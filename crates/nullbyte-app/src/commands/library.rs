//! Bibliotekos Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į `db::games`.

use tauri::State;

use crate::db::games::{self, GameFilter, PlatformSummary};
use crate::db::models::Game;
use crate::error::AppError;
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
