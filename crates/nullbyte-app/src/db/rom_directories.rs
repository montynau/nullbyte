//! ROM katalogų CRUD (`rom_directories` lentelė, MVP.md P7.5). `library::scanner::scan()`
//! pati skaito TIK `enabled` įrašus — šis modulis skirtas UI valdymui (visų įrašų sąrašas,
//! pridėjimas, pašalinimas).

use rusqlite::{params, Connection};

use crate::db::models::RomDirectory;
use crate::error::AppError;

pub fn list_rom_directories(conn: &Connection) -> Result<Vec<RomDirectory>, AppError> {
    let mut stmt =
        conn.prepare("SELECT id, path, recursive, enabled FROM rom_directories ORDER BY path")?;
    let rows = stmt.query_map([], |row| {
        Ok(RomDirectory {
            id: row.get(0)?,
            path: row.get(1)?,
            recursive: row.get(2)?,
            enabled: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

/// Idempotentiška — pakartotinis pridėjimas TO PATIES kelio atnaujina `recursive` ir vėl
/// įjungia (`enabled = 1`), ne meta `UNIQUE` klaidos (`path` stulpelis unikalus).
pub fn add_rom_directory(
    conn: &Connection,
    path: &str,
    recursive: bool,
) -> Result<RomDirectory, AppError> {
    conn.execute(
        "INSERT INTO rom_directories (path, recursive, enabled) VALUES (?1, ?2, 1)
         ON CONFLICT(path) DO UPDATE SET recursive = excluded.recursive, enabled = 1",
        params![path, recursive],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM rom_directories WHERE path = ?1",
        params![path],
        |row| row.get(0),
    )?;
    Ok(RomDirectory {
        id,
        path: path.to_string(),
        recursive,
        enabled: true,
    })
}

pub fn remove_rom_directory(conn: &Connection, id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM rom_directories WHERE id = ?1", params![id])?;
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
    fn add_then_list_returns_the_new_directory() {
        let conn = open_test_db();
        let added = add_rom_directory(&conn, "/roms/snes", true).unwrap();
        assert!(added.id > 0);
        assert_eq!(added.path, "/roms/snes");
        assert!(added.recursive);
        assert!(added.enabled);

        let dirs = list_rom_directories(&conn).unwrap();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].path, "/roms/snes");
    }

    #[test]
    fn adding_the_same_path_twice_updates_instead_of_erroring() {
        let conn = open_test_db();
        add_rom_directory(&conn, "/roms/snes", true).unwrap();
        let second = add_rom_directory(&conn, "/roms/snes", false).unwrap();
        assert!(!second.recursive);

        let dirs = list_rom_directories(&conn).unwrap();
        assert_eq!(dirs.len(), 1);
        assert!(!dirs[0].recursive);
    }

    #[test]
    fn remove_deletes_the_directory() {
        let conn = open_test_db();
        let added = add_rom_directory(&conn, "/roms/snes", true).unwrap();
        remove_rom_directory(&conn, added.id).unwrap();
        assert!(list_rom_directories(&conn).unwrap().is_empty());
    }
}
