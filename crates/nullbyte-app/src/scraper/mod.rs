//! ScreenScraper API klientas, kvotos, media atsisiuntimas, scraping orkestracija
//! (CLAUDE.md §3.1, §9, Faza 6).
//!
//! `types`/`screenscraper` (P6.1) — API klientas. `rate_limit` (P6.2) — cache + kvota.
//! `media` (P6.3) — failų atsisiuntimas. Šis failas (P6.4) — juos visus sujungianti
//! orkestracija: vienas žaidimas arba visa `scrape_status = 'pending'` eilė.
//!
//! SĄMONINGAI Tauri-nepriklausoma (tas pats principas kaip `library::scanner` — žr. jo
//! modulio doc): `on_progress` yra paprastas `FnMut`, ne `tauri::ipc::Channel`, kad
//! orkestracijos logika liktų testuojama be `tauri::test` scaffolding'o. Plonas
//! `commands::scraper` sluoksnis (P6.4) persiunčia progresą per `Channel<ScrapeProgress>` ir
//! valdo `CancellationToken` gyvavimo trukmę per `AppState`.
//!
//! **SĄMONINGAI NEGENERALIZUOTA per injektuojamą `fetch` closure'ą** (skirtingai nuo
//! `rate_limit::cached_lookup`, kuri tą daro) — čia `scrape_one_game` visada kviečia TIKRĄ
//! `screenscraper::lookup_game`. Generalizavimas šiame sluoksnyje reikalautų arba HRTB
//! (`for<'r> Fn(&'r RomIdentity<'r>) -> Fut`, kuris konfliktuoja su vienu fiksuotu `Fut` tipo
//! parametru), arba `Box<dyn Fn(..) -> Pin<Box<dyn Future>>>` dinaminio dispatch'o — abu
//! neproporcingai sudėtingi šiam vieninteliam kvietimo taškui, kai `cached_lookup` savo ruožtu
//! JAU pilnai unit-testuota (P6.2) su injektuotu `fetch`. Šio sluoksnio korektiškumas
//! patikrinamas gyvais `#[ignore]` testais (žr. `tests` modulį), o `rom_filename`/DB
//! funkcijos (`db::games::set_scrape_status`/`apply_scrape_result`) testuojamos greitai atskirai.

pub mod image_dimensions;
pub mod media;
pub mod rate_limit;
pub mod screenscraper;
pub mod types;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use tokio_util::sync::CancellationToken;

use crate::db::games;
use crate::db::models::{Game, Platform};
use crate::error::AppError;
use screenscraper::{QuotaInfo, ScrapeOutcome, ScreenScraperCredentials};

/// Vieno žaidimo progreso pranešimas — CLAUDE.md §7.3 (IPC struct'ai: `Serialize` +
/// camelCase). MVP.md P6.4: `Channel<ScrapeProgress { current, total, title, status,
/// quota_left }>`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeProgress {
    pub current: u32,
    pub total: u32,
    pub title: String,
    /// `"ok" | "notfound" | "unsupported" | "error"` — PLATESNIS žodynas nei
    /// `games.scrape_status` DB stulpelis (kuris `"unsupported"` neturi, žr.
    /// [`GameOutcome::status_str`] doc), nes šis laukas yra tik efemeriška UI informacija.
    pub status: String,
    /// `None`, kol dar negauta nė vieno GYVO (ne cache) atsakymo su kvotos informacija.
    pub quota_left: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeSummary {
    pub found: u32,
    pub not_found: u32,
    pub errored: u32,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameOutcome {
    Found,
    NotFound,
    /// Platformos `screenscraper_id` NEŽINOMAS (P5.1 seed — kai kurios platformos sąmoningai
    /// paliktos su `NULL`, žr. migracijos komentarą) — negalima net bandyti paieškos.
    Unsupported,
    Error,
}

impl GameOutcome {
    /// DB stulpeliui `games.scrape_status` rašoma TIESIOGIAI (`"error"`/`"notfound"`/`"ok"`
    /// literalais atskiruose kvietimuose, žr. `scrape_one_game`), NE per šią funkciją — ji
    /// skirta TIK `ScrapeProgress.status` (efemeriška UI reikšmė, gali būti smulkesnė už DB
    /// keturių reikšmių žodyną, pvz. `"unsupported"`).
    fn status_str(self) -> &'static str {
        match self {
            GameOutcome::Found => "ok",
            GameOutcome::NotFound => "notfound",
            GameOutcome::Unsupported => "unsupported",
            GameOutcome::Error => "error",
        }
    }
}

/// ROM'o failo vardas paieškai — vidinis archyvo failas, jei suarchyvuota, kitaip
/// `rom_path`'o paskutinis komponentas (žr. `RomIdentity.filename` doc — ScreenScraper
/// `romnom` parametrui reikia PAČIO ROM failo vardo, ne archyvo).
fn rom_filename(game: &Game) -> String {
    game.archive_inner.clone().unwrap_or_else(|| {
        Path::new(&game.rom_path)
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| game.rom_path.clone())
    })
}

fn set_status_logged(db: &Mutex<Connection>, id: i64, status: &str) {
    let conn = db.lock().unwrap();
    if let Err(error) = games::set_scrape_status(&conn, id, status) {
        tracing::warn!(game_id = id, %error, "nepavyko atnaujinti scrape_status");
    }
}

/// Sugrupuoja bendrus scraping'o priklausomumus vienu parametru — be šito `scrape_pending_games`
/// viršytų clippy `too_many_arguments` limitą (7). Visi laukai — nuorodos, gyvuojančios visos
/// vienos `scrape_game`/`scrape_library` Tauri komandos vykdymo trukmę.
pub struct ScrapeContext<'a> {
    pub client: &'a reqwest::Client,
    pub credentials: &'a ScreenScraperCredentials,
    pub limiter: &'a rate_limit::RateLimiter,
    pub db: &'a Mutex<Connection>,
    pub media_dir: &'a Path,
}

/// Vieno žaidimo scraping'as: paieška → (jei rasta) media atsisiuntimas → DB rašymas.
///
/// `Err` grąžinamas TIK jei PATI `rate_limit::cached_lookup` nepavyko (paprastai — kvota
/// išnaudota po viso backoff'o, žr. `rate_limit.rs`) — tai signalas kviečiančiajam
/// (`scrape_pending_games`) sustabdyti VISĄ likusią eilę, ne tik šį žaidimą (MVP.md P6.4
/// acceptance: „Kvotos pabaiga sustabdo švariai"). Visos KITOS problemos (nepalaikoma
/// platforma, media katalogo klaida, DB rašymo klaida) — `Ok` su `GameOutcome::Error`/
/// `Unsupported`, nes tai yra TO VIENO žaidimo problema, neturėtų sustabdyti likusios eilės.
async fn scrape_one_game(
    ctx: &ScrapeContext<'_>,
    game: &Game,
    platform: &Platform,
) -> Result<(GameOutcome, Option<QuotaInfo>), AppError> {
    let Some(systemeid) = platform.screenscraper_id else {
        set_status_logged(ctx.db, game.id, "error");
        return Ok((GameOutcome::Unsupported, None));
    };

    let filename = rom_filename(game);
    let key = rate_limit::cache_key(game.crc32.as_deref(), &platform.slug, &filename);
    let rom = screenscraper::RomIdentity {
        crc32: game.crc32.as_deref(),
        md5: game.md5.as_deref(),
        sha1: game.sha1.as_deref(),
        size: Some(game.rom_size as u64),
        filename: &filename,
        systemeid,
    };

    let cached = rate_limit::cached_lookup(ctx.db, ctx.limiter, &key, || {
        screenscraper::lookup_game(ctx.client, ctx.credentials, &rom)
    })
    .await?;
    let quota = cached.quota;

    match cached.outcome {
        ScrapeOutcome::NotFound => {
            set_status_logged(ctx.db, game.id, "notfound");
            Ok((GameOutcome::NotFound, quota))
        }
        ScrapeOutcome::Found(metadata) => {
            match media::download_game_media(ctx.client, ctx.media_dir, game.id, &metadata.medias)
                .await
            {
                Ok(media_paths) => {
                    let write_result = {
                        let conn = ctx.db.lock().unwrap();
                        games::apply_scrape_result(&conn, game.id, &metadata, &media_paths)
                    };
                    match write_result {
                        Ok(()) => Ok((GameOutcome::Found, quota)),
                        Err(error) => {
                            tracing::warn!(game_id = game.id, %error, "nepavyko įrašyti scraping rezultato į DB");
                            set_status_logged(ctx.db, game.id, "error");
                            Ok((GameOutcome::Error, quota))
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(game_id = game.id, %error, "media katalogo klaida");
                    set_status_logged(ctx.db, game.id, "error");
                    Ok((GameOutcome::Error, quota))
                }
            }
        }
    }
}

fn quota_left(quota: &QuotaInfo) -> i64 {
    (quota
        .max_requests_per_day
        .saturating_sub(quota.requests_today)) as i64
}

/// Vieno žaidimo scraping'as, apvilktas progreso pranešimu ir santrauka — MVP.md P6.4
/// `scrape_game(id)`.
pub async fn scrape_single_game(
    ctx: &ScrapeContext<'_>,
    game_id: i64,
    mut on_progress: impl FnMut(ScrapeProgress),
) -> Result<ScrapeSummary, AppError> {
    let (game, platform) = {
        let conn = ctx.db.lock().unwrap();
        let game = games::get_game(&conn, game_id)?
            .ok_or_else(|| AppError::Other(format!("žaidimas #{game_id} nerastas")))?;
        let platform = games::list_platforms(&conn)?
            .into_iter()
            .map(|summary| summary.platform)
            .find(|p| p.id == game.platform_id)
            .ok_or_else(|| AppError::Other(format!("platforma #{} nerasta", game.platform_id)))?;
        (game, platform)
    };

    let (outcome, quota) = scrape_one_game(ctx, &game, &platform).await?;

    let mut summary = ScrapeSummary::default();
    match outcome {
        GameOutcome::Found => summary.found = 1,
        GameOutcome::NotFound => summary.not_found = 1,
        GameOutcome::Unsupported | GameOutcome::Error => summary.errored = 1,
    }

    on_progress(ScrapeProgress {
        current: 1,
        total: 1,
        title: game.title.clone(),
        status: outcome.status_str().to_string(),
        quota_left: quota.as_ref().map(quota_left),
    });

    Ok(summary)
}

/// Visos `scrape_status = 'pending'` eilės scraping'as, neprivalomai filtruojant pagal
/// platformą — MVP.md P6.4 `scrape_library(platform_id?)`.
///
/// Atšaukimas tikrinamas PRIEŠ kiekvieną žaidimą IR lenktyniaujamas (`tokio::select!`) prieš
/// PATĮ VIENO ŽAIDIMO scraping'ą — taip atšaukimas suveikia net vidury `retry_with_backoff`
/// laukimo (MVP.md acceptance „Atšaukimas veikia iškart"), ne tik tarp žaidimų. Kai
/// `select!` pasirenka atšaukimo šaką, `scrape_one_game`'o Future tiesiog numetamas (Rust
/// async cooperative cancellation per `Drop`) — jokių resursų nutekėjimo, nes viduje
/// nėra jokių `unsafe`/rankinių resursų, tik `OwnedSemaphorePermit` (RAII) ir tinklo/failų
/// rankenos, kurios visos turi teisingą `Drop`.
pub async fn scrape_pending_games(
    ctx: &ScrapeContext<'_>,
    platform_id: Option<i64>,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(ScrapeProgress),
) -> Result<ScrapeSummary, AppError> {
    let (pending_games, platform_by_id): (Vec<Game>, HashMap<i64, Platform>) = {
        let conn = ctx.db.lock().unwrap();
        let pending_games = games::list_pending_games(&conn, platform_id)?;
        let platform_by_id = games::list_platforms(&conn)?
            .into_iter()
            .map(|summary| (summary.platform.id, summary.platform))
            .collect();
        (pending_games, platform_by_id)
    };

    let total = pending_games.len() as u32;
    let mut summary = ScrapeSummary::default();
    let mut latest_quota: Option<QuotaInfo> = None;

    for (index, game) in pending_games.iter().enumerate() {
        if cancel.is_cancelled() {
            summary.cancelled = true;
            break;
        }

        let Some(platform) = platform_by_id.get(&game.platform_id) else {
            // Schemos FOREIGN KEY garantija — praktiškai nepasiekiama, bet netikimasi panikos.
            summary.errored += 1;
            continue;
        };

        let scrape_future = scrape_one_game(ctx, game, platform);
        let outcome_result = tokio::select! {
            biased;
            () = cancel.cancelled() => {
                summary.cancelled = true;
                break;
            }
            result = scrape_future => result,
        };

        let (outcome, quota) = match outcome_result {
            Ok(pair) => pair,
            Err(_lookup_error) => {
                // `scrape_one_game` grąžina `Err` TIK kai `cached_lookup` PATI nepavyko — žr.
                // jos doc. Sustabdome VISĄ likusią eilę švariai, likusieji lieka `pending`
                // kitam bandymui (MVP.md acceptance „Kvotos pabaiga sustabdo švariai").
                on_progress(ScrapeProgress {
                    current: index as u32 + 1,
                    total,
                    title: game.title.clone(),
                    status: "error".to_string(),
                    quota_left: latest_quota.as_ref().map(quota_left),
                });
                break;
            }
        };

        if let Some(q) = quota {
            latest_quota = Some(q);
        }

        match outcome {
            GameOutcome::Found => summary.found += 1,
            GameOutcome::NotFound => summary.not_found += 1,
            GameOutcome::Unsupported | GameOutcome::Error => summary.errored += 1,
        }

        on_progress(ScrapeProgress {
            current: index as u32 + 1,
            total,
            title: game.title.clone(),
            status: outcome.status_str().to_string(),
            quota_left: latest_quota.as_ref().map(quota_left),
        });
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Mutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        for &(_, sql) in crate::db::migrations::MIGRATIONS {
            conn.execute_batch(sql).unwrap();
        }
        Mutex::new(conn)
    }

    fn snes_platform_id(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM platforms WHERE slug = 'snes'", [], |r| {
            r.get(0)
        })
        .unwrap()
    }

    /// `intellivision` P5.1 seed'e turi `screenscraper_id = NULL` (nepatikrintas, žr.
    /// migracijos komentarą) — tinka `Unsupported` šakai testuoti be tinklo.
    fn platform_without_screenscraper_id(conn: &Connection) -> i64 {
        conn.query_row(
            "SELECT id FROM platforms WHERE slug = 'intellivision'",
            [],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn insert_game(conn: &Connection, platform_id: i64, title: &str, rom_path: &str) -> i64 {
        conn.execute(
            "INSERT INTO games (platform_id, title, sort_title, rom_path, rom_size, added_at, file_mtime)
             VALUES (?1, ?2, ?3, ?4, 0, 0, 0)",
            rusqlite::params![platform_id, title, title.to_lowercase(), rom_path],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn rom_filename_prefers_archive_inner_over_rom_path() {
        let game = games_test_row(
            "/roms/collection.zip",
            Some("Super Metroid.sfc".to_string()),
        );
        assert_eq!(rom_filename(&game), "Super Metroid.sfc");
    }

    #[test]
    fn rom_filename_falls_back_to_rom_path_basename() {
        let game = games_test_row("/roms/snes/Super Metroid.sfc", None);
        assert_eq!(rom_filename(&game), "Super Metroid.sfc");
    }

    fn games_test_row(rom_path: &str, archive_inner: Option<String>) -> Game {
        Game {
            id: 1,
            platform_id: 1,
            title: "x".to_string(),
            sort_title: "x".to_string(),
            rom_path: rom_path.to_string(),
            rom_size: 0,
            archive_inner,
            crc32: None,
            md5: None,
            sha1: None,
            description: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            release_date: None,
            rating: None,
            region: None,
            cover_path: None,
            cover_width: None,
            cover_height: None,
            screenshot_path: None,
            wheel_path: None,
            video_path: None,
            scrape_status: "pending".to_string(),
            scraped_at: None,
            last_played: None,
            play_count: 0,
            play_time_seconds: 0,
            favorite: false,
            added_at: 0,
            file_mtime: 0,
        }
    }

    /// Platforma be žinomo `screenscraper_id` → `Unsupported`, BE JOKIO tinklo kvietimo
    /// (jei kodas VIS TIEK bandytų tinklą, `credentials`/`client` yra tušti/netikri ir
    /// kvietimas užstrigtų arba grąžintų klaidą — testas naudoja trumpą timeout'ą per
    /// apibrėžimą to nedaro, tad šis testas iš principo veiktų BE tinklo prieigos apskritai).
    #[tokio::test]
    async fn unsupported_platform_short_circuits_without_network() {
        let db = test_db();
        let media_dir = std::env::temp_dir().join(format!(
            "nullbyte_scrape_test_{}_{}",
            std::process::id(),
            "unsupported"
        ));
        let platform_id = {
            let conn = db.lock().unwrap();
            platform_without_screenscraper_id(&conn)
        };
        let game_id = {
            let conn = db.lock().unwrap();
            insert_game(&conn, platform_id, "Astrosmash", "/roms/astrosmash.int")
        };
        let (game, platform) = {
            let conn = db.lock().unwrap();
            let game = games::get_game(&conn, game_id).unwrap().unwrap();
            let platform = games::list_platforms(&conn)
                .unwrap()
                .into_iter()
                .map(|s| s.platform)
                .find(|p| p.id == platform_id)
                .unwrap();
            (game, platform)
        };

        let client = reqwest::Client::new();
        let credentials = ScreenScraperCredentials {
            devid: "unused".to_string(),
            devpassword: "unused".to_string(),
            ssid: None,
            sspassword: None,
        };
        let limiter = rate_limit::RateLimiter::new();
        let ctx = ScrapeContext {
            client: &client,
            credentials: &credentials,
            limiter: &limiter,
            db: &db,
            media_dir: &media_dir,
        };

        let (outcome, quota) = scrape_one_game(&ctx, &game, &platform).await.unwrap();

        assert_eq!(outcome, GameOutcome::Unsupported);
        assert!(quota.is_none());

        let conn = db.lock().unwrap();
        let updated = games::get_game(&conn, game_id).unwrap().unwrap();
        assert_eq!(updated.scrape_status, "error");
    }

    /// MVP.md P6.4 acceptance: „Atšaukimas veikia iškart" — atšaukta PRIEŠ pradedant, tad
    /// `scrape_pending_games` neturėtų apdoroti NĖ VIENO žaidimo, net jei eilėje jų yra.
    #[tokio::test]
    async fn cancellation_before_start_processes_nothing() {
        let db = test_db();
        let snes = { snes_platform_id(&db.lock().unwrap()) };
        {
            let conn = db.lock().unwrap();
            insert_game(&conn, snes, "Chrono Trigger", "/roms/ct.sfc");
            insert_game(&conn, snes, "EarthBound", "/roms/eb.sfc");
        }

        let client = reqwest::Client::new();
        let credentials = ScreenScraperCredentials {
            devid: "unused".to_string(),
            devpassword: "unused".to_string(),
            ssid: None,
            sspassword: None,
        };
        let limiter = rate_limit::RateLimiter::new();
        let media_dir = std::env::temp_dir();
        let ctx = ScrapeContext {
            client: &client,
            credentials: &credentials,
            limiter: &limiter,
            db: &db,
            media_dir: &media_dir,
        };
        let cancel = CancellationToken::new();
        cancel.cancel();

        let mut progress_calls = 0;
        let summary = scrape_pending_games(&ctx, None, &cancel, |_| progress_calls += 1)
            .await
            .unwrap();

        assert!(summary.cancelled);
        assert_eq!(summary.found + summary.not_found + summary.errored, 0);
        assert_eq!(progress_calls, 0);
    }

    #[test]
    fn quota_left_subtracts_used_from_max() {
        let quota = QuotaInfo {
            maxthreads: 1,
            requests_today: 30,
            max_requests_per_day: 100,
            closed_for_nonmember: false,
            closed_for_leecher: false,
        };
        assert_eq!(quota_left(&quota), 70);
    }

    /// REALUS tinklo kvietimas — `scrape_single_game` sujungtas su TIKRU
    /// `screenscraper::lookup_game` IR TIKRU media atsisiuntimu, patikrina, kad DB eilutė
    /// REALIAI atsinaujina (ne tik kad kodas nepanikuoja). `#[ignore]`: priklauso nuo tinklo
    /// IR realių `.env` kredencialų. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_scrape_single_game_updates_db_row -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_scrape_single_game_updates_db_row() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db = test_db();
            let snes = snes_platform_id(&db.lock().unwrap());
            let game_id = {
                let conn = db.lock().unwrap();
                conn.execute(
                    "INSERT INTO games (platform_id, title, sort_title, rom_path, rom_size, crc32, added_at, file_mtime)
                     VALUES (?1, 'Super Metroid', 'super metroid', '/roms/Super Metroid.sfc', 3145728, 'AD2CBF9C', 0, 0)",
                    [snes],
                )
                .unwrap();
                conn.last_insert_rowid()
            };

            let client = reqwest::Client::new();
            let credentials = ScreenScraperCredentials::from_env()
                .expect(".env turėtų turėti SCREENSCRAPER_DEV_ID/DEV_PASSWORD");
            let limiter = rate_limit::RateLimiter::new();
            let media_dir = std::env::temp_dir()
                .join(format!("nullbyte_scrape_single_test_{}", std::process::id()));
            let ctx = ScrapeContext {
                client: &client,
                credentials: &credentials,
                limiter: &limiter,
                db: &db,
                media_dir: &media_dir,
            };

            let mut progress_events = Vec::new();
            let summary = scrape_single_game(&ctx, game_id, |p| progress_events.push(p))
                .await
                .unwrap();

            assert_eq!(summary.found, 1);
            assert_eq!(progress_events.len(), 1);
            assert_eq!(progress_events[0].status, "ok");
            assert_eq!(progress_events[0].current, 1);
            assert_eq!(progress_events[0].total, 1);

            let conn = db.lock().unwrap();
            let updated = games::get_game(&conn, game_id).unwrap().unwrap();
            eprintln!("atnaujintas žaidimas: {updated:?}");
            assert_eq!(updated.scrape_status, "ok");
            assert_eq!(updated.developer.as_deref(), Some("Nintendo"));
            assert!(updated.cover_path.is_some());
            assert_eq!(updated.title, "Super Metroid"); // NEPAKEISTA — žr. `apply_scrape_result` doc.

            drop(conn);
            std::fs::remove_dir_all(&media_dir).ok();
        });
    }

    /// REALUS integracinis testas — TIKRAS `library::scanner::scan()` + TIKRAS
    /// `scrape_pending_games()` prieš ~90 realių fixture ROM'ų
    /// (`crates/nullbyte-core/roms/{snes,megadrive,gba,psx}/`). MVP.md P6.4 acceptance:
    /// „50 žaidimų scraping'as baigiasi be klaidų" — TIESIOGINIS patikrinimas su TIKRA eile,
    /// ne ekstrapoliacija iš vieno žaidimo testo. `#[ignore]`: lėtas (dešimtys realių HTTP
    /// kvietimų — `maxthreads=1`, tad NUOSEKLIAI, kol pirmas atsakymas nepasako kitaip),
    /// priklauso nuo tinklo, realių `.env` kredencialų IR realių fixture ROM'ų disko.
    /// Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_scrape_library_processes_90_real_games -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_scrape_library_processes_90_real_games() {
        let roms_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core/roms");
        if !roms_dir.exists() {
            eprintln!("PRALEISTA: {roms_dir:?} neegzistuoja šioje mašinoje (fixture ROM'ai — žr. .gitignore)");
            return;
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let db = test_db();
            {
                let mut conn = db.lock().unwrap();
                conn.execute(
                    "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
                    [roms_dir.to_string_lossy().to_string()],
                )
                .unwrap();
                let scan_summary = crate::library::scanner::scan(&mut conn, |_| {}).unwrap();
                eprintln!("skenavimas: {scan_summary:?}");
                assert!(
                    scan_summary.added >= 50,
                    "tikėtasi bent 50 pridėtų žaidimų, gauta {}",
                    scan_summary.added
                );
            }

            let client = reqwest::Client::new();
            let credentials = ScreenScraperCredentials::from_env()
                .expect(".env turėtų turėti SCREENSCRAPER_DEV_ID/DEV_PASSWORD");
            let limiter = rate_limit::RateLimiter::new();
            let media_dir = std::env::temp_dir()
                .join(format!("nullbyte_scrape_library_test_{}", std::process::id()));
            let ctx = ScrapeContext {
                client: &client,
                credentials: &credentials,
                limiter: &limiter,
                db: &db,
                media_dir: &media_dir,
            };
            let cancel = CancellationToken::new();

            let mut events: Vec<ScrapeProgress> = Vec::new();
            let summary = scrape_pending_games(&ctx, None, &cancel, |p| events.push(p))
                .await
                .unwrap();

            eprintln!("scraping santrauka: {summary:?}");
            eprintln!("iš viso progreso pranešimų: {}", events.len());

            assert!(!summary.cancelled);
            assert_eq!(
                events.len() as u32,
                summary.found + summary.not_found + summary.errored
            );
            assert!(
                summary.found + summary.not_found >= 50,
                "tikėtasi bent 50 sėkmingai apdorotų (rasta+nerasta), gauta found={} notfound={} errored={}",
                summary.found,
                summary.not_found,
                summary.errored
            );

            std::fs::remove_dir_all(&media_dir).ok();
        });
    }
}
