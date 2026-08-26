//! SRAM (in-game save) skaitymas/rašymas (MVP.md P8.2, CLAUDE.md §8.8).
//!
//! Atskirai nuo save state'ų (`core::savestate`) — skirtinga libretro operacija
//! (`retro_get_memory_data`/`size`, ne `retro_serialize`) ir skirtinga panaudojimo semantika:
//! SRAM yra PATIES ŽAIDIMO in-game save'as (RPG progresas ir pan.), atsinaujina progresyviai
//! visos sesijos metu, o ne vieną tašką „užšaldo" kaip save state'as. Failo kelią
//! (`saves_dir()/{rom_basename}.srm`) parenka kviečiantysis (`core::runner`) — šis modulis
//! TIK ima `path`, kaip ir `core::savestate`.

use std::path::Path;

use super::loader::CoreHandle;
use super::savestate::write_atomic;
use crate::error::CoreError;

/// Įrašo core'o SRAM turinį į `path` (atominiu rašymu — `.tmp` → `rename`). `Ok(())` ir NIEKO
/// nedaro, jei core'as neturi SRAM (`CoreHandle::sram()` grąžina `None`) — CLAUDE.md §8.8:
/// daug core'ų (pvz. arcade) battery-backed atminties tiesiog neturi, tai NĖRA klaida.
///
/// # Safety
/// Turi būti kviečiama tik iš emuliavimo gijos, po sėkmingo `CoreHandle::load_game()`
/// (žr. `CoreHandle::sram` doc).
pub unsafe fn save_sram(core: &CoreHandle, path: &Path) -> Result<(), CoreError> {
    let Some(data) = (unsafe { core.sram() }) else {
        return Ok(());
    };
    if data.is_empty() {
        return Ok(());
    }
    write_atomic(path, data)
}

/// Įkelia `.srm` failo turinį į core'o SRAM. `Ok(())` ir nieko nedaro, jei core'as neturi
/// SRAM, ARBA jei `path` neegzistuoja (nauja sesija be ankstesnio in-game save'o — TAI NĖRA
/// klaida, žaidimas tiesiog prasideda nuo pradžių, kaip ir realiame emuliatoriuje).
///
/// Jei failo dydis nesutampa su core'o SRAM dydžiu (pvz. core versija pasikeitė), kopijuojama
/// TIK bendra (mažesnioji) dalis — likusi core'o atmintis paliekama tokia, kokią ją
/// inicializavo pats core'as. Tai apsaugo nuo buferio perviršio be papildomo dydžio patikrinimo
/// iš kviečiančiosios pusės.
///
/// # Safety
/// Turi būti kviečiama tik iš emuliavimo gijos, po sėkmingo `CoreHandle::load_game()`, PRIEŠ
/// pirmą `run()` (kad in-game save'as būtų matomas nuo pat pirmo kadro).
pub unsafe fn load_sram(core: &CoreHandle, path: &Path) -> Result<(), CoreError> {
    let Some(sram) = (unsafe { core.sram_mut() }) else {
        return Ok(());
    };
    if sram.is_empty() || !path.exists() {
        return Ok(());
    }
    let data = std::fs::read(path)?;
    let len = data.len().min(sram.len());
    sram[..len].copy_from_slice(&data[..len]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::callbacks::{install_context, take_context, EmuContext};
    use crate::core::loader::RetroCallbacks;
    use std::path::PathBuf;

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

    /// Realus core'as + ROM'as: paleidžia kelis kadrus (kad core'as inicializuotų savo SRAM
    /// buferį realistiškai), rankiniu būdu užrašo žinomą baitų šabloną TIESIOG į core'o SRAM
    /// (imituoja in-game save'ą be reikalo laukti realaus žaidimo scenarijaus), `save_sram` →
    /// NAUJAS `CoreHandle` (imituoja „uždaryti → paleisti iš naujo") → `load_sram` → core'o
    /// SRAM turi turėti TĄ PATĮ šabloną.
    #[test]
    fn save_then_load_on_a_fresh_core_restores_identical_sram_prefix() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let Some(rom_path) = first_sfc_rom() else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        let tmp_srm = std::env::temp_dir().join("nullbyte_test_sram.srm");
        std::fs::remove_file(&tmp_srm).ok();

        install_context(EmuContext::default());
        let core_a = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core_a.init(stub_callbacks()) };
        unsafe { core_a.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        for _ in 0..5 {
            unsafe { core_a.run() };
        }

        let Some(sram_a) = (unsafe { core_a.sram_mut() }) else {
            eprintln!("praleista: snes9x šis ROM'as nepraneša SRAM (size == 0)");
            unsafe {
                core_a.unload_game();
                core_a.deinit();
            }
            take_context();
            return;
        };
        // Žinomas, atpažįstamas šablonas (ne visi 0x00/0xFF — tie yra dažni SRAM „tuščios"
        // reikšmės, prastas testo signalas).
        let pattern: Vec<u8> = (0..sram_a.len()).map(|i| (i % 251) as u8).collect();
        sram_a.copy_from_slice(&pattern);

        unsafe { save_sram(&core_a, &tmp_srm) }.expect("save_sram turėtų pavykti");
        unsafe {
            core_a.unload_game();
            core_a.deinit();
        }
        take_context();

        install_context(EmuContext::default());
        let core_b = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { core_b.init(stub_callbacks()) };
        unsafe { core_b.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        unsafe { load_sram(&core_b, &tmp_srm) }.expect("load_sram turėtų pavykti");

        let sram_b = unsafe { core_b.sram() }.expect("core_b turėtų pranešti tą patį SRAM dydį");
        assert_eq!(
            sram_b, pattern,
            "atstatytas SRAM turėtų sutapti baitas-į-baitą su tuo, kuris buvo įrašytas"
        );

        unsafe {
            core_b.unload_game();
            core_b.deinit();
        }
        take_context();
        std::fs::remove_file(&tmp_srm).ok();
    }

    #[test]
    fn load_sram_with_missing_file_is_a_silent_noop_not_an_error() {
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

        let missing = std::env::temp_dir().join("nullbyte_test_definitely_missing.srm");
        std::fs::remove_file(&missing).ok();

        let result = unsafe { load_sram(&core, &missing) };
        assert!(
            result.is_ok(),
            "trūkstamas .srm failas (nauja sesija) turėtų būti tyliai ignoruojamas, gauta {result:?}"
        );

        unsafe {
            core.unload_game();
            core.deinit();
        }
        take_context();
    }
}
