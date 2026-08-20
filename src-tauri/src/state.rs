//! Aplikacijos globalus būvis, laikomas kaip Tauri managed state.
//!
//! Laiko išspręstus duomenų katalogus ir (nuo P2.3) emuliatoriaus lango `Renderer`.
//! DB pool'as prisijungs P5.1.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::AppError;
use crate::paths;
use crate::video::renderer::Renderer;

pub struct AppState {
    pub data_dir: PathBuf,
    pub cores_dir: PathBuf,
    pub system_dir: PathBuf,
    pub saves_dir: PathBuf,
    pub states_dir: PathBuf,
    pub media_dir: PathBuf,
    pub db_path: PathBuf,
    /// Emuliatoriaus lango wgpu būvis — `None`, kol `open_emulator_window` komanda
    /// dar nebuvo kviesta (P2.3).
    pub renderer: Mutex<Option<Renderer>>,
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
            renderer: Mutex::new(None),
        })
    }
}
