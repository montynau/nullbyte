//! Core (`.dylib` / `.so`) įkėlimas per `libloading` (CLAUDE.md §8.1, §8.2).

use std::ffi::{c_char, c_void, CStr};
use std::path::{Path, PathBuf};

use libloading::Library;

use crate::error::AppError;

use super::ffi::{
    retro_audio_sample_batch_t, retro_audio_sample_t, retro_environment_t, retro_game_info,
    retro_input_poll_t, retro_input_state_t, retro_system_av_info, retro_system_info,
    retro_video_refresh_t, RETRO_API_VERSION,
};

/// Simbolis be gyvavimo trukmės parametro — leidžia laikyti jį kartu su `Library` tame
/// pačiame struct'e (kitaip `Symbol<'lib, T>` sukurtų self-referential struct problemą).
///
/// SAFETY: kiekvienas šio tipo reikšmė galioja tik tol, kol gyva `Library`, iš kurios ji
/// gauta. `CoreHandle` tai užtikrina lauko deklaravimo tvarka (žr. žemiau).
type RawSymbol<T> = libloading::os::unix::Symbol<T>;

type RetroApiVersionFn = unsafe extern "C" fn() -> u32;
type RetroSetEnvironmentFn = unsafe extern "C" fn(retro_environment_t);
type RetroSetVideoRefreshFn = unsafe extern "C" fn(retro_video_refresh_t);
type RetroSetAudioSampleFn = unsafe extern "C" fn(retro_audio_sample_t);
type RetroSetAudioSampleBatchFn = unsafe extern "C" fn(retro_audio_sample_batch_t);
type RetroSetInputPollFn = unsafe extern "C" fn(retro_input_poll_t);
type RetroSetInputStateFn = unsafe extern "C" fn(retro_input_state_t);
type RetroInitFn = unsafe extern "C" fn();
type RetroDeinitFn = unsafe extern "C" fn();
type RetroGetSystemInfoFn = unsafe extern "C" fn(*mut retro_system_info);
type RetroGetSystemAvInfoFn = unsafe extern "C" fn(*mut retro_system_av_info);
type RetroSetControllerPortDeviceFn = unsafe extern "C" fn(u32, u32);
type RetroLoadGameFn = unsafe extern "C" fn(*const retro_game_info) -> bool;
type RetroUnloadGameFn = unsafe extern "C" fn();
type RetroRunFn = unsafe extern "C" fn();
type RetroResetFn = unsafe extern "C" fn();
type RetroSerializeSizeFn = unsafe extern "C" fn() -> usize;
type RetroSerializeFn = unsafe extern "C" fn(*mut c_void, usize) -> bool;
type RetroUnserializeFn = unsafe extern "C" fn(*const c_void, usize) -> bool;
type RetroGetMemoryDataFn = unsafe extern "C" fn(u32) -> *mut c_void;
type RetroGetMemorySizeFn = unsafe extern "C" fn(u32) -> usize;
type RetroGetRegionFn = unsafe extern "C" fn() -> u32;

/// Visi privalomi libretro simboliai iš CLAUDE.md §8.1.
#[allow(dead_code)] // dauguma laukų dar nenaudojami — prisijungs P1.4–P1.7
pub struct CoreSymbols {
    retro_api_version: RawSymbol<RetroApiVersionFn>,
    retro_set_environment: RawSymbol<RetroSetEnvironmentFn>,
    retro_set_video_refresh: RawSymbol<RetroSetVideoRefreshFn>,
    retro_set_audio_sample: RawSymbol<RetroSetAudioSampleFn>,
    retro_set_audio_sample_batch: RawSymbol<RetroSetAudioSampleBatchFn>,
    retro_set_input_poll: RawSymbol<RetroSetInputPollFn>,
    retro_set_input_state: RawSymbol<RetroSetInputStateFn>,
    retro_init: RawSymbol<RetroInitFn>,
    retro_deinit: RawSymbol<RetroDeinitFn>,
    retro_get_system_info: RawSymbol<RetroGetSystemInfoFn>,
    retro_get_system_av_info: RawSymbol<RetroGetSystemAvInfoFn>,
    retro_set_controller_port_device: RawSymbol<RetroSetControllerPortDeviceFn>,
    retro_load_game: RawSymbol<RetroLoadGameFn>,
    retro_unload_game: RawSymbol<RetroUnloadGameFn>,
    retro_run: RawSymbol<RetroRunFn>,
    retro_reset: RawSymbol<RetroResetFn>,
    retro_serialize_size: RawSymbol<RetroSerializeSizeFn>,
    retro_serialize: RawSymbol<RetroSerializeFn>,
    retro_unserialize: RawSymbol<RetroUnserializeFn>,
    retro_get_memory_data: RawSymbol<RetroGetMemoryDataFn>,
    retro_get_memory_size: RawSymbol<RetroGetMemorySizeFn>,
    retro_get_region: RawSymbol<RetroGetRegionFn>,
}

impl CoreSymbols {
    /// # Safety
    /// `lib` privalo gyventi bent tiek pat, kiek bus naudojami grąžinti simboliai
    /// (užtikrina `CoreHandle` laukų tvarka).
    #[allow(dead_code)] // kviečia tik CoreHandle::load ir testai — dar nenaudojama P1.7 runner.rs
    unsafe fn load(lib: &Library) -> Result<Self, AppError> {
        macro_rules! sym {
            ($name:expr, $ty:ty) => {{
                let raw: libloading::Symbol<'_, $ty> = lib
                    .get(concat!($name, "\0").as_bytes())
                    .map_err(|_| AppError::Other(format!("core'e trūksta simbolio `{}`", $name)))?;
                raw.into_raw()
            }};
        }

        Ok(Self {
            retro_api_version: sym!("retro_api_version", RetroApiVersionFn),
            retro_set_environment: sym!("retro_set_environment", RetroSetEnvironmentFn),
            retro_set_video_refresh: sym!("retro_set_video_refresh", RetroSetVideoRefreshFn),
            retro_set_audio_sample: sym!("retro_set_audio_sample", RetroSetAudioSampleFn),
            retro_set_audio_sample_batch: sym!(
                "retro_set_audio_sample_batch",
                RetroSetAudioSampleBatchFn
            ),
            retro_set_input_poll: sym!("retro_set_input_poll", RetroSetInputPollFn),
            retro_set_input_state: sym!("retro_set_input_state", RetroSetInputStateFn),
            retro_init: sym!("retro_init", RetroInitFn),
            retro_deinit: sym!("retro_deinit", RetroDeinitFn),
            retro_get_system_info: sym!("retro_get_system_info", RetroGetSystemInfoFn),
            retro_get_system_av_info: sym!("retro_get_system_av_info", RetroGetSystemAvInfoFn),
            retro_set_controller_port_device: sym!(
                "retro_set_controller_port_device",
                RetroSetControllerPortDeviceFn
            ),
            retro_load_game: sym!("retro_load_game", RetroLoadGameFn),
            retro_unload_game: sym!("retro_unload_game", RetroUnloadGameFn),
            retro_run: sym!("retro_run", RetroRunFn),
            retro_reset: sym!("retro_reset", RetroResetFn),
            retro_serialize_size: sym!("retro_serialize_size", RetroSerializeSizeFn),
            retro_serialize: sym!("retro_serialize", RetroSerializeFn),
            retro_unserialize: sym!("retro_unserialize", RetroUnserializeFn),
            retro_get_memory_data: sym!("retro_get_memory_data", RetroGetMemoryDataFn),
            retro_get_memory_size: sym!("retro_get_memory_size", RetroGetMemorySizeFn),
            retro_get_region: sym!("retro_get_region", RetroGetRegionFn),
        })
    }
}

/// Įkeltas libretro core: bendrina `Library` ir jos simbolius.
///
/// Laukų tvarka SVARBI: `symbols` deklaruotas prieš `lib`, nes Rust drop'ina struct
/// laukus deklaravimo tvarka — taip `symbols` (rodyklės į `lib` atmintį) visada
/// atlaisvinami PRIEŠ `lib` iškraunamas (`dlclose`).
#[allow(dead_code)] // pilnai prijungiama P1.7 (emuliavimo gija) — kol kas naudoja tik testai
pub struct CoreHandle {
    symbols: CoreSymbols,
    path: PathBuf,
    lib: Library,
}

#[allow(dead_code)] // load/path naudoja tik testai, kol P1.7 runner.rs neparašytas
impl CoreHandle {
    /// Įkelia core'ą iš `.dylib` / `.so` failo ir patikrina `retro_api_version() == 1`.
    ///
    /// # Safety motyvacija
    /// `Library::new` yra `unsafe`, nes bibliotekos įkėlimas gali vykdyti savavališką kodą
    /// per jos konstruktorius/inicializatorius — tai žinoma, priimta `libloading` rizika.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref().to_path_buf();

        // SAFETY: rizika, kad įkeliamas failas paleis savavališką kodą per savo
        // konstruktorius, yra būdinga bet kokiam dinaminiam bibliotekos įkėlimui.
        // Vartotojas pats renkasi, kokius core'us deda į cores_dir (CLAUDE.md §11.2).
        let lib = unsafe { Library::new(&path) }.map_err(|e| {
            AppError::Other(format!("nepavyko įkelti core'o {}: {e}", path.display()))
        })?;

        // SAFETY: `lib` gyvuos bent tiek, kiek `symbols` — abu laikomi tame pačiame
        // `CoreHandle`, o laukų tvarka garantuoja teisingą Drop seką.
        let symbols = unsafe { CoreSymbols::load(&lib) }?;

        // SAFETY: `retro_api_version` neima jokių argumentų ir negali panikuoti be
        // core'o pačio klaidos; kviečiame ją tik patikrinti API suderinamumą.
        let api_version = unsafe { (symbols.retro_api_version)() };
        if api_version != RETRO_API_VERSION {
            return Err(AppError::Other(format!(
                "core'as {} turi nesuderinamą API versiją {api_version} (tikimasi {RETRO_API_VERSION})",
                path.display()
            )));
        }

        Ok(Self { symbols, path, lib })
    }

    /// Kelias, iš kurio core'as įkeltas.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Kviečia `retro_get_system_info()` ir grąžina laukus kaip savarankiškas (owned)
    /// reikšmes — naudojama `core::info` (P1.3) core'ų metaduomenims surinkti.
    pub fn system_info(&self) -> CoreSystemInfo {
        // SAFETY: `info` užpildomas paties core'o retro_get_system_info() implementacijos;
        // libretro kontraktas garantuoja, kad grąžintos *const c_char rodyklės yra arba
        // NULL, arba galioja bent tiek, kiek core'as įkeltas (statinės eilutės).
        let mut info: retro_system_info = unsafe { std::mem::zeroed() };
        unsafe { (self.symbols.retro_get_system_info)(&mut info) };

        CoreSystemInfo {
            library_name: unsafe { c_str_to_string(info.library_name) },
            library_version: unsafe { c_str_to_string(info.library_version) },
            valid_extensions: unsafe { c_str_to_string(info.valid_extensions) },
            need_fullpath: info.need_fullpath,
            block_extract: info.block_extract,
        }
    }
}

/// `retro_get_system_info()` laukai, konvertuoti į savarankiškus (owned) tipus.
#[allow(dead_code)] // laukus skaito core::info (P1.3), kol kas naudoja tik testai
#[derive(Debug, Clone)]
pub struct CoreSystemInfo {
    pub library_name: String,
    pub library_version: String,
    /// Neapdorotas `|`-skirtas sąrašas, pvz. `"smc|sfc|swc|fig|bs"`.
    pub valid_extensions: String,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

/// # Safety
/// `ptr` privalo būti arba NULL, arba rodyti į teisingai nul-terminuotą C eilutę, kurios
/// gyvavimo trukmė apima šio iškvietimo momentą.
unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

impl Drop for CoreHandle {
    /// `retro_deinit()` PRIVALO būti iškviestas PRIEŠ šį drop (CLAUDE.md §8.2, žingsnis 14:
    /// `retro_unload_game()` → `retro_deinit()` → `drop(Library)`). Tai callerio (runner.rs,
    /// P1.7) atsakomybė — šis `Drop` tik atlaisvina `Library`/`symbols` ir nekviečia
    /// `retro_deinit` už tave, nes tam reikia žinoti emuliavimo būvį (ar žaidimas įkeltas ir
    /// pan.), kurio `CoreHandle` pats nelaiko.
    fn drop(&mut self) {
        tracing::debug!(core = %self.path.display(), "core'o Library iškraunama");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `src-tauri/cores/` yra `.gitignore`'inta (CLAUDE.md §11.2) — vartotojas pats deda
    /// realius core'us lokaliam testavimui. CI aplinkoje jo nėra, todėl testas praleidžiamas,
    /// jei failo nerandama, o ne suveikia klaidingai.
    fn test_core_path() -> Option<PathBuf> {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/genesis_plus_gx_libretro.dylib");
        path.exists().then_some(path)
    }

    #[test]
    fn loads_real_core_and_reports_api_version_1() {
        let Some(path) = test_core_path() else {
            eprintln!(
                "praleista: src-tauri/cores/genesis_plus_gx_libretro.dylib nerastas (lokalus test fixture)"
            );
            return;
        };

        // CoreHandle::load() viduje jau atmeta core'ą, jei api_version != 1 — sėkmingas
        // grąžinimas ir yra įrodymas, kad api_version == 1.
        let handle = CoreHandle::load(&path).expect("core'as turėtų sėkmingai įsikelti");
        assert_eq!(handle.path(), path);
    }

    #[test]
    fn missing_file_returns_app_error_not_panic() {
        let result = CoreHandle::load("/no/such/path/definitely_missing_core.dylib");
        assert!(matches!(result, Err(AppError::Other(_))));
    }

    #[test]
    fn non_libretro_library_reports_missing_symbol() {
        // Sisteminė zlib biblioteka — egzistuoja tiek macOS, tiek dauguma Linux distribucijų,
        // bet neturi jokių libretro simbolių.
        let candidates = [
            "/usr/lib/libz.dylib",
            "/usr/lib/x86_64-linux-gnu/libz.so.1",
            "/lib/x86_64-linux-gnu/libz.so.1",
            "/usr/lib/aarch64-linux-gnu/libz.so.1",
        ];
        let Some(path) = candidates.iter().find(|p| Path::new(p).exists()) else {
            eprintln!("praleista: sistemos libz nerasta");
            return;
        };

        let err = match CoreHandle::load(path) {
            Ok(_) => panic!("libz neturi libretro simbolių, bet CoreHandle::load pavyko"),
            Err(e) => e,
        };
        let message = err.to_string();
        assert!(
            message.contains("trūksta simbolio"),
            "klaida turėtų minėti trūkstamą simbolį, gauta: {message}"
        );
    }
}
