//! ROM hash'avimas — CRC32/MD5/SHA1 vienu perėjimu (MVP.md P5.2).
//!
//! Archyvams (`.zip`/`.7z`) hash'uojamas VIDINIS failas (CLAUDE.md §9.1 „Suarchyvuotiems
//! ROM'ams hash'uok vidinį failą, ne archyvą") — naudoja `nullbyte_core::archive`, tą patį
//! modulį, kurį naudoja `core::loader` (žr. `library` modulio doc dėl kodėl jis gyvena
//! `nullbyte-core`, ne čia). Archyvo atveju STREAMING NEĮGYVENDINTAS — `extract_first_match`
//! jau skaito visą vidinį failą į atmintį (esamos `nullbyte_core::archive` API riba, bendra
//! su core'o krovimu); MVP.md P5.2 „Failams > 64 MB: streaming" reikalavimas taikomas
//! NEARCHYVUOTIEMS failams (dažniausias didelio failo atvejis — CD/DVD atvaizdai be archyvo).
//!
//! **Header skip:** įgyvendintas NES (iNES) atvejis — MAGIC baitai `4E 45 53 1A` failo
//! pradžioje identifikuoja iNES formatą NEPRIKLAUSOMAI nuo plėtinio, tad header'io
//! aptikimas veikia be jokios platformos metaduomenų (MVP.md P5.2 „implementuok bent NES
//! atvejį"). SNES copier header'is (512 baitų) NEĮGYVENDINTAS — jis neturi patikimo magic
//! baito, tad reikalautų platformos konteksto per funkcijos parametrus; atidėta iki tikrai
//! prireiks (dauguma šiuolaikinių SNES ROM'ų platinami BE copier header'io).

#![allow(dead_code)] // pilnai išnaudos P5.3 skeneris

use std::io::Read;

use md5::{Digest, Md5};
use sha1::Sha1;

use nullbyte_core::archive;

use crate::error::AppError;

/// Failų, didesnių už šią ribą, hash'avimas vyksta streaming'u (skaitoma 1 MB gabalais), o
/// ne visas failas iškart į atmintį.
const STREAMING_THRESHOLD_BYTES: u64 = 64 * 1024 * 1024;
const STREAM_CHUNK_SIZE: usize = 1024 * 1024;

const INES_MAGIC: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const INES_HEADER_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomHashes {
    /// 8 hex simboliai, DIDŽIOSIOS raidės (retro-scene/ScreenScraper konvencija).
    pub crc32: String,
    /// Mažosios raidės — sutampa su `md5sum` komandinės eilutės išvestimi.
    pub md5: String,
    /// Mažosios raidės — sutampa su `sha1sum` komandinės eilutės išvestimi.
    pub sha1: String,
    /// Realiai hash'uotų baitų kiekis (PO archyvo išpakavimo IR header skip'o) —
    /// `games.rom_size`/ScreenScraper `romtaille` parametrui (CLAUDE.md §9.1).
    pub size: u64,
    /// Archyvo VIDINIO failo pavadinimas (`games.archive_inner`), `None` neaarchyvuotiems
    /// ROM'ams. Naudoja P5.3 skeneris.
    pub archive_inner: Option<String>,
}

/// Apskaičiuoja CRC32/MD5/SHA1 vienu perėjimu. `path` — ROM'o (arba jo archyvo) kelias;
/// `archive_extensions` — kokie plėtiniai laikomi ROM'u ARCHYVO viduje (paprastai
/// platformos `extensions` stulpelis, žr. [`crate::db::models::Platform`]) — naudojama TIK
/// jei `path` pats yra `.zip`/`.7z`.
pub fn hash_rom(
    path: &std::path::Path,
    archive_extensions: &[String],
) -> Result<RomHashes, AppError> {
    let is_archive = matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase)
            .as_deref(),
        Some("zip") | Some("7z")
    );

    if is_archive {
        let (inner_name, data) = archive::extract_first_match(path, archive_extensions)?;
        let mut hashes = hash_bytes(&data);
        hashes.archive_inner = Some(inner_name);
        return Ok(hashes);
    }

    let metadata = std::fs::metadata(path)?;
    if metadata.len() > STREAMING_THRESHOLD_BYTES {
        hash_file_streaming(path)
    } else {
        let data = std::fs::read(path)?;
        Ok(hash_bytes(&data))
    }
}

fn strip_known_header(data: &[u8]) -> &[u8] {
    if data.len() > INES_HEADER_LEN && data[..4] == INES_MAGIC {
        &data[INES_HEADER_LEN..]
    } else {
        data
    }
}

fn hash_bytes(data: &[u8]) -> RomHashes {
    let data = strip_known_header(data);

    let mut crc = crc32fast::Hasher::new();
    crc.update(data);

    RomHashes {
        crc32: format!("{:08X}", crc.finalize()),
        md5: hex_lower(&Md5::digest(data)),
        sha1: hex_lower(&Sha1::digest(data)),
        size: data.len() as u64,
        archive_inner: None,
    }
}

fn hash_file_streaming(path: &std::path::Path) -> Result<RomHashes, AppError> {
    let mut file = std::fs::File::open(path)?;
    let mut crc = crc32fast::Hasher::new();
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();
    let mut buf = vec![0u8; STREAM_CHUNK_SIZE];
    let mut total: u64 = 0;
    let mut first_chunk = true;

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut chunk = &buf[..n];
        if first_chunk {
            first_chunk = false;
            chunk = strip_known_header(chunk);
        }
        crc.update(chunk);
        md5.update(chunk);
        sha1.update(chunk);
        total += chunk.len() as u64;
    }

    Ok(RomHashes {
        crc32: format!("{:08X}", crc.finalize()),
        md5: hex_lower(&md5.finalize()),
        sha1: hex_lower(&sha1.finalize()),
        size: total,
        archive_inner: None,
    })
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Publikuoti, plačiai patikrinti test vektoriai `"abc"` tekstui (CRC-32/IEEE 802.3,
    /// MD5, SHA1) — nepriklausomas nuo jokio failo I/O, patikrina VIEN algoritmų teisingumą.
    #[test]
    fn known_test_vectors_for_abc() {
        let hashes = hash_bytes(b"abc");
        assert_eq!(hashes.crc32, "352441C2");
        assert_eq!(hashes.md5, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(hashes.sha1, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(hashes.size, 3);
    }

    #[test]
    fn streaming_matches_in_memory_for_same_content() {
        let mut data = vec![0u8; 5 * 1024 * 1024]; // 5 MiB — kelios `STREAM_CHUNK_SIZE` ribos.
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 251) as u8; // pseudo-atsitiktinis, bet deterministinis turinys.
        }

        let in_memory = hash_bytes(&data);

        let dir = std::env::temp_dir().join(format!("nullbyte_hasher_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("big.bin");
        std::fs::write(&path, &data).unwrap();

        let streamed = hash_file_streaming(&path).unwrap();

        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(in_memory, streamed);
    }

    #[test]
    fn ines_header_is_stripped_before_hashing() {
        let mut with_header = vec![0x4E, 0x45, 0x53, 0x1A]; // iNES magic
        with_header.extend_from_slice(&[0u8; 12]); // likusi header'io dalis (16 baitų iš viso)
        with_header.extend_from_slice(b"abc"); // „payload" — tas pats vektorius kaip aukščiau

        let hashes = hash_bytes(&with_header);
        assert_eq!(hashes.crc32, "352441C2");
        assert_eq!(
            hashes.size, 3,
            "header'is turėjo būti nuimtas prieš skaičiuojant dydį"
        );
    }

    #[test]
    fn non_ines_data_is_not_stripped() {
        // Panašus prefiksas, bet NE tikslus iNES magic — turi likti nepaliestas.
        let data = b"NESX abc";
        let hashes = hash_bytes(data);
        assert_eq!(hashes.size, data.len() as u64);
    }

    #[test]
    fn zip_wrapped_rom_hashes_the_inner_file_not_the_archive() {
        let mut buf = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut buf);
            let mut writer = zip::ZipWriter::new(cursor);
            writer
                .start_file("Game.sfc", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"abc").unwrap();
            writer.finish().unwrap();
        }

        let dir =
            std::env::temp_dir().join(format!("nullbyte_hasher_zip_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zip_path = dir.join("Game.zip");
        std::fs::write(&zip_path, &buf).unwrap();

        let extensions = vec!["sfc".to_string()];
        let hashes = hash_rom(&zip_path, &extensions).expect("turėtų rasti .sfc archyve");

        std::fs::remove_dir_all(&dir).ok();

        // Tas pats „abc" vektorius kaip `known_test_vectors_for_abc` — patvirtina, kad
        // hash'uotas VIDINIS failas, ne pats .zip.
        assert_eq!(hashes.crc32, "352441C2");
        assert_eq!(hashes.md5, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(hashes.archive_inner.as_deref(), Some("Game.sfc"));
    }

    /// P5.2 acceptance: „Hash'ai sutampa su sha1sum/md5sum komandinės eilutės rezultatais" —
    /// realus ROM'as, palyginta su NEPRIKLAUSOMU nuo mūsų kodo šaltiniu (macOS `md5`/
    /// `shasum` sistemos komandos, ne pačios `md-5`/`sha1` crate'os, kurias testuojame).
    ///
    /// `#[ignore]`: priklauso nuo `nullbyte-core/roms/snes/` test fixture'ų (gitignore'inti,
    /// žr. atminties įrašą apie test assets) IR macOS `md5`/`shasum` binarų. Paleisti
    /// rankiniu būdu:
    /// `cargo test -p nullbyte-app real_rom_hash_matches_system_tools -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_rom_hash_matches_system_tools() {
        let roms_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core/roms/snes");
        let rom_path = std::fs::read_dir(&roms_dir)
            .expect("roms/snes turėtų egzistuoti šioje dev aplinkoje")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some("sfc"))
            .expect("bent vienas .sfc turėtų būti roms/snes/");

        let hashes = hash_rom(&rom_path, &[]).expect("hash_rom turėtų pavykti");

        let md5_output = std::process::Command::new("md5")
            .arg("-q")
            .arg(&rom_path)
            .output()
            .expect("md5 komanda turėtų paleisti (macOS coreutils)");
        let system_md5 = String::from_utf8_lossy(&md5_output.stdout)
            .trim()
            .to_lowercase();

        let sha1_output = std::process::Command::new("shasum")
            .args(["-a", "1"])
            .arg(&rom_path)
            .output()
            .expect("shasum komanda turėtų paleisti (macOS coreutils)");
        let system_sha1 = String::from_utf8_lossy(&sha1_output.stdout)
            .split_whitespace()
            .next()
            .expect("shasum išvestis turėtų prasidėti hash'u")
            .to_lowercase();

        assert_eq!(hashes.md5, system_md5, "MD5 nesutampa su sistemos `md5`");
        assert_eq!(
            hashes.sha1, system_sha1,
            "SHA1 nesutampa su sistemos `shasum -a 1`"
        );
    }

    /// P5.2 acceptance: „100 failų (~2 GB) hash'avimas < 30 s SSD'e" — visi realūs test
    /// fixture ROM'ai (`nullbyte-core/{roms,roms/megadrive,roms/psx,roms/gba}`), ne
    /// dirbtinai sugeneruoti duomenys, nes tikri PSX `.zip` archyvai (400-500 MB kiekvienas)
    /// realiai iškviečia streaming/archyvo kelius, kurių sintetinis testas nepatikrintų.
    ///
    /// `#[ignore]`: priklauso nuo test fixture'ų IR trunka realiu laiku (sekundės, ne ms).
    /// Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app --release hashing_100_files_under_30_seconds -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn hashing_100_files_under_30_seconds() {
        let roms_root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core/roms");
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(&roms_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // `roms/mame/` — kito, nesusijusio (P4.0.5 MAME smoke test) manual patikrinimo
            // liekana. MAME archyvai neturi vieno „vidinio" failo (daugybė ROM chip'ų
            // viename zip'e) — kitokia semantika, ne šio testo apimtis.
            if entry.path().components().any(|c| c.as_os_str() == "mame") {
                continue;
            }
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        assert!(
            !files.is_empty(),
            "roms/ turėtų turėti bent kelis test fixture failus"
        );

        let total_bytes: u64 = files
            .iter()
            .filter_map(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();
        let start = std::time::Instant::now();

        // Kiekvienos platformos katalogo TIKRI ROM plėtiniai (ne pačio archyvo plėtinys —
        // `path.extension()` archyvuotam failui grąžintų „zip", o mums reikia to, KO
        // ieškoti VIDUJE, žr. `hash_rom` doc dėl `archive_extensions`).
        let platform_extensions: &[(&str, &[&str])] = &[
            ("snes", &["sfc", "smc"]),
            ("megadrive", &["md", "smd", "bin", "gen"]),
            ("psx", &["cue", "bin", "chd", "pbp"]),
            ("gba", &["gba"]),
        ];

        for path in &files {
            let platform_dir = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let extensions: Vec<String> = platform_extensions
                .iter()
                .find(|(dir, _)| *dir == platform_dir)
                .map(|(_, exts)| exts.iter().map(|e| e.to_string()).collect())
                .unwrap_or_default();

            hash_rom(path, &extensions).unwrap_or_else(|e| {
                panic!("hash_rom nepavyko {}: {e}", path.display());
            });
        }

        let elapsed = start.elapsed();
        eprintln!(
            "{} failų ({:.2} GB) hash'uota per {:.2}s",
            files.len(),
            total_bytes as f64 / 1_073_741_824.0,
            elapsed.as_secs_f64()
        );
        assert!(
            elapsed.as_secs() < 30,
            "hash'avimas užtruko {:.2}s, tikėtasi < 30s",
            elapsed.as_secs_f64()
        );
    }
}
