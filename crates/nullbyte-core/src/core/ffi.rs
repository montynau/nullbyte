//! `libretro.h` C tipai, konstantos ir callback signatūros.
//!
//! Šaltinis: <https://github.com/libretro/RetroArch/blob/master/libretro-common/include/libretro.h>
//! (CLAUDE.md §15). Kiekviena konstanta ir struct'as patikrinti prieš originalą eilutė po
//! eilutės (P1.1 acceptance).
//!
//! **Pastaba:** CLAUDE.md §8.3 lentelėje trys reikšmės neatitiko tikro header'io —
//! `GET_CAN_DUPE` (buvo 13, iš tikrųjų 3), `SHUTDOWN` (buvo 23, iš tikrųjų 7 — 23 yra
//! `GET_RUMBLE_INTERFACE`) ir `GET_INPUT_BITMASKS` (buvo 52, iš tikrųjų `51 | EXPERIMENTAL` —
//! 52 yra `GET_CORE_OPTIONS_VERSION`). Čia naudojamos ištaisytos reikšmės iš originalo.
//!
//! Struct'ų vardai sąmoningai `snake_case`, tiksliai atkartojantys C pavadinimus — taip
//! lengviau audituoti prieš originalą (bindgen konvencija).

// Struct'ų/tipų vardai atitinka C 1:1 (audito patogumui), ne Rust UpperCamelCase konvenciją.
#![allow(non_camel_case_types)]
// Šis modulis tik apibrėžia FFI tipus/konstantas — juos realiai naudos loader.rs (P1.2),
// callbacks.rs (P1.4) ir environment.rs (P1.5), kurie dar neparašyti.
#![allow(dead_code)]

use std::ffi::{c_char, c_void};
use std::mem::size_of;

// ---------------------------------------------------------------------------------------------
// Versija
// ---------------------------------------------------------------------------------------------

pub const RETRO_API_VERSION: u32 = 1;

// ---------------------------------------------------------------------------------------------
// retro_environment cmd ID's (CLAUDE.md §8.3 — bent šios privalomos MVP metu)
// ---------------------------------------------------------------------------------------------

/// Žymė, pridedama prie eksperimentinių komandų ID (pvz. `GET_INPUT_BITMASKS`).
pub const RETRO_ENVIRONMENT_EXPERIMENTAL: u32 = 0x10000;

pub const RETRO_ENVIRONMENT_GET_CAN_DUPE: u32 = 3;
pub const RETRO_ENVIRONMENT_SET_MESSAGE: u32 = 6;
pub const RETRO_ENVIRONMENT_SHUTDOWN: u32 = 7;
pub const RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL: u32 = 8;
pub const RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY: u32 = 9;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: u32 = 10;
pub const RETRO_ENVIRONMENT_GET_VARIABLE: u32 = 15;
pub const RETRO_ENVIRONMENT_SET_VARIABLES: u32 = 16;
pub const RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE: u32 = 17;
pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: u32 = 27;
pub const RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY: u32 = 31;
pub const RETRO_ENVIRONMENT_GET_LANGUAGE: u32 = 39;
pub const RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION: u32 = 52;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS: u32 = 53;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL: u32 = 54;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY: u32 = 55;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2: u32 = 67;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL: u32 = 68;
pub const RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: u32 = 69;

/// `51 | RETRO_ENVIRONMENT_EXPERIMENTAL` — vis dar stabili, plačiai palaikoma komanda,
/// nepaisant EXPERIMENTAL žymės (CLAUDE.md §8.3).
pub const RETRO_ENVIRONMENT_GET_INPUT_BITMASKS: u32 = 51 | RETRO_ENVIRONMENT_EXPERIMENTAL;

// ---------------------------------------------------------------------------------------------
// Pixel formatai (RETRO_ENVIRONMENT_SET_PIXEL_FORMAT duomenys)
// ---------------------------------------------------------------------------------------------

pub const RETRO_PIXEL_FORMAT_0RGB1555: u32 = 0;
pub const RETRO_PIXEL_FORMAT_XRGB8888: u32 = 1;
pub const RETRO_PIXEL_FORMAT_RGB565: u32 = 2;

// ---------------------------------------------------------------------------------------------
// Įvestis — RETRO_DEVICE_JOYPAD ir mygtukų ID's
// ---------------------------------------------------------------------------------------------

pub const RETRO_DEVICE_JOYPAD: u32 = 1;

pub const RETRO_DEVICE_ID_JOYPAD_B: u32 = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: u32 = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: u32 = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: u32 = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: u32 = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: u32 = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: u32 = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: u32 = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: u32 = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: u32 = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: u32 = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: u32 = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: u32 = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: u32 = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: u32 = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: u32 = 15;

/// `retro_input_state_t` su šiuo ID grąžina visų 16 mygtukų būvį kaip bitmask
/// (žr. `RETRO_ENVIRONMENT_GET_INPUT_BITMASKS`).
pub const RETRO_DEVICE_ID_JOYPAD_MASK: u32 = 256;

// ---------------------------------------------------------------------------------------------
// Atmintis (retro_get_memory_data/size)
// ---------------------------------------------------------------------------------------------

pub const RETRO_MEMORY_SAVE_RAM: u32 = 0;

// ---------------------------------------------------------------------------------------------
// Log lygiai
// ---------------------------------------------------------------------------------------------

pub const RETRO_LOG_DEBUG: i32 = 0;
pub const RETRO_LOG_INFO: i32 = 1;
pub const RETRO_LOG_WARN: i32 = 2;
pub const RETRO_LOG_ERROR: i32 = 3;

// ---------------------------------------------------------------------------------------------
// Struct'ai (#[repr(C)] — laukų tvarka ir tipai tikslūs pagal libretro.h)
// ---------------------------------------------------------------------------------------------

#[repr(C)]
pub struct retro_system_info {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

// 3 rodyklės (3×8) + 2 bool (2×1) = 26, paddinama iki 32 (8 baitų alignment nuo rodyklių).
const _: () = assert!(size_of::<retro_system_info>() == 32);

#[repr(C)]
pub struct retro_game_geometry {
    pub base_width: u32,
    pub base_height: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub aspect_ratio: f32,
}

const _: () = assert!(size_of::<retro_game_geometry>() == 20);

#[repr(C)]
pub struct retro_system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

const _: () = assert!(size_of::<retro_system_timing>() == 16);

#[repr(C)]
pub struct retro_system_av_info {
    pub geometry: retro_game_geometry,
    pub timing: retro_system_timing,
}

const _: () = assert!(size_of::<retro_system_av_info>() == 40);

#[repr(C)]
pub struct retro_game_info {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

// 4 laukai po 8 baitus (path, data, size, meta) — visi 8 baitų alignment, be paddingo.
const _: () = assert!(size_of::<retro_game_info>() == 32);

#[repr(C)]
pub struct retro_variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

const _: () = assert!(size_of::<retro_variable>() == 16);

#[repr(C)]
pub struct retro_message {
    pub msg: *const c_char,
    pub frames: u32,
}

// 8 (msg) + 4 (frames) = 12, paddinama iki 16 (8 baitų alignment nuo rodyklės).
const _: () = assert!(size_of::<retro_message>() == 16);

/// `void (*)(enum retro_log_level level, const char *fmt, ...)`.
pub type retro_log_printf_t = unsafe extern "C" fn(level: i32, fmt: *const c_char, ...);

#[repr(C)]
pub struct retro_log_callback {
    pub log: retro_log_printf_t,
}

const _: () = assert!(size_of::<retro_log_callback>() == 8);

// ---------------------------------------------------------------------------------------------
// Callback tipų aliasai (visi be `user_data` — žr. CLAUDE.md §3.3)
// ---------------------------------------------------------------------------------------------

pub type retro_environment_t = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

pub type retro_video_refresh_t =
    unsafe extern "C" fn(data: *const c_void, width: u32, height: u32, pitch: usize);

pub type retro_audio_sample_t = unsafe extern "C" fn(left: i16, right: i16);

pub type retro_audio_sample_batch_t =
    unsafe extern "C" fn(data: *const i16, frames: usize) -> usize;

pub type retro_input_poll_t = unsafe extern "C" fn();

pub type retro_input_state_t =
    unsafe extern "C" fn(port: u32, device: u32, index: u32, id: u32) -> i16;
