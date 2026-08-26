//! `games`/`platforms`/... lentelių Rust atitikmenys — 1:1 su `migrations/001_initial.sql`
//! stulpeliais (CLAUDE.md §7.3: IPC ribą kertantys struct'ai turi `Serialize`/`Deserialize`
//! ir `rename_all = "camelCase"`). CRUD (P5.4 „Bibliotekos užklausos") dar neparašytas — šie
//! struct'ai kol kas naudojami tik testuose.

#![allow(dead_code)] // pilnai išnaudos P5.2-P5.4 (hash'avimas, skeneris, užklausos)

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub screenscraper_id: Option<i64>,
    /// Kableliais atskirti plėtiniai, be taško (pvz. `"sfc,smc,fig"`).
    pub extensions: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub platform_id: i64,
    pub title: String,
    /// Be `"The "` prefikso, mažosiomis raidėmis — rikiavimui (P5.3 skeneris).
    pub sort_title: String,
    /// ABSOLIUTUS kelias — SKIRTINGAI nuo media cache (CLAUDE.md §9.4), ROM katalogai gali
    /// būti bet kur diske ir jų gali būti keli vienu metu, tad santykinis kelias būtų
    /// dviprasmiškas be papildomo JOIN'o į `rom_directories` (žr. `library::scanner` doc).
    pub rom_path: String,
    pub rom_size: i64,
    /// Failas archyve (`.zip`/`.7z`), jei ROM'as suarchyvuotas.
    pub archive_inner: Option<String>,
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<i64>,
    pub release_date: Option<String>,
    pub rating: Option<f64>,
    pub region: Option<String>,
    pub cover_path: Option<String>,
    /// Tikri viršelio matmenys (ADR-021) — `None`, kol dar nenuskaityti (žr.
    /// `scraper::image_dimensions`). Naudojama `GameGrid` „packed row" layout'ui, kad
    /// skirtingų proporcijų viršeliai (PSX kvadratas vs SNES platus vs Genesis aukštas)
    /// nebūtų apkerpami vienoda 3:4 dėže.
    pub cover_width: Option<i64>,
    pub cover_height: Option<i64>,
    pub screenshot_path: Option<String>,
    pub wheel_path: Option<String>,
    pub video_path: Option<String>,
    /// `"pending" | "ok" | "notfound" | "error"` (MVP.md P5.1 schema komentaras).
    pub scrape_status: String,
    pub scraped_at: Option<i64>,
    pub last_played: Option<i64>,
    pub play_count: i64,
    pub play_time_seconds: i64,
    pub favorite: bool,
    pub added_at: i64,
    /// ROM failo `mtime` pakartotinio skenavimo metu — pakeitus failą, `file_mtime`
    /// nesutaps, tad P5.3 skeneris žinos, kad reikia perskaityti hash'us iš naujo.
    pub file_mtime: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveState {
    pub id: i64,
    pub game_id: i64,
    /// `0` = quick save (P4.4 hotkey'ų konvencija), `1..=4` — numeruoti slot'ai.
    pub slot: i64,
    pub path: String,
    pub thumb_path: Option<String>,
    pub core_name: String,
    pub core_version: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRecord {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub version: Option<String>,
    pub extensions: String,
    pub need_fullpath: bool,
    pub last_seen: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCorePref {
    pub platform_id: i64,
    pub core_id: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RomDirectory {
    pub id: i64,
    pub path: String,
    pub recursive: bool,
    pub enabled: bool,
    /// `None` = automatinis platformos nustatymas pagal plėtinį per skenavimą (senas
    /// elgesys). `Some` — vartotojo eksplicitiškai nurodyta platforma šiam katalogui,
    /// pašalinanti dviprasmiškumą tarp platformų, dalinančių tuos pačius archyvo vidinius
    /// plėtinius (PSX/Saturn/SegaCD, žr. MVP.md ADR-020).
    pub platform_id: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeCacheEntry {
    /// `"crc:ABCD1234"` arba `"name:snes:Super Mario"` (MVP.md P5.1 schema komentaras).
    pub hash_key: String,
    /// `None`, jei `found == false` (P6.2 „Sėkmingi ir nesėkmingi rezultatai cache'uojami").
    pub response: Option<String>,
    pub found: bool,
    pub fetched_at: i64,
}
