//! Migracijos per `PRAGMA user_version` (MVP.md P5.1 „Ką daryti").

use rusqlite::Connection;

use crate::error::AppError;

/// Kiekviena migracija: `(versija, SQL turinys)`. Versijos numeruojamos NUO 1 — naujo DB
/// failo `PRAGMA user_version` numatytoji reikšmė yra `0`, kuri reiškia „jokia migracija dar
/// netaikyta", tad `0` pati savaime negali būti galiojanti migracijos versija.
pub(crate) const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../../migrations/001_initial.sql")),
    (
        2,
        include_str!("../../migrations/002_fix_archive_extensions.sql"),
    ),
    (
        3,
        include_str!("../../migrations/003_games_fts_sync_triggers.sql"),
    ),
    (
        4,
        include_str!("../../migrations/004_fix_gba_archive_extension.sql"),
    ),
    (
        5,
        include_str!("../../migrations/005_rom_directory_platform_hint.sql"),
    ),
    (
        6,
        include_str!("../../migrations/006_game_cover_dimensions.sql"),
    ),
];

/// Atveria (arba sukuria, jei neegzistuoja — kartu su tėviniais katalogais) SQLite DB
/// `path`, pritaiko visas dar netaikytas migracijas, grąžina paruoštą `Connection`.
///
/// `foreign_keys = ON` nustatoma PRIE ŠIO KONKRETAUS ryšio, ne vieną kartą migracijos SQL
/// faile — SQLite tai laiko per-connection nustatymu (numatytai OFF kiekvienam naujam
/// `Connection`), CLAUDE.md §10 „SQLite" tai eksplicitiškai reikalauja daryti „prie
/// kiekvieno prisijungimo".
pub fn open_and_migrate(path: &std::path::Path) -> Result<Connection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;

    run_migrations(&conn)?;

    Ok(conn)
}

fn current_version(conn: &Connection) -> Result<u32, AppError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    let current = current_version(conn)?;

    for &(version, sql) in MIGRATIONS {
        if version <= current {
            continue; // Jau pritaikyta ankstesnio paleidimo metu — idempotentiškumas.
        }
        conn.execute_batch(sql)?;
        conn.pragma_update(None, "user_version", version)?;
        tracing::info!(version, "DB migracija pritaikyta");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_platforms(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM platforms", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn fresh_db_creates_schema_and_seed_platforms() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let count = count_platforms(&conn);
        assert!(
            count >= 20,
            "tikėtasi bent 20 seed platformų, gauta {count}"
        );

        let snes_id: i64 = conn
            .query_row(
                "SELECT screenscraper_id FROM platforms WHERE slug = 'snes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(snes_id, 4);
    }

    /// P5.1 acceptance: „Migracijos idempotentiškos (paleisk 3 kartus)".
    #[test]
    fn migrations_are_idempotent_across_three_runs() {
        let conn = Connection::open_in_memory().unwrap();

        run_migrations(&conn).unwrap();
        let after_first = count_platforms(&conn);
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        assert_eq!(
            count_platforms(&conn),
            after_first,
            "pakartotinis migracijų paleidimas neturėjo dubliuoti seed duomenų"
        );
    }

    /// P5.1 acceptance: „DB sukuriama pirmą kartą paleidus" — realus failas, ne
    /// `:memory:`, ir tėvinis katalogas, kurio dar nėra (kaip realiu paleidimu — `data_dir`
    /// gali dar neegzistuoti, žr. `AppState::new`).
    #[test]
    fn open_and_migrate_creates_parent_directory_and_file() {
        let dir = std::env::temp_dir().join(format!("nullbyte_db_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let db_path = dir.join("nested/nullbyte.db");

        let conn = open_and_migrate(&db_path).expect("turėtų sukurti katalogą ir DB failą");
        assert!(db_path.exists());
        assert!(count_platforms(&conn) >= 20);

        drop(conn);
        std::fs::remove_dir_all(&dir).ok();
    }
}
