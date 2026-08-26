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

/// P9.1: konkretaus žaidimo save state'ų katalogas — siunčiamas `nullbyte-emu`'ui kaip
/// `EmuCommand::Load.states_dir` (žr. jo doc, P8.1). Raktas — `game_id`, NE ROM'o failo
/// vardas: keli žaidimai skirtinguose kataloguose gali turėti TĄ PATĮ failo vardą
/// (`game.sfc` dviejose skirtingose kolekcijose), o `game_id` yra garantuotai unikalus ir
/// stabilus (DB primary key).
pub fn game_states_dir(game_id: i64) -> Result<PathBuf, AppError> {
    Ok(states_dir()?.join(game_id.to_string()))
}

/// P9.1: konkretaus žaidimo SRAM (`.srm`) failo kelias — siunčiamas kaip
/// `EmuCommand::Load.sram_path` (žr. jo doc, P8.2). Tas pats `game_id`-raktas argumentas
/// kaip [`game_states_dir`] — MVP.md P8.2 juodraštis siūlė `{rom_basename}.srm`, bet tai
/// turėtų TĄ PAČIĄ kolizijos riziką, tad sąmoningai naudojame `game_id`.
pub fn game_sram_path(game_id: i64) -> Result<PathBuf, AppError> {
    Ok(saves_dir()?.join(format!("{game_id}.srm")))
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

    #[test]
    fn per_game_paths_nest_under_states_and_saves_dir_by_id() {
        assert_eq!(
            game_states_dir(42).unwrap(),
            states_dir().unwrap().join("42")
        );
        assert_eq!(
            game_sram_path(42).unwrap(),
            saves_dir().unwrap().join("42.srm")
        );
    }
}
