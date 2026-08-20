//! `nullbyte-core`'o klaidų tipas — naudojamas IR `nullbyte-emu` (vaiko procese), IR
//! `nullbyte-app` (kuri savo `AppError`'yje turi `Core(#[from] CoreError)` variantą, žr.
//! CLAUDE.md §4/ADR-016). Sąmoningai NETURI `rusqlite`/`reqwest` variantų — šis crate'as
//! nepriklauso nuo nei vieno, tiktai `nullbyte-app` (DB/scraper) tai daro.

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("I/O klaida: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl CoreError {
    fn kind(&self) -> &'static str {
        match self {
            CoreError::Io(_) => "io",
            CoreError::Other(_) => "other",
        }
    }
}

impl Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CoreError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
