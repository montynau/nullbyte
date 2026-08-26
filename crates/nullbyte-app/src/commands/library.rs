//! Bibliotekos Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į `db::games`
//! (žaidimai/platformos) ir `db::rom_directories` + `library::scanner` (ROM katalogai/skenavimas,
//! MVP.md P7.5).

use tauri::ipc::Channel;
use tauri::State;

use crate::db::games::{self, GameFilter, PlatformSummary};
use crate::db::models::{Game, RomDirectory};
use crate::db::rom_directories;
use crate::error::AppError;
use crate::library::scanner::{self, ScanProgress, ScanSummary};
use crate::state::AppState;

#[tauri::command]
pub fn list_games(state: State<'_, AppState>, filter: GameFilter) -> Result<Vec<Game>, AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    games::list_games(&conn, &filter)
}

#[tauri::command]
pub fn get_game(state: State<'_, AppState>, id: i64) -> Result<Option<Game>, AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    games::get_game(&conn, id)
}

#[tauri::command]
pub fn set_favorite(state: State<'_, AppState>, id: i64, favorite: bool) -> Result<(), AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    games::set_favorite(&conn, id, favorite)
}

#[tauri::command]
pub fn record_play(state: State<'_, AppState>, id: i64, seconds: i64) -> Result<(), AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    games::record_play(&conn, id, seconds)
}

#[tauri::command]
pub fn list_platforms(state: State<'_, AppState>) -> Result<Vec<PlatformSummary>, AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    games::list_platforms(&conn)
}

#[tauri::command]
pub fn list_rom_directories(state: State<'_, AppState>) -> Result<Vec<RomDirectory>, AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    rom_directories::list_rom_directories(&conn)
}

#[tauri::command]
pub fn add_rom_directory(
    state: State<'_, AppState>,
    path: String,
    recursive: bool,
) -> Result<RomDirectory, AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    rom_directories::add_rom_directory(&conn, &path, recursive)
}

#[tauri::command]
pub fn remove_rom_directory(state: State<'_, AppState>, id: i64) -> Result<(), AppError> {
    let conn = state.db.lock().expect("DB Mutex poisoned");
    rom_directories::remove_rom_directory(&conn, id)
}

/// Nuskenuoja visus įjungtus ROM katalogus — MVP.md P7.5 „Skenuoti" mygtukas.
#[tauri::command]
pub fn scan_library(
    state: State<'_, AppState>,
    progress: Channel<ScanProgress>,
) -> Result<ScanSummary, AppError> {
    let mut conn = state.db.lock().expect("DB Mutex poisoned");
    scanner::scan(&mut conn, move |update| {
        if let Err(error) = progress.send(update) {
            tracing::warn!(%error, "nepavyko išsiųsti skenavimo progreso į UI");
        }
    })
}
