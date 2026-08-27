//! Save state Tauri komandos (MVP.md P8.1 UI sluoksnis) — PLONAS sluoksnis. Faktinis DB
//! rašymas (`upsert_save_state`) vyksta `commands::emulator::start_game`'o `on_status`
//! callback'e (žr. jo doc), NE čia — TIK ten yra visas reikiamas kontekstas (`game_id`,
//! `states_dir`, core pavadinimas/versija) tuo momentu, kai `EmuStatus::StateSaved` realiai
//! atkeliauja iš `nullbyte-emu`. Šis modulis TIK skaito jau esančius įrašus ir juos trina.

use rusqlite::Connection;
use tauri::State;

use crate::db::models::SaveState;
use crate::db::save_states;
use crate::error::AppError;
use crate::state::AppState;

/// Visi žaidimo save state'ai, rikiuoti pagal slot'ą — žaidimo detalių puslapio „Save
/// states" sekcija (P7.4/P8.1).
#[tauri::command]
pub fn list_save_states(
    state: State<'_, AppState>,
    game_id: i64,
) -> Result<Vec<SaveState>, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    save_states::list_save_states(&conn, game_id)
}

/// Ištrina save state'ą — DB įrašą IR faktinius failus diske (`.state`/`.png`). Jei kuris
/// nors failas jau neegzistuoja (pvz. rankiniu būdu ištrintas anksčiau) — TAI NĖRA klaida,
/// tiesiog praleidžiama (`std::fs::remove_file(...).ok()`), nes galutinis tikslas („šio
/// save state'o daugiau nebėra") jau pasiektas.
///
/// Logika iškelta į [`delete_save_state_impl`] (grynas `&Connection`, ne `State<AppState>`),
/// kad būtų testuojama be pilno Tauri app'o — ta pati technika kaip
/// `commands::settings::resolve_preferred_core_path`.
#[tauri::command]
pub fn delete_save_state(
    state: State<'_, AppState>,
    game_id: i64,
    slot: i64,
) -> Result<(), AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    delete_save_state_impl(&conn, game_id, slot)
}

fn delete_save_state_impl(conn: &Connection, game_id: i64, slot: i64) -> Result<(), AppError> {
    if let Some(existing) = save_states::get_save_state(conn, game_id, slot)? {
        std::fs::remove_file(&existing.path).ok();
        if let Some(thumb) = &existing.thumb_path {
            std::fs::remove_file(thumb).ok();
        }
    }
    save_states::delete_save_state(conn, game_id, slot)
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

    fn insert_test_game(conn: &Connection) -> i64 {
        let platform_id: i64 = conn
            .query_row("SELECT id FROM platforms WHERE slug = 'snes'", [], |row| {
                row.get(0)
            })
            .unwrap();
        conn.execute(
            "INSERT INTO games (platform_id, title, sort_title, rom_path, rom_size, added_at, file_mtime)
             VALUES (?1, 'Test Game', 'test game', '/roms/test.sfc', 1024, 0, 0)",
            [platform_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn delete_save_state_removes_db_row_and_real_files_on_disk() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        let dir = std::env::temp_dir().join("nullbyte_delete_save_state_test");
        std::fs::create_dir_all(&dir).unwrap();
        let state_path = dir.join("0.state");
        let thumb_path = dir.join("0.png");
        std::fs::write(&state_path, b"fake save state bytes").unwrap();
        std::fs::write(&thumb_path, b"fake png bytes").unwrap();

        save_states::upsert_save_state(
            &conn,
            game_id,
            0,
            state_path.to_str().unwrap(),
            Some(thumb_path.to_str().unwrap()),
            "Snes9x",
            "1.62.3",
            1000,
        )
        .unwrap();

        delete_save_state_impl(&conn, game_id, 0).unwrap();

        assert!(
            save_states::get_save_state(&conn, game_id, 0)
                .unwrap()
                .is_none(),
            "DB įrašas turėjo dingti"
        );
        assert!(!state_path.exists(), ".state failas turėjo būti ištrintas");
        assert!(!thumb_path.exists(), ".png failas turėjo būti ištrintas");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_save_state_with_no_thumb_and_missing_files_does_not_error() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        // Sąmoningai NEEGZISTUOJANTYS failai — DB įrašas rodo į kelią, kurio realiai nėra
        // (pvz. rankiniu būdu ištrintas anksčiau) — turi likti idempotentiška, ne klaida.
        save_states::upsert_save_state(
            &conn,
            game_id,
            1,
            "/tmp/nullbyte_definitely_missing_12345.state",
            None,
            "Snes9x",
            "1.62.3",
            1000,
        )
        .unwrap();

        delete_save_state_impl(&conn, game_id, 1).unwrap();

        assert!(save_states::get_save_state(&conn, game_id, 1)
            .unwrap()
            .is_none());
    }

    #[test]
    fn delete_save_state_for_nonexistent_slot_is_a_noop_not_an_error() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);

        // Joks save state'as niekada nebuvo įrašytas šiam slot'ui — turi tiesiog nieko
        // nedaryti, ne klysti.
        delete_save_state_impl(&conn, game_id, 3).unwrap();
    }
}
