//! Core metaduomenys: `retro_get_system_info()` + libretro `.info` failų parsinimas
//! (CLAUDE.md §3.1, Faza 1 P1.3).

// Naudos commands/settings.rs (list_cores) ir library/scanner.rs (P5.3) — kol jie
// neparašyti, šis modulis pilnai išnaudojamas tik testuose.
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::AppError;

use super::loader::CoreHandle;

/// Vieno core'o metaduomenys — `retro_get_system_info()` laukai + papildoma info iš
/// gretimo `.info` failo (jei toks yra; jo nebuvimas nėra klaida).
#[derive(Debug, Clone)]
pub struct CoreInfo {
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub block_extract: bool,
    pub system_name: Option<String>,
    pub manufacturer: Option<String>,
    pub categories: Option<String>,
    pub database: Option<String>,
}

/// Nuskaito katalogą, aptinka visus `*_libretro.dylib` / `*_libretro.so` failus. Kiekvienas
/// trumpam įkeliamas (`CoreHandle::load`), kad gautume `retro_get_system_info()`, tada
/// iškart atlaisvinamas (žr. `CoreHandle::Drop`) — vienu metu įkeltas tik vienas core'as
/// (CLAUDE.md §3.2 taisyklė #2).
///
/// Core'ai, kurių nepavyksta įkelti (bloga architektūra, sugadintas failas ir pan.), yra
/// praleidžiami su `tracing::warn!`, o ne nutraukia visą skenavimą.
pub fn scan_cores_dir(dir: impl AsRef<Path>) -> Result<Vec<CoreInfo>, AppError> {
    let dir = dir.as_ref();
    let mut cores = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| {
        AppError::Other(format!(
            "nepavyko skaityti core'ų katalogo {}: {e}",
            dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|e| AppError::Other(format!("klaida skenuojant {}: {e}", dir.display())))?;
        let path = entry.path();

        if !is_core_file(&path) {
            continue;
        }

        match load_core_info(&path) {
            Ok(info) => cores.push(info),
            Err(error) => {
                tracing::warn!(core = %path.display(), %error, "nepavyko įkelti core'o — praleidžiama");
            }
        }
    }

    Ok(cores)
}

fn is_core_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name.ends_with("_libretro.dylib") || name.ends_with("_libretro.so")
}

fn load_core_info(path: &Path) -> Result<CoreInfo, AppError> {
    let handle = CoreHandle::load(path)?;
    let raw = handle.system_info();

    let valid_extensions = raw
        .valid_extensions
        .split('|')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let meta = parse_info_file(&info_file_path(path));

    Ok(CoreInfo {
        path: path.to_path_buf(),
        name: raw.library_name,
        version: raw.library_version,
        valid_extensions,
        need_fullpath: raw.need_fullpath,
        block_extract: raw.block_extract,
        system_name: meta.as_ref().and_then(|m| m.get("systemname").cloned()),
        manufacturer: meta.as_ref().and_then(|m| m.get("manufacturer").cloned()),
        categories: meta.as_ref().and_then(|m| m.get("categories").cloned()),
        database: meta.as_ref().and_then(|m| m.get("database").cloned()),
    })
}

/// `snes9x_libretro.dylib` → `snes9x_libretro.info` (libretro standartas — `.info` failas
/// guli greta core'o, tą patį bazinį vardą, tik su `.info` plėtiniu).
fn info_file_path(core_path: &Path) -> PathBuf {
    core_path.with_extension("info")
}

/// Parsina paprastą `key = "value"` formatą, kurį naudoja libretro `.info` failai.
/// Grąžina `None`, jei failo nėra arba jo nepavyksta perskaityti — tai NĖRA klaida
/// (P1.3 acceptance: „.info failo nebuvimas nesulaužo skenavimo").
fn parse_info_file(path: &Path) -> Option<HashMap<String, String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        map.insert(
            key.trim().to_string(),
            value.trim().trim_matches('"').to_string(),
        );
    }

    Some(map)
}

/// Sudaro plėtinio → core'ų pavadinimų mapping'ą iš jau nuskaitytų core'ų sąrašo
/// (pvz. `"sfc" → ["Snes9x", "Snes9x - Current"]`).
pub fn extension_to_cores(cores: &[CoreInfo]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for core in cores {
        for ext in &core.valid_extensions {
            map.entry(ext.clone()).or_default().push(core.name.clone());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture katalogas su 5 realiais (dubliuotais) core'ais + vienu `.info` failu —
    /// paruoštas rankiniu būdu iš `src-tauri/cores/` (žr. sesijos pastabas). Visas
    /// `src-tauri/cores/` yra `.gitignore`'intas (CLAUDE.md §11.2), tad CI aplinkoje šio
    /// katalogo nėra — testas praleidžiamas švelniai, jei jo nerandama.
    fn fixture_dir() -> Option<PathBuf> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/scan_fixture");
        dir.is_dir().then_some(dir)
    }

    #[test]
    fn scans_directory_with_five_plus_cores() {
        let Some(dir) = fixture_dir() else {
            eprintln!("praleista: src-tauri/cores/scan_fixture nerastas (lokalus fixture)");
            return;
        };

        let cores = scan_cores_dir(&dir).expect("skenavimas neturėtų klaidos");
        assert!(
            cores.len() >= 5,
            "tikėtasi bent 5 core'ų, gauta {}",
            cores.len()
        );

        for core in &cores {
            assert!(
                !core.name.is_empty(),
                "core'o pavadinimas neturėtų būti tuščias"
            );
            assert!(
                !core.valid_extensions.is_empty(),
                "core'as {} turėtų turėti bent vieną plėtinį",
                core.name
            );
        }
    }

    #[test]
    fn missing_info_file_does_not_break_scan() {
        let Some(dir) = fixture_dir() else {
            eprintln!("praleista: src-tauri/cores/scan_fixture nerastas (lokalus fixture)");
            return;
        };

        // Bent vienas fixture core'as sąmoningai neturi gretimo .info failo.
        let cores = scan_cores_dir(&dir).expect("skenavimas neturėtų klaidos");
        let without_info = cores.iter().any(|c| c.system_name.is_none());
        assert!(
            without_info,
            "fixture turėtų turėti bent vieną core'ą be .info failo"
        );
    }

    #[test]
    fn parses_info_file_fields() {
        let Some(dir) = fixture_dir() else {
            eprintln!("praleista: src-tauri/cores/scan_fixture nerastas (lokalus fixture)");
            return;
        };

        let cores = scan_cores_dir(&dir).expect("skenavimas neturėtų klaidos");
        let with_info = cores.iter().find(|c| c.system_name.is_some());
        assert!(
            with_info.is_some(),
            "fixture turėtų turėti bent vieną core'ą su .info failu"
        );
    }

    #[test]
    fn builds_extension_to_cores_mapping() {
        let Some(dir) = fixture_dir() else {
            eprintln!("praleista: src-tauri/cores/scan_fixture nerastas (lokalus fixture)");
            return;
        };

        let cores = scan_cores_dir(&dir).expect("skenavimas neturėtų klaidos");
        let mapping = extension_to_cores(&cores);
        assert!(!mapping.is_empty(), "mapping neturėtų būti tuščias");
        for (ext, core_names) in &mapping {
            assert!(!ext.is_empty());
            assert!(!core_names.is_empty());
        }
    }
}
