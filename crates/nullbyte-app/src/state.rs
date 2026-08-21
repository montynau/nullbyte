//! Aplikacijos globalus būvis, laikomas kaip Tauri managed state.
//!
//! Laiko išspręstus duomenų katalogus. DB pool'as prisijungs P5.1; `nullbyte-emu` vaiko
//! proceso rankena (`crate::ipc::EmuClient`) — P9.1, kai bus realus žaidimo paleidimo
//! srautas (žr. `crate::commands` modulio doc dėl P2.3-eros lokalaus `Renderer`/`EmuThread`
//! pašalinimo P4.0.3 metu — ADR-016 juos perkėlė į atskirą procesą).

use std::path::PathBuf;

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
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            data_dir: paths::data_dir()?,
            cores_dir: paths::cores_dir()?,
            system_dir: paths::system_dir()?,
            saves_dir: paths::saves_dir()?,
            states_dir: paths::states_dir()?,
            media_dir: paths::media_dir()?,
            db_path: paths::db_path()?,
        })
    }
}
