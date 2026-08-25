//! Aplikacijos globalus būvis, laikomas kaip Tauri managed state.
//!
//! Laiko išspręstus duomenų katalogus ir DB ryšį. `nullbyte-emu` vaiko proceso rankena
//! (`crate::ipc::EmuClient`) — P9.1, kai bus realus žaidimo paleidimo srautas (žr.
//! `crate::commands` modulio doc dėl P2.3-eros lokalaus `Renderer`/`EmuThread` pašalinimo
//! P4.0.3 metu — ADR-016 juos perkėlė į atskirą procesą).

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::migrations;
use crate::error::AppError;
use crate::paths;

pub struct AppState {
    pub data_dir: PathBuf,
    pub cores_dir: PathBuf,
    pub system_dir: PathBuf,
    pub saves_dir: PathBuf,
    pub states_dir: PathBuf,
    pub media_dir: PathBuf,
    pub db_path: PathBuf,
    /// `rusqlite::Connection` NĖRA `Sync` (CLAUDE.md §10 „SQLite") — `Mutex<Connection>` MVP
    /// metu pakanka (vienas ryšys, ne pool'as; `r2d2_sqlite` — post-MVP, jei tikrai reikės).
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let db_path = paths::db_path()?;
        let db = migrations::open_and_migrate(&db_path)?;

        Ok(Self {
            data_dir: paths::data_dir()?,
            cores_dir: paths::cores_dir()?,
            system_dir: paths::system_dir()?,
            saves_dir: paths::saves_dir()?,
            states_dir: paths::states_dir()?,
            media_dir: paths::media_dir()?,
            db_path,
            db: Mutex::new(db),
        })
    }
}
