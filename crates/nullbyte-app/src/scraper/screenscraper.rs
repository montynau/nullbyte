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
use crate::scraper::types::{Genre, Jeu, JeuInfosResponse, LangText, RegionText};

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
#[derive(Debug)]
pub enum ScrapeOutcome {
    Found(GameMetadata),
    NotFound,
}

/// Švarūs, jau ištraukti (regiono/kalbos prioritetai pritaikyti) metaduomenys — tiesiogiai
/// atitinka `games` lentelės stulpelius (P5.1 schema), paruošti P6.4 DB rašymui.
#[derive(Debug, Clone, PartialEq)]
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
}

/// Ieško žaidimo metaduomenų ScreenScraper'yje. Vienas HTTP kvietimas su visais turimais
/// identifikatoriais (žr. modulio doc dėl strategijos).
///
/// **404 = NotFound, NE klaida.** API grąžina HTTP 404 su PAPRASTU TEKSTU (ne JSON) ROM'o
/// nesuradus — patikrinta realiu užklausimu (2026-08-25). Tai tikrinama PRIEŠ bandant JSON
/// parsinimą, kitaip kiekvienas „nerasta" atvejis būtų klaidingai traktuojamas kaip sugadintas
/// atsakymas.
pub async fn lookup_game(
    client: &reqwest::Client,
    credentials: &ScreenScraperCredentials,
    rom: &RomIdentity<'_>,
) -> Result<ScrapeOutcome, AppError> {
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

    let response = client.get(API_URL).query(&query).send().await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ScrapeOutcome::NotFound);
    }
    if !response.status().is_success() {
        return Err(AppError::Other(format!(
            "ScreenScraper HTTP {}",
            response.status()
        )));
    }

    let body = response.text().await?;
    parse_response(&body)
}

/// Atskirta nuo `lookup_game` tam, kad būtų testuojama be tinklo (žr. `tests` modulį).
fn parse_response(body: &str) -> Result<ScrapeOutcome, AppError> {
    let parsed: JeuInfosResponse = serde_json::from_str(body).map_err(|error| {
        AppError::Other(format!("ScreenScraper JSON parsinimo klaida: {error}"))
    })?;

    let Some(response_body) = parsed.response else {
        return Ok(ScrapeOutcome::NotFound);
    };

    Ok(ScrapeOutcome::Found(extract_metadata(&response_body.jeu)))
}

fn pick_region_text(items: &[RegionText]) -> Option<String> {
    for region in REGION_PRIORITY {
        if let Some(item) = items.iter().find(|i| i.region == *region) {
            return Some(item.text.clone());
        }
    }
    items.first().map(|i| i.text.clone())
}

fn pick_lang_text(items: &[LangText]) -> Option<String> {
    for lang in LANG_PRIORITY {
        if let Some(item) = items.iter().find(|i| i.langue == *lang) {
            return Some(item.text.clone());
        }
    }
    items.first().map(|i| i.text.clone())
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
        developer: jeu.developpeur.as_ref().map(|d| d.text.clone()),
        publisher: jeu.editeur.as_ref().map(|e| e.text.clone()),
        genre: pick_genre(&genres),
        players: jeu.joueurs.as_ref().and_then(|j| j.text.parse().ok()),
        release_date: pick_region_text(&dates),
        rating: jeu.note.as_ref().and_then(|n| n.text.parse().ok()),
        region: pick_available_region(&noms),
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
            }
        }
    }"#;

    #[test]
    fn parses_real_snes_response_shape_and_extracts_correct_metadata() {
        let outcome = parse_response(REAL_SNES_RESPONSE).unwrap();
        let ScrapeOutcome::Found(metadata) = outcome else {
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
    }

    /// MVP.md P6.1 acceptance: „Blogas JSON nesulaužo (graceful degradation)" — grąžina
    /// `Err`, NE panic'ina.
    #[test]
    fn malformed_json_returns_err_not_panic() {
        let result = parse_response("{ šitas JSON sąmoningai sugadintas");
        assert!(result.is_err());
    }

    #[test]
    fn valid_json_missing_response_key_is_not_found() {
        let result = parse_response(r#"{"header": {"success": "false", "error": "kažkas"}}"#);
        assert!(matches!(result.unwrap(), ScrapeOutcome::NotFound));
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
        let outcome = parse_response(json).unwrap();
        let ScrapeOutcome::Found(metadata) = outcome else {
            panic!("tikėtasi Found");
        };
        assert_eq!(metadata.title, "Solo Game");
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

            let outcome = lookup_game(&client, &credentials, &rom).await.unwrap();
            let ScrapeOutcome::Found(metadata) = outcome else {
                panic!("tikėtasi Found realiam Super Metroid CRC'ui");
            };
            eprintln!("gauta: {metadata:?}");
            assert_eq!(metadata.title, "Super Metroid");
            assert_eq!(metadata.developer.as_deref(), Some("Nintendo"));
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

            let outcome = lookup_game(&client, &credentials, &rom).await.unwrap();
            assert!(matches!(outcome, ScrapeOutcome::NotFound));
        });
    }
}
