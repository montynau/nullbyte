//! Settings Tauri komandos (CLAUDE.md §6.3) — P7.6 Cores panelė.
//!
//! `list_cores` yra PILNAI FUNKCIONALUS (skaito `cores_dir`, jokio P9.1 blokerio — žr.
//! `commands::input` modulio doc dėl to bloko). `get_preferred_cores`/`set_preferred_cores`
//! kenčia nuo TO PATIES apribojimo kaip `commands::input` mapping'as: išsaugoma, bet
//! realaus žaidimo paleidimo dar niekas nenaudoja (P9.1 dar neįgyvendinta).

use tauri::State;

use crate::db::settings;
use crate::error::AppError;
use crate::state::AppState;

/// `nullbyte_core::core::info::CoreInfo` IPC-saugi versija — CLAUDE.md §7.3 (camelCase,
/// `PathBuf` → `String`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfoDto {
    pub path: String,
    pub name: String,
    pub version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub system_name: Option<String>,
    pub manufacturer: Option<String>,
}

impl From<nullbyte_core::core::info::CoreInfo> for CoreInfoDto {
    fn from(info: nullbyte_core::core::info::CoreInfo) -> Self {
        Self {
            path: info.path.to_string_lossy().into_owned(),
            name: info.name,
            version: info.version,
            valid_extensions: info.valid_extensions,
            need_fullpath: info.need_fullpath,
            system_name: info.system_name,
            manufacturer: info.manufacturer,
        }
    }
}

/// Aptinka `cores_dir` esančius libretro core'us (P1.3 `scan_cores_dir` perpanaudotas
/// nepakeistas). Tuščias sąrašas — NE klaida — jei katalogas dar neegzistuoja (naujam
/// diegimui core'ų dar gali nebūti atsisiųsta, žr. MVP.md P1.3 doc).
#[tauri::command]
pub fn list_cores(state: State<'_, AppState>) -> Result<Vec<CoreInfoDto>, AppError> {
    if !state.cores_dir.is_dir() {
        return Ok(Vec::new());
    }
    let cores = nullbyte_core::core::info::scan_cores_dir(&state.cores_dir)?;
    Ok(cores.into_iter().map(CoreInfoDto::from).collect())
}

/// Vieno platformos → core'o priskyrimo įrašas — CLAUDE.md §7.3 (camelCase).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCorePreference {
    pub platform_slug: String,
    pub core_path: String,
}

const PREFERRED_CORES_KEY: &str = "core.preferred";

#[tauri::command]
pub fn get_preferred_cores(
    state: State<'_, AppState>,
) -> Result<Vec<PlatformCorePreference>, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    match settings::get(&conn, PREFERRED_CORES_KEY)? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| AppError::Other(format!("sugadintas core.preferred JSON: {error}"))),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn set_preferred_cores(
    state: State<'_, AppState>,
    preferences: Vec<PlatformCorePreference>,
) -> Result<(), AppError> {
    let json = serde_json::to_string(&preferences).map_err(|error| {
        AppError::Other(format!("nepavyko serializuoti core.preferred: {error}"))
    })?;
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, PREFERRED_CORES_KEY, &json)
}
