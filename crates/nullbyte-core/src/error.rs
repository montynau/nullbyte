//! `nullbyte-core`'o klaidų tipas — naudojamas IR `nullbyte-emu` (vaiko procese), IR
//! `nullbyte-app` (kuri savo `AppError`'yje turi `Core(#[from] CoreError)` variantą, žr.
//! CLAUDE.md §4/ADR-016). Sąmoningai NETURI `rusqlite`/`reqwest` variantų — šis crate'as
//! nepriklauso nuo nei vieno, tiktai `nullbyte-app` (DB/scraper) tai daro.
//!
//! **Nuo P4.0.3:** `CoreError` kerta `nullbyte-emu` ↔ `nullbyte-app` IPC ribą TIESIOGIAI,
//! struktūriškai (`EmuStatus::Error(CoreError)`, žr. `crate::ipc`) — NE kaip suplokštinta
//! `{kind, message}` eilutė. Todėl turi pilną `Serialize`/`Deserialize` (apverčiamą), ne
//! rankinį suplokštinantį `impl Serialize`. Suplokštinimas `{kind, message}` UI kontraktui
//! (CLAUDE.md §6.1) vyksta TIK ties `nullbyte-app::error::AppError` → Tauri frontend riba
//! (`AppError::kind()` deleguoja į `CoreError::kind()` Core variantui) — vienintelė vieta,
//! kur struktūra iš tikrųjų nebereikalinga (JS pusė gauna tik string/string).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CoreError {
    /// `std::io::Error` pati neturi `Serialize`/`Deserialize` (gali nešti platform-specifinį
    /// OS klaidos kodą arba savavališką `Box<dyn Error>`) — `io_error_wire` žemiau paverčia
    /// ją į paprastą tekstą IPC laidui. Lauko TIPAS lieka `std::io::Error`, kad `#[from]`
    /// (taigi ir `?` visuose esamuose call site'uose `loader.rs`/`archive.rs`) veiktų
    /// nepakitęs — keičiasi tik SERIALIZACIJOS būdas šiam vienam laukui.
    #[error("I/O klaida: {0}")]
    Io(
        #[from]
        #[serde(with = "io_error_wire")]
        std::io::Error,
    ),

    /// Core'as (`.dylib`/`.so`) nepavyko įkelti — arba `Library::new` (failas neegzistuoja,
    /// negalioja kaip dinaminė biblioteka), arba jame trūksta privalomo libretro simbolio
    /// (CLAUDE.md §8.1). UI turėtų nurodyti vartotojui patikrinti core'o failą.
    #[error("nepavyko įkelti core'o {path}: {reason}")]
    CoreLoad { path: PathBuf, reason: String },

    /// `retro_api_version()` negrąžino tikėtos reikšmės (CLAUDE.md §8.2 žingsnis 2) —
    /// core'as nesuderinamas su libretro API, kurį palaiko Nullbyte.
    #[error("core'as {path} turi nesuderinamą API versiją {actual} (tikimasi {expected})")]
    ApiVersion {
        path: PathBuf,
        expected: u32,
        actual: u32,
    },

    /// `retro_load_game()` grąžino `false` — core'as pats atmetė ROM'ą (blogas failas,
    /// nesuderinamas regionas, trūksta priklausomybių ir pan.).
    #[error("core'as atmetė ROM'ą: {}", rom_path.display())]
    RomLoad { rom_path: PathBuf },

    /// Core'ui reikalingas BIOS failas nerastas system directory (pvz. PSX `scph5501.bin`).
    #[error("core'ui {core} trūksta BIOS failo: {bios_file}")]
    MissingBios { core: String, bios_file: String },

    /// Core'as (per `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT`) paprašė formato, kurio Nullbyte
    /// nekonvertuoja (palaikom tik `0RGB1555`/`XRGB8888`/`RGB565`, CLAUDE.md §8.4).
    #[error("core'as {core} prašo nepalaikomo pixel format ({format})")]
    UnsupportedPixelFormat { core: String, format: u32 },

    /// `retro_serialize`/`retro_unserialize` klaida (CLAUDE.md §8.7) — save state'as
    /// nesuderinamas su dabartine core versija arba serializacija/deserializacija nepavyko.
    #[error("save state klaida: {0}")]
    SaveState(String),

    /// SRAM (in-game save, CLAUDE.md §8.8) skaitymo/rašymo klaida — ATSKIRAI nuo `SaveState`,
    /// nes semantiškai skirtinga operacija (retro_get_memory_data/size, ne retro_serialize).
    #[error("SRAM klaida: {0}")]
    Sram(String),

    #[error("{0}")]
    Other(String),
}

impl CoreError {
    /// Stabilus, mašinai skaitomas kategorijos vardas — naudoja
    /// `nullbyte-app::error::AppError::kind()` suplokštinant `{kind, message}` ties Tauri
    /// frontend riba (žr. modulio doc). `pub`, nes kviečiama iš kito crate'o.
    pub fn kind(&self) -> &'static str {
        match self {
            CoreError::Io(_) => "io",
            CoreError::CoreLoad { .. } => "core_load",
            CoreError::ApiVersion { .. } => "api_version",
            CoreError::RomLoad { .. } => "rom_load",
            CoreError::MissingBios { .. } => "missing_bios",
            CoreError::UnsupportedPixelFormat { .. } => "unsupported_pixel_format",
            CoreError::SaveState(_) => "save_state",
            CoreError::Sram(_) => "sram",
            CoreError::Other(_) => "other",
        }
    }
}

/// `serde(with = ...)` shim vieninteliam `CoreError::Io` laukui — žr. jo doc komentarą.
/// `io::ErrorKind` (kaip ir pats `io::Error`) neturi serde impl'ų standartinėje bibliotekoje,
/// tad round-trip'inam tik pranešimo tekstą (`ErrorKind::Other` po deserializacijos — niekas
/// šiame kode nesišakoja pagal konkretų `ErrorKind` po IPC ribos, tik pagal `CoreError::kind()`).
mod io_error_wire {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::io;

    pub fn serialize<S: Serializer>(err: &io::Error, serializer: S) -> Result<S::Ok, S::Error> {
        err.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<io::Error, D::Error> {
        String::deserialize(deserializer).map(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(err: CoreError) -> CoreError {
        let json = serde_json::to_string(&err).expect("CoreError turėtų serializuotis");
        serde_json::from_str(&json).expect("CoreError turėtų deserializuotis atgal")
    }

    #[test]
    fn core_load_roundtrips_with_all_fields_and_kind() {
        let original = CoreError::CoreLoad {
            path: PathBuf::from("/cores/snes9x_libretro.dylib"),
            reason: "trūksta simbolio `retro_run`".to_string(),
        };
        let restored = roundtrip(original);
        assert_eq!(restored.kind(), "core_load");
        match restored {
            CoreError::CoreLoad { path, reason } => {
                assert_eq!(path, PathBuf::from("/cores/snes9x_libretro.dylib"));
                assert_eq!(reason, "trūksta simbolio `retro_run`");
            }
            other => panic!("tikėtasi CoreLoad, gauta {other:?}"),
        }
    }

    #[test]
    fn api_version_roundtrips_numeric_fields() {
        let original = CoreError::ApiVersion {
            path: PathBuf::from("/cores/old_core.dylib"),
            expected: 1,
            actual: 2,
        };
        let restored = roundtrip(original);
        assert_eq!(restored.kind(), "api_version");
        match restored {
            CoreError::ApiVersion {
                expected, actual, ..
            } => {
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("tikėtasi ApiVersion, gauta {other:?}"),
        }
    }

    #[test]
    fn io_variant_roundtrips_message_via_shim() {
        let original: CoreError =
            std::io::Error::new(std::io::ErrorKind::NotFound, "nerastas failas").into();
        let restored = roundtrip(original);
        assert_eq!(restored.kind(), "io");
        assert!(
            restored.to_string().contains("nerastas failas"),
            "pranešimas turėtų išlikti per shim'ą, gauta: {restored}"
        );
    }

    #[test]
    fn all_variants_have_distinct_kind() {
        let variants = [
            CoreError::Io(std::io::Error::other("x")),
            CoreError::CoreLoad {
                path: PathBuf::new(),
                reason: String::new(),
            },
            CoreError::ApiVersion {
                path: PathBuf::new(),
                expected: 0,
                actual: 0,
            },
            CoreError::RomLoad {
                rom_path: PathBuf::new(),
            },
            CoreError::MissingBios {
                core: String::new(),
                bios_file: String::new(),
            },
            CoreError::UnsupportedPixelFormat {
                core: String::new(),
                format: 0,
            },
            CoreError::SaveState(String::new()),
            CoreError::Sram(String::new()),
            CoreError::Other(String::new()),
        ];
        let kinds: std::collections::HashSet<_> = variants.iter().map(CoreError::kind).collect();
        assert_eq!(
            kinds.len(),
            variants.len(),
            "kiekvienas variantas turi turėti unikalų kind()"
        );
    }
}
