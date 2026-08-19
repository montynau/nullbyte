//! Aplikacijos globalus būvis, laikomas kaip Tauri managed state.
//!
//! Kol kas laiko tik išspręstus duomenų katalogus; DB pool'as ir emu handle prisijungs
//! vėlesnėse fazėse (P5.1 DB, P1.7 emuliavimo gija).

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
