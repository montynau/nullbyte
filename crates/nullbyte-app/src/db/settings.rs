//! Bendra key/value nustatymų lentelė (`settings`, migracija 001) — MVP.md P7.6.
//!
//! Sąmoningai plika `String -> String` sąsaja (jokio tipizuoto `Settings` struct'o): įvairūs
//! domenai (scraper credentials, vėliau core/video/audio nustatymai) turi visiškai skirtingus
//! raktus ir gyvavimo ciklus, tad bendras struct'as tik pridėtų netiesiogiškumo be naudos.
//! Domeno moduliai (pvz. `scraper::screenscraper::ScreenScraperCredentials::load`) žino savo
//! raktų vardus ir kviečia šias funkcijas tiesiogiai.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;

pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, AppError> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
    .map_err(AppError::from)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, key: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
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

    #[test]
    fn missing_key_returns_none() {
        let conn = open_test_db();
        assert_eq!(get(&conn, "nonexistent").unwrap(), None);
    }

    #[test]
    fn set_then_get_roundtrips() {
        let conn = open_test_db();
        set(&conn, "scraper.dev_id", "abc123").unwrap();
        assert_eq!(
            get(&conn, "scraper.dev_id").unwrap(),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn set_twice_overwrites_not_errors() {
        let conn = open_test_db();
        set(&conn, "k", "first").unwrap();
        set(&conn, "k", "second").unwrap();
        assert_eq!(get(&conn, "k").unwrap(), Some("second".to_string()));
    }

    #[test]
    fn delete_removes_the_key() {
        let conn = open_test_db();
        set(&conn, "k", "v").unwrap();
        delete(&conn, "k").unwrap();
        assert_eq!(get(&conn, "k").unwrap(), None);
    }
}
