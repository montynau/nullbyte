//! Settings Tauri komandos (CLAUDE.md §6.3) — P7.6 Cores panelė.
//!
//! `list_cores` yra PILNAI FUNKCIONALUS (skaito `cores_dir`, jokio P9.1 blokerio — žr.
//! `commands::input` modulio doc dėl to bloko). `get_preferred_cores`/`set_preferred_cores`
//! kenčia nuo TO PATIES apribojimo kaip `commands::input` mapping'as: išsaugoma, bet
//! realaus žaidimo paleidimo dar niekas nenaudoja (P9.1 dar neįgyvendinta).

use tauri::State;

use crate::db::settings;
use crate::error::AppError;
use crate::state::AppState;

/// `nullbyte_core::core::info::CoreInfo` IPC-saugi versija — CLAUDE.md §7.3 (camelCase,
/// `PathBuf` → `String`).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfoDto {
    pub path: String,
    pub name: String,
    pub version: String,
    pub valid_extensions: Vec<String>,
    pub need_fullpath: bool,
    pub system_name: Option<String>,
    pub manufacturer: Option<String>,
    /// Kuruotas (rankiniu būdu patikrintas) platformų `slug` sąrašas, kurias šis core'as
    /// TIKRAI palaiko — žr. `known_core_platforms` doc. `None`, jei core'o pavadinimo NĖRA
    /// lentelėje: UI TURI traktuoti tai kaip „nepatikrinta", ne „nepalaiko nieko" (rodyti
    /// tokį core'ą VISUR, ne slėpti).
    pub supported_platforms: Option<Vec<String>>,
}

/// Kuruota core'o pavadinimo (`retro_get_system_info().library_name`, TIKSLIAI kaip core'as
/// PATS praneša — NE core'o failo vardas) → palaikomų platformų `slug` sąrašas lentelė.
///
/// **KODĖL šitai reikalinga, ne extension'ų sutapimas:** libretro API NETURI „kokias
/// platformas palaikau" lauko — TIK `valid_extensions` (failo plėtiniai). ADR-024 (P7.6
/// realiu patikrinimu, 2026-08-26) parodė, kad extension'ų sutapimas duoda KLAIDINGUS
/// teigiamus rezultatus: PicoDrive/Genesis Plus GX (abu palaiko Sega CD) IR MAME (plati
/// `zip`/`7z` deklaracija) atitinka PSX plėtinius (`cue`/`chd`/`m3u` — net `m3u`, kurį
/// tikėtasi esant PSX-unikaliu, PicoDrive/Genesis Plus GX naudoja savo Sega CD daugiadiskių
/// sąrašams). Sega CD ir Saturn `platforms.extensions` yra BYTE-FOR-BYTE identiški
/// (`cue,iso,chd,zip,7z`) — tarp jų NĖRA JOKIO extension signalo, kad ir kaip
/// sudėtinga heuristika būtų sugalvota. `.info` failai (kuriuose būtų patikimesnis
/// `systemname` laukas) — NEBŪTINAS, atskiras atsisiuntimas, dažnai jo NĖRA (šioje
/// aplinkoje jo NĖRA nė vienam core'ui). Tad vienintelis TIKSLUS sprendimas —
/// rankiniu būdu patikrinta lentelė, ta pati filosofija kaip `platforms` seed'as (P5.1) su
/// kuruotais ScreenScraper ID'ais.
///
/// **Įtraukti TIK REALIAI patikrinti šioje sesijoje esantys core'ai** (2026-08-26,
/// `crates/nullbyte-core/cores/*.dylib`) — nespėliojama apie neturimus core'us. Pridedant
/// naują core'ą ateityje: patikrink TIKSLŲ jo praneštą `library_name` (ne spėk iš failo
/// vardo — pvz. visi trys `bsnes_mercury_*_libretro.dylib` failai praneša TĄ PATĮ pavadinimą
/// „bsnes-mercury", nepriklausomai nuo failo vardo), tada jo REALIAI dokumentuotą sistemų
/// sąrašą (libretro core'o README/dokumentacija).
fn known_core_platforms(core_name: &str) -> Option<&'static [&'static str]> {
    match core_name {
        "Snes9x" => Some(&["snes"]),
        "bsnes-mercury" => Some(&["snes"]),
        "mGBA" => Some(&["gba", "gb", "gbc"]),
        "Genesis Plus GX" => Some(&["genesis", "segacd", "mastersystem", "gamegear"]),
        "PicoDrive" => Some(&["genesis", "sega32x", "segacd", "mastersystem", "gamegear"]),
        "Beetle PSX" => Some(&["psx"]),
        "Beetle PSX HW" => Some(&["psx"]),
        "SwanStation" => Some(&["psx"]),
        "MAME" => Some(&["arcade"]),
        _ => None,
    }
}

/// Kai KELETAS core'ų palaiko tą pačią platformą (žr. `known_core_platforms`), ši lentelė
/// nurodo REKOMENDUOJAMĄ tvarką (pirmas rastas `cores_dir` laimi) — naudojama P7.6 Cores
/// panelės automatiniam pasiūlymui, kai vartotojas dar nieko nepasirinko rankiniu būdu
/// (vartotojo prašymas: „galima iškart uždėti jei randa rekomenduojamą... nenorėčiau
/// įsirašęs programą viską nuo 0 suvedinėti"). Prioritetas — bendrai priimta libretro
/// bendruomenės nuomonė (tikslumas/suderinamumas), NE griežtai išmatuota — vartotojas visada
/// gali pakeisti rankiniu būdu.
const CORE_PRIORITY_ORDER: &[(&str, &[&str])] = &[
    ("snes", &["Snes9x", "bsnes-mercury"]),
    ("gba", &["mGBA"]),
    ("gb", &["mGBA"]),
    ("gbc", &["mGBA"]),
    ("genesis", &["Genesis Plus GX", "PicoDrive"]),
    ("mastersystem", &["Genesis Plus GX", "PicoDrive"]),
    ("gamegear", &["Genesis Plus GX", "PicoDrive"]),
    ("segacd", &["Genesis Plus GX", "PicoDrive"]),
    ("sega32x", &["PicoDrive"]),
    ("psx", &["SwanStation", "Beetle PSX HW", "Beetle PSX"]),
    ("arcade", &["MAME"]),
];

/// Grynai statiniai duomenys (jokio I/O, jokio `cores_dir` skenavimo) — frontend'as sujungia
/// su JAU turimu `list_cores` rezultatu, kad IŠVENGTŲ pakartotinio core'ų įkėlimo (kai kurie,
/// pvz. MAME, ~400MB — brangu skenuoti du kartus).
#[tauri::command]
pub fn get_core_priority() -> std::collections::HashMap<String, Vec<String>> {
    CORE_PRIORITY_ORDER
        .iter()
        .map(|&(slug, names)| {
            (
                slug.to_string(),
                names.iter().map(|s| s.to_string()).collect(),
            )
        })
        .collect()
}

impl From<nullbyte_core::core::info::CoreInfo> for CoreInfoDto {
    fn from(info: nullbyte_core::core::info::CoreInfo) -> Self {
        let supported_platforms = known_core_platforms(&info.name)
            .map(|slugs| slugs.iter().map(|s| s.to_string()).collect());
        Self {
            path: info.path.to_string_lossy().into_owned(),
            name: info.name,
            version: info.version,
            valid_extensions: info.valid_extensions,
            need_fullpath: info.need_fullpath,
            system_name: info.system_name,
            manufacturer: info.manufacturer,
            supported_platforms,
        }
    }
}

/// Aptinka `cores_dir` esančius libretro core'us (P1.3 `scan_cores_dir` perpanaudotas
/// nepakeistas). Tuščias sąrašas — NE klaida — jei katalogas dar neegzistuoja (naujam
/// diegimui core'ų dar gali nebūti atsisiųsta, žr. MVP.md P1.3 doc).
#[tauri::command]
pub fn list_cores(state: State<'_, AppState>) -> Result<Vec<CoreInfoDto>, AppError> {
    if !state.cores_dir.is_dir() {
        return Ok(Vec::new());
    }
    let cores = nullbyte_core::core::info::scan_cores_dir(&state.cores_dir)?;
    Ok(cores.into_iter().map(CoreInfoDto::from).collect())
}

/// Vieno platformos → core'o priskyrimo įrašas — CLAUDE.md §7.3 (camelCase).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCorePreference {
    pub platform_slug: String,
    pub core_path: String,
}

const PREFERRED_CORES_KEY: &str = "core.preferred";

#[tauri::command]
pub fn get_preferred_cores(
    state: State<'_, AppState>,
) -> Result<Vec<PlatformCorePreference>, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    match settings::get(&conn, PREFERRED_CORES_KEY)? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| AppError::Other(format!("sugadintas core.preferred JSON: {error}"))),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn set_preferred_cores(
    state: State<'_, AppState>,
    preferences: Vec<PlatformCorePreference>,
) -> Result<(), AppError> {
    let json = serde_json::to_string(&preferences).map_err(|error| {
        AppError::Other(format!("nepavyko serializuoti core.preferred: {error}"))
    })?;
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, PREFERRED_CORES_KEY, &json)
}

/// P7.6 Video panelė — CLAUDE.md §7.3 (camelCase). `filter`/`scaleMode` reikšmės TIKSLIAI
/// atitinka `nullbyte_core::video::renderer::{FilterMode, ScaleMode}` variantus (žemyn
/// suserializuotus kaip `"nearest"|"linear"` / `"fit"|"integer"`) — kad P9.1 wiring metu
/// nereikėtų perkelti reikšmių, tik parse'inti į jau egzistuojantį enum'ą. **TIK
/// persistencija** — `Renderer::set_filter`/`set_scale_mode` JAU EGZISTUOJA ir veikia, bet
/// nėra JOKIO IPC kanalo (naujas `EmuCommand` variantas) šiai reikšmei nuo `nullbyte-app`
/// pasiekti `nullbyte-emu` vaiko procesą — tas pats P9.1 apribojimas kaip Input/Cores.
/// `vsync`/`startFullscreen` neturi JOKIO esamo runtime hook'o net Rust pusėje (vsync
/// „baked" į `SurfaceConfiguration` kūrimo metu, fullscreen tik F11 toggle) — abu reikalautų
/// naujo kodo net PRIEŠ P9.1 wiring'ą.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoSettings {
    pub filter: String,
    pub scale_mode: String,
    pub vsync: bool,
    pub start_fullscreen: bool,
}

impl Default for VideoSettings {
    /// Atitinka `FilterMode`/`ScaleMode` `#[default]` variantus (`renderer.rs`) — kad UI
    /// numatytoji reikšmė pirmą kartą atidarius sutaptų su tuo, ką core'as REALIAI naudotų,
    /// jei šis pasirinkimas šiandien turėtų P9.1 vamzdyną.
    fn default() -> Self {
        Self {
            filter: "nearest".to_string(),
            scale_mode: "fit".to_string(),
            vsync: true,
            start_fullscreen: false,
        }
    }
}

const VIDEO_SETTINGS_KEY: &str = "video.settings";

#[tauri::command]
pub fn get_video_settings(state: State<'_, AppState>) -> Result<VideoSettings, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    match settings::get(&conn, VIDEO_SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| AppError::Other(format!("sugadintas video.settings JSON: {error}"))),
        None => Ok(VideoSettings::default()),
    }
}

#[tauri::command]
pub fn set_video_settings(
    state: State<'_, AppState>,
    value: VideoSettings,
) -> Result<(), AppError> {
    let json = serde_json::to_string(&value).map_err(|error| {
        AppError::Other(format!("nepavyko serializuoti video.settings: {error}"))
    })?;
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, VIDEO_SETTINGS_KEY, &json)
}

/// P7.6 Audio panelė. **`device`/`volume`/`bufferMs` visi TIK persistencija** — skirtingai
/// nuo Video (kur bent `filter`/`scaleMode` jau turi veikiantį Rust API), garso pusėje
/// (`audio/output.rs`) NĖRA JOKIO esamo mechanizmo pasirinktam įrenginiui atidaryti (dabar
/// visada `host.default_output_device()`), garsumui taikyti (sample'ai keliauja
/// nekeisti), ar buferio dydžiui keisti be perkompiliavimo (`TARGET_LATENCY_MS` — hardkodinta
/// konstanta) — VISI trys reikalautų naujo kodo `nullbyte-core`/`nullbyte-emu` pusėje, NE
/// vien P9.1 IPC wiring'o.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    /// `None` = numatytasis sistemos įrenginys. Kitaip — TIKSLUS `cpal` įrenginio
    /// pavadinimas (žr. `list_audio_devices`).
    pub device: Option<String>,
    /// 0.0..=1.0, apkerpama `set_audio_settings` viduje.
    pub volume: f32,
    /// Milisekundės — atitinka `nullbyte_core::audio::output::TARGET_LATENCY_MS` (dabar
    /// hardkodinta `50`).
    pub buffer_ms: u32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            device: None,
            volume: 1.0,
            buffer_ms: 50,
        }
    }
}

const AUDIO_SETTINGS_KEY: &str = "audio.settings";

#[tauri::command]
pub fn get_audio_settings(state: State<'_, AppState>) -> Result<AudioSettings, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    match settings::get(&conn, AUDIO_SETTINGS_KEY)? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| AppError::Other(format!("sugadintas audio.settings JSON: {error}"))),
        None => Ok(AudioSettings::default()),
    }
}

#[tauri::command]
pub fn set_audio_settings(
    state: State<'_, AppState>,
    mut value: AudioSettings,
) -> Result<(), AppError> {
    value.volume = value.volume.clamp(0.0, 1.0);
    let json = serde_json::to_string(&value).map_err(|error| {
        AppError::Other(format!("nepavyko serializuoti audio.settings: {error}"))
    })?;
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, AUDIO_SETTINGS_KEY, &json)
}

/// Realiai veikiantis (jokio P9.1 blokerio) sistemos garso išvesties įrenginių sąrašas —
/// `cpal` enumeracija VEIKIA nepriklausomai nuo to, ar koks nors garso srautas šiuo metu
/// atidarytas (tai tik OS užklausa, ne aktyvi srauto operacija), tad `nullbyte-app` gali ją
/// kviesti tiesiogiai, NEATIDARANT jokio srauto ir NELAUKIANT `nullbyte-emu` vaiko proceso.
/// Tuščias sąrašas (NE panic/klaida) klaidos atveju — loginama, bet UI tiesiog rodo „Default
/// only".
#[tauri::command]
pub fn list_audio_devices() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    match host.output_devices() {
        Ok(devices) => devices.filter_map(|d| d.name().ok()).collect(),
        Err(error) => {
            tracing::warn!(%error, "nepavyko išvardinti garso išvesties įrenginių");
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN_CORE_NAMES: &[&str] = &[
        "Snes9x",
        "bsnes-mercury",
        "mGBA",
        "Genesis Plus GX",
        "PicoDrive",
        "Beetle PSX",
        "Beetle PSX HW",
        "SwanStation",
        "MAME",
    ];

    /// Kiekvienas `known_core_platforms` grąžintas `slug` PRIVALO egzistuoti realioje seed'o
    /// lentelėje (P5.1 migracija 001) — kitaip UI tyliai niekada nerastų šios platformos
    /// `library.platforms` sąraše ir preferuojamo core'o Select'as jos net nerodytų.
    #[test]
    fn every_known_core_platform_slug_exists_in_the_seed_table() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::db::migrations::MIGRATIONS
            .iter()
            .for_each(|(_, sql)| conn.execute_batch(sql).unwrap());

        for &core_name in KNOWN_CORE_NAMES {
            let slugs = known_core_platforms(core_name).unwrap_or_else(|| {
                panic!("{core_name} turėtų būti known_core_platforms lentelėje")
            });
            for &slug in slugs {
                let exists: bool = conn
                    .query_row(
                        "SELECT EXISTS(SELECT 1 FROM platforms WHERE slug = ?1)",
                        rusqlite::params![slug],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert!(
                    exists,
                    "{core_name} nurodo neegzistuojantį platform slug '{slug}'"
                );
            }
        }
    }

    /// ADR-024 esmė: PSX turi rodyti TIK tikrus PSX core'us, NE PicoDrive/Genesis Plus GX/MAME
    /// (kurie klaidingai atitikdavo per extension'ų sutapimą prieš šį pataisymą).
    #[test]
    fn psx_maps_to_exactly_the_three_real_psx_cores() {
        let psx_cores: Vec<&str> = KNOWN_CORE_NAMES
            .iter()
            .filter(|&&name| known_core_platforms(name).unwrap().contains(&"psx"))
            .copied()
            .collect();
        assert_eq!(
            psx_cores,
            vec!["Beetle PSX", "Beetle PSX HW", "SwanStation"],
        );
    }

    #[test]
    fn unrecognized_core_name_returns_none() {
        assert_eq!(known_core_platforms("Some Future Core"), None);
    }

    /// Apsauga nuo lentelių išsiskyrimo (typo, pamirštas atnaujinti vieną iš dviejų): kiekvienas
    /// `CORE_PRIORITY_ORDER` core'o vardas TURI būti realiai `known_core_platforms` nurodytas
    /// KAIP palaikantis TĄ PAČIĄ platformą — kitaip rekomendacija tyliai niekada nesutaptų su
    /// jokiu core'u.
    #[test]
    fn every_priority_entry_is_consistent_with_known_core_platforms() {
        for &(slug, names) in CORE_PRIORITY_ORDER {
            for &name in names {
                let supported = known_core_platforms(name).unwrap_or_else(|| {
                    panic!(
                        "{name} (CORE_PRIORITY_ORDER['{slug}']) neturi known_core_platforms įrašo"
                    )
                });
                assert!(
                    supported.contains(&slug),
                    "{name} yra CORE_PRIORITY_ORDER['{slug}'], bet known_core_platforms jo nesieja su '{slug}'"
                );
            }
        }
    }

    /// `VideoSettings::default()` TURI atitikti `nullbyte_core::video::renderer::{FilterMode,
    /// ScaleMode}` `#[default]` variantus — kitaip UI pirmą kartą rodytų kitokią reikšmę nei
    /// core'as REALIAI naudotų, jei šiandien turėtų P9.1 vamzdyną.
    #[test]
    fn video_settings_default_matches_renderer_defaults() {
        assert_eq!(
            VideoSettings::default().filter,
            format!(
                "{:?}",
                nullbyte_core::video::renderer::FilterMode::default()
            )
            .to_lowercase()
        );
        assert_eq!(
            VideoSettings::default().scale_mode,
            format!("{:?}", nullbyte_core::video::renderer::ScaleMode::default()).to_lowercase()
        );
    }

    #[test]
    fn video_settings_roundtrips_through_json() {
        let original = VideoSettings {
            filter: "linear".to_string(),
            scale_mode: "integer".to_string(),
            vsync: false,
            start_fullscreen: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: VideoSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.filter, original.filter);
        assert_eq!(parsed.scale_mode, original.scale_mode);
        assert_eq!(parsed.vsync, original.vsync);
        assert_eq!(parsed.start_fullscreen, original.start_fullscreen);
    }

    #[test]
    fn audio_settings_default_is_system_default_device_full_volume() {
        let default = AudioSettings::default();
        assert_eq!(default.device, None);
        assert_eq!(default.volume, 1.0);
        assert_eq!(default.buffer_ms, 50);
    }
}
