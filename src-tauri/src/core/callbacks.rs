//! libretro callback'ai ir `thread_local` emuliavimo kontekstas (CLAUDE.md §3.3).
//!
//! libretro callback'ai yra paprastos C funkcijų rodyklės be `user_data` parametro — nėra kur
//! perduoti `&mut self`. Sprendimas: `thread_local! { static CTX: RefCell<Option<EmuContext>> }`.
//! Visi `retro_*` kvietimai vyksta iš vienos (emuliavimo) gijos, tad `thread_local` yra saugus
//! ir be sinchronizacijos kaštų (CLAUDE.md §3.2 taisyklė #1, ADR-007).
//!
//! **Pastaba dėl placeholder laukų:** `video_frame` ir `audio_samples_written` čia yra
//! minimalūs — realų lock-free triple buffer'į video kadrams prijungs P2.2, o realų rtrb ring
//! buffer'į garsui — P3.2. Kol jų nėra, `video_frame` laiko tik PASKUTINĮ kadrą (fiksuoto
//! dydžio, ne augantis), o garsui laikomas tik skaitliukas — taip išvengiama neribotai
//! augančio buferio, kol niekas jo nenuleidžia (drain).

// Naudos runner.rs (P1.7) ir environment.rs (P1.5) — kol jie neparašyti, callback'ai ir
// EmuContext laukai pilnai išnaudojami tik testuose.
#![allow(dead_code)]

use std::ffi::{c_void, CString};

use super::ffi::{RETRO_DEVICE_ID_JOYPAD_MASK, RETRO_DEVICE_JOYPAD, RETRO_PIXEL_FORMAT_0RGB1555};

/// Paskutinio gauto vaizdo kadro metaduomenys ir baitai.
#[derive(Default)]
pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    pub pitch: usize,
    pub data: Vec<u8>,
}

/// Emuliavimo gijos būvis, pasiekiamas iš visų libretro callback'ų per `thread_local`.
pub struct EmuContext {
    /// Viena iš `RETRO_PIXEL_FORMAT_*` (ffi.rs) — numatytoji `0RGB1555`, kol core'as
    /// nepakeičia per `RETRO_ENVIRONMENT_SET_PIXEL_FORMAT` (P1.5).
    pub pixel_format: u32,
    pub video_frame: VideoFrame,
    pub video_frame_count: u64,
    /// Placeholder P3.2 ring buffer producer'iui — kol kas tik skaitliukas.
    pub audio_samples_written: u64,
    /// Mygtukų bitmask kiekvienam iš 4 portų (bitas N = `RETRO_DEVICE_ID_JOYPAD_N`).
    pub input_state: [u16; 4],
    /// Užpildoma `RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY` apdorojime (P1.5).
    pub system_dir: Option<CString>,
    /// Užpildoma `RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY` apdorojime (P1.5).
    pub save_dir: Option<CString>,
    /// Užpildoma `RETRO_ENVIRONMENT_GET_LOG_INTERFACE` apdorojime (P1.5).
    pub log_callback: Option<super::ffi::retro_log_printf_t>,
}

impl Default for EmuContext {
    fn default() -> Self {
        Self {
            pixel_format: RETRO_PIXEL_FORMAT_0RGB1555,
            video_frame: VideoFrame::default(),
            video_frame_count: 0,
            audio_samples_written: 0,
            input_state: [0; 4],
            system_dir: None,
            save_dir: None,
            log_callback: None,
        }
    }
}

thread_local! {
    static CTX: std::cell::RefCell<Option<EmuContext>> = const { std::cell::RefCell::new(None) };
}

/// Įdiegia naują `EmuContext` einamai gijai (turi būti kviečiama emuliavimo gijoje, prieš
/// `retro_init()`). Naudos `runner.rs` (P1.7).
#[allow(dead_code)]
pub fn install_context(ctx: EmuContext) {
    CTX.with_borrow_mut(|slot| *slot = Some(ctx));
}

/// Vykdo `f` su prieiga prie einamos gijos `EmuContext`, jei jis įdiegtas. Naudos
/// `runner.rs` (P1.7) kadrų/garso nuskaitymui ir `environment.rs` (P1.5) komandų apdorojimui.
#[allow(dead_code)]
pub fn with_context<R>(f: impl FnOnce(&mut EmuContext) -> R) -> Option<R> {
    CTX.with_borrow_mut(|slot| slot.as_mut().map(f))
}

/// Pašalina einamos gijos `EmuContext` (kviečiama po `retro_deinit()`). Naudos `runner.rs`.
#[allow(dead_code)]
pub fn take_context() -> Option<EmuContext> {
    CTX.with_borrow_mut(|slot| slot.take())
}

/// `retro_environment_t` — dar neapdoroja jokios komandos (tai P1.5 `environment.rs` darbas).
/// Kol P1.5 neparašytas, elgiasi kaip su nežinoma komanda: logina ir grąžina `false`.
///
/// # Safety
/// Kviečia core'as bet kuriuo metu tarp `retro_set_environment()` ir `retro_deinit()`.
/// `data` reikšmė ir tipas priklauso nuo `cmd` — šis stub'as jos neliečia, tad jokių
/// papildomų invariantų nereikalaujama.
pub unsafe extern "C" fn environment_cb(cmd: u32, _data: *mut c_void) -> bool {
    tracing::debug!(cmd, "retro_environment komanda dar neapdorota (P1.5)");
    false
}

/// `retro_video_refresh_t`.
///
/// # Safety
/// Core garantuoja, kad jei `data` ne NULL, jis rodo į bent `height * pitch` baitų buferį,
/// galiojantį šio iškvietimo metu (libretro kontraktas). `data == NULL` reiškia „pakartok
/// paskutinį kadrą" (dupe frame) — tokiu atveju nieko neliečiame ir grįžtame.
pub unsafe extern "C" fn video_refresh_cb(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    if data.is_null() {
        return;
    }

    let len = height as usize * pitch;
    // SAFETY: žr. funkcijos SAFETY komentarą — core garantuoja `len` baitų buferį.
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, len) };

    CTX.with_borrow_mut(|slot| {
        if let Some(ctx) = slot.as_mut() {
            ctx.video_frame.width = width;
            ctx.video_frame.height = height;
            ctx.video_frame.pitch = pitch;
            ctx.video_frame.data.clear();
            ctx.video_frame.data.extend_from_slice(bytes);
            ctx.video_frame_count += 1;
        }
    });
}

/// `retro_audio_sample_t` — pavienis (ne batch) audio sample'as. Retai naudojamas realių
/// core'ų (dauguma naudoja `audio_sample_batch_cb`), bet libretro reikalauja abiejų.
///
/// # Safety
/// Neliečia jokių žaliavinių (raw) rodyklių — `left`/`right` yra paprastos reikšmės.
/// Funkcija pažymėta `unsafe extern "C"`, nes ją tiesiogiai kviečia core per FFI ribą.
pub unsafe extern "C" fn audio_sample_cb(_left: i16, _right: i16) {
    CTX.with_borrow_mut(|slot| {
        if let Some(ctx) = slot.as_mut() {
            ctx.audio_samples_written += 1;
        }
    });
}

/// `retro_audio_sample_batch_t`.
///
/// # Safety
/// Core garantuoja, kad jei `data` ne NULL, jis rodo į bent `frames * 2` `i16` reikšmių
/// (L/R kanalai, interleaved), galiojančių šio iškvietimo metu.
pub unsafe extern "C" fn audio_sample_batch_cb(data: *const i16, frames: usize) -> usize {
    if data.is_null() || frames == 0 {
        return 0;
    }

    CTX.with_borrow_mut(|slot| {
        if let Some(ctx) = slot.as_mut() {
            ctx.audio_samples_written += frames as u64;
        }
    });

    frames
}

/// `retro_input_poll_t`.
///
/// # Safety
/// Neima jokių argumentų ir neliečia jokių žaliavinių rodyklių. `EmuContext.input_state`
/// atnaujinamas iš išorės (runner.rs komandų kanalu, P4.3), prieš kiekvieną `retro_run()` —
/// šiai funkcijai papildomai nieko daryti nereikia.
pub unsafe extern "C" fn input_poll_cb() {}

/// `retro_input_state_t`.
///
/// # Safety
/// Neliečia jokių žaliavinių rodyklių. Palaiko tik `RETRO_DEVICE_JOYPAD`; kitiems
/// įrenginiams ir portams > 4 grąžina 0 (libretro kontraktas: nepalaikomos reikšmės → 0).
pub unsafe extern "C" fn input_state_cb(port: u32, device: u32, _index: u32, id: u32) -> i16 {
    if device != RETRO_DEVICE_JOYPAD || port >= 4 {
        return 0;
    }

    CTX.with_borrow(|slot| {
        let Some(ctx) = slot.as_ref() else {
            return 0;
        };
        let buttons = ctx.input_state[port as usize];

        if id == RETRO_DEVICE_ID_JOYPAD_MASK {
            buttons as i16
        } else if id < 16 {
            ((buttons >> id) & 1) as i16
        } else {
            0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::super::ffi::{RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B};
    use super::*;

    fn with_fresh_context<R>(f: impl FnOnce() -> R) -> R {
        install_context(EmuContext::default());
        let result = f();
        take_context();
        result
    }

    #[test]
    fn null_video_frame_does_not_panic_or_update_state() {
        with_fresh_context(|| {
            // SAFETY: testas — data == NULL yra tiksliai tas atvejis, kurį funkcija privalo
            // saugiai apdoroti (dupe frame), nieko neskaitydama iš rodyklės.
            unsafe { video_refresh_cb(std::ptr::null(), 256, 224, 512) };

            let count = with_context(|ctx| ctx.video_frame_count).unwrap();
            assert_eq!(count, 0, "NULL kadras neturėtų padidinti skaitliuko");
        });
    }

    #[test]
    fn video_frame_updates_context_and_stays_bounded() {
        with_fresh_context(|| {
            let frame = vec![0xABu8; 256 * 2 * 224];
            // SAFETY: testas — `frame` gyvena visą iškvietimo metu, dydis atitinka
            // width*2 (RGB565) * height.
            unsafe {
                video_refresh_cb(frame.as_ptr() as *const c_void, 256, 224, 256 * 2);
            }

            with_context(|ctx| {
                assert_eq!(ctx.video_frame_count, 1);
                assert_eq!(ctx.video_frame.width, 256);
                assert_eq!(ctx.video_frame.height, 224);
                assert_eq!(ctx.video_frame.data.len(), frame.len());
            })
            .unwrap();
        });
    }

    #[test]
    fn audio_batch_increments_counter_without_growing_unboundedly() {
        with_fresh_context(|| {
            let samples = [0i16; 32 * 2]; // 32 kadrai, L/R
            for _ in 0..1000 {
                // SAFETY: testas — `samples` gyvena visą iškvietimo metu.
                let consumed =
                    unsafe { audio_sample_batch_cb(samples.as_ptr(), samples.len() / 2) };
                assert_eq!(consumed, 32);
            }

            with_context(|ctx| {
                assert_eq!(ctx.audio_samples_written, 32_000);
            })
            .unwrap();
        });
    }

    #[test]
    fn input_state_reads_bitmask_and_individual_buttons() {
        with_fresh_context(|| {
            with_context(|ctx| {
                ctx.input_state[0] = 1 << RETRO_DEVICE_ID_JOYPAD_A;
            })
            .unwrap();

            // SAFETY: testas — jokių žaliavinių rodyklių, tik paprastos reikšmės.
            let a_pressed =
                unsafe { input_state_cb(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_A) };
            assert_eq!(a_pressed, 1);

            let b_pressed =
                unsafe { input_state_cb(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_B) };
            assert_eq!(b_pressed, 0);

            let mask =
                unsafe { input_state_cb(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_MASK) };
            assert_eq!(mask, 1 << RETRO_DEVICE_ID_JOYPAD_A);
        });
    }
}
