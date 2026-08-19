//! `retro_environment` komandų apdorojimas (CLAUDE.md §8.3, P1.5).
//!
//! Visos komandos iš CLAUDE.md §8.3 lentelės (pataisytos reikšmės — žr. `ffi.rs` pastabą apie
//! P1.1 metu rastas klaidas). Nežinomos komandos → `tracing::debug!` su ID ir `false`.
//!
//! **`GET_LOG_INTERFACE` ABI apribojimas:** libretro `retro_log_printf_t` yra C-variadic
//! (`void (*)(level, fmt, ...)`), o stabilus Rust (dar) negali apibrėžti tokių funkcijų
//! (rust-lang/rust#44930 — planuojama stabilizuoti Rust 1.99). `core_log_printf` čia priima
//! tik `level`+`fmt` ir NESKAITO varargs — core'ų log pranešimai su `%s`/`%d` bus rodomi
//! NEIŠPLĖSTU formatu. Funkcijos rodyklė transmute'inama į variadic tipą; tai veikia System V
//! AMD64 / AAPCS64 (macOS + Linux, x86_64 + aarch64 — mūsų vieninteliai taikiniai), nes
//! fiksuotų parametrų perdavimas identiškas variadic ir ne-variadic funkcijoms C ABI lygyje.

use std::ffi::{c_char, c_void, CStr, CString};

use super::callbacks::with_context;
use super::ffi::{
    retro_log_callback, retro_log_printf_t, retro_message, retro_variable,
    RETRO_ENVIRONMENT_GET_CAN_DUPE, RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION,
    RETRO_ENVIRONMENT_GET_INPUT_BITMASKS, RETRO_ENVIRONMENT_GET_LANGUAGE,
    RETRO_ENVIRONMENT_GET_LOG_INTERFACE, RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
    RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY, RETRO_ENVIRONMENT_GET_VARIABLE,
    RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE, RETRO_ENVIRONMENT_SET_CORE_OPTIONS,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK,
    RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2, RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL,
    RETRO_ENVIRONMENT_SET_MESSAGE, RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL,
    RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, RETRO_ENVIRONMENT_SET_VARIABLES,
    RETRO_ENVIRONMENT_SHUTDOWN, RETRO_LOG_DEBUG, RETRO_LOG_ERROR, RETRO_LOG_WARN,
    RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
};

/// Pagrindinis `retro_environment` dispatch'as — kviečiamas iš `callbacks::environment_cb`.
///
/// # Safety
/// `data` reikšmė ir tipas priklauso nuo `cmd`; kiekvienos šakos SAFETY komentaras
/// dokumentuoja, ko iš `data` reikalaujama tai konkrečiai komandai (libretro kontraktas).
pub unsafe fn handle(cmd: u32, data: *mut c_void) -> bool {
    match cmd {
        RETRO_ENVIRONMENT_GET_LOG_INTERFACE => unsafe { get_log_interface(data) },
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => unsafe { set_pixel_format(data) },
        RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => unsafe {
            write_optional_dir(data, |ctx| ctx.system_dir.as_ref())
        },
        RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => unsafe {
            write_optional_dir(data, |ctx| ctx.save_dir.as_ref())
        },
        RETRO_ENVIRONMENT_GET_CAN_DUPE => unsafe { write_bool(data, true) },
        RETRO_ENVIRONMENT_SET_VARIABLES => unsafe { set_variables(data) },
        RETRO_ENVIRONMENT_GET_VARIABLE => unsafe { get_variable(data) },
        RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => unsafe { write_bool(data, false) },
        RETRO_ENVIRONMENT_SET_MESSAGE => unsafe { set_message(data) },
        RETRO_ENVIRONMENT_SHUTDOWN => {
            tracing::info!("core'as paprašė RETRO_ENVIRONMENT_SHUTDOWN");
            with_context(|ctx| ctx.shutdown_requested = true);
            true
        }
        RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL => true,
        RETRO_ENVIRONMENT_GET_LANGUAGE => unsafe { write_u32(data, 0) }, // RETRO_LANGUAGE_ENGLISH
        RETRO_ENVIRONMENT_GET_INPUT_BITMASKS => true, // input_state_cb jau palaiko (P1.4)
        RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => false, // → core'as naudos legacy SET_VARIABLES
        RETRO_ENVIRONMENT_SET_CORE_OPTIONS
        | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL
        | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY
        | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2
        | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL
        | RETRO_ENVIRONMENT_SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK => {
            tracing::debug!(
                cmd,
                "core options komanda pripažinta, bet neparsinama (Core options UI — post-MVP, CLAUDE.md §1.3)"
            );
            true
        }
        _ => {
            tracing::debug!(cmd, "nežinoma arba nepalaikoma retro_environment komanda");
            false
        }
    }
}

/// # Safety
/// `data` turi rodyti į galiojantį `retro_log_callback`, kai `cmd == GET_LOG_INTERFACE`.
unsafe fn get_log_interface(data: *mut c_void) -> bool {
    if data.is_null() {
        return false;
    }

    // SAFETY: core_log_printf turi tą pačią fiksuotų parametrų (level, fmt) ABI kaip
    // retro_log_printf_t — žr. modulio doc komentarą apie C-variadic apribojimą.
    let log_fn: retro_log_printf_t = unsafe {
        std::mem::transmute::<unsafe extern "C" fn(i32, *const c_char), retro_log_printf_t>(
            core_log_printf,
        )
    };

    let out = data as *mut retro_log_callback;
    // SAFETY: žr. funkcijos SAFETY komentarą.
    unsafe { (*out).log = log_fn };
    with_context(|ctx| ctx.log_callback = Some(log_fn));
    true
}

/// Faktinis log callback'as, kurį core'as kviečia (transmute'intas per `get_log_interface`).
/// Neskaito varargs — žr. modulio doc komentarą.
unsafe extern "C" fn core_log_printf(level: i32, fmt: *const c_char) {
    if fmt.is_null() {
        return;
    }
    // SAFETY: core garantuoja `fmt` yra galiojanti nul-terminuota eilutė šio iškvietimo metu.
    let message = unsafe { CStr::from_ptr(fmt) }.to_string_lossy();
    let message = message.trim_end_matches('\n');

    if level == RETRO_LOG_ERROR {
        tracing::error!(target: "core", "{message}");
    } else if level == RETRO_LOG_WARN {
        tracing::warn!(target: "core", "{message}");
    } else if level == RETRO_LOG_DEBUG {
        tracing::debug!(target: "core", "{message}");
    } else {
        tracing::info!(target: "core", "{message}");
    }
}

/// # Safety
/// `data` turi rodyti į galiojantį `int` (retro_pixel_format), kai `cmd == SET_PIXEL_FORMAT`.
unsafe fn set_pixel_format(data: *mut c_void) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: žr. funkcijos SAFETY komentarą.
    let format = unsafe { *(data as *const i32) } as u32;

    match format {
        RETRO_PIXEL_FORMAT_0RGB1555 | RETRO_PIXEL_FORMAT_XRGB8888 | RETRO_PIXEL_FORMAT_RGB565 => {
            with_context(|ctx| ctx.pixel_format = format);
            tracing::debug!(format, "pixel format nustatytas");
            true
        }
        _ => {
            tracing::warn!(format, "core'as prašo nepalaikomo pixel format");
            false
        }
    }
}

/// # Safety
/// `data` turi rodyti į galiojantį `*const c_char` (rašymui), kai `cmd` yra `GET_SYSTEM_DIRECTORY`
/// arba `GET_SAVE_DIRECTORY`.
unsafe fn write_optional_dir(
    data: *mut c_void,
    select: impl FnOnce(&super::callbacks::EmuContext) -> Option<&CString>,
) -> bool {
    if data.is_null() {
        return false;
    }
    let out = data as *mut *const c_char;

    with_context(|ctx| {
        let ptr = select(ctx).map(|c| c.as_ptr()).unwrap_or(std::ptr::null());
        // SAFETY: žr. funkcijos SAFETY komentarą.
        unsafe { *out = ptr };
    });
    true
}

/// # Safety
/// `data` turi rodyti į galiojantį `bool` (rašymui).
unsafe fn write_bool(data: *mut c_void, value: bool) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: žr. funkcijos SAFETY komentarą.
    unsafe { *(data as *mut bool) = value };
    true
}

/// # Safety
/// `data` turi rodyti į galiojantį `u32` (rašymui).
unsafe fn write_u32(data: *mut c_void, value: u32) -> bool {
    if data.is_null() {
        return false;
    }
    // SAFETY: žr. funkcijos SAFETY komentarą.
    unsafe { *(data as *mut u32) = value };
    true
}

/// # Safety
/// `data` turi rodyti į NULL-terminuotą `retro_variable` masyvą (paskutinis įrašas su
/// `key == NULL`), kai `cmd == SET_VARIABLES`.
unsafe fn set_variables(data: *mut c_void) -> bool {
    if data.is_null() {
        return true; // NULL legacy specifikacijoje reiškia „nieko papildomai" — ne klaida.
    }

    with_context(|ctx| {
        ctx.core_options.clear();
        let mut ptr = data as *const retro_variable;
        loop {
            // SAFETY: masyvas baigiasi įrašu su key == NULL (libretro kontraktas).
            let entry = unsafe { &*ptr };
            if entry.key.is_null() {
                break;
            }
            // SAFETY: core garantuoja galiojančias nul-terminuotas eilutes.
            let key = unsafe { CStr::from_ptr(entry.key) }
                .to_string_lossy()
                .into_owned();

            let default_value = if entry.value.is_null() {
                String::new()
            } else {
                // SAFETY: core garantuoja galiojančią nul-terminuotą eilutę.
                let raw = unsafe { CStr::from_ptr(entry.value) }.to_string_lossy();
                parse_legacy_default(&raw)
            };

            if let Ok(value) = CString::new(default_value) {
                ctx.core_options.insert(key, value);
            }

            // SAFETY: masyvas turi bent vieną daugiau įrašą (terminatorių), tad +1 saugus.
            ptr = unsafe { ptr.add(1) };
        }
    });

    true
}

/// Legacy `SET_VARIABLES` reikšmės formatas: `"Aprašymas; default|opt2|opt3"`.
/// Grąžina `default` dalį.
fn parse_legacy_default(raw: &str) -> String {
    raw.split_once(';')
        .map_or(raw, |(_, rest)| rest)
        .split('|')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// # Safety
/// `data` turi rodyti į galiojantį `retro_variable` (su ne-NULL `key` užklausai), kai
/// `cmd == GET_VARIABLE`.
unsafe fn get_variable(data: *mut c_void) -> bool {
    if data.is_null() {
        return false;
    }
    let var = data as *mut retro_variable;
    // SAFETY: žr. funkcijos SAFETY komentarą.
    let key_ptr = unsafe { (*var).key };
    if key_ptr.is_null() {
        return false;
    }
    // SAFETY: core garantuoja galiojančią nul-terminuotą eilutę.
    let key = unsafe { CStr::from_ptr(key_ptr) }
        .to_string_lossy()
        .into_owned();

    with_context(|ctx| {
        let value_ptr = ctx
            .core_options
            .get(&key)
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());
        // SAFETY: žr. funkcijos SAFETY komentarą.
        unsafe { (*var).value = value_ptr };
    });

    true
}

/// # Safety
/// `data` turi rodyti į galiojantį `retro_message`, kai `cmd == SET_MESSAGE`.
unsafe fn set_message(data: *mut c_void) -> bool {
    if data.is_null() {
        return false;
    }
    let msg = data as *const retro_message;
    // SAFETY: žr. funkcijos SAFETY komentarą.
    let text_ptr = unsafe { (*msg).msg };
    if !text_ptr.is_null() {
        // SAFETY: core garantuoja galiojančią nul-terminuotą eilutę.
        let text = unsafe { CStr::from_ptr(text_ptr) }.to_string_lossy();
        // TODO(P9.x): persiųsti į UI kaip toast per Tauri event — reikia AppHandle,
        // kurio emuliavimo gijos EmuContext (P1.4) šiuo metu neturi.
        tracing::info!(target: "core", "{text}");
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::callbacks::{install_context, take_context, with_context, EmuContext};
    use crate::core::ffi::RETRO_PIXEL_FORMAT_RGB565;
    use crate::core::loader::{CoreHandle, RetroCallbacks};
    use std::path::PathBuf;

    fn with_fresh_context<R>(f: impl FnOnce() -> R) -> R {
        install_context(EmuContext::default());
        let result = f();
        take_context();
        result
    }

    #[test]
    fn unknown_command_returns_false_without_panic() {
        with_fresh_context(|| {
            let handled = unsafe { handle(0xFFFF, std::ptr::null_mut()) };
            assert!(!handled);
        });
    }

    #[test]
    fn set_pixel_format_accepts_valid_and_rejects_invalid() {
        with_fresh_context(|| {
            let mut valid = RETRO_PIXEL_FORMAT_RGB565 as i32;
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
                    &mut valid as *mut i32 as *mut c_void,
                )
            };
            assert!(ok);
            with_context(|ctx| assert_eq!(ctx.pixel_format, RETRO_PIXEL_FORMAT_RGB565)).unwrap();

            let mut invalid = 99i32;
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
                    &mut invalid as *mut i32 as *mut c_void,
                )
            };
            assert!(!ok);
        });
    }

    #[test]
    fn get_system_and_save_directory_return_configured_paths() {
        with_fresh_context(|| {
            with_context(|ctx| {
                ctx.system_dir = Some(CString::new("/tmp/nullbyte/system").unwrap());
                ctx.save_dir = Some(CString::new("/tmp/nullbyte/saves").unwrap());
            })
            .unwrap();

            let mut sys_ptr: *const c_char = std::ptr::null();
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY,
                    &mut sys_ptr as *mut *const c_char as *mut c_void,
                )
            };
            assert!(ok);
            let sys = unsafe { CStr::from_ptr(sys_ptr) }.to_str().unwrap();
            assert_eq!(sys, "/tmp/nullbyte/system");

            let mut save_ptr: *const c_char = std::ptr::null();
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY,
                    &mut save_ptr as *mut *const c_char as *mut c_void,
                )
            };
            assert!(ok);
            let save = unsafe { CStr::from_ptr(save_ptr) }.to_str().unwrap();
            assert_eq!(save, "/tmp/nullbyte/saves");
        });
    }

    #[test]
    fn get_can_dupe_writes_true() {
        with_fresh_context(|| {
            let mut value = false;
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_CAN_DUPE,
                    &mut value as *mut bool as *mut c_void,
                )
            };
            assert!(ok);
            assert!(value);
        });
    }

    #[test]
    fn set_variables_then_get_variable_round_trips_default() {
        with_fresh_context(|| {
            let key = CString::new("nullbyte_test_opt").unwrap();
            let value = CString::new("Test option; foo|bar|baz").unwrap();
            let entries = [
                retro_variable {
                    key: key.as_ptr(),
                    value: value.as_ptr(),
                },
                retro_variable {
                    key: std::ptr::null(),
                    value: std::ptr::null(),
                },
            ];

            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_SET_VARIABLES,
                    entries.as_ptr() as *mut c_void,
                )
            };
            assert!(ok);

            let mut query = retro_variable {
                key: key.as_ptr(),
                value: std::ptr::null(),
            };
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_VARIABLE,
                    &mut query as *mut retro_variable as *mut c_void,
                )
            };
            assert!(ok);
            assert!(!query.value.is_null());
            let got = unsafe { CStr::from_ptr(query.value) }.to_str().unwrap();
            assert_eq!(got, "foo");
        });
    }

    #[test]
    fn get_variable_unknown_key_writes_null() {
        with_fresh_context(|| {
            let key = CString::new("unknown_key").unwrap();
            let mut query = retro_variable {
                key: key.as_ptr(),
                value: std::ptr::null(),
            };
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_VARIABLE,
                    &mut query as *mut retro_variable as *mut c_void,
                )
            };
            assert!(ok);
            assert!(query.value.is_null());
        });
    }

    #[test]
    fn shutdown_sets_flag_on_context() {
        with_fresh_context(|| {
            let ok = unsafe { handle(RETRO_ENVIRONMENT_SHUTDOWN, std::ptr::null_mut()) };
            assert!(ok);
            with_context(|ctx| assert!(ctx.shutdown_requested)).unwrap();
        });
    }

    #[test]
    fn get_log_interface_installs_callback() {
        with_fresh_context(|| {
            let mut cb = retro_log_callback {
                log: unsafe {
                    std::mem::transmute::<
                        unsafe extern "C" fn(i32, *const c_char),
                        retro_log_printf_t,
                    >(core_log_printf)
                },
            };
            let ok = unsafe {
                handle(
                    RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
                    &mut cb as *mut retro_log_callback as *mut c_void,
                )
            };
            assert!(ok);
            with_context(|ctx| assert!(ctx.log_callback.is_some())).unwrap();

            let msg = CString::new("testinis log pranešimas").unwrap();
            unsafe { (cb.log)(0, msg.as_ptr()) };
        });
    }

    /// Fixture core'as (žr. `loader.rs`/`info.rs` testus) — gitignored, lokalus.
    fn test_core_path() -> Option<PathBuf> {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/genesis_plus_gx_libretro.dylib");
        path.exists().then_some(path)
    }

    #[test]
    fn real_core_initializes_without_panic_via_environment_cb() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();

        let Some(path) = test_core_path() else {
            eprintln!("praleista: src-tauri/cores/genesis_plus_gx_libretro.dylib nerastas");
            return;
        };

        with_fresh_context(|| {
            let handle_core = CoreHandle::load(&path).expect("core'as turėtų įsikelti");
            unsafe {
                handle_core.init(RetroCallbacks {
                    environment: crate::core::callbacks::environment_cb,
                    video_refresh: crate::core::callbacks::video_refresh_cb,
                    input_poll: crate::core::callbacks::input_poll_cb,
                    input_state: crate::core::callbacks::input_state_cb,
                    audio_sample: crate::core::callbacks::audio_sample_cb,
                    audio_sample_batch: crate::core::callbacks::audio_sample_batch_cb,
                });
            }
            // Sėkmingas grįžimas be panic'o/segfault'o IR yra acceptance kriterijus
            // („core'as inicializuojasi be klaidų log'e").
        });
    }
}
