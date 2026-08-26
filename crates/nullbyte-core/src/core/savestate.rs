//! Save state serializacija/atkūrimas + preview paveiksliukas (MVP.md P8.1, CLAUDE.md §8.7).
//!
//! Sąmoningai `nullbyte-core` pusėje (ne `nullbyte-app`) — ADR-016: šis kodas VISADA
//! kviečiamas iš emuliavimo gijos `nullbyte-emu` vaiko procese
//! (`retro_serialize`/`retro_unserialize` taisyklė, CLAUDE.md §3.2 taisyklė #1), o preview
//! paveiksliukas skaitomas iš PAČIO proceso triple buffer'io (ADR-016, MVP.md §14: „paimk
//! kadrą → PNG" TURI vykti vaiko pusėje, NE persiunčiant žalius kadro baitus per IPC). Failų
//! įrašymas/skaitymas taip pat vyksta ČIA — tėvo pusė (`nullbyte-app`) tik gauna GRĄŽTA kelią
//! per IPC ir įrašo jį į DB (CLAUDE.md §10 „IPC riba turi likti PLONA").

use std::path::{Path, PathBuf};

use super::loader::CoreHandle;
use crate::error::CoreError;
use crate::video::frame_buffer::VideoFrameData;
use crate::video::png_encoder;

/// Išsaugo dabartinį žaidimo būvį į `path` (atominiu rašymu — `.tmp` → `rename`, kad
/// nutrūkęs rašymas niekada nepaliktų sugadinto failo) ir, jei `frame` bei `thumb_path`
/// abu pateikti, preview paveiksliuką į `thumb_path`.
///
/// `retro_serialize_size()` kviečiama IŠ NAUJO KIEKVIENĄ kartą (CLAUDE.md §8.7 — dydis GALI
/// pasikeisti tarp iškvietimų), niekada necache'uojama.
///
/// # Safety
/// Turi būti kviečiama tik iš emuliavimo gijos, TARP `retro_run()` kvietimų, po sėkmingo
/// `CoreHandle::load_game()` (žr. `CoreHandle::serialize` doc).
pub unsafe fn save_state(
    core: &CoreHandle,
    frame: Option<&VideoFrameData>,
    path: &Path,
    thumb_path: Option<&Path>,
) -> Result<(), CoreError> {
    let size = unsafe { core.serialize_size() };
    if size == 0 {
        return Err(CoreError::SaveState(
            "retro_serialize_size() grąžino 0 — core'as nepalaiko save state'ų".to_string(),
        ));
    }
    let mut buffer = vec![0u8; size];
    if !unsafe { core.serialize(&mut buffer) } {
        return Err(CoreError::SaveState(
            "retro_serialize() atmetė — core'as atsisakė serializuoti".to_string(),
        ));
    }

    write_atomic(path, &buffer)?;

    if let (Some(frame), Some(thumb_path)) = (frame, thumb_path) {
        match png_encoder::encode_rgba8(frame.width, frame.height, &frame.data) {
            Some(png) => {
                // Preview'o nepavykimas NĖRA kritiška klaida pačiam save state'ui (kuris jau
                // sėkmingai įrašytas aukščiau) — tik loginam, negrąžinam Err.
                if let Err(error) = write_atomic(thumb_path, &png) {
                    tracing::warn!(
                        %error,
                        path = %thumb_path.display(),
                        "nepavyko įrašyti save state preview'o"
                    );
                }
            }
            None => tracing::warn!(
                width = frame.width,
                height = frame.height,
                data_len = frame.data.len(),
                "save state preview'o kadras neatitiko RGBA8 dydžio — praleista"
            ),
        }
    }

    Ok(())
}

/// Atstato žaidimo būvį iš `path`. `false` grąžinimas iš `retro_unserialize()` NĖRA
/// netikėta klaida (CLAUDE.md §8.7 — save state'ai nesuderinami tarp core versijų), bet vis
/// tiek grąžinamas kaip `Err`, kad kviečiančioji pusė galėtų parodyti aiškų pranešimą.
///
/// # Safety
/// Turi būti kviečiama tik iš emuliavimo gijos, TARP `retro_run()` kvietimų, po sėkmingo
/// `CoreHandle::load_game()`.
pub unsafe fn load_state(core: &CoreHandle, path: &Path) -> Result<(), CoreError> {
    let buffer = std::fs::read(path)?;
    if !unsafe { core.unserialize(&buffer) } {
        return Err(CoreError::SaveState(format!(
            "retro_unserialize() atmetė failą {} — greičiausiai nesuderinama core versija",
            path.display()
        )));
    }
    Ok(())
}

/// `.tmp` failas PRIE PILNO originalaus kelio (ne `with_extension`, kuris PAKEISTŲ, ne
/// papildytų — `foo.state` → `foo.tmp` prarastų `.state` dalį, o mums reikia `foo.state.tmp`).
///
/// `pub(super)`, nes `core::sram` (P8.2) naudoja TĄ PATĮ atominio rašymo šabloną SRAM
/// failams — nėra prasmės dubliuoti.
pub(super) fn write_atomic(path: &Path, data: &[u8]) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = PathBuf::from(format!("{}.tmp", path.display()));
    std::fs::write(&tmp_path, data)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::callbacks::{install_context, take_context, EmuContext};
    use crate::core::loader::RetroCallbacks;

    fn snes9x_path() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/snes9x_libretro.dylib");
        path.exists().then_some(path)
    }

    fn first_sfc_rom() -> Option<PathBuf> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/snes");
        std::fs::read_dir(&dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_lowercase)
                    == Some("sfc".to_string())
            })
    }

    fn stub_callbacks() -> RetroCallbacks {
        RetroCallbacks {
            environment: crate::core::callbacks::environment_cb,
            video_refresh: crate::core::callbacks::video_refresh_cb,
            input_poll: crate::core::callbacks::input_poll_cb,
            input_state: crate::core::callbacks::input_state_cb,
            audio_sample: crate::core::callbacks::audio_sample_cb,
            audio_sample_batch: crate::core::callbacks::audio_sample_batch_cb,
        }
    }

    /// Realus core'as + ROM'as: `retro_run()` keliskart, tada save → NAUJAS `CoreHandle`
    /// (imituoja „uždaryti → paleisti iš naujo" P8.1 acceptance) → load → serializuota
    /// būsena TURI sutapti baitas-į-baitą su ta, kuri buvo TIESIOG PO save (CLAUDE.md §8.7
    /// „tas pats taškas" — patikrinama objektyviai, ne subjektyviu žaidimu, per pačią
    /// `retro_serialize()` išvestį, kuri IR YRA visas core'o vidinis būvis).
    #[test]
    fn save_then_load_on_a_fresh_core_restores_identical_state() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let Some(rom_path) = first_sfc_rom() else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        let tmp_state = std::env::temp_dir().join("nullbyte_test_savestate.state");
        std::fs::remove_file(&tmp_state).ok();

        // --- Pirma sesija: paleisk, žaisk kelis kadrus, išsaugok. ---
        install_context(EmuContext::default());
        let core_a = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core_a.init(stub_callbacks()) };
        unsafe { core_a.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        for _ in 0..30 {
            unsafe { core_a.run() };
        }
        unsafe { save_state(&core_a, None, &tmp_state, None) }.expect("save_state turėtų pavykti");

        // Referencinis būvis TIESIOG PO save — tikriname PRIEŠ tai, ar failas sutampa.
        let size_after_save = unsafe { core_a.serialize_size() };
        let mut reference = vec![0u8; size_after_save];
        assert!(unsafe { core_a.serialize(&mut reference) });

        unsafe {
            core_a.unload_game();
            core_a.deinit();
        }
        take_context();

        // --- Antra sesija: NAUJAS CoreHandle (imituoja procesą iš naujo), load_state. ---
        install_context(EmuContext::default());
        let core_b = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core_b.init(stub_callbacks()) };
        unsafe { core_b.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        unsafe { load_state(&core_b, &tmp_state) }.expect("load_state turėtų pavykti");

        let size_after_load = unsafe { core_b.serialize_size() };
        let mut restored = vec![0u8; size_after_load];
        assert!(unsafe { core_b.serialize(&mut restored) });

        assert_eq!(
            restored, reference,
            "atstatytas būvis turėtų sutapti baitas-į-baitą su tuo, kuris buvo IŠ KARTO po save"
        );

        unsafe {
            core_b.unload_game();
            core_b.deinit();
        }
        take_context();
        std::fs::remove_file(&tmp_state).ok();
    }

    #[test]
    fn load_state_with_missing_file_returns_io_error_not_panic() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let Some(rom_path) = first_sfc_rom() else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext::default());
        let core = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core.init(stub_callbacks()) };
        unsafe { core.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");

        let missing = std::env::temp_dir().join("nullbyte_test_definitely_missing.state");
        std::fs::remove_file(&missing).ok();

        let result = unsafe { load_state(&core, &missing) };
        assert!(matches!(result, Err(CoreError::Io(_))));

        unsafe {
            core.unload_game();
            core.deinit();
        }
        take_context();
    }

    #[test]
    fn save_state_writes_a_valid_preview_png_when_frame_given() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let Some(rom_path) = first_sfc_rom() else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext::default());
        let core = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core.init(stub_callbacks()) };
        unsafe { core.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        unsafe { core.run() };

        let frame = VideoFrameData {
            width: 4,
            height: 4,
            aspect_ratio: 1.0,
            generation: 1,
            data: vec![255u8; 4 * 4 * 4],
        };
        let tmp_state = std::env::temp_dir().join("nullbyte_test_savestate_with_thumb.state");
        let tmp_thumb = std::env::temp_dir().join("nullbyte_test_savestate_with_thumb.png");
        std::fs::remove_file(&tmp_state).ok();
        std::fs::remove_file(&tmp_thumb).ok();

        unsafe { save_state(&core, Some(&frame), &tmp_state, Some(&tmp_thumb)) }
            .expect("save_state turėtų pavykti");

        assert!(tmp_thumb.exists(), "preview PNG turėjo būti įrašytas");
        let png_bytes = std::fs::read(&tmp_thumb).unwrap();
        assert_eq!(
            &png_bytes[..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        unsafe {
            core.unload_game();
            core.deinit();
        }
        take_context();
        std::fs::remove_file(&tmp_state).ok();
        std::fs::remove_file(&tmp_thumb).ok();
    }
}
