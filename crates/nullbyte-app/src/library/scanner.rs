//! ROM katalogų skenavimas (MVP.md P5.3).
//!
//! `scan()` SĄMONINGAI nepriklauso nuo Tauri tipų (`on_progress` — paprastas `FnMut`
//! atgalinis kvietimas, ne `tauri::ipc::Channel`) — kviečiančioji Tauri komanda (P7 UI
//! sluoksnis) jį persiunčia per `Channel<ScanProgress>` (CLAUDE.md §6.3), bet pati skenavimo
//! logika lieka testuojama be `tauri::test` scaffolding'o (tas pats principas kaip
//! `core::runner`/`input::gamepad` — domeno moduliai nepriklauso nuo UI/IPC karkaso).
//!
//! **`rom_path` saugomas ABSOLIUČIAI**, ne santykinai su `rom_directories.path` — skirtingai
//! nuo media cache (CLAUDE.md §9.4 „DB laiko tik santykinius kelius"), kuris visada gyvena
//! `{app_data}/media/` viduje, ROM katalogai gali būti BET KUR diske, ir jų gali būti keli,
//! tad „santykinis nuo ko" būtų dviprasmiškas be papildomo JOIN'o į `rom_directories`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};

use crate::db::models::{Platform, RomDirectory};
use crate::error::AppError;
use crate::library::hasher;

/// Vieno apdoroto failo progreso pranešimas — žr. modulio doc dėl KODĖL `scan()` pati
/// nepriklauso nuo `tauri::ipc::Channel`. `Serialize` + camelCase (CLAUDE.md §7.3) — P7.5
/// komandų sluoksnis šitą persiunčia per `Channel<ScanProgress>` nepakeistą.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub removed: usize,
    pub skipped_unknown_extension: usize,
}

/// Nuskenuoja visus `enabled` `rom_directories` įrašus, sinchronizuoja `games` lentelę su
/// disko turiniu VIENOJE SQLite transakcijoje (MVP.md P5.3 „Ką daryti"). `on_progress`
/// kviečiamas po KIEKVIENO apdoroto failo (net jei jis praleistas dėl nežinomo plėtinio ar
/// nepakitęs — vartotojui progreso juosta turi judėti tolygiai, ne tik „naujiems" failams).
pub fn scan(
    conn: &mut Connection,
    mut on_progress: impl FnMut(ScanProgress),
) -> Result<ScanSummary, AppError> {
    let directories = load_enabled_directories(conn)?;
    let platforms = load_platforms(conn)?;

    let mut files_by_directory: Vec<(RomDirectory, Vec<PathBuf>)> = Vec::new();
    for dir in directories {
        let mut walker = walkdir::WalkDir::new(&dir.path);
        if !dir.recursive {
            walker = walker.max_depth(1);
        }
        let files: Vec<PathBuf> = walker
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();
        files_by_directory.push((dir, files));
    }

    let total: usize = files_by_directory
        .iter()
        .map(|(_, files)| files.len())
        .sum();
    let mut summary = ScanSummary::default();
    let mut processed = 0usize;

    let tx = conn.transaction()?;
    let mut found_paths: HashSet<String> = HashSet::new();

    for (dir, files) in &files_by_directory {
        for path in files {
            processed += 1;
            on_progress(ScanProgress {
                current: processed,
                total,
                current_file: path.display().to_string(),
            });

            // `canonicalize()` sprendžia `..`/simlink'us — tas pats failas visada duoda tą
            // patį `rom_path`, nepriklausomai nuo to, kokiu keliu jis pasiekiamas per
            // skirtingus (persidengiančius) `rom_directories` įrašus.
            let Ok(canonical) = path.canonicalize() else {
                continue; // Failas dingo tarp `walkdir` ir `canonicalize()` — praleidžiam.
            };
            let rom_path = canonical.to_string_lossy().into_owned();
            found_paths.insert(rom_path.clone());

            let metadata = std::fs::metadata(path)?;
            let file_mtime = mtime_unix_seconds(&metadata);
            let existing = existing_game_state(&tx, &rom_path)?;
            let existing_mtime = existing.map(|(mtime, _)| mtime);

            // Inkrementinis skenavimas (MVP.md P5.3): nepakitusiam failui NĖRA prasmės nei
            // spėti platformą, nei hash'uoti iš naujo — tai buvo padaryta ANKSTESNIO
            // skenavimo metu, o rezultatas jau DB'je. IŠIMTIS (ADR-020): jei katalogui
            // NUSTATYTAS `platform_id` hint'as, kuris SKIRIASI nuo jau įrašytos platformos —
            // priverstinai perklasifikuojame, nepaisant mtime (vartotojas ką tik ištaisė
            // dviprasmybę pridėdamas hint'ą, tikisi pataisymo be failo modifikavimo).
            let platform_matches_hint = match (dir.platform_id, existing) {
                (Some(hint), Some((_, existing_platform_id))) => hint == existing_platform_id,
                _ => true,
            };
            if existing_mtime == Some(file_mtime) && platform_matches_hint {
                summary.unchanged += 1;
                continue;
            }

            let Some(extension) = path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
            else {
                summary.skipped_unknown_extension += 1;
                continue;
            };

            let Some((platform, hashes)) =
                resolve_platform_and_hashes(path, &platforms, &extension, dir.platform_id)
            else {
                summary.skipped_unknown_extension += 1;
                continue;
            };

            upsert_game(
                &tx,
                &rom_path,
                platform,
                path,
                &hashes,
                file_mtime,
                existing_mtime.is_none(),
            )?;
            if existing_mtime.is_none() {
                summary.added += 1;
            } else {
                summary.updated += 1;
            }
        }

        // `dir.path` kanonizuojamas ČIA, PRIEŠ palyginimą su `games.rom_path` — pastarasis
        // visada kanonizuotas (žr. `canonical` aukščiau). Be šito, `/tmp` (simlink į
        // `/private/tmp` macOS'e) ir panašūs atvejai reikštų, kad joks `rom_path` niekada
        // nesutaptų su prefiksu, ir IŠTRINTI failai NIEKADA nebūtų aptikti (žr. testą
        // `deleted_file_is_removed_from_games_on_rescan`, kuris tai iš tikrųjų aptiko).
        let canonical_dir_path = std::fs::canonicalize(&dir.path)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| dir.path.clone());
        summary.removed += remove_missing_games(&tx, &canonical_dir_path, &found_paths)?;
    }

    tx.commit()?;

    Ok(summary)
}

fn load_enabled_directories(conn: &Connection) -> Result<Vec<RomDirectory>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, path, recursive, enabled, platform_id FROM rom_directories WHERE enabled = 1",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(RomDirectory {
            id: row.get(0)?,
            path: row.get(1)?,
            recursive: row.get::<_, i64>(2)? != 0,
            enabled: row.get::<_, i64>(3)? != 0,
            platform_id: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_platforms(conn: &Connection) -> Result<Vec<Platform>, AppError> {
    let mut stmt =
        conn.prepare("SELECT id, slug, name, screenscraper_id, extensions FROM platforms")?;
    let rows = stmt.query_map([], |row| {
        Ok(Platform {
            id: row.get(0)?,
            slug: row.get(1)?,
            name: row.get(2)?,
            screenscraper_id: row.get(3)?,
            extensions: row.get(4)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Nustato platformą IR PAKELIUI apskaičiuoja hash'us — abu atliekami VIENAME žingsnyje,
/// nes archyvams (`.zip`/`.7z`) vien plėtinio NEPAKANKA: KELIOS platformos gali dalintis
/// bendru archyvo plėtiniu (žr. `002_fix_archive_extensions.sql` — PSX/Saturn/SegaCD visos
/// naudoja `.zip`), tad be `platform_hint` vienareikšmiškai nustatoma bandant KIEKVIENĄ
/// kandidatą ir žiūrint, kurio VIDINIS plėtinys realiai atsiranda archyve. Tai PIGU:
/// `archive::extract_first_match` tikrina įrašų VARDUS (zip TOC), NEDEKOMPRESUOJA turinio,
/// kol nerado sutampančio — klaidingas kandidatas kainuoja tik sąrašo skaitymą, teisingas —
/// vienintelę realią dekompresiją.
///
/// `platform_hint` (ADR-020, `rom_directories.platform_id`) — kai `Some`, kandidatų sąrašas
/// susiaurinamas iki VIENOS nurodytos platformos: pašalina dviprasmybę VISIŠKAI (realus
/// atvejis: PSX `.zip` be hint'o klaidingai atsidurdavo po Sega CD, nes abi priima `.cue`
/// archyvo viduje, o Sega CD anksčiau pasitaiko `platforms` sąraše).
fn resolve_platform_and_hashes<'a>(
    path: &Path,
    platforms: &'a [Platform],
    extension: &str,
    platform_hint: Option<i64>,
) -> Option<(&'a Platform, hasher::RomHashes)> {
    let candidates: Vec<&Platform> = match platform_hint {
        Some(hint_id) => platforms.iter().filter(|p| p.id == hint_id).collect(),
        None => platforms.iter().collect(),
    };

    let is_archive = matches!(extension, "zip" | "7z");

    if !is_archive {
        let platform = candidates.into_iter().find(|p| {
            p.extensions
                .split(',')
                .any(|e| e.trim().eq_ignore_ascii_case(extension))
        })?;
        let hashes = hasher::hash_rom(path, &[]).ok()?;
        return Some((platform, hashes));
    }

    for platform in candidates.into_iter().filter(|p| {
        p.extensions
            .split(',')
            .any(|e| e.trim().eq_ignore_ascii_case(extension))
    }) {
        let inner_extensions: Vec<String> = platform
            .extensions
            .split(',')
            .map(|e| e.trim().to_string())
            .filter(|e| e != "zip" && e != "7z") // Nešaukdinam paties archyvo plėtinio.
            .collect();
        if let Ok(hashes) = hasher::hash_rom(path, &inner_extensions) {
            return Some((platform, hashes));
        }
    }
    None
}

fn mtime_unix_seconds(metadata: &std::fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Grąžina `(file_mtime, platform_id)` jau įrašytam žaidimui, arba `None`, jei jo dar nėra.
fn existing_game_state(
    tx: &rusqlite::Transaction,
    rom_path: &str,
) -> Result<Option<(i64, i64)>, AppError> {
    tx.query_row(
        "SELECT file_mtime, platform_id FROM games WHERE rom_path = ?1",
        params![rom_path],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map(Some)
    .or_else(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(AppError::from(other)),
    })
}

/// Įterpia (`is_new`) arba atnaujina `games` eilutę. Atnaujinant TYČIA nekeičiami
/// `added_at`/`play_count`/`play_time_seconds`/`favorite`/`scrape_status` — tai vartotojo
/// istorija, nepriklausanti nuo failo turinio pasikeitimo.
fn upsert_game(
    tx: &rusqlite::Transaction,
    rom_path: &str,
    platform: &Platform,
    original_path: &Path,
    hashes: &hasher::RomHashes,
    file_mtime: i64,
    is_new: bool,
) -> Result<(), AppError> {
    let (title, region) = clean_title(
        original_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default(),
    );
    let sort_title = make_sort_title(&title);

    if is_new {
        tx.execute(
            "INSERT INTO games (
                platform_id, title, sort_title, rom_path, rom_size, archive_inner,
                crc32, md5, sha1, region, added_at, file_mtime
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                platform.id,
                title,
                sort_title,
                rom_path,
                hashes.size,
                hashes.archive_inner,
                hashes.crc32,
                hashes.md5,
                hashes.sha1,
                region,
                now_unix_seconds(),
                file_mtime,
            ],
        )?;
    } else {
        tx.execute(
            "UPDATE games SET
                platform_id = ?1, title = ?2, sort_title = ?3, rom_size = ?4,
                archive_inner = ?5, crc32 = ?6, md5 = ?7, sha1 = ?8, region = ?9,
                file_mtime = ?10
             WHERE rom_path = ?11",
            params![
                platform.id,
                title,
                sort_title,
                hashes.size,
                hashes.archive_inner,
                hashes.crc32,
                hashes.md5,
                hashes.sha1,
                region,
                file_mtime,
                rom_path,
            ],
        )?;
    }

    Ok(())
}

/// Pašalina `games` eilutes, kurių `rom_path` PRIKLAUSO šiam skenuotam katalogui (`dir_path`
/// prefiksas), bet NEBUVO rasta šio skenavimo metu (MVP.md P5.3 „Ištrintų failų aptikimas").
/// Apribota konkrečiu katalogu — kitų (šį kartą neskenuotų arba išjungtų)
/// `rom_directories` žaidimai NELIEČIAMI.
fn remove_missing_games(
    tx: &rusqlite::Transaction,
    dir_path: &str,
    found_paths: &HashSet<String>,
) -> Result<usize, AppError> {
    let prefix = format!("{dir_path}%");
    let mut stmt = tx.prepare("SELECT rom_path FROM games WHERE rom_path LIKE ?1")?;
    let candidates: Vec<String> = stmt
        .query_map(params![prefix], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    drop(stmt);

    let mut removed = 0;
    for rom_path in candidates {
        if !found_paths.contains(&rom_path) {
            tx.execute("DELETE FROM games WHERE rom_path = ?1", params![rom_path])?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// Regionai, kuriuos atpažįstame parentezėse (No-Intro/GoodTools konvencija) — pirmas
/// atitinkantis tag'as tampa `region` reikšme. NE išsamus sąrašas, bet padengia dažniausius.
const KNOWN_REGIONS: &[&str] = &[
    "USA",
    "Europe",
    "Japan",
    "World",
    "Germany",
    "France",
    "Spain",
    "Italy",
    "Australia",
    "Korea",
    "China",
    "Brazil",
    "Netherlands",
    "Sweden",
    "Asia",
    "UK",
    "Canada",
];

/// Išvalo ROM failo pavadinimą į rodomą pavadinimą + regioną (MVP.md P5.3 „Pavadinimo
/// valymas": `"Super Mario World (USA) [!].sfc"` → `"Super Mario World"`, regionas → `"USA"`).
/// Pašalina VISUS `(...)`/`[...]` fragmentus iš title'o, nepriklausomai nuo to, ar jie
/// atpažinti kaip regionas — likę (pvz. `[!]`, `(Rev A)`, `(Beta)`) yra metaduomenys, ne
/// pavadinimo dalis.
fn clean_title(filename: &str) -> (String, Option<String>) {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let mut region = None;
    let mut cleaned = String::with_capacity(stem.len());
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut tag_buf = String::new();

    for ch in stem.chars() {
        match ch {
            '(' => {
                depth_paren += 1;
                tag_buf.clear();
            }
            ')' => {
                if depth_paren > 0 {
                    depth_paren -= 1;
                    if region.is_none() {
                        let candidate = tag_buf.trim();
                        if KNOWN_REGIONS
                            .iter()
                            .any(|r| r.eq_ignore_ascii_case(candidate))
                        {
                            region = Some(candidate.to_string());
                        }
                    }
                }
            }
            '[' => {
                depth_bracket += 1;
            }
            ']' => {
                depth_bracket = (depth_bracket - 1).max(0);
            }
            _ if depth_paren > 0 => tag_buf.push(ch),
            _ if depth_bracket > 0 => {}
            _ => cleaned.push(ch),
        }
    }

    (cleaned.trim().to_string(), region)
}

/// `games.sort_title` — be `"The "` prefikso, mažosiomis raidėmis (MVP.md P5.1 schema
/// komentaras). Rikiavimui, ne rodymui — `title` laukas išlaiko originalų regisrą.
fn make_sort_title(title: &str) -> String {
    let lower = title.to_lowercase();
    lower.strip_prefix("the ").unwrap_or(&lower).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MVP.md P5.3 acceptance: „Pavadinimai išvalyti teisingai (testai su 20 pavyzdžių)".
    #[test]
    fn clean_title_handles_twenty_real_world_filenames() {
        let cases: &[(&str, &str, Option<&str>)] = &[
            (
                "Super Mario World (USA) [!].sfc",
                "Super Mario World",
                Some("USA"),
            ),
            ("Super Mario World.sfc", "Super Mario World", None),
            (
                "Chrono Trigger (Japan).sfc",
                "Chrono Trigger",
                Some("Japan"),
            ),
            (
                "The Legend of Zelda (USA).nes",
                "The Legend of Zelda",
                Some("USA"),
            ),
            (
                "Sonic The Hedgehog 2 (World).md",
                "Sonic The Hedgehog 2",
                Some("World"),
            ),
            (
                "Streets of Rage (Europe) (Rev A).md",
                "Streets of Rage",
                Some("Europe"),
            ),
            (
                "Final Fantasy VI (USA) (Rev 1) [!].sfc",
                "Final Fantasy VI",
                Some("USA"),
            ),
            ("Metroid.nes", "Metroid", None),
            (
                "Contra III - The Alien Wars (USA).sfc",
                "Contra III - The Alien Wars",
                Some("USA"),
            ),
            (
                "Donkey Kong Country 2 - Diddy's Kong Quest (USA).sfc",
                "Donkey Kong Country 2 - Diddy's Kong Quest",
                Some("USA"),
            ),
            ("EarthBound (USA) (Rev 1).sfc", "EarthBound", Some("USA")),
            ("F-Zero (Japan, USA).sfc", "F-Zero", None),
            (
                "Tetris & Dr. Mario (USA).sfc",
                "Tetris & Dr. Mario",
                Some("USA"),
            ),
            ("Mega Man X (Europe).sfc", "Mega Man X", Some("Europe")),
            ("Golden Axe (World) [b1].md", "Golden Axe", Some("World")),
            (
                "Gunstar Heroes (Japan, USA) (En).md",
                "Gunstar Heroes",
                None,
            ),
            ("Vectorman (USA, Europe).md", "Vectorman", None),
            (
                "Panel de Pon (Japan) (Beta).sfc",
                "Panel de Pon",
                Some("Japan"),
            ),
            ("Wario's Woods (USA).sfc", "Wario's Woods", Some("USA")),
            (
                "[BIOS] Game Boy Boot ROM (World).gb",
                "Game Boy Boot ROM",
                Some("World"),
            ),
        ];
        assert_eq!(cases.len(), 20, "acceptance reikalauja lygiai 20 pavyzdžių");

        for (input, expected_title, expected_region) in cases {
            let (title, region) = clean_title(input);
            assert_eq!(&title, expected_title, "title nesutapo failui {input}");
            assert_eq!(
                region.as_deref(),
                *expected_region,
                "region nesutapo failui {input}"
            );
        }
    }

    #[test]
    fn sort_title_strips_the_prefix_and_lowercases() {
        assert_eq!(make_sort_title("The Legend of Zelda"), "legend of zelda");
        assert_eq!(make_sort_title("Chrono Trigger"), "chrono trigger");
        assert_eq!(make_sort_title("THE Elder Scrolls"), "elder scrolls");
    }

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

    fn make_test_rom(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn scan_inserts_new_games_and_skips_unknown_extensions() {
        let mut conn = open_test_db();
        let dir = std::env::temp_dir().join(format!("nullbyte_scan_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        make_test_rom(&dir, "Super Mario World (USA).sfc", b"fake rom data");
        make_test_rom(&dir, "readme.txt", b"not a rom");

        conn.execute(
            "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
            params![dir.to_string_lossy()],
        )
        .unwrap();

        let mut progress_calls = 0;
        let summary = scan(&mut conn, |_| progress_calls += 1).unwrap();

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(summary.added, 1);
        assert_eq!(summary.skipped_unknown_extension, 1);
        assert_eq!(progress_calls, 2, "progresas turėjo suveikti abiem failams");

        let title: String = conn
            .query_row("SELECT title FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(title, "Super Mario World");

        let platform_id: i64 = conn
            .query_row("SELECT platform_id FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(platform_id, snes_platform_id(&conn));
    }

    #[test]
    fn rescanning_unchanged_directory_is_a_noop() {
        let mut conn = open_test_db();
        let dir = std::env::temp_dir().join(format!("nullbyte_scan_test2_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        make_test_rom(&dir, "Metroid.sfc", b"fake rom data");

        conn.execute(
            "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
            params![dir.to_string_lossy()],
        )
        .unwrap();

        let first = scan(&mut conn, |_| {}).unwrap();
        let second = scan(&mut conn, |_| {}).unwrap();

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(first.added, 1);
        assert_eq!(second.added, 0);
        assert_eq!(second.unchanged, 1);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "pakartotinis skenavimas neturėjo dubliuoti įrašo");
    }

    #[test]
    fn deleted_file_is_removed_from_games_on_rescan() {
        let mut conn = open_test_db();
        let dir = std::env::temp_dir().join(format!("nullbyte_scan_test3_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rom_path = make_test_rom(&dir, "Metroid.sfc", b"fake rom data");

        conn.execute(
            "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
            params![dir.to_string_lossy()],
        )
        .unwrap();

        scan(&mut conn, |_| {}).unwrap();
        std::fs::remove_file(&rom_path).unwrap();
        let second = scan(&mut conn, |_| {}).unwrap();

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(second.removed, 1);
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn changed_file_content_updates_hashes_and_mtime() {
        let mut conn = open_test_db();
        let dir = std::env::temp_dir().join(format!("nullbyte_scan_test4_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rom_path = make_test_rom(&dir, "Metroid.sfc", b"version one");

        conn.execute(
            "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
            params![dir.to_string_lossy()],
        )
        .unwrap();

        scan(&mut conn, |_| {}).unwrap();
        let crc_before: String = conn
            .query_row("SELECT crc32 FROM games", [], |r| r.get(0))
            .unwrap();

        // `file_mtime` granuliuotumas — sekundės; be dirbtinio pauzavimo testas galėtų
        // netyčia gauti tą pačią mtime reikšmę ir klaidingai praleisti atnaujinimą.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        std::fs::write(&rom_path, b"version two, totally different content").unwrap();

        let second = scan(&mut conn, |_| {}).unwrap();
        let crc_after: String = conn
            .query_row("SELECT crc32 FROM games", [], |r| r.get(0))
            .unwrap();

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(second.updated, 1);
        assert_ne!(crc_before, crc_after);
    }

    /// P5.3 acceptance: „500 ROM'ų katalogas nuskenuojamas < 60s" ir „Pakartotinis
    /// skenavimas be pakeitimų < 2s" — REALŪS test fixture ROM'ai (SNES/Genesis/PSX/GBA,
    /// tas pats rinkinys kaip P5.2 `hashing_100_files_under_30_seconds`), ne sintetiniai
    /// duomenys. 91 failas, ne 500 — bet extrapoliuojant iš P5.2 rezultato (91 failas / 1.53
    /// GB per 14.12s release'e), 500 mažesnių SNES/NES-dydžio ROM'ų liktų giliai po 60s riba.
    ///
    /// `#[ignore]`: priklauso nuo test fixture'ų. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app --release scan_real_fixtures_is_fast -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn scan_real_fixtures_is_fast() {
        let mut conn = open_test_db();
        let roms_root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core/roms");

        for sub in ["snes", "megadrive", "psx", "gba"] {
            let dir = roms_root.join(sub);
            if dir.exists() {
                conn.execute(
                    "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, 1, 1)",
                    params![dir.to_string_lossy()],
                )
                .unwrap();
            }
        }

        let start = std::time::Instant::now();
        let first = scan(&mut conn, |_| {}).unwrap();
        let first_elapsed = start.elapsed();

        let start2 = std::time::Instant::now();
        let second = scan(&mut conn, |_| {}).unwrap();
        let second_elapsed = start2.elapsed();

        eprintln!(
            "pirmas skenavimas: {} nauji, {:.2}s | pakartotinis: {} nepakitę, {:.2}s",
            first.added,
            first_elapsed.as_secs_f64(),
            second.unchanged,
            second_elapsed.as_secs_f64()
        );

        assert!(
            first.added > 0,
            "turėtų rasti bent vieną ROM'ą test fixture'uose"
        );
        assert_eq!(second.added, 0);
        assert_eq!(second.updated, 0);
        assert_eq!(second.unchanged, first.added);
        assert!(
            first_elapsed.as_secs() < 60,
            "pirmas skenavimas užtruko {:.2}s, tikėtasi < 60s",
            first_elapsed.as_secs_f64()
        );
        assert!(
            second_elapsed.as_secs_f64() < 2.0,
            "pakartotinis skenavimas užtruko {:.2}s, tikėtasi < 2s",
            second_elapsed.as_secs_f64()
        );
    }

    fn make_test_zip(dir: &Path, name: &str, inner_name: &str, content: &[u8]) -> PathBuf {
        use std::io::Write;

        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);
            writer
                .start_file(inner_name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(content).unwrap();
            writer.finish().unwrap();
        }
        let path = dir.join(name);
        std::fs::write(&path, &buf).unwrap();
        path
    }

    fn platform_id_by_slug(conn: &Connection, slug: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM platforms WHERE slug = ?1",
            params![slug],
            |r| r.get(0),
        )
        .unwrap()
    }

    /// ADR-020: PSX/Saturn/SegaCD visos priima `.cue` archyvo viduje, tad be `platform_id`
    /// hint'o skeneris negali jų vienareikšmiškai atskirti — pasirenka PIRMĄ tinkantį
    /// kandidatą `platforms` sąrašo tvarka (Sega CD, nes jos `id` mažesnis už PSX, žr.
    /// `001_initial.sql` seed eiliškumą). Realus radinys P7.5 metu (3 tikri PSX žaidimai
    /// atsidūrė po Sega CD). Šis testas patikrina IR pačią dviprasmybę, IR kad hint'o
    /// pridėjimas + pakartotinis skenavimas PERKLASIFIKUOJA jau įrašytą žaidimą, nors jo
    /// failas nepasikeitė (mtime tas pats) — savaiminis pasitaisymas be rankinio DB taisymo.
    #[test]
    fn ambiguous_cue_zip_resolves_via_hint_and_self_heals_on_rescan() {
        let mut conn = open_test_db();
        let psx_id = platform_id_by_slug(&conn, "psx");
        let segacd_id = platform_id_by_slug(&conn, "segacd");
        let dir = std::env::temp_dir().join(format!(
            "nullbyte_scan_ambiguous_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        make_test_zip(&dir, "Tekken 3.zip", "Tekken 3.cue", b"fake cue data");

        conn.execute(
            "INSERT INTO rom_directories (path, recursive, enabled, platform_id) VALUES (?1, 1, 1, NULL)",
            params![dir.to_string_lossy()],
        )
        .unwrap();
        let first = scan(&mut conn, |_| {}).unwrap();
        assert_eq!(first.added, 1);
        let resolved: i64 = conn
            .query_row(
                "SELECT platform_id FROM games WHERE title LIKE 'Tekken%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            resolved, segacd_id,
            "be hint'o žinomai susimaišo su Sega CD (dokumentuotas apribojimas)"
        );

        conn.execute(
            "UPDATE rom_directories SET platform_id = ?1 WHERE path = ?2",
            params![psx_id, dir.to_string_lossy()],
        )
        .unwrap();
        let second = scan(&mut conn, |_| {}).unwrap();
        assert_eq!(
            second.updated, 1,
            "hint'o pridėjimas turėtų priversti perklasifikavimą, ne 'unchanged'"
        );
        assert_eq!(second.added, 0);
        let resolved2: i64 = conn
            .query_row(
                "SELECT platform_id FROM games WHERE title LIKE 'Tekken%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(resolved2, psx_id, "po hint'o turėtų būti PSX");

        std::fs::remove_dir_all(&dir).ok();
    }
}
