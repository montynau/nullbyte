//! Save state metaduomenų CRUD (`save_states` lentelė, MVP.md P8.1). Pati serializacija/
//! atkūrimas vyksta `nullbyte-core::core::savestate` (vaiko procese, ADR-016) — šis modulis
//! TIK saugo/skaito DB įrašus apie JAU egzistuojančius failus (kelias, preview, core
//! pavadinimas/versija core-mismatch įspėjimui, laikas).
//!
//! `upsert_save_state` kviečiama iš `commands::emulator::start_game`'o `on_status`
//! callback'o (P8.1 UI sluoksnis) kaskart, kai `nullbyte-emu` atsiunčia
//! `EmuStatus::StateSaved`; `list_save_states`/`delete_save_state` — iš `commands::savestate`
//! (žaidimo detalių puslapio „Save states" sekcija).

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::models::SaveState;
use crate::error::AppError;

const SAVE_STATE_COLUMNS: &str =
    "id, game_id, slot, path, thumb_path, core_name, core_version, created_at";

fn save_state_from_row(row: &rusqlite::Row) -> rusqlite::Result<SaveState> {
    Ok(SaveState {
        id: row.get(0)?,
        game_id: row.get(1)?,
        slot: row.get(2)?,
        path: row.get(3)?,
        thumb_path: row.get(4)?,
        core_name: row.get(5)?,
        core_version: row.get(6)?,
        created_at: row.get(7)?,
    })
}

/// Visi žaidimo save state'ai, rikiuoti pagal `slot` (P4.4 konvencija: `0` = quick save,
/// `1..=4` — numeruoti slot'ai).
pub fn list_save_states(conn: &Connection, game_id: i64) -> Result<Vec<SaveState>, AppError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SAVE_STATE_COLUMNS} FROM save_states WHERE game_id = ?1 ORDER BY slot"
    ))?;
    let rows = stmt.query_map(params![game_id], save_state_from_row)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn get_save_state(
    conn: &Connection,
    game_id: i64,
    slot: i64,
) -> Result<Option<SaveState>, AppError> {
    conn.query_row(
        &format!("SELECT {SAVE_STATE_COLUMNS} FROM save_states WHERE game_id = ?1 AND slot = ?2"),
        params![game_id, slot],
        save_state_from_row,
    )
    .optional()
    .map_err(AppError::from)
}

/// Įrašo (arba, jei `(game_id, slot)` jau egzistuoja, PERRAŠO — `UNIQUE(game_id, slot)`
/// migracijoje 001) save state'o metaduomenis PO to, kai `nullbyte-emu` jau sėkmingai
/// įrašė patį failą (žr. `nullbyte_core::core::savestate::save_state`) — šis kvietimas
/// TIK sinchronizuoja DB su tuo, kas jau yra diske.
#[allow(clippy::too_many_arguments)]
pub fn upsert_save_state(
    conn: &Connection,
    game_id: i64,
    slot: i64,
    path: &str,
    thumb_path: Option<&str>,
    core_name: &str,
    core_version: &str,
    created_at: i64,
) -> Result<SaveState, AppError> {
    conn.execute(
        "INSERT INTO save_states (game_id, slot, path, thumb_path, core_name, core_version, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(game_id, slot) DO UPDATE SET
             path = excluded.path,
             thumb_path = excluded.thumb_path,
             core_name = excluded.core_name,
             core_version = excluded.core_version,
             created_at = excluded.created_at",
        params![
            game_id,
            slot,
            path,
            thumb_path,
            core_name,
            core_version,
            created_at
        ],
    )?;
    get_save_state(conn, game_id, slot)?.ok_or_else(|| {
        AppError::Other("save state įrašytas, bet nerandamas iškart po to".to_string())
    })
}

pub fn delete_save_state(conn: &Connection, game_id: i64, slot: i64) -> Result<(), AppError> {
    conn.execute(
        "DELETE FROM save_states WHERE game_id = ?1 AND slot = ?2",
        params![game_id, slot],
    )?;
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

    /// `games`/`platforms` FK reikalauja realaus `game_id` — sukuria minimalų fixture įrašą.
    fn insert_test_game(conn: &Connection) -> i64 {
        let platform_id: i64 = conn
            .query_row("SELECT id FROM platforms WHERE slug = 'snes'", [], |row| {
                row.get(0)
            })
            .unwrap();
        conn.execute(
            "INSERT INTO games (platform_id, title, sort_title, rom_path, rom_size, added_at, file_mtime)
             VALUES (?1, 'Test Game', 'test game', '/roms/test.sfc', 1024, 0, 0)",
            params![platform_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn list_is_empty_for_a_game_with_no_saves() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);
        assert!(list_save_states(&conn, game_id).unwrap().is_empty());
    }

    #[test]
    fn upsert_then_get_roundtrips() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        let saved = upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            Some("/states/1.png"),
            "Snes9x",
            "1.62.3",
            1000,
        )
        .unwrap();
        assert_eq!(saved.slot, 1);
        assert_eq!(saved.path, "/states/1.state");
        assert_eq!(saved.thumb_path.as_deref(), Some("/states/1.png"));

        let fetched = get_save_state(&conn, game_id, 1).unwrap().unwrap();
        assert_eq!(fetched.id, saved.id);
        assert_eq!(fetched.core_name, "Snes9x");
    }

    #[test]
    fn upsert_same_slot_twice_overwrites_not_duplicates() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();
        upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "2.0",
            2000,
        )
        .unwrap();

        let all = list_save_states(&conn, game_id).unwrap();
        assert_eq!(
            all.len(),
            1,
            "tas pats (game_id, slot) turėtų PERRAŠYTI, ne dubliuoti"
        );
        assert_eq!(all[0].core_version, "2.0");
    }

    #[test]
    fn different_slots_coexist() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        upsert_save_state(
            &conn,
            game_id,
            0,
            "/states/0.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();
        upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();
        upsert_save_state(
            &conn,
            game_id,
            2,
            "/states/2.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();

        let all = list_save_states(&conn, game_id).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(
            all.iter().map(|s| s.slot).collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn delete_removes_only_the_targeted_slot() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();
        upsert_save_state(
            &conn,
            game_id,
            2,
            "/states/2.state",
            None,
            "Snes9x",
            "1.0",
            1000,
        )
        .unwrap();

        delete_save_state(&conn, game_id, 1).unwrap();

        let all = list_save_states(&conn, game_id).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].slot, 2);
    }
}
