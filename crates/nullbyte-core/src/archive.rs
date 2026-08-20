//! Suarchyvuotų ROM'ų skaitymas — `.zip` / `.7z` (CLAUDE.md §3.1, P1.6).
//!
//! Ieško archyve pirmo failo, kurio plėtinys sutampa su core'o `valid_extensions`, ir grąžina
//! jo turinį atmintyje (naudojama P1.6 `core::loader::load_game`).

use std::io::Read;
use std::path::Path;

use sevenz_rust::{Password, SevenZReader};
use zip::ZipArchive;

use crate::error::CoreError;

/// Randa pirmą archyvo viduje esantį failą, kurio plėtinys yra tarp `valid_extensions`
/// (be taško, nepriklausomai nuo registro, pvz. `["sfc", "smc"]`), ir grąžina jo bazinį
/// pavadinimą bei turinį atmintyje.
pub fn extract_first_match(
    archive_path: &Path,
    valid_extensions: &[String],
) -> Result<(String, Vec<u8>), CoreError> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    match ext.as_deref() {
        Some("zip") => extract_from_zip(archive_path, valid_extensions),
        Some("7z") => extract_from_7z(archive_path, valid_extensions),
        _ => Err(CoreError::Other(format!(
            "nepalaikomas archyvo formatas: {}",
            archive_path.display()
        ))),
    }
}

/// Kaip [`extract_first_match`], bet išpakuoja į laikiną failą — naudojama `need_fullpath`
/// core'ams, kurie žaidimą atsidaro patys pagal kelią, ne per atminties buferį.
pub fn extract_first_match_to_temp(
    archive_path: &Path,
    valid_extensions: &[String],
) -> Result<std::path::PathBuf, CoreError> {
    let (name, data) = extract_first_match(archive_path, valid_extensions)?;
    let temp_dir = std::env::temp_dir().join("nullbyte");
    std::fs::create_dir_all(&temp_dir)?;
    let temp_path = temp_dir.join(name);
    std::fs::write(&temp_path, &data)?;
    Ok(temp_path)
}

fn has_valid_extension(name: &str, valid_extensions: &[String]) -> bool {
    let Some(ext) = Path::new(name).extension().and_then(|e| e.to_str()) else {
        return false;
    };
    let ext = ext.to_lowercase();
    valid_extensions.iter().any(|v| v.to_lowercase() == ext)
}

fn base_name(name: &str) -> String {
    Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name)
        .to_string()
}

fn no_match_error(archive_path: &Path, valid_extensions: &[String]) -> CoreError {
    CoreError::Other(format!(
        "archyve {} nerasta tinkamo failo (laukiami plėtiniai: {})",
        archive_path.display(),
        valid_extensions.join(", ")
    ))
}

fn extract_from_zip(
    path: &Path,
    valid_extensions: &[String],
) -> Result<(String, Vec<u8>), CoreError> {
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| CoreError::Other(format!("nepavyko atidaryti zip {}: {e}", path.display())))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CoreError::Other(format!("zip skaitymo klaida: {e}")))?;

        if entry.is_dir() || !has_valid_extension(entry.name(), valid_extensions) {
            continue;
        }

        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data)?;
        return Ok((base_name(entry.name()), data));
    }

    Err(no_match_error(path, valid_extensions))
}

fn extract_from_7z(
    path: &Path,
    valid_extensions: &[String],
) -> Result<(String, Vec<u8>), CoreError> {
    let mut reader = SevenZReader::open(path, Password::empty())
        .map_err(|e| CoreError::Other(format!("nepavyko atidaryti 7z {}: {e}", path.display())))?;

    let mut result: Option<(String, Vec<u8>)> = None;
    reader
        .for_each_entries(|entry, entry_reader| {
            if result.is_some()
                || entry.is_directory()
                || !has_valid_extension(entry.name(), valid_extensions)
            {
                return Ok(true); // tęsti kitiems įrašams
            }
            let mut data = Vec::new();
            entry_reader.read_to_end(&mut data)?;
            result = Some((base_name(entry.name()), data));
            Ok(true)
        })
        .map_err(|e| CoreError::Other(format!("7z skaitymo klaida: {e}")))?;

    result.ok_or_else(|| no_match_error(path, valid_extensions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);
            for (name, data) in entries {
                writer
                    .start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(data).unwrap();
            }
            writer.finish().unwrap();
        }
        buf
    }

    #[test]
    fn extracts_first_matching_file_from_zip() {
        let zip_bytes = make_zip(&[
            ("readme.txt", b"not a rom"),
            ("Super Game.sfc", b"fake snes rom bytes"),
        ]);
        let path = std::env::temp_dir().join("nullbyte_test_extract.zip");
        std::fs::write(&path, &zip_bytes).unwrap();

        let extensions = vec!["sfc".to_string(), "smc".to_string()];
        let (name, data) = extract_first_match(&path, &extensions).expect("turėtų rasti .sfc");

        assert_eq!(name, "Super Game.sfc");
        assert_eq!(data, b"fake snes rom bytes");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn missing_extension_in_zip_returns_clear_error() {
        let zip_bytes = make_zip(&[("readme.txt", b"not a rom")]);
        let path = std::env::temp_dir().join("nullbyte_test_no_match.zip");
        std::fs::write(&path, &zip_bytes).unwrap();

        let extensions = vec!["sfc".to_string()];
        let result = extract_first_match(&path, &extensions);
        assert!(result.is_err());

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn unsupported_archive_extension_returns_error() {
        let path = Path::new("/tmp/nullbyte_test.rar");
        let result = extract_first_match(path, &["sfc".to_string()]);
        assert!(result.is_err());
    }
}
