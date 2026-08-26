mod commands;
mod db;
mod error;
mod ipc;
mod library;
mod paths;
mod scraper;
mod state;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use state::AppState;

/// Inicializuoja `tracing`: stdout + rotuojantis failas `data_dir()/logs/`.
///
/// `RUST_LOG` aplinkos kintamasis valdo lygį (pvz. `RUST_LOG=nullbyte=debug`); jei nenustatytas,
/// numatytasis lygis — `info` mūsų pačių kodui, tyliau trečiųjų šalių bibliotekoms.
///
/// Grąžintas `WorkerGuard` privalo gyventi tol, kol veikia programa — jį numetus, failo
/// writer'io fone veikianti gija sustoja ir likę log'ai gali neišsirašyti (CLAUDE.md §14 ADR-010).
fn init_logging(log_dir: &std::path::Path) -> tracing_appender::non_blocking::WorkerGuard {
    std::fs::create_dir_all(log_dir).expect("nepavyko sukurti log katalogo");

    let file_appender = tracing_appender::rolling::daily(log_dir, "nullbyte.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,nullbyte_lib=debug"));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .with(env_filter)
        .init();

    guard
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    platform: String,
    data_dir: String,
    cores_dir: String,
    system_dir: String,
    saves_dir: String,
    states_dir: String,
    media_dir: String,
    db_path: String,
}

/// Grąžina versiją, platformą ir išspręstus katalogus — naudinga UI Nustatymų ekrane
/// ir debug'inant (CLAUDE.md §5 P0.5).
#[tauri::command]
fn get_app_info(state: tauri::State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        data_dir: state.data_dir.display().to_string(),
        cores_dir: state.cores_dir.display().to_string(),
        system_dir: state.system_dir.display().to_string(),
        saves_dir: state.saves_dir.display().to_string(),
        states_dir: state.states_dir.display().to_string(),
        media_dir: state.media_dir.display().to_string(),
        db_path: state.db_path.display().to_string(),
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new().expect("nepavyko nustatyti duomenų katalogų");
    let log_dir = app_state.data_dir.join("logs");
    let _log_guard = init_logging(&log_dir);

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Nullbyte paleidžiamas");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_info,
            commands::library::list_games,
            commands::library::get_game,
            commands::library::set_favorite,
            commands::library::record_play,
            commands::emulator::start_game,
            commands::emulator::stop_game,
            commands::emulator::is_game_running,
            commands::library::list_platforms,
            commands::library::list_rom_directories,
            commands::library::add_rom_directory,
            commands::library::remove_rom_directory,
            commands::library::scan_library,
            commands::scraper::scrape_game,
            commands::scraper::scrape_library,
            commands::scraper::cancel_scrape,
            commands::scraper::get_scraper_status,
            commands::scraper::get_scraper_quota,
            commands::scraper::set_scraper_credentials,
            commands::scraper::clear_scraper_credentials,
            commands::input::get_input_mapping,
            commands::input::set_input_mapping,
            commands::input::reset_input_mapping,
            commands::settings::list_cores,
            commands::settings::get_preferred_cores,
            commands::settings::set_preferred_cores,
            commands::settings::get_core_priority,
            commands::settings::get_video_settings,
            commands::settings::set_video_settings,
            commands::settings::get_audio_settings,
            commands::settings::set_audio_settings,
            commands::settings::list_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
