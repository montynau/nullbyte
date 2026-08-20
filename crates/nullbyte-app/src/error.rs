//! Bendras klaidų tipas, kertantis Tauri IPC ribą (žr. CLAUDE.md §6.1).
//!
//! `Core` variantas apgaubia `nullbyte_core::error::CoreError` — leidžia `?` operatoriui
//! veikti skambinant iš `nullbyte-app` į `nullbyte-core` funkcijas (pvz. `Renderer::new`,
//! `AudioOutput::open`), nekeičiant CLAUDE.md §6.1 „vieno klaidų tipo" taisyklės kiekvieno
//! crate'o viduje (P4.0.1, ADR-016 — `nullbyte-core` negali priklausyti nuo `rusqlite`/
//! `reqwest`, tad negali dalintis šiuo pačiu `AppError` tipu).

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O klaida: {0}")]
    Io(#[from] std::io::Error),

    #[error("duomenų bazės klaida: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("tinklo klaida: {0}")]
    Network(#[from] reqwest::Error),

    #[error("branduolio klaida: {0}")]
    Core(#[from] nullbyte_core::error::CoreError),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io",
            AppError::Database(_) => "database",
            AppError::Network(_) => "network",
            AppError::Core(_) => "core",
            AppError::Other(_) => "other",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
