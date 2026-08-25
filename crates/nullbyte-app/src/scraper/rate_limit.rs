//! Cache + kvotos apsauga aplink `screenscraper::lookup_game` (MVP.md P6.2, CLAUDE.md §9.3).
//!
//! Sluoksniavimas sąmoningai atskirtas nuo `screenscraper.rs`: TEN gyvena „kaip kalbėtis su
//! ScreenScraper per vieną HTTP kvietimą", ČIA — „kaip elgtis su DAUG kvietimų per laiką"
//! (cache, semaforas, backoff). `cached_lookup` priima `fetch` closure'ą, ne tiesiogiai
//! `screenscraper::lookup_game`, kad testai galėtų patikrinti cache/backoff logiką BE tinklo
//! (žr. `tests` modulį).

#![allow(dead_code)] // pilnai išnaudos P6.4 (scraping orkestracija sujungs su scanner/db)

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension};
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};

use crate::error::AppError;
use crate::scraper::screenscraper::{GameMetadata, LookupError, LookupSuccess, QuotaInfo};

/// „notfound" įrašų TTL — CLAUDE.md §9.3 taisyklė 1: „nesėkmingi rezultatai cache'uojami —
/// su TTL, pvz. 7 dienos". Sėkmingi (`found=1`) įrašai TTL neturi (žr. `read_cache`).
const NOTFOUND_TTL_SECONDS: i64 = 7 * 24 * 60 * 60;

/// MVP.md P6.2 „Ką daryti": „Exponential backoff: 429/430/API closed → 2s, 4s, 8s, 16s, tada
/// sustok". 4 laukimai tarp bandymų + PASKUTINIS bandymas be laukimo (žr. `retry_with_backoff`)
/// = iš viso 5 bandymai, atitinka „tada sustok" po keturių pakartotinių bandymų.
const BACKOFF_SCHEDULE_SECONDS: &[u64] = &[2, 4, 8, 16];

/// Rezultatas su `from_cache` žyma — naudinga UI/log'ams atskirti „nereikėjo tinklo" nuo
/// „gauta gyvai".
#[derive(Debug)]
pub struct CachedLookup {
    pub outcome: crate::scraper::screenscraper::ScrapeOutcome,
    pub quota: Option<QuotaInfo>,
    pub from_cache: bool,
}

/// Vienalaikių ScreenScraper užklausų limitas (CLAUDE.md §9.3 taisyklė 2). Prasideda nuo
/// VIENO leidimo (saugiausia numatytoji, kol dar nežinomas realus `maxthreads`) ir AUGA
/// aukštyn, kai atsakymas atneša didesnę `maxthreads` reikšmę.
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    granted: AtomicUsize,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(1)),
            granted: AtomicUsize::new(1),
        }
    }

    /// AUGA tik AUKŠTYN — tokio `Semaphore` neturi saugaus būdo ATIMTI jau išduotus leidimus
    /// aktyvioms `acquire` operacijoms. MVP supaprastinimas (dokumentuota CLAUDE.md
    /// prasme — nekomplikuojam sprendimo dėl atvejo, kurio šią sesiją API neparodė): jei
    /// serveris kada nors sumažintų `maxthreads`, limitas liktų senas platesnis iki
    /// proceso persileidimo.
    pub fn update_maxthreads(&self, maxthreads: u32) {
        let maxthreads = maxthreads.max(1) as usize;
        let previous = self.granted.fetch_max(maxthreads, Ordering::SeqCst);
        if maxthreads > previous {
            self.semaphore.add_permits(maxthreads - previous);
        }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        Arc::clone(&self.semaphore)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache rakto formatas: CRC pirmenybė (unikaliausias tiksliam ROM'ui, atsparus pervadinimui),
/// kitaip vardas+platforma+failas. Šis formatas — ŠIO modulio sprendimas, ne kažkur kitur
/// dokumentuotas kontraktas, tad keičiant reikia migruoti/išvalyti esamą `scrape_cache` turinį.
pub fn cache_key(crc32: Option<&str>, platform_slug: &str, filename: &str) -> String {
    match crc32 {
        Some(crc) => format!("crc:{}", crc.to_uppercase()),
        None => format!("name:{platform_slug}:{filename}"),
    }
}

/// Cache'uotas + kvotos-saugus ScreenScraper lookup'as.
///
/// 1. Tikrina `scrape_cache` (be tinklo).
/// 2. Cache miss → laukia semaforo leidimo (CLAUDE.md §9.3 „Gerbk maxthreads").
/// 3. Kviečia `fetch` su exponential backoff, kai atsakymas — rate limit.
/// 4. Sėkmę/„notfound" rašo atgal į `scrape_cache`.
///
/// `fetch` — injektuojamas (ne tiesioginis `screenscraper::lookup_game` kvietimas), kad testai
/// patikrintų cache/backoff logiką be realaus tinklo (žr. `tests` modulį — „cache hit niekada
/// nekviečia fetch").
pub async fn cached_lookup<F, Fut>(
    conn: &Mutex<Connection>,
    limiter: &RateLimiter,
    hash_key: &str,
    fetch: F,
) -> Result<CachedLookup, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<LookupSuccess, LookupError>>,
{
    if let Some(cached) = read_cache(conn, hash_key)? {
        return Ok(cached);
    }

    let permit = limiter
        .semaphore()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Other("rate limiter semaforas uždarytas".to_string()))?;

    let result = retry_with_backoff(&fetch).await;
    drop(permit);

    let success = result?;
    if let Some(quota) = &success.quota {
        limiter.update_maxthreads(quota.maxthreads);
    }
    write_cache(conn, hash_key, &success.outcome)?;

    Ok(CachedLookup {
        outcome: success.outcome,
        quota: success.quota,
        from_cache: false,
    })
}

/// Bando `fetch` iki `BACKOFF_SCHEDULE_SECONDS.len() + 1` kartų, laukdamas tarpuose, kai
/// atsakymas — `LookupError::RateLimited`. Kitos klaidos (`LookupError::Failed`) grąžinamos
/// IŠKART, be pakartojimo — tik rate-limit yra „palauk ir bandyk vėl" atvejis.
async fn retry_with_backoff<F, Fut>(fetch: &F) -> Result<LookupSuccess, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<LookupSuccess, LookupError>>,
{
    for &delay in BACKOFF_SCHEDULE_SECONDS {
        match fetch().await {
            Ok(success) => return Ok(success),
            Err(LookupError::Failed(error)) => return Err(error),
            Err(LookupError::RateLimited { status }) => {
                tracing::warn!("ScreenScraper rate limit (HTTP {status}), laukiama {delay}s");
                sleep(Duration::from_secs(delay)).await;
            }
        }
    }

    match fetch().await {
        Ok(success) => Ok(success),
        Err(LookupError::Failed(error)) => Err(error),
        Err(LookupError::RateLimited { status }) => Err(AppError::Other(format!(
            "ScreenScraper kvota viršyta net po {} bandymų (paskutinis HTTP {status})",
            BACKOFF_SCHEDULE_SECONDS.len() + 1
        ))),
    }
}

fn read_cache(conn: &Mutex<Connection>, hash_key: &str) -> Result<Option<CachedLookup>, AppError> {
    let conn = conn.lock().unwrap();
    let row: Option<(Option<String>, i64, i64)> = conn
        .query_row(
            "SELECT response, found, fetched_at FROM scrape_cache WHERE hash_key = ?1",
            [hash_key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    drop(conn);

    let Some((response, found, fetched_at)) = row else {
        return Ok(None);
    };

    if found == 0 {
        let age = now_unix() - fetched_at;
        if age > NOTFOUND_TTL_SECONDS {
            return Ok(None); // TTL pasibaigė — traktuojam kaip cache miss, bandom vėl.
        }
        return Ok(Some(CachedLookup {
            outcome: crate::scraper::screenscraper::ScrapeOutcome::NotFound,
            quota: None,
            from_cache: true,
        }));
    }

    let metadata: GameMetadata = response
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|error| AppError::Other(format!("scrape_cache JSON klaida: {error}")))?
        .ok_or_else(|| AppError::Other("scrape_cache: found=1, bet response=NULL".to_string()))?;

    Ok(Some(CachedLookup {
        outcome: crate::scraper::screenscraper::ScrapeOutcome::Found(metadata),
        quota: None, // cache'uotas įrašas neturi kvotos — ji aktuali TIK gyvo atsakymo metu.
        from_cache: true,
    }))
}

fn write_cache(
    conn: &Mutex<Connection>,
    hash_key: &str,
    outcome: &crate::scraper::screenscraper::ScrapeOutcome,
) -> Result<(), AppError> {
    use crate::scraper::screenscraper::ScrapeOutcome;

    let (found, response) = match outcome {
        ScrapeOutcome::Found(metadata) => (
            1i64,
            Some(serde_json::to_string(metadata).map_err(|error| {
                AppError::Other(format!("scrape_cache serializavimo klaida: {error}"))
            })?),
        ),
        ScrapeOutcome::NotFound => (0i64, None),
    };

    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO scrape_cache (hash_key, response, found, fetched_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(hash_key) DO UPDATE SET
             response = excluded.response,
             found = excluded.found,
             fetched_at = excluded.fetched_at",
        rusqlite::params![hash_key, response, found, now_unix()],
    )?;

    Ok(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("sistemos laikrodis prieš 1970")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraper::screenscraper::ScrapeOutcome;
    use std::sync::atomic::AtomicU32;

    /// Tas pats šablonas kaip `db::games`/`library::scanner` testuose: `Connection::open`
    /// su `":memory:"` KELIU sukurtų realų failą pavadinimu „:memory:" (ne in-memory DB) —
    /// tikras in-memory ryšys reikalauja `open_in_memory()`, tad `open_and_migrate` (kuris
    /// visada eina per `Connection::open(path)`) čia netinka.
    fn test_db() -> Mutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        for &(_, sql) in crate::db::migrations::MIGRATIONS {
            conn.execute_batch(sql).unwrap();
        }
        Mutex::new(conn)
    }

    fn sample_metadata() -> GameMetadata {
        GameMetadata {
            title: "Test Game".to_string(),
            description: Some("desc".to_string()),
            developer: Some("Dev".to_string()),
            publisher: Some("Pub".to_string()),
            genre: Some("Platform".to_string()),
            players: Some(1),
            release_date: Some("1994-07-28".to_string()),
            rating: Some(16.0),
            region: Some("eu".to_string()),
            medias: vec![],
        }
    }

    #[test]
    fn cache_key_prefers_crc_over_name() {
        assert_eq!(cache_key(Some("ad2cbf9c"), "snes", "x.sfc"), "crc:AD2CBF9C");
        assert_eq!(
            cache_key(None, "snes", "Super Metroid.sfc"),
            "name:snes:Super Metroid.sfc"
        );
    }

    /// MVP.md P6.2 acceptance: „Pakartotinė užklausa nesikreipia į tinklą" — `fetch` closure
    /// turi būti kviečiamas LYGIAI VIENĄ kartą (pirmam, cache-miss lookup'ui); antras
    /// `cached_lookup` kvietimas su tuo pačiu raktu turi rasti cache'e ir NEBEKVIESTI `fetch`.
    #[tokio::test]
    async fn second_lookup_with_same_key_hits_cache_and_skips_fetch() {
        let db = test_db();
        let limiter = RateLimiter::new();
        let calls = AtomicU32::new(0);

        let fetch = || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Ok(LookupSuccess {
                    outcome: ScrapeOutcome::Found(sample_metadata()),
                    quota: Some(QuotaInfo {
                        maxthreads: 1,
                        requests_today: 1,
                        max_requests_per_day: 20000,
                        closed_for_nonmember: false,
                        closed_for_leecher: false,
                    }),
                })
            }
        };

        let first = cached_lookup(&db, &limiter, "crc:AAAA", fetch)
            .await
            .unwrap();
        assert!(!first.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let second = cached_lookup(&db, &limiter, "crc:AAAA", fetch)
            .await
            .unwrap();
        assert!(second.from_cache);
        assert_eq!(calls.load(Ordering::SeqCst), 1); // fetch NEBUVO iškviestas antrą kartą.

        let ScrapeOutcome::Found(metadata) = second.outcome else {
            panic!("tikėtasi Found iš cache");
        };
        assert_eq!(metadata.title, "Test Game");
    }

    /// „notfound" taip pat cache'uojamas (CLAUDE.md §9.3 taisyklė 1) — antras kvietimas
    /// negrįžta prie tinklo, kol TTL negaliojantis.
    #[tokio::test]
    async fn notfound_is_cached_within_ttl() {
        let db = test_db();
        let limiter = RateLimiter::new();
        let calls = AtomicU32::new(0);

        let fetch = || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Ok(LookupSuccess {
                    outcome: ScrapeOutcome::NotFound,
                    quota: None,
                })
            }
        };

        cached_lookup(&db, &limiter, "crc:BBBB", fetch)
            .await
            .unwrap();
        let second = cached_lookup(&db, &limiter, "crc:BBBB", fetch)
            .await
            .unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(second.from_cache);
        assert!(matches!(second.outcome, ScrapeOutcome::NotFound));
    }

    /// Pasibaigęs TTL → cache miss, `fetch` kviečiamas vėl. Rašom praeities `fetched_at`
    /// tiesiogiai į DB (ne per `write_cache`, kuris visada rašo dabartinį laiką), kad
    /// simuliuotume „prieš 8 dienas cache'uotą notfound".
    #[tokio::test]
    async fn expired_notfound_ttl_triggers_new_fetch() {
        let db = test_db();
        let limiter = RateLimiter::new();
        let calls = AtomicU32::new(0);
        let eight_days_ago = now_unix() - (8 * 24 * 60 * 60);

        {
            let conn = db.lock().unwrap();
            conn.execute(
                "INSERT INTO scrape_cache (hash_key, response, found, fetched_at) VALUES (?1, NULL, 0, ?2)",
                rusqlite::params!["crc:CCCC", eight_days_ago],
            )
            .unwrap();
        }

        let fetch = || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Ok(LookupSuccess {
                    outcome: ScrapeOutcome::NotFound,
                    quota: None,
                })
            }
        };

        let result = cached_lookup(&db, &limiter, "crc:CCCC", fetch)
            .await
            .unwrap();
        assert!(!result.from_cache); // pasenęs TTL → traktuojama kaip miss, gyvas fetch iškviestas.
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    /// MVP.md P6.2 acceptance: „Kvotos viršijimas nesulaužo — sustoja su aiškiu pranešimu
    /// UI". Naudoja `RateLimited` be jokio realaus laukimo laiko patikros (testas nelaukia
    /// realių 2/4/8/16s — tikrina tik GALUTINĮ elgesį: `Err`, aiškus pranešimas, `fetch`
    /// iškviestas TIKSLIAI backoff'o numatytą kartų skaičių).
    #[tokio::test]
    async fn persistent_rate_limit_gives_up_with_clear_error_after_all_retries() {
        let db = test_db();
        let limiter = RateLimiter::new();
        let calls = AtomicU32::new(0);

        let fetch = || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Err(LookupError::RateLimited {
                    status: reqwest::StatusCode::TOO_MANY_REQUESTS,
                })
            }
        };

        let result = cached_lookup(&db, &limiter, "crc:DDDD", fetch).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("kvota"));
        assert_eq!(
            calls.load(Ordering::SeqCst) as usize,
            BACKOFF_SCHEDULE_SECONDS.len() + 1
        );
    }

    /// Ne-rate-limit klaida (`LookupError::Failed`) grąžinama IŠKART, be pakartojimo —
    /// backoff skirtas TIK kvotos atvejui (MVP.md P6.2 „Ką daryti").
    #[tokio::test]
    async fn non_rate_limit_error_fails_fast_without_retry() {
        let db = test_db();
        let limiter = RateLimiter::new();
        let calls = AtomicU32::new(0);

        let fetch = || {
            calls.fetch_add(1, Ordering::SeqCst);
            async {
                Err(LookupError::Failed(AppError::Other(
                    "sugadinta".to_string(),
                )))
            }
        };

        let result = cached_lookup(&db, &limiter, "crc:EEEE", fetch).await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn update_maxthreads_only_grows_and_adds_correct_permit_delta() {
        let limiter = RateLimiter::new();
        assert_eq!(limiter.semaphore().available_permits(), 1);

        limiter.update_maxthreads(4);
        assert_eq!(limiter.semaphore().available_permits(), 4);

        limiter.update_maxthreads(2); // mažesnė reikšmė — ignoruojama (žr. doc).
        assert_eq!(limiter.semaphore().available_permits(), 4);

        limiter.update_maxthreads(6);
        assert_eq!(limiter.semaphore().available_permits(), 6);
    }

    /// REALUS tinklo kvietimas — `cached_lookup` sujungtas su TIKRU
    /// `screenscraper::lookup_game`, ne injektuotu `fetch` (skirtingai nuo likusių šio
    /// modulio testų). Tikrina, kad P6.1 sluoksnis (HTTP+JSON) ir P6.2 sluoksnis
    /// (cache+semaforas) realiai sujungia be tipų/signatūrų neatitikimų — vien unit testai su
    /// injektuotu `fetch` to NEIŠBANDO (žr. sesijos pastabą: klaidos dažniausiai išlenda
    /// TIK kai NAUJAS sluoksnis realiai panaudoja ANKSTESNĮ). `#[ignore]`: priklauso nuo
    /// tinklo IR realių `.env` kredencialų. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_lookup_populates_cache_then_second_call_is_cache_hit -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_lookup_populates_cache_then_second_call_is_cache_hit() {
        let dir =
            std::env::temp_dir().join(format!("nullbyte_rate_limit_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let db_path = dir.join("nullbyte.db");
        let conn = crate::db::migrations::open_and_migrate(&db_path).unwrap();
        let db = Mutex::new(conn);
        let limiter = RateLimiter::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let credentials = crate::scraper::screenscraper::ScreenScraperCredentials::from_env()
                .expect(".env turėtų turėti SCREENSCRAPER_DEV_ID/DEV_PASSWORD");
            let client = reqwest::Client::new();
            let rom = crate::scraper::screenscraper::RomIdentity {
                crc32: Some("AD2CBF9C"),
                md5: None,
                sha1: None,
                size: Some(3_145_728),
                filename: "Super Metroid.sfc",
                systemeid: 4,
            };
            let key = cache_key(rom.crc32, "snes", rom.filename);

            let fetch = || crate::scraper::screenscraper::lookup_game(&client, &credentials, &rom);

            let first = cached_lookup(&db, &limiter, &key, fetch).await.unwrap();
            assert!(!first.from_cache);
            let ScrapeOutcome::Found(ref metadata) = first.outcome else {
                panic!("tikėtasi Found realiam Super Metroid CRC'ui");
            };
            assert_eq!(metadata.title, "Super Metroid");
            assert!(first.quota.is_some());
            eprintln!("kvota po pirmo kvietimo: {:?}", first.quota);

            let second = cached_lookup(&db, &limiter, &key, fetch).await.unwrap();
            assert!(second.from_cache); // antras kvietimas — TIK cache, be tinklo.
            let ScrapeOutcome::Found(ref metadata2) = second.outcome else {
                panic!("tikėtasi Found iš cache");
            };
            assert_eq!(metadata2.title, "Super Metroid");
        });

        drop(db);
        std::fs::remove_dir_all(&dir).ok();
    }
}
