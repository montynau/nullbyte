//! Core (`.dylib` / `.so`) įkėlimas per `libloading` (CLAUDE.md §8.1, §8.2).

use std::ffi::{c_char, c_void, CStr, CString};
use std::path::{Path, PathBuf};

use libloading::Library;

use crate::archive;
use crate::error::CoreError;

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
    unsafe fn load(lib: &Library) -> Result<Self, CoreError> {
        macro_rules! sym {
            ($name:expr, $ty:ty) => {{
                let raw: libloading::Symbol<'_, $ty> =
                    lib.get(concat!($name, "\0").as_bytes()).map_err(|_| {
                        CoreError::Other(format!("core'e trūksta simbolio `{}`", $name))
                    })?;
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
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CoreError> {
        let path = path.as_ref().to_path_buf();

        // SAFETY: rizika, kad įkeliamas failas paleis savavališką kodą per savo
        // konstruktorius, yra būdinga bet kokiam dinaminiam bibliotekos įkėlimui.
        // Vartotojas pats renkasi, kokius core'us deda į cores_dir (CLAUDE.md §11.2).
        let lib = unsafe { Library::new(&path) }.map_err(|e| {
            CoreError::Other(format!("nepavyko įkelti core'o {}: {e}", path.display()))
        })?;

        // SAFETY: `lib` gyvuos bent tiek, kiek `symbols` — abu laikomi tame pačiame
        // `CoreHandle`, o laukų tvarka garantuoja teisingą Drop seką.
        let symbols = unsafe { CoreSymbols::load(&lib) }?;

        // SAFETY: `retro_api_version` neima jokių argumentų ir negali panikuoti be
        // core'o pačio klaidos; kviečiame ją tik patikrinti API suderinamumą.
        let api_version = unsafe { (symbols.retro_api_version)() };
        if api_version != RETRO_API_VERSION {
            return Err(CoreError::Other(format!(
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

    /// Registruoja visus 6 libretro callback'us ir kviečia `retro_init()`
    /// (CLAUDE.md §8.2, žingsniai 3–9). Pagrindas P1.7 runner.rs emuliavimo gijai;
    /// šiame etape naudojama P1.5 integraciniam testui („core'as inicializuojasi be klaidų").
    ///
    /// # Safety
    /// Kaip ir visi `retro_*` kvietimai, turi būti kviečiama tik iš gijos, kurioje bus
    /// naudojamas `thread_local` `EmuContext` (CLAUDE.md §3.2 taisyklė #1) — callback'ai
    /// per jį pasiekia būvį.
    pub unsafe fn init(&self, callbacks: RetroCallbacks) {
        unsafe {
            (self.symbols.retro_set_environment)(callbacks.environment);
            (self.symbols.retro_set_video_refresh)(callbacks.video_refresh);
            (self.symbols.retro_set_input_poll)(callbacks.input_poll);
            (self.symbols.retro_set_input_state)(callbacks.input_state);
            (self.symbols.retro_set_audio_sample)(callbacks.audio_sample);
            (self.symbols.retro_set_audio_sample_batch)(callbacks.audio_sample_batch);
            (self.symbols.retro_init)();
        }
    }

    /// Įkelia ROM'ą (CLAUDE.md §8.2 žingsniai 11–12). Palaiko archyvus (`.zip`/`.7z`) —
    /// išpakuoja pirmą core'o `valid_extensions` atitinkantį failą. `retro_get_system_av_info()`
    /// kviečiama TIK po `retro_load_game()` — kai kurie core'ai (pvz. Mednafen) prieš tai
    /// grąžina neteisingus duomenis, nes AV info priklauso nuo ROM'o.
    ///
    /// # Safety
    /// Turi būti kviečiama tik iš emuliavimo gijos, po `CoreHandle::init()` (CLAUDE.md §3.2
    /// taisyklė #1) — `retro_load_game` gali kviesti bet kurį iš anksčiau užregistruotų
    /// callback'ų core'o viduje.
    pub unsafe fn load_game(&self, rom_path: &Path) -> Result<LoadedGameInfo, CoreError> {
        let sysinfo = self.system_info();
        let valid_extensions: Vec<String> = sysinfo
            .valid_extensions
            .split('|')
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();

        let is_archive = matches!(
            rom_path
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
                .as_deref(),
            Some("zip") | Some("7z")
        );

        // path_cstring/data_owned privalo gyventi bent tiek, kiek `game_info` naudojamas
        // žemiau — abu laikomi šio bloko viduje iki pat retro_load_game() kvietimo.
        let path_cstring: CString;
        let data_owned: Option<Vec<u8>>;

        if sysinfo.need_fullpath {
            let actual_path = if is_archive {
                archive::extract_first_match_to_temp(rom_path, &valid_extensions)?
            } else {
                rom_path.to_path_buf()
            };
            path_cstring = path_to_cstring(&actual_path)?;
            data_owned = None;
        } else {
            let bytes = if is_archive {
                archive::extract_first_match(rom_path, &valid_extensions)?.1
            } else {
                std::fs::read(rom_path)?
            };
            // libretro.h: net kai need_fullpath == false, geriau paduoti tikrą kelią nei
            // NULL — kai kurie core'ai jį naudoja kaip nuorodą kitiems failams sudaryti.
            path_cstring = path_to_cstring(rom_path)?;
            data_owned = Some(bytes);
        }

        let game_info = retro_game_info {
            path: path_cstring.as_ptr(),
            data: data_owned
                .as_ref()
                .map(|d| d.as_ptr() as *const c_void)
                .unwrap_or(std::ptr::null()),
            size: data_owned.as_ref().map(|d| d.len()).unwrap_or(0),
            meta: std::ptr::null(),
        };

        // SAFETY: `game_info` laukai (path_cstring/data_owned) gyvena visą šio kvietimo metu.
        let loaded = unsafe { (self.symbols.retro_load_game)(&game_info) };
        if !loaded {
            return Err(CoreError::Other(format!(
                "core'as atmetė ROM'ą: {}",
                rom_path.display()
            )));
        }

        let mut av_info: retro_system_av_info = unsafe { std::mem::zeroed() };
        // SAFETY: kviečiama TIK po sėkmingo retro_load_game() (žr. funkcijos doc komentarą).
        unsafe { (self.symbols.retro_get_system_av_info)(&mut av_info) };

        Ok(LoadedGameInfo {
            fps: av_info.timing.fps,
            sample_rate: av_info.timing.sample_rate,
            base_width: av_info.geometry.base_width,
            base_height: av_info.geometry.base_height,
            max_width: av_info.geometry.max_width,
            max_height: av_info.geometry.max_height,
            aspect_ratio: av_info.geometry.aspect_ratio,
        })
    }

    /// `retro_unload_game()`.
    ///
    /// # Safety
    /// Turi būti kviečiama tik iš emuliavimo gijos, po sėkmingo `load_game()`
    /// (CLAUDE.md §8.2 žingsnis 14).
    pub unsafe fn unload_game(&self) {
        unsafe { (self.symbols.retro_unload_game)() };
    }

    /// `retro_deinit()`. PRIVALO būti iškviesta prieš `CoreHandle` `drop` — žr.
    /// `Drop for CoreHandle` dokumentaciją (CLAUDE.md §8.2 žingsnis 14).
    ///
    /// # Safety
    /// Turi būti kviečiama tik iš emuliavimo gijos, po `unload_game()`.
    pub unsafe fn deinit(&self) {
        unsafe { (self.symbols.retro_deinit)() };
    }

    /// `retro_run()` — vykdo vieną kadrą. Naudos P1.7 `runner.rs` emuliavimo gijos loop'e.
    ///
    /// # Safety
    /// Turi būti kviečiama tik iš emuliavimo gijos, po sėkmingo `load_game()`
    /// (CLAUDE.md §3.2 taisyklė #1).
    pub unsafe fn run(&self) {
        unsafe { (self.symbols.retro_run)() };
    }

    /// `retro_reset()`.
    ///
    /// # Safety
    /// Turi būti kviečiama tik iš emuliavimo gijos, po sėkmingo `load_game()`.
    pub unsafe fn reset(&self) {
        unsafe { (self.symbols.retro_reset)() };
    }
}

fn path_to_cstring(path: &Path) -> Result<CString, CoreError> {
    let s = path
        .to_str()
        .ok_or_else(|| CoreError::Other(format!("kelias nėra UTF-8: {}", path.display())))?;
    CString::new(s)
        .map_err(|_| CoreError::Other(format!("kelias turi nul baitą: {}", path.display())))
}

/// Tikri `retro_get_system_av_info()` duomenys po `retro_load_game()` — fps/sample_rate/
/// geometry priklauso nuo konkretaus ROM'o, tad prieš load_game jie gali būti neteisingi.
#[derive(Debug, Clone)]
#[allow(dead_code)] // skaitys P1.7 runner.rs (frame pacing, geometry)
pub struct LoadedGameInfo {
    pub fps: f64,
    pub sample_rate: f64,
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
}

/// Visi 6 libretro callback'ai, reikalingi `CoreHandle::init()` (CLAUDE.md §8.2 žingsniai 3–8).
#[allow(dead_code)] // konstruoja tik P1.5 testai ir bus P1.7 runner.rs
pub struct RetroCallbacks {
    pub environment: retro_environment_t,
    pub video_refresh: retro_video_refresh_t,
    pub input_poll: retro_input_poll_t,
    pub input_state: retro_input_state_t,
    pub audio_sample: retro_audio_sample_t,
    pub audio_sample_batch: retro_audio_sample_batch_t,
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

    /// `crates/nullbyte-core/cores/` yra `.gitignore`'inta (CLAUDE.md §11.2) — vartotojas pats deda
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
                "praleista: crates/nullbyte-core/cores/genesis_plus_gx_libretro.dylib nerastas (lokalus test fixture)"
            );
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        // CoreHandle::load() viduje jau atmeta core'ą, jei api_version != 1 — sėkmingas
        // grąžinimas ir yra įrodymas, kad api_version == 1.
        let handle = CoreHandle::load(&path).expect("core'as turėtų sėkmingai įsikelti");
        assert_eq!(handle.path(), path);
    }

    #[test]
    fn missing_file_returns_app_error_not_panic() {
        let result = CoreHandle::load("/no/such/path/definitely_missing_core.dylib");
        assert!(matches!(result, Err(CoreError::Other(_))));
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

    // --- P1.6: ROM įkėlimas ---------------------------------------------------------------

    use super::super::callbacks::{install_context, take_context, EmuContext};

    fn snes9x_path() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/snes9x_libretro.dylib");
        path.exists().then_some(path)
    }

    fn mednafen_psx_path() -> Option<PathBuf> {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/mednafen_psx_libretro.dylib");
        path.exists().then_some(path)
    }

    fn bios_dir() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bios");
        path.is_dir().then_some(path)
    }

    /// Pirmas rastas failas kataloge su nurodytu plėtiniu — nepriklauso nuo konkretaus
    /// failo pavadinimo, tik nuo to, kad `crates/nullbyte-core/roms/<platforma>/` turi bent vieną.
    fn first_file_with_ext(dir: &Path, ext: &str) -> Option<PathBuf> {
        std::fs::read_dir(dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_lowercase)
                    .as_deref()
                    == Some(ext)
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

    #[test]
    fn snes_rom_loads_with_correct_fps() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let roms_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/snes");
        let Some(rom_path) = first_file_with_ext(&roms_dir, "sfc") else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext::default());
        let handle = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { handle.init(stub_callbacks()) };

        let info = unsafe { handle.load_game(&rom_path) }.expect("ROM'as turėtų įsikelti");
        // `read_dir` tvarka neapibrėžta — gali pataikyti ir į NTSC (~60.098), ir į PAL
        // (~50.0) ROM'ą. Svarbu, kad fps būtų TIKRA aparatūros reikšmė, ne apvalinta 60/50.
        let is_ntsc = (info.fps - 60.098).abs() < 1.0;
        let is_pal = (info.fps - 50.0).abs() < 1.0;
        assert!(
            is_ntsc || is_pal,
            "tikėtasi SNES NTSC (≈60.098) arba PAL (≈50.0) fps, gauta {}",
            info.fps
        );
        assert!(info.base_width > 0 && info.base_height > 0);

        unsafe {
            handle.unload_game();
            handle.deinit();
        }
        take_context();
    }

    #[test]
    fn zip_wrapped_rom_loads_via_archive_extraction() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };
        let roms_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/snes");
        let Some(rom_path) = first_file_with_ext(&roms_dir, "sfc") else {
            eprintln!("praleista: nė vieno .sfc faile roms/snes/ nerasta");
            return;
        };

        // Suvyniojame realų .sfc į laikiną .zip, kad patikrintume archyvo išpakavimo kelią.
        let rom_bytes = std::fs::read(&rom_path).unwrap();
        let inner_name = rom_path.file_name().unwrap().to_str().unwrap().to_string();
        let zip_path = std::env::temp_dir().join("nullbyte_test_snes_wrap.zip");
        {
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file(&inner_name, zip::write::SimpleFileOptions::default())
                .unwrap();
            use std::io::Write;
            writer.write_all(&rom_bytes).unwrap();
            writer.finish().unwrap();
        }

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext::default());
        let handle = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { handle.init(stub_callbacks()) };

        let result = unsafe { handle.load_game(&zip_path) };
        assert!(
            result.is_ok(),
            "zip'uotas ROM'as turėtų įsikelti: {result:?}"
        );

        unsafe {
            handle.unload_game();
            handle.deinit();
        }
        take_context();
        std::fs::remove_file(&zip_path).ok();
    }

    #[test]
    fn psx_core_with_need_fullpath_receives_path_not_buffer() {
        let Some(core_path) = mednafen_psx_path() else {
            eprintln!("praleista: mednafen_psx_libretro.dylib nerastas");
            return;
        };
        let Some(bios) = bios_dir() else {
            eprintln!("praleista: crates/nullbyte-core/bios/ nerastas");
            return;
        };
        let roms_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/psx");
        let Some(rom_path) = first_file_with_ext(&roms_dir, "zip") else {
            eprintln!("praleista: nė vieno .zip faile roms/psx/ nerasta");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext {
            system_dir: Some(path_to_cstring(&bios).expect("bios kelias turėtų būti UTF-8")),
            ..EmuContext::default()
        });

        let handle = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        assert!(
            handle.system_info().need_fullpath,
            "mednafen_psx turėtų reikalauti need_fullpath"
        );
        unsafe { handle.init(stub_callbacks()) };

        let result = unsafe { handle.load_game(&rom_path) };
        assert!(
            result.is_ok(),
            "PSX žaidimas su teisingu BIOS turėtų įsikelti: {result:?}"
        );

        unsafe {
            handle.unload_game();
            handle.deinit();
        }
        take_context();
    }

    #[test]
    fn missing_rom_file_returns_app_error_not_crash() {
        let Some(core_path) = snes9x_path() else {
            eprintln!("praleista: snes9x_libretro.dylib nerastas");
            return;
        };

        // Pastaba: snes9x (kaip dauguma SNES core'ų) LoROM header validaciją daro labai
        // atlaidžiai — bandymas paduoti šiukšlių baitus su .sfc plėtiniu realiai PAVYKO
        // (core'as juos interpretavo kaip validų LoROM). Todėl patikimam "blogo ROM'o"
        // scenarijui naudojame neegzistuojantį failą — tai testuoja mūsų PAČIŲ kelio
        // skaitymo klaidos apdorojimą (std::fs::read → CoreError::Io), nepriklausomai nuo
        // konkretaus core'o header validacijos griežtumo.
        let missing_rom = std::env::temp_dir().join("nullbyte_test_definitely_missing.sfc");
        std::fs::remove_file(&missing_rom).ok();

        let _core_lock = crate::core::test_support::lock_core_load();
        install_context(EmuContext::default());
        let handle = CoreHandle::load(&core_path).expect("core'as turėtų įsikelti");
        unsafe { handle.init(stub_callbacks()) };

        let result = unsafe { handle.load_game(&missing_rom) };
        assert!(
            matches!(result, Err(CoreError::Io(_))),
            "neegzistuojantis ROM'as turėtų grąžinti CoreError::Io, gauta {result:?}"
        );

        take_context();
    }
}
