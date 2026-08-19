//! Aplikacijos duomenų katalogų sprendimas (CLAUDE.md §4, §6.1).
//!
//! macOS: `~/Library/Application Support/Nullbyte`
//! Linux: `$XDG_DATA_HOME/nullbyte` arba `~/.local/share/nullbyte`

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("Nullbyte MVP palaiko tik macOS ir Linux (CLAUDE.md §11.5)");

use std::path::PathBuf;

use crate::error::AppError;

fn home_dir() -> Result<PathBuf, AppError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::Other("HOME aplinkos kintamasis nenustatytas".into()))
}

/// Šakninis Nullbyte duomenų katalogas.
pub fn data_dir() -> Result<PathBuf, AppError> {
    #[cfg(target_os = "macos")]
    {
        Ok(home_dir()?.join("Library/Application Support/Nullbyte"))
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg).join("nullbyte"));
        }
        Ok(home_dir()?.join(".local/share/nullbyte"))
    }
}

/// Katalogas, kuriame vartotojas laiko libretro core'us (`.dylib` / `.so`).
pub fn cores_dir() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("cores"))
}

/// BIOS / core system failų katalogas (`GET_SYSTEM_DIRECTORY`).
pub fn system_dir() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("system"))
}

/// SRAM (`.srm`) failų katalogas.
pub fn saves_dir() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("saves"))
}

/// Save state (`.state`) failų katalogas.
pub fn states_dir() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("states"))
}

/// ScreenScraper media cache (viršeliai, screenshot'ai, video).
pub fn media_dir() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("media"))
}

/// SQLite duomenų bazės failo kelias.
pub fn db_path() -> Result<PathBuf, AppError> {
    Ok(data_dir()?.join("nullbyte.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_follows_platform_convention() {
        let dir = data_dir().expect("HOME turėtų būti nustatytas testų aplinkoje");

        #[cfg(target_os = "macos")]
        assert!(dir.ends_with("Library/Application Support/Nullbyte"));

        #[cfg(target_os = "linux")]
        assert!(dir.ends_with("nullbyte"));
    }

    #[test]
    fn derived_dirs_nest_under_data_dir() {
        let data = data_dir().unwrap();
        assert_eq!(cores_dir().unwrap(), data.join("cores"));
        assert_eq!(system_dir().unwrap(), data.join("system"));
        assert_eq!(saves_dir().unwrap(), data.join("saves"));
        assert_eq!(states_dir().unwrap(), data.join("states"));
        assert_eq!(media_dir().unwrap(), data.join("media"));
        assert_eq!(db_path().unwrap(), data.join("nullbyte.db"));
    }
}
