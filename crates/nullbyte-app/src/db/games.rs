//! Bibliotekos užklausos — filtruotas žaidimų sąrašas (FTS5 paieška, platforma, favorite,
//! rūšiavimas, puslapiavimas), pavienio žaidimo CRUD, platformos su žaidimų kiekiu
//! (MVP.md P5.4).

use rusqlite::{params_from_iter, Connection, Row};

use crate::db::models::{Game, Platform};
use crate::error::AppError;

/// `list_games` filtras — visi laukai NEPRIVALOMI (išskyrus rūšiavimą/puslapiavimą, kurie
/// turi numatytąsias reikšmes per `Default`). CLAUDE.md §7.3: kerta Tauri IPC ribą.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFilter {
    pub platform_id: Option<i64>,
    /// Laisvo teksto paieška (FTS5, `title`+`description`) — `None`/tuščia reiškia „visi".
    pub search: Option<String>,
    #[serde(default)]
    pub favorites_only: bool,
    #[serde(default)]
    pub sort: SortField,
    #[serde(default)]
    pub sort_direction: SortDirection,
    #[serde(default = "default_page_size")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

impl Default for GameFilter {
    fn default() -> Self {
        Self {
            platform_id: None,
            search: None,
            favorites_only: false,
            sort: SortField::default(),
            sort_direction: SortDirection::default(),
            page: 0,
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

const DEFAULT_PAGE_SIZE: u32 = 50;
fn default_page_size() -> u32 {
    DEFAULT_PAGE_SIZE
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    #[default]
    Title,
    LastPlayed,
    AddedAt,
}

impl SortField {
    fn column(self) -> &'static str {
        match self {
            SortField::Title => "g.sort_title",
            SortField::LastPlayed => "g.last_played",
            SortField::AddedAt => "g.added_at",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    fn sql(self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// Platforma + kiek `games` eilučių jai priklauso — `#[serde(flatten)]`, kad JSON pusėje
/// `Platform` laukai ir `gameCount` atrodytų kaip vienas plokščias objektas.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSummary {
    #[serde(flatten)]
    pub platform: Platform,
    pub game_count: i64,
}

/// Visi `games` stulpeliai TEISINGA tvarka, atitinkančia [`game_from_row`] — vienas šaltinis
/// abiem SELECT sakiniams (`list_games`/`get_game`), kad nebūtų dviejų vietų, kurias reikia
/// sinchronizuoti pakeitus schemą.
const GAME_COLUMNS: &str = "g.id, g.platform_id, g.title, g.sort_title, g.rom_path, g.rom_size, \
     g.archive_inner, g.crc32, g.md5, g.sha1, g.description, g.developer, g.publisher, \
     g.genre, g.players, g.release_date, g.rating, g.region, g.cover_path, g.screenshot_path, \
     g.wheel_path, g.video_path, g.scrape_status, g.scraped_at, g.last_played, g.play_count, \
     g.play_time_seconds, g.favorite, g.added_at, g.file_mtime, g.cover_width, g.cover_height";

fn game_from_row(row: &Row) -> rusqlite::Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        platform_id: row.get(1)?,
        title: row.get(2)?,
        sort_title: row.get(3)?,
        rom_path: row.get(4)?,
        rom_size: row.get(5)?,
        archive_inner: row.get(6)?,
        crc32: row.get(7)?,
        md5: row.get(8)?,
        sha1: row.get(9)?,
        description: row.get(10)?,
        developer: row.get(11)?,
        publisher: row.get(12)?,
        genre: row.get(13)?,
        players: row.get(14)?,
        release_date: row.get(15)?,
        rating: row.get(16)?,
        region: row.get(17)?,
        cover_path: row.get(18)?,
        screenshot_path: row.get(19)?,
        wheel_path: row.get(20)?,
        video_path: row.get(21)?,
        scrape_status: row.get(22)?,
        scraped_at: row.get(23)?,
        last_played: row.get(24)?,
        play_count: row.get(25)?,
        play_time_seconds: row.get(26)?,
        favorite: row.get(27)?,
        added_at: row.get(28)?,
        file_mtime: row.get(29)?,
        cover_width: row.get(30)?,
        cover_height: row.get(31)?,
    })
}

/// Paverčia laisvo teksto paiešką į FTS5 `MATCH` sintaksę: kiekvienas žodis tampa prefikso
/// paieška (`mario*`), tarp žodžių numanomas AND. `None`, jei po išvalymo nelieka nieko
/// paieškotino (vien skyryba/tarpai) — kviečiantysis kodas tada praleidžia FTS5 JOIN'ą
/// visai, ne siunčia tuščią/beprasmį MATCH.
fn fts_query(search: &str) -> Option<String> {
    let terms: Vec<String> = search
        .split_whitespace()
        .map(|word| {
            let cleaned: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            format!("{cleaned}*")
        })
        .filter(|w| w.len() > 1) // vien "*" (iš tuščio žodžio) nėra galiojanti FTS5 paieška.
        .collect();

    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

/// Filtruotas, surikiuotas, puslapiuotas žaidimų sąrašas (MVP.md P5.4).
pub fn list_games(conn: &Connection, filter: &GameFilter) -> Result<Vec<Game>, AppError> {
    let mut sql = format!("SELECT {GAME_COLUMNS} FROM games g");
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    let fts_match = filter.search.as_deref().and_then(fts_query);
    if let Some(match_query) = &fts_match {
        sql.push_str(" JOIN games_fts f ON f.rowid = g.id");
        conditions.push("f.games_fts MATCH ?".to_string());
        params.push(Box::new(match_query.clone()));
    }

    if let Some(platform_id) = filter.platform_id {
        conditions.push("g.platform_id = ?".to_string());
        params.push(Box::new(platform_id));
    }

    if filter.favorites_only {
        conditions.push("g.favorite = 1".to_string());
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(
        " ORDER BY {} {} LIMIT ? OFFSET ?",
        filter.sort.column(),
        filter.sort_direction.sql()
    ));
    params.push(Box::new(filter.page_size));
    params.push(Box::new(filter.page * filter.page_size));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params_from_iter(params.iter().map(|p| p.as_ref())),
        game_from_row,
    )?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Vienas žaidimas pagal ID, arba `None`, jei tokio nėra.
pub fn get_game(conn: &Connection, id: i64) -> Result<Option<Game>, AppError> {
    let sql = format!("SELECT {GAME_COLUMNS} FROM games g WHERE g.id = ?1");
    conn.query_row(&sql, [id], game_from_row)
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(AppError::from(other)),
        })
}

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool) -> Result<(), AppError> {
    conn.execute(
        "UPDATE games SET favorite = ?1 WHERE id = ?2",
        rusqlite::params![favorite, id],
    )?;
    Ok(())
}

/// Fiksuoja žaidimo sesiją: `play_count += 1`, `play_time_seconds += seconds`,
/// `last_played = dabar` (MVP.md P9.1 „Ką daryti" — kviečiama uždarius žaidimą).
pub fn record_play(conn: &Connection, id: i64, seconds: i64) -> Result<(), AppError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    conn.execute(
        "UPDATE games SET play_count = play_count + 1, play_time_seconds = play_time_seconds + ?1, \
         last_played = ?2 WHERE id = ?3",
        rusqlite::params![seconds, now, id],
    )?;
    Ok(())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `scrape_status = 'pending'` žaidimai, neprivalomai filtruoti pagal platformą — MVP.md P6.4
/// „scrape_library(platform_id?) — visi scrape_status = 'pending'".
pub fn list_pending_games(
    conn: &Connection,
    platform_id: Option<i64>,
) -> Result<Vec<Game>, AppError> {
    let mut sql = format!("SELECT {GAME_COLUMNS} FROM games g WHERE g.scrape_status = 'pending'");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(id) = platform_id {
        sql.push_str(" AND g.platform_id = ?");
        params.push(Box::new(id));
    }
    sql.push_str(" ORDER BY g.id");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(
        params_from_iter(params.iter().map(|p| p.as_ref())),
        game_from_row,
    )?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Pažymi žaidimą apdorotu su konkrečiu statusu, BE metaduomenų (P6.4: „notfound"/„error"
/// atvejai — sėkmingam radiniui naudok [`apply_scrape_result`], kuris rašo ir statusą, ir
/// duomenis viename sakinyje). `scrape_status` reikšmės pagal P5.1 schemos komentarą:
/// `"pending" | "ok" | "notfound" | "error"`.
pub fn set_scrape_status(conn: &Connection, id: i64, status: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE games SET scrape_status = ?1, scraped_at = ?2 WHERE id = ?3",
        rusqlite::params![status, unix_now(), id],
    )?;
    Ok(())
}

/// Įrašo sėkmingo scraping'o rezultatą: ScreenScraper metaduomenis + media kelius, pažymi
/// `scrape_status = 'ok'`.
///
/// **SĄMONINGAI NEKEIČIA `title`/`sort_title`** — skenerio (`library::scanner`) parinktas
/// pavadinimas iš ROM failo vardo lieka autoritetingas rodymui/rikiavimui; ScreenScraper
/// pavadinimas patenka TIK per `description`/kitus laukus, ne perrašo tai, ką vartotojas mato
/// bibliotekoje. Priežastis: netikėtas žaidimo PERVADINIMAS vidury scraping'o būtų
/// nemalonus siurprizas, o `sort_title` perskaičiavimas reikalautų `library::scanner`
/// logikos importo į DB sluoksnį vien šiam retam atvejui — nepakankama nauda šiai MVP daliai.
pub fn apply_scrape_result(
    conn: &Connection,
    id: i64,
    metadata: &crate::scraper::screenscraper::GameMetadata,
    media: &crate::scraper::media::MediaPaths,
) -> Result<(), AppError> {
    conn.execute(
        "UPDATE games SET
            description = ?1, developer = ?2, publisher = ?3, genre = ?4, players = ?5,
            release_date = ?6, rating = ?7, region = ?8,
            cover_path = ?9, screenshot_path = ?10, wheel_path = ?11, video_path = ?12,
            cover_width = ?13, cover_height = ?14,
            scrape_status = 'ok', scraped_at = ?15
         WHERE id = ?16",
        rusqlite::params![
            metadata.description,
            metadata.developer,
            metadata.publisher,
            metadata.genre,
            metadata.players,
            metadata.release_date,
            metadata.rating,
            metadata.region,
            media.cover_path,
            media.screenshot_path,
            media.wheel_path,
            media.video_path,
            media.cover_width,
            media.cover_height,
            unix_now(),
            id,
        ],
    )?;
    Ok(())
}

/// Visos platformos su jų žaidimų kiekiu (`LEFT JOIN`, kad platformos be nė vieno žaidimo
/// vis tiek pasirodytų su `game_count = 0`, ne visai dingtų iš sąrašo).
pub fn list_platforms(conn: &Connection) -> Result<Vec<PlatformSummary>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.slug, p.name, p.screenscraper_id, p.extensions, COUNT(g.id) as game_count
         FROM platforms p
         LEFT JOIN games g ON g.platform_id = p.id
         GROUP BY p.id
         ORDER BY p.name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PlatformSummary {
            platform: Platform {
                id: row.get(0)?,
                slug: row.get(1)?,
                name: row.get(2)?,
                screenscraper_id: row.get(3)?,
                extensions: row.get(4)?,
            },
            game_count: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::MIGRATIONS
            .iter()
            .for_each(|(_, sql)| conn.execute_batch(sql).unwrap());
        conn
    }

    fn snes_platform_id(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM platforms WHERE slug = 'snes'", [], |r| {
            r.get(0)
        })
        .unwrap()
    }

    fn genesis_platform_id(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM platforms WHERE slug = 'genesis'", [], |r| {
            r.get(0)
        })
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
    fn search_finds_games_by_title_prefix() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        insert_game(&conn, snes, "Super Mario World", "/roms/smw.sfc");
        insert_game(&conn, snes, "Super Mario Kart", "/roms/smk.sfc");
        insert_game(&conn, snes, "Chrono Trigger", "/roms/ct.sfc");

        let results = list_games(
            &conn,
            &GameFilter {
                search: Some("mario".to_string()),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|g| g.title.contains("Mario")));
    }

    #[test]
    fn filter_by_platform_excludes_other_platforms() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let genesis = genesis_platform_id(&conn);
        insert_game(&conn, snes, "Super Metroid", "/roms/sm.sfc");
        insert_game(&conn, genesis, "Sonic The Hedgehog", "/roms/sonic.md");

        let results = list_games(
            &conn,
            &GameFilter {
                platform_id: Some(snes),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Super Metroid");
    }

    #[test]
    fn favorites_only_filters_correctly() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let fav_id = insert_game(&conn, snes, "Chrono Trigger", "/roms/ct.sfc");
        insert_game(&conn, snes, "EarthBound", "/roms/eb.sfc");

        set_favorite(&conn, fav_id, true).unwrap();

        let results = list_games(
            &conn,
            &GameFilter {
                favorites_only: true,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Chrono Trigger");
    }

    #[test]
    fn pagination_returns_correct_slice() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        for i in 0..5 {
            insert_game(
                &conn,
                snes,
                &format!("Game {i}"),
                &format!("/roms/g{i}.sfc"),
            );
        }

        let page0 = list_games(
            &conn,
            &GameFilter {
                page: 0,
                page_size: 2,
                ..Default::default()
            },
        )
        .unwrap();
        let page1 = list_games(
            &conn,
            &GameFilter {
                page: 1,
                page_size: 2,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(page0.len(), 2);
        assert_eq!(page1.len(), 2);
        assert_ne!(page0[0].id, page1[0].id);
    }

    #[test]
    fn record_play_increments_count_and_time() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let id = insert_game(&conn, snes, "EarthBound", "/roms/eb.sfc");

        record_play(&conn, id, 120).unwrap();
        record_play(&conn, id, 60).unwrap();

        let game = get_game(&conn, id).unwrap().unwrap();
        assert_eq!(game.play_count, 2);
        assert_eq!(game.play_time_seconds, 180);
        assert!(game.last_played.is_some());
    }

    #[test]
    fn list_pending_games_excludes_non_pending_and_other_platforms() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let genesis = genesis_platform_id(&conn);
        let pending = insert_game(&conn, snes, "Chrono Trigger", "/roms/ct.sfc");
        let other_platform_pending = insert_game(&conn, genesis, "Sonic", "/roms/sonic.md");
        let already_done = insert_game(&conn, snes, "EarthBound", "/roms/eb.sfc");
        set_scrape_status(&conn, already_done, "ok").unwrap();

        let all_pending = list_pending_games(&conn, None).unwrap();
        assert_eq!(all_pending.len(), 2);
        assert!(all_pending.iter().any(|g| g.id == pending));
        assert!(all_pending.iter().any(|g| g.id == other_platform_pending));

        let snes_pending = list_pending_games(&conn, Some(snes)).unwrap();
        assert_eq!(snes_pending.len(), 1);
        assert_eq!(snes_pending[0].id, pending);
    }

    #[test]
    fn set_scrape_status_updates_status_and_timestamp_without_touching_metadata() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let id = insert_game(&conn, snes, "Chrono Trigger", "/roms/ct.sfc");

        set_scrape_status(&conn, id, "notfound").unwrap();

        let game = get_game(&conn, id).unwrap().unwrap();
        assert_eq!(game.scrape_status, "notfound");
        assert!(game.scraped_at.is_some());
        assert_eq!(game.title, "Chrono Trigger"); // nepakito.
    }

    fn sample_scrape_metadata() -> crate::scraper::screenscraper::GameMetadata {
        crate::scraper::screenscraper::GameMetadata {
            title: "Ignoruojama".to_string(), // apply_scrape_result SĄMONINGAI nerašo title.
            description: Some("Aprašymas".to_string()),
            developer: Some("Nintendo".to_string()),
            publisher: Some("Nintendo".to_string()),
            genre: Some("Platform".to_string()),
            players: Some(2),
            release_date: Some("1994-07-28".to_string()),
            rating: Some(16.0),
            region: Some("eu".to_string()),
            medias: vec![],
        }
    }

    #[test]
    fn apply_scrape_result_writes_metadata_and_media_but_preserves_scanned_title() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let id = insert_game(&conn, snes, "Chrono Trigger (USA)", "/roms/ct.sfc");

        let media = crate::scraper::media::MediaPaths {
            cover_path: Some("covers/1.png".to_string()),
            cover_width: Some(680),
            cover_height: Some(497),
            screenshot_path: None,
            wheel_path: None,
            video_path: Some("videos/1.mp4".to_string()),
        };

        apply_scrape_result(&conn, id, &sample_scrape_metadata(), &media).unwrap();

        let game = get_game(&conn, id).unwrap().unwrap();
        assert_eq!(game.title, "Chrono Trigger (USA)"); // NEPAKEISTA — žr. funkcijos doc.
        assert_eq!(game.description.as_deref(), Some("Aprašymas"));
        assert_eq!(game.developer.as_deref(), Some("Nintendo"));
        assert_eq!(game.genre.as_deref(), Some("Platform"));
        assert_eq!(game.players, Some(2));
        assert_eq!(game.rating, Some(16.0));
        assert_eq!(game.cover_path.as_deref(), Some("covers/1.png"));
        assert_eq!(game.cover_width, Some(680));
        assert_eq!(game.cover_height, Some(497));
        assert_eq!(game.video_path.as_deref(), Some("videos/1.mp4"));
        assert_eq!(game.screenshot_path, None);
        assert_eq!(game.scrape_status, "ok");
        assert!(game.scraped_at.is_some());
    }

    #[test]
    fn list_platforms_includes_zero_count_platforms() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        insert_game(&conn, snes, "Super Metroid", "/roms/sm.sfc");

        let platforms = list_platforms(&conn).unwrap();
        let snes_summary = platforms
            .iter()
            .find(|p| p.platform.slug == "snes")
            .unwrap();
        let nes_summary = platforms.iter().find(|p| p.platform.slug == "nes").unwrap();

        assert_eq!(snes_summary.game_count, 1);
        assert_eq!(nes_summary.game_count, 0);
    }

    #[test]
    fn deleted_game_disappears_from_search_too() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);
        let id = insert_game(&conn, snes, "Super Mario World", "/roms/smw.sfc");

        conn.execute("DELETE FROM games WHERE id = ?1", [id])
            .unwrap();

        let results = list_games(
            &conn,
            &GameFilter {
                search: Some("mario".to_string()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            results.is_empty(),
            "FTS5 sync trigeris turėjo pašalinti įrašą kartu su games"
        );
    }

    /// P5.4 acceptance: „Paieška 'mario' randa visus Mario žaidimus < 50 ms su 5000 įrašų".
    /// Sintetiniai duomenys — realių fixture ROM'ų nėra 5000 (turime ~90).
    #[test]
    fn search_under_50ms_with_5000_rows() {
        let conn = open_test_db();
        let snes = snes_platform_id(&conn);

        let tx_conn = &conn;
        for i in 0..5000 {
            let title = if i % 137 == 0 {
                format!("Mario Adventure {i}")
            } else {
                format!("Generic Game {i}")
            };
            insert_game(tx_conn, snes, &title, &format!("/roms/g{i}.sfc"));
        }

        let start = std::time::Instant::now();
        let results = list_games(
            &conn,
            &GameFilter {
                search: Some("mario".to_string()),
                page_size: 5000,
                ..Default::default()
            },
        )
        .unwrap();
        let elapsed = start.elapsed();

        eprintln!(
            "paieška 5000 įrašų tarp: {:.2}ms, {} rezultatų",
            elapsed.as_secs_f64() * 1000.0,
            results.len()
        );
        assert!(!results.is_empty());
        assert!(
            elapsed.as_millis() < 50,
            "paieška užtruko {:.2}ms, tikėtasi < 50ms",
            elapsed.as_secs_f64() * 1000.0
        );
    }
}
