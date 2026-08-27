//! ScreenScraper API klientas (MVP.md P6.1) — `jeuInfos.php` užklausa, JSON parsinimas,
//! metaduomenų ištraukimas su regiono/kalbos prioritetais (CLAUDE.md §9.2).
//!
//! **Strategija (CLAUDE.md §9.1):** dokumentas aprašo tai kaip du žingsnius („pirma hash'ais,
//! jei nerado — pavadinimu"), bet REALUS API elgesys (patikrinta gyvu užklausimu, 2026-08-25)
//! priima VISUS turimus identifikatorius (`crc`/`md5`/`sha1`/`romtaille`/`romnom`/`systemeid`)
//! VIENOJE užklausoje ir pats bando hash'us PIRMIAU, o pavadinimą — jei hash'ai nerado.
//! Todėl `lookup_game()` siunčia visus turimus laukus vienu HTTP kvietimu, ne dviem — nereikia
//! kliento pusėje kartoti serverio jau atliekamos logikos.

#![allow(dead_code)] // pilnai išnaudos P6.2/P6.4 (rate limiting, scraping orkestracija)

use crate::error::AppError;
use crate::scraper::types::{
    Genre, Jeu, JeuInfosBody, JeuInfosResponse, LangText, Media, RegionText,
};

const API_URL: &str = "https://www.screenscraper.fr/api2/jeuInfos.php";

/// Regionų prioritetas pavadinimui/datai (CLAUDE.md §9.2).
const REGION_PRIORITY: &[&str] = &["wor", "eu", "us", "jp", "ss"];
/// Kalbų prioritetas aprašymui/žanrui (CLAUDE.md §9.2: „en → lt (jei bus) → pirmas prieinamas").
const LANG_PRIORITY: &[&str] = &["en", "lt"];

/// ScreenScraper kredencialai — TIK iš `.env`/aplinkos kintamųjų, niekada hardkodinti
/// (MVP.md P6.1 acceptance). `ssid`/`sspassword` neprivalomi (CLAUDE.md §9.3: be jų —
/// „labai maža kvota", ne klaida).
#[derive(Debug, Clone)]
pub struct ScreenScraperCredentials {
    pub devid: String,
    pub devpassword: String,
    pub ssid: Option<String>,
    pub sspassword: Option<String>,
}

impl ScreenScraperCredentials {
    /// Užkrauna `.env` (jei yra — production build'e jo gali nebūti, tada pasikliaujama
    /// tikru proceso environment'u, pvz. per OS nustatymus) ir skaito kredencialus.
    pub fn from_env() -> Result<Self, AppError> {
        let _ = dotenvy::dotenv(); // Tyliai ignoruoja, jei .env nerastas — NE klaida.

        let devid = std::env::var("SCREENSCRAPER_DEV_ID").map_err(|_| {
            AppError::Other("SCREENSCRAPER_DEV_ID nenustatytas (.env arba aplinka)".to_string())
        })?;
        let devpassword = std::env::var("SCREENSCRAPER_DEV_PASSWORD").map_err(|_| {
            AppError::Other(
                "SCREENSCRAPER_DEV_PASSWORD nenustatytas (.env arba aplinka)".to_string(),
            )
        })?;

        Ok(Self {
            devid,
            devpassword,
            ssid: std::env::var("SCREENSCRAPER_SSID").ok(),
            sspassword: std::env::var("SCREENSCRAPER_SSPASSWORD").ok(),
        })
    }

    /// Raktai, po kuriais UI redaguoti kredencialai laikomi `settings` lentelėje (P7.6
    /// Scraper panelė) — vieša, kad `commands::scraper` galėtų juos naudoti rašydama, nesant
    /// šio modulio viduje dviejų nesinchronizuotų raktų vardų sąrašų.
    pub const KEY_DEV_ID: &'static str = "scraper.dev_id";
    pub const KEY_DEV_PASSWORD: &'static str = "scraper.dev_password";
    pub const KEY_SSID: &'static str = "scraper.ssid";
    pub const KEY_SSPASSWORD: &'static str = "scraper.sspassword";

    /// Kaip [`Self::from_env`], bet `settings` lentelėje (P7.6 UI) įrašytos reikšmės TURI
    /// PIRMENYBĘ prieš `.env` — vartotojas redaguoja per Settings ekraną, ne failą, tad jo
    /// paskutinis veiksmas laimi. Tuščia (arba nesanti) DB reikšmė krinta atgal į `.env`.
    /// Klaida grąžinama TIK jei nei DB, nei `.env` neturi PRIVALOMŲ `devid`/`devpassword`.
    pub fn load(conn: &rusqlite::Connection) -> Result<Self, AppError> {
        use crate::db::settings;

        let env = Self::from_env().ok();

        let db_devid = settings::get(conn, Self::KEY_DEV_ID)?.filter(|s| !s.is_empty());
        let db_devpassword = settings::get(conn, Self::KEY_DEV_PASSWORD)?.filter(|s| !s.is_empty());
        let db_ssid = settings::get(conn, Self::KEY_SSID)?.filter(|s| !s.is_empty());
        let db_sspassword = settings::get(conn, Self::KEY_SSPASSWORD)?.filter(|s| !s.is_empty());

        let devid = db_devid.or_else(|| env.as_ref().map(|c| c.devid.clone()));
        let devpassword = db_devpassword.or_else(|| env.as_ref().map(|c| c.devpassword.clone()));

        let (Some(devid), Some(devpassword)) = (devid, devpassword) else {
            return Err(AppError::Other(
                "ScreenScraper dev credentials nesukonfigūruoti (Settings arba .env)".to_string(),
            ));
        };

        Ok(Self {
            devid,
            devpassword,
            ssid: db_ssid.or_else(|| env.as_ref().and_then(|c| c.ssid.clone())),
            sspassword: db_sspassword.or_else(|| env.as_ref().and_then(|c| c.sspassword.clone())),
        })
    }
}

/// ROM'o identifikuojanti informacija, siunčiama ScreenScraper'iui — paprastai iš
/// `library::hasher::RomHashes` (P5.2) + failo vardo + `platforms.screenscraper_id` (P5.1).
#[derive(Debug, Clone)]
pub struct RomIdentity<'a> {
    pub crc32: Option<&'a str>,
    pub md5: Option<&'a str>,
    pub sha1: Option<&'a str>,
    pub size: Option<u64>,
    pub filename: &'a str,
    pub systemeid: i64,
}

/// Paieškos rezultatas — `NotFound` yra TEISĖTAS, ne klaida (MVP.md P6.1 acceptance
/// „Nežinomas ROM'as → NotFound, ne klaida"). Skiriasi nuo `Err(AppError)`, kuris reiškia,
/// kad UŽKLAUSA/ATSAKYMAS buvo sugadintas — semantiškai skirtingi atvejai kviečiančiajai
/// pusei (P6.4): NotFound → pažymėk `scrape_status='notfound'` ir cache'uok; Err → pakartok
/// vėliau arba loginink kaip realią problemą.
// `GameMetadata` (su P6.4 pridėtu `medias: Vec<Media>`) yra ~224 baitų — clippy siūlo
// `Box<GameMetadata>`, bet `ScrapeOutcome` sukuriamas VIENĄ kartą per žaidimo scraping'ą
// (ne per kadrą/hot path'e), tad 224 baitų steko skirtumas praktiškai nesvarbus; boxinimas
// tik pridėtų netiesioginumo visose `match`/konstrukcijos vietose be realios naudos.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ScrapeOutcome {
    Found(GameMetadata),
    NotFound,
}

/// Švarūs, jau ištraukti (regiono/kalbos prioritetai pritaikyti) metaduomenys — tiesiogiai
/// atitinka `games` lentelės stulpelius (P5.1 schema), paruošti P6.4 DB rašymui.
///
/// `Serialize`/`Deserialize` (nuo P6.2) — šis tipas keliauja per `scrape_cache.response`
/// stulpelį kaip JSON tekstas (žr. `rate_limit::write_cache`/`read_cache`), ne tik per API.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GameMetadata {
    pub title: String,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<i64>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
    pub region: Option<String>,
    /// Žaliaviniai media įrašai (P6.4) — `scraper::media::download_game_media` juos vėliau
    /// pasirenka/atsisiunčia. Laikomi ČIA (ne atskirai grąžinami iš `lookup_game`), kad
    /// cache'uotas (`scrape_cache`) atsakymas galėtų pakartotinai atsisiųsti media BE naujo
    /// gyvo API kvietimo — žr. `types.rs::Media` doc.
    pub medias: Vec<Media>,
}

/// Vartotojo kvota, ištraukta iš `response.ssuser`/`response.serveurs` (kiekviename
/// sėkmingame `jeuInfos.php` atsakyme, žr. `types.rs` modulio doc) — MVP.md P6.2 „Kvotos
/// likutis... → rodyk UI" ir „Semaforas pagal ssuser.maxthreads".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuotaInfo {
    pub maxthreads: u32,
    pub requests_today: u64,
    pub max_requests_per_day: u64,
    pub closed_for_nonmember: bool,
    pub closed_for_leecher: bool,
}

/// Sėkmingo `lookup_game` rezultatas — `quota` `None`, jei atsakymas (retai) neturėjo
/// `ssuser` bloko (žr. `types.rs`: laukas apsaugotas `Option`, nes NĖRA garantuotas API
/// kontraktu, tik PASTEBĖTAS visuose šios sesijos realiuose atsakymuose).
#[derive(Debug)]
pub struct LookupSuccess {
    pub outcome: ScrapeOutcome,
    pub quota: Option<QuotaInfo>,
}

/// Skiria „reikia palaukti ir bandyti vėl" nuo „tikra klaida" — MVP.md P6.2 acceptance
/// reikalauja atskiro elgesio (exponential backoff vs. tiesiog `Err`).
#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error("ScreenScraper kvota viršyta arba API laikinai uždaryta (HTTP {status})")]
    RateLimited { status: reqwest::StatusCode },
    #[error(transparent)]
    Failed(#[from] AppError),
}

/// Ieško žaidimo metaduomenų ScreenScraper'yje. Vienas HTTP kvietimas su visais turimais
/// identifikatoriais (žr. modulio doc dėl strategijos).
///
/// **404 = NotFound, NE klaida.** API grąžina HTTP 404 su PAPRASTU TEKSTU (ne JSON) ROM'o
/// nesuradus — patikrinta realiu užklausimu (2026-08-25). Tai tikrinama PRIEŠ bandant JSON
/// parsinimą, kitaip kiekvienas „nerasta" atvejis būtų klaidingai traktuojamas kaip sugadintas
/// atsakymas.
///
/// **429/430 = rate limit** (MVP.md P6.2 „Ką daryti": „429/430/`API closed`"). **NEVERIFIKUOTA
/// gyvu atsakymu šią sesiją** — realaus limito pasiekimas reikalautų sąmoningai išeikvoti
/// vartotojo dienos kvotą, kas nebūtų atsakinga naudoti dev kredencialus, tad ši šaka remiasi
/// TIK MVP.md specifikacijos tekstu, ne stebėtu API elgesiu (skirtingai nuo likusios šio
/// failo logikos — žr. modulio doc apie kitas, TIKRAI patikrintas ypatybes).
pub async fn lookup_game(
    client: &reqwest::Client,
    credentials: &ScreenScraperCredentials,
    rom: &RomIdentity<'_>,
) -> Result<LookupSuccess, LookupError> {
    let mut query: Vec<(&str, String)> = vec![
        ("devid", credentials.devid.clone()),
        ("devpassword", credentials.devpassword.clone()),
        (
            "softname",
            format!("Nullbyte-{}", env!("CARGO_PKG_VERSION")),
        ),
        ("output", "json".to_string()),
        ("romnom", rom.filename.to_string()),
        ("systemeid", rom.systemeid.to_string()),
    ];
    if let Some(ssid) = &credentials.ssid {
        query.push(("ssid", ssid.clone()));
    }
    if let Some(sspassword) = &credentials.sspassword {
        query.push(("sspassword", sspassword.clone()));
    }
    if let Some(crc) = rom.crc32 {
        query.push(("crc", crc.to_string()));
    }
    if let Some(md5) = rom.md5 {
        query.push(("md5", md5.to_string()));
    }
    if let Some(sha1) = rom.sha1 {
        query.push(("sha1", sha1.to_string()));
    }
    if let Some(size) = rom.size {
        query.push(("romtaille", size.to_string()));
    }

    let response = client
        .get(API_URL)
        .query(&query)
        .send()
        .await
        .map_err(AppError::from)?;
    let status = response.status();

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.as_u16() == 430 {
        return Err(LookupError::RateLimited { status });
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(LookupSuccess {
            outcome: ScrapeOutcome::NotFound,
            quota: None,
        });
    }
    if !status.is_success() {
        return Err(LookupError::Failed(AppError::Other(format!(
            "ScreenScraper HTTP {status}"
        ))));
    }

    let body = response.text().await.map_err(AppError::from)?;
    parse_response(&body, status)
}

/// Atskirta nuo `lookup_game` tam, kad būtų testuojama be tinklo (žr. `tests` modulį).
/// `status` reikalingas TIK „API closed" tekstinio atsakymo šakai (žr. `lookup_game` doc) —
/// ten HTTP kodas realiai yra 200, bet MVP.md traktuoja tai kaip rate-limit signalą.
fn parse_response(body: &str, status: reqwest::StatusCode) -> Result<LookupSuccess, LookupError> {
    let parsed: JeuInfosResponse = match serde_json::from_str(body) {
        Ok(parsed) => parsed,
        Err(error) => {
            if body.to_lowercase().contains("api closed") {
                return Err(LookupError::RateLimited { status });
            }
            return Err(LookupError::Failed(AppError::Other(format!(
                "ScreenScraper JSON parsinimo klaida: {error}"
            ))));
        }
    };

    let Some(response_body) = parsed.response else {
        return Ok(LookupSuccess {
            outcome: ScrapeOutcome::NotFound,
            quota: None,
        });
    };

    let quota = extract_quota(&response_body);
    Ok(LookupSuccess {
        outcome: ScrapeOutcome::Found(extract_metadata(&response_body.jeu)),
        quota,
    })
}

/// `None`, jei atsakymas neturėjo `ssuser` bloko — apsauga, ne spėjama numatytoji reikšmė
/// (žr. `QuotaInfo` doc).
fn extract_quota(body: &JeuInfosBody) -> Option<QuotaInfo> {
    let ssuser = body.ssuser.as_ref()?;
    let serveurs = body.serveurs.as_ref();

    Some(QuotaInfo {
        maxthreads: ssuser.maxthreads.parse().unwrap_or(1),
        requests_today: ssuser.requeststoday.parse().unwrap_or(0),
        max_requests_per_day: ssuser.maxrequestsperday.parse().unwrap_or(0),
        closed_for_nonmember: serveurs.is_some_and(|s| s.closefornomember == "1"),
        closed_for_leecher: serveurs.is_some_and(|s| s.closeforleecher == "1"),
    })
}

/// ScreenScraper API grąžina laisvo teksto laukus (aprašymus, žanrus, kūrėją/leidėją) JAU
/// HTML-escape'intus (pvz. `&quot;The Master&quot;` vietoj `"The Master"`) — REALIAI
/// pastebėta P9.6 galutinio patikrinimo metu (ActRaiser aprašymas rodė `&quot;` tiesiogiai
/// UI, ne kabutės ženklą). Svelte `{expr}` teisingai apsaugo NUO XSS, bet NEIŠVYNIOJA jau
/// esamų entity'ų atgal — tad tai turi būti padaryta ČIA, gaunant duomenis, ne UI pusėje.
/// Apima standartinius pavadintus entity'us IR skaitmeninius (`&#39;`/`&#x27;`), nes abu
/// realiai pasitaiko API atsakymuose.
fn unescape_html_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp_pos) = rest.find('&') {
        result.push_str(&rest[..amp_pos]);
        let after_amp = &rest[amp_pos + 1..];
        let Some(semi_pos) = after_amp.find(';').filter(|&p| p <= 10) else {
            result.push('&');
            rest = after_amp;
            continue;
        };
        let entity = &after_amp[..semi_pos];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" | "#x27" | "#X27" => Some('\''),
            _ if entity.starts_with('#') => entity[1..]
                .strip_prefix(['x', 'X'])
                .map_or_else(
                    || entity[1..].parse::<u32>().ok(),
                    |hex| u32::from_str_radix(hex, 16).ok(),
                )
                .and_then(char::from_u32),
            _ => None,
        };
        match decoded {
            Some(c) => {
                result.push(c);
                rest = &after_amp[semi_pos + 1..];
            }
            None => {
                // Nepavyko atpažinti kaip entity'o — palik '&' kaip yra, tęsk NUO jo, kad
                // netyčia nepraleistum kito, teisingo entity'o iškart po jo.
                result.push('&');
                rest = after_amp;
            }
        }
    }
    result.push_str(rest);
    result
}

fn pick_region_text(items: &[RegionText]) -> Option<String> {
    for region in REGION_PRIORITY {
        if let Some(item) = items.iter().find(|i| i.region == *region) {
            return Some(unescape_html_entities(&item.text));
        }
    }
    items.first().map(|i| unescape_html_entities(&i.text))
}

fn pick_lang_text(items: &[LangText]) -> Option<String> {
    for lang in LANG_PRIORITY {
        if let Some(item) = items.iter().find(|i| i.langue == *lang) {
            return Some(unescape_html_entities(&item.text));
        }
    }
    items.first().map(|i| unescape_html_entities(&i.text))
}

fn pick_genre(genres: &[Genre]) -> Option<String> {
    let first = genres.first()?;
    pick_lang_text(&first.noms.clone().into_vec())
}

fn extract_metadata(jeu: &Jeu) -> GameMetadata {
    let noms = jeu.noms.clone().into_vec();
    let dates = jeu.dates.clone().into_vec();
    let genres = jeu.genres.clone().into_vec();
    let synopsis = jeu.synopsis.clone().into_vec();

    GameMetadata {
        title: pick_region_text(&noms).unwrap_or_default(),
        description: pick_lang_text(&synopsis),
        developer: jeu
            .developpeur
            .as_ref()
            .map(|d| unescape_html_entities(&d.text)),
        publisher: jeu
            .editeur
            .as_ref()
            .map(|e| unescape_html_entities(&e.text)),
        genre: pick_genre(&genres),
        players: jeu.joueurs.as_ref().and_then(|j| j.text.parse().ok()),
        release_date: pick_region_text(&dates),
        rating: jeu.note.as_ref().and_then(|n| n.text.parse().ok()),
        region: pick_available_region(&noms),
        medias: jeu.medias.clone().into_vec(),
    }
}

/// Grąžina AUKŠČIAUSIO prioriteto regioną, kuris REALIAI yra `noms` sąraše — SKIRTINGAI nuo
/// naivaus `noms.iter().find(...)`, kuris grąžintų PIRMĄ JSON masyve esantį regioną,
/// nepriklausomai nuo prioriteto (realus radinys: `noms` API atsakyme atėjo `[ss, us, jp,
/// eu]` tvarka — naivus variantas grąžintų „ss", nors „eu" turėtų laimėti pagal CLAUDE.md
/// §9.2 prioritetą). Iteruojam PER PRIORITETO sąrašą, ne per `noms`.
fn pick_available_region(noms: &[RegionText]) -> Option<String> {
    for region in REGION_PRIORITY {
        if noms.iter().any(|n| n.region == *region) {
            return Some((*region).to_string());
        }
    }
    noms.first().map(|n| n.region.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rust testai VIENAME binare paleidžiami LYGIAGREČIAI — trys testai šiame modulyje
    /// mutuoja tuos pačius PROCESO GLOBALIUS `SCREENSCRAPER_*` env kintamuosius
    /// (`credentials_from_env_reads_required_and_optional_fields`,
    /// `load_prefers_settings_table_over_env`, `load_falls_back_to_env_when_settings_table_empty`).
    /// Be šio lock'o jie realiai lenktyniauja (pastebėta: vieno testo `set_var` „laimėdavo"
    /// prieš kito `assert`) — šis `Mutex` priverčia juos vykdytis nuosekliai vienas po kito.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// P9.6 galutinio patikrinimo metu pastebėtas REALUS bug'as: ActRaiser aprašymas
    /// (tikras ScreenScraper atsakymas) turėjo `&quot;The Master&quot;`, kurį UI rodė
    /// TIESIOGINIU tekstu, ne kabučių ženklu.
    #[test]
    fn unescape_html_entities_decodes_real_actraiser_case() {
        let input = r#"You play &quot;The Master&quot;, and must save the world."#;
        let expected = r#"You play "The Master", and must save the world."#;
        assert_eq!(unescape_html_entities(input), expected);
    }

    #[test]
    fn unescape_html_entities_handles_all_standard_named_entities() {
        assert_eq!(unescape_html_entities("Q&amp;A"), "Q&A");
        assert_eq!(unescape_html_entities("a &lt; b &gt; c"), "a < b > c");
        assert_eq!(unescape_html_entities("it&apos;s"), "it's");
        assert_eq!(unescape_html_entities("it&#39;s"), "it's");
        assert_eq!(unescape_html_entities("it&#x27;s"), "it's");
    }

    #[test]
    fn unescape_html_entities_leaves_bare_ampersand_untouched() {
        // "R&D" — jokio ';' po '&' pakankamai arti, kad tai būtų entity — turi likti kaip yra.
        assert_eq!(unescape_html_entities("R&D Games"), "R&D Games");
    }

    #[test]
    fn unescape_html_entities_leaves_unknown_entity_untouched() {
        assert_eq!(unescape_html_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn unescape_html_entities_handles_empty_and_no_entities() {
        assert_eq!(unescape_html_entities(""), "");
        assert_eq!(unescape_html_entities("plain text"), "plain text");
    }

    /// Sutrumpinta, kredencialų NETURINTI versija REALAUS ScreenScraper `jeuInfos.php`
    /// atsakymo (Super Metroid, SNES, `crc=AD2CBF9C`, gauta 2026-08-25) — laukų PAVADINIMAI
    /// ir FORMA (`noms`/`dates` masyvai, `developpeur`/`editeur` pavieniai objektai ir t.t.)
    /// tikslūs, tik `medias`/`roms` sutrumpinti (nereikalingi šiam testui), o visi `url`
    /// laukai pakeisti placeholder'iais (originalūs turėjo devid/devpassword/ssid/sspassword
    /// atviru tekstu — žr. `types.rs` modulio doc dėl šios API ypatybės).
    const REAL_SNES_RESPONSE: &str = r#"{
        "header": {"success": "true", "error": ""},
        "response": {
            "jeu": {
                "id": "2147",
                "noms": [
                    {"region": "ss", "text": "Super Metroid"},
                    {"region": "us", "text": "Super Metroid"},
                    {"region": "jp", "text": "Super Metroid"},
                    {"region": "eu", "text": "Super Metroid"}
                ],
                "synopsis": [
                    {"langue": "en", "text": "Join forces with the power of Samus..."},
                    {"langue": "fr", "text": "Associez-vous avec la puissance de Samus..."}
                ],
                "developpeur": {"id": "286", "text": "Nintendo"},
                "editeur": {"id": "286", "text": "Nintendo"},
                "joueurs": {"text": "1"},
                "note": {"text": "16"},
                "dates": [
                    {"region": "us", "text": "1994-04-18"},
                    {"region": "jp", "text": "1994-03-19"},
                    {"region": "eu", "text": "1994-07-28"}
                ],
                "genres": [
                    {
                        "id": "7",
                        "noms": [
                            {"langue": "en", "text": "Platform"},
                            {"langue": "fr", "text": "Plateforme"}
                        ]
                    }
                ],
                "medias": [
                    {"type": "box-2D", "region": "us", "url": "https://example.invalid/box.png", "format": "png"}
                ],
                "rom": {
                    "romcrc": "AD2CBF9C",
                    "rommd5": "3D64F89499A403D17D530388854A7DA5",
                    "romfilename": "Super Metroid (Europe) (En,Fr,De).sfc"
                }
            },
            "serveurs": {"closefornomember": "0", "closeforleecher": "0"},
            "ssuser": {"maxthreads": "1", "requeststoday": "0", "maxrequestsperday": "20000"}
        }
    }"#;

    #[test]
    fn parses_real_snes_response_shape_and_extracts_correct_metadata() {
        let success = parse_response(REAL_SNES_RESPONSE, reqwest::StatusCode::OK).unwrap();
        let ScrapeOutcome::Found(metadata) = success.outcome else {
            panic!("tikėtasi Found, gauta NotFound");
        };

        assert_eq!(metadata.title, "Super Metroid"); // "eu" prioritetas prieš "us"/"jp"/"ss"
        assert_eq!(metadata.developer.as_deref(), Some("Nintendo"));
        assert_eq!(metadata.publisher.as_deref(), Some("Nintendo"));
        assert_eq!(metadata.genre.as_deref(), Some("Platform"));
        assert_eq!(metadata.players, Some(1));
        assert_eq!(metadata.release_date.as_deref(), Some("1994-07-28")); // "eu" prioritetas
        assert_eq!(metadata.rating, Some(16.0));
        assert_eq!(metadata.region.as_deref(), Some("eu"));
        assert!(metadata.description.unwrap().starts_with("Join forces"));

        // P6.2: kvota ištraukta iš to PAČIO atsakymo (žr. `types.rs` modulio doc — `serveurs`/
        // `ssuser` yra kiekviename sėkmingame `jeuInfos.php` atsakyme, ne tik dedikuotame
        // kvotos endpoint'e).
        let quota = success.quota.unwrap();
        assert_eq!(quota.maxthreads, 1);
        assert_eq!(quota.requests_today, 0);
        assert_eq!(quota.max_requests_per_day, 20000);
        assert!(!quota.closed_for_nonmember);
        assert!(!quota.closed_for_leecher);

        // P6.4: `medias` perkeliami į `GameMetadata` be pakeitimų — `scraper::media` juos
        // vėliau pasirenka/atsisiunčia (žr. `GameMetadata` doc).
        assert_eq!(metadata.medias.len(), 1);
        assert_eq!(metadata.medias[0].media_type, "box-2D");
    }

    /// MVP.md P6.1 acceptance: „Blogas JSON nesulaužo (graceful degradation)" — grąžina
    /// `Err`, NE panic'ina.
    #[test]
    fn malformed_json_returns_err_not_panic() {
        let result = parse_response(
            "{ šitas JSON sąmoningai sugadintas",
            reqwest::StatusCode::OK,
        );
        assert!(result.is_err());
    }

    /// MVP.md P6.2: „429/430/API closed" → `LookupError::RateLimited`, ne bendra klaida —
    /// tikrina TEKSTINIO (ne JSON) „API closed" atsakymo šaką (žr. `lookup_game` doc dėl
    /// šio unverifikuoto elgesio statuso).
    #[test]
    fn api_closed_text_body_is_rate_limited_not_generic_error() {
        let result = parse_response("API closed for now", reqwest::StatusCode::OK);
        assert!(matches!(result, Err(LookupError::RateLimited { .. })));
    }

    #[test]
    fn valid_json_missing_response_key_is_not_found() {
        let result = parse_response(
            r#"{"header": {"success": "false", "error": "kažkas"}}"#,
            reqwest::StatusCode::OK,
        );
        assert!(matches!(result.unwrap().outcome, ScrapeOutcome::NotFound));
    }

    /// Vienas elementas — patikrina, kad `OneOrMany` priima IR pavienį objektą ten, kur
    /// realus atsakymas paprastai duoda masyvą (žr. `types.rs` modulio doc).
    #[test]
    fn one_or_many_accepts_single_object_where_array_is_typical() {
        let json = r#"{
            "header": {"success": "true", "error": ""},
            "response": {
                "jeu": {
                    "id": "1",
                    "noms": {"region": "us", "text": "Solo Game"},
                    "developpeur": {"id": "1", "text": "Dev"},
                    "editeur": {"id": "1", "text": "Pub"}
                }
            }
        }"#;
        let success = parse_response(json, reqwest::StatusCode::OK).unwrap();
        let ScrapeOutcome::Found(metadata) = success.outcome else {
            panic!("tikėtasi Found");
        };
        assert_eq!(metadata.title, "Solo Game");
        assert!(success.quota.is_none()); // atsakymas be `ssuser` bloko — None, ne spėta reikšmė.
    }

    #[test]
    fn region_priority_prefers_eu_over_us_jp_ss() {
        let items = vec![
            RegionText {
                region: "ss".into(),
                text: "ss-value".into(),
            },
            RegionText {
                region: "jp".into(),
                text: "jp-value".into(),
            },
            RegionText {
                region: "us".into(),
                text: "us-value".into(),
            },
            RegionText {
                region: "eu".into(),
                text: "eu-value".into(),
            },
        ];
        assert_eq!(pick_region_text(&items).as_deref(), Some("eu-value"));
    }

    #[test]
    fn credentials_from_env_reads_required_and_optional_fields() {
        let _guard = lock_env();
        // VISI keturi kintamieji nustatomi EKSPLICITIŠKAI (ne pašalinami) — `dotenvy::dotenv()`
        // viduje `from_env()` NEPERRAŠO jau nustatytų kintamųjų (standartinis dotenv elgesys),
        // bet TIKRAS šio repo `.env` failas (kurį `dotenv()` randa einant per tėvinius
        // katalogus nuo CWD) UŽPILDYTŲ bet kurį PALIKTĄ nenustatytą kintamąjį savo tikra
        // reikšme — tad testas, bandantis simuliuoti „nenustatyta" per `remove_var`, būtų
        // netikras/priklausomas nuo to, ar dev mašinoje yra `.env` (buvo realiai pagauta:
        // testas KRITO su tikra `.env` reikšme vietoj tikėtos `None`).
        std::env::set_var("SCREENSCRAPER_DEV_ID", "test-id");
        std::env::set_var("SCREENSCRAPER_DEV_PASSWORD", "test-pass");
        std::env::set_var("SCREENSCRAPER_SSID", "test-ssid");
        std::env::set_var("SCREENSCRAPER_SSPASSWORD", "test-sspass");

        let creds = ScreenScraperCredentials::from_env().unwrap();
        assert_eq!(creds.devid, "test-id");
        assert_eq!(creds.devpassword, "test-pass");
        assert_eq!(creds.ssid.as_deref(), Some("test-ssid"));
        assert_eq!(creds.sspassword.as_deref(), Some("test-sspass"));

        std::env::remove_var("SCREENSCRAPER_DEV_ID");
        std::env::remove_var("SCREENSCRAPER_DEV_PASSWORD");
        std::env::remove_var("SCREENSCRAPER_SSID");
        std::env::remove_var("SCREENSCRAPER_SSPASSWORD");
    }

    fn open_test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::MIGRATIONS
            .iter()
            .for_each(|(_, sql)| conn.execute_batch(sql).unwrap());
        conn
    }

    /// P7.6 Scraper panelė: UI įrašyti kredencialai (`settings` lentelė) TURI PIRMENYBĘ prieš
    /// `.env` (žr. `load` doc).
    #[test]
    fn load_prefers_settings_table_over_env() {
        let _guard = lock_env();
        std::env::set_var("SCREENSCRAPER_DEV_ID", "env-id");
        std::env::set_var("SCREENSCRAPER_DEV_PASSWORD", "env-pass");

        let conn = open_test_db();
        crate::db::settings::set(&conn, ScreenScraperCredentials::KEY_DEV_ID, "db-id").unwrap();
        crate::db::settings::set(&conn, ScreenScraperCredentials::KEY_DEV_PASSWORD, "db-pass")
            .unwrap();

        let creds = ScreenScraperCredentials::load(&conn).unwrap();
        assert_eq!(creds.devid, "db-id");
        assert_eq!(creds.devpassword, "db-pass");

        std::env::remove_var("SCREENSCRAPER_DEV_ID");
        std::env::remove_var("SCREENSCRAPER_DEV_PASSWORD");
    }

    /// Kai `settings` lentelėje nieko nėra — `load` krenta atgal į `.env`, elgiasi kaip
    /// `from_env`.
    #[test]
    fn load_falls_back_to_env_when_settings_table_empty() {
        let _guard = lock_env();
        std::env::set_var("SCREENSCRAPER_DEV_ID", "env-only-id");
        std::env::set_var("SCREENSCRAPER_DEV_PASSWORD", "env-only-pass");

        let conn = open_test_db();

        let creds = ScreenScraperCredentials::load(&conn).unwrap();
        assert_eq!(creds.devid, "env-only-id");
        assert_eq!(creds.devpassword, "env-only-pass");

        std::env::remove_var("SCREENSCRAPER_DEV_ID");
        std::env::remove_var("SCREENSCRAPER_DEV_PASSWORD");
    }

    /// P6.1 acceptance: „Žinomas SNES ROM'as randa teisingus metaduomenis" — REALUS tinklo
    /// kvietimas, tikri `.env` kredencialai. `#[ignore]`: priklauso nuo tinklo IR realaus
    /// `nullbyte-core/roms/snes/Super Metroid.sfc` fixture'o. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_snes_rom_finds_metadata -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_snes_rom_finds_metadata() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let credentials = ScreenScraperCredentials::from_env()
                .expect(".env turėtų turėti SCREENSCRAPER_DEV_ID/DEV_PASSWORD");
            let client = reqwest::Client::new();

            let rom = RomIdentity {
                crc32: Some("AD2CBF9C"),
                md5: None,
                sha1: None,
                size: Some(3_145_728),
                filename: "Super Metroid.sfc",
                systemeid: 4, // SNES (P5.1 seed, patikrinta)
            };

            let success = lookup_game(&client, &credentials, &rom).await.unwrap();
            let ScrapeOutcome::Found(metadata) = success.outcome else {
                panic!("tikėtasi Found realiam Super Metroid CRC'ui");
            };
            eprintln!("gauta: {metadata:?}");
            eprintln!("kvota: {:?}", success.quota);
            assert_eq!(metadata.title, "Super Metroid");
            assert_eq!(metadata.developer.as_deref(), Some("Nintendo"));
            assert!(success.quota.is_some()); // P6.2: realiame atsakyme visada yra `ssuser`.
        });
    }

    /// P6.1 acceptance: „Nežinomas ROM'as → NotFound, ne klaida" — REALUS tinklo kvietimas.
    /// `#[ignore]`: priklauso nuo tinklo. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_unknown_rom_is_not_found -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_unknown_rom_is_not_found() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let credentials = ScreenScraperCredentials::from_env().unwrap();
            let client = reqwest::Client::new();

            let rom = RomIdentity {
                crc32: Some("00000000"),
                md5: None,
                sha1: None,
                size: None,
                filename: "ThisRomDefinitelyDoesNotExist12345.sfc",
                systemeid: 4,
            };

            let success = lookup_game(&client, &credentials, &rom).await.unwrap();
            assert!(matches!(success.outcome, ScrapeOutcome::NotFound));
        });
    }
}
