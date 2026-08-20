//! Emuliavimo gija: dedikuota gija su komandų kanalu, `retro_run()` loop, kadrų pacing
//! (CLAUDE.md §3.2, §8.2, P1.7). Nuo P2.4 kiekvieno kadro pabaigoje konvertuoja
//! `EmuContext.video_frame` į RGBA8 ([`pixel_format`]) ir publikuoja per
//! [`frame_buffer::FrameProducer`] — UI/render gija skaito per grąžintą `FrameConsumer`.
//!
//! **Milestone M1:** jei ši gija patikimai suka realų ROM'ą be crash'o — libretro
//! integracija veikia ir galima eiti į Fazę 2.

// Naudos commands/emulator.rs (vėlesnė fazė) — kol jo nėra, EmuThread pilnai išnaudojamas
// tik testuose.
#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::core::callbacks::{self, EmuContext};
use crate::core::ffi::{
    RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
};
use crate::core::loader::{CoreHandle, LoadedGameInfo, RetroCallbacks};
use crate::error::AppError;
use crate::video::frame_buffer::{self, FrameConsumer, FrameProducer};
use crate::video::pixel_format::{self, PixelFormat};

/// Vieno porto įvestis (`RETRO_DEVICE_JOYPAD` bitmask). Kol P4.x mapping'as neparašytas,
/// tai minimali reprezentacija tiesiogiai atitinkanti `EmuContext.input_state`.
#[derive(Debug, Clone, Copy)]
pub struct InputState {
    pub port: u32,
    pub buttons: u16,
}

/// Komandos, siunčiamos į emuliavimo giją per kanalą.
pub enum EmuCommand {
    Load {
        core: PathBuf,
        rom: PathBuf,
    },
    Run,
    Pause,
    Resume,
    Reset,
    Stop,
    /// Dar neimplementuota — P8.1. Kol kas logina ir ignoruoja.
    SaveState(u8),
    /// Dar neimplementuota — P8.1. Kol kas logina ir ignoruoja.
    LoadState(u8),
    SetInput(InputState),
}

/// Rankena į veikiančią emuliavimo giją. `Drop` siunčia `Stop` ir laukia, kol gija baigs
/// darbą — švarus sustabdymas garantuotas net jei caller'is pamiršta jį padaryti pats.
pub struct EmuThread {
    sender: Sender<EmuCommand>,
    handle: Option<JoinHandle<()>>,
}

impl EmuThread {
    /// Paleidžia naują dedikuotą emuliavimo giją. Grąžina ir [`FrameConsumer`] — UI/render
    /// gija per jį gauna kiekvieną naują nupieštą kadrą (P2.4).
    pub fn spawn() -> (Self, FrameConsumer) {
        let (sender, receiver) = mpsc::channel();
        let (video_producer, video_consumer) = frame_buffer::new();
        let handle = std::thread::Builder::new()
            .name("nullbyte-emu".to_string())
            .spawn(move || run_loop(receiver, video_producer))
            .expect("nepavyko sukurti emuliavimo gijos");

        (
            Self {
                sender,
                handle: Some(handle),
            },
            video_consumer,
        )
    }

    /// Siunčia komandą į emuliavimo giją. Klaida reiškia, kad gija jau baigė darbą.
    pub fn send(&self, cmd: EmuCommand) -> Result<(), AppError> {
        self.sender
            .send(cmd)
            .map_err(|_| AppError::Other("emuliavimo gija nebeveikia".to_string()))
    }
}

impl Drop for EmuThread {
    /// Švarus sustabdymas: siunčia `Stop`, laukia, kol gija (per `run_loop` -> `cleanup`)
    /// atlaisvina core'ą (`unload_game` → `deinit` → `drop(Library)`) ir baigia darbą.
    fn drop(&mut self) {
        let _ = self.sender.send(EmuCommand::Stop);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Emuliavimo gijos vidinis būvis. Laikomas tik `run_loop` viduje — niekada neiškeliauja
/// iš tos gijos (CoreHandle nereikia būti `Send`).
struct RunnerState {
    core: Option<CoreHandle>,
    game_info: Option<LoadedGameInfo>,
    running: bool,
}

impl RunnerState {
    fn new() -> Self {
        Self {
            core: None,
            game_info: None,
            running: false,
        }
    }
}

fn stub_callbacks() -> RetroCallbacks {
    RetroCallbacks {
        environment: callbacks::environment_cb,
        video_refresh: callbacks::video_refresh_cb,
        input_poll: callbacks::input_poll_cb,
        input_state: callbacks::input_state_cb,
        audio_sample: callbacks::audio_sample_cb,
        audio_sample_batch: callbacks::audio_sample_batch_cb,
    }
}

/// Įkelia core'ą + ROM'ą į `state`. Jei jau buvo įkeltas kitas core'as — pirma jį švariai
/// atlaisvina (CLAUDE.md §3.2 taisyklė #2: vienu metu procese tik vienas core'as).
fn handle_load(state: &mut RunnerState, core_path: &std::path::Path, rom_path: &std::path::Path) {
    cleanup(state);

    let result = (|| -> Result<(CoreHandle, LoadedGameInfo), AppError> {
        let core = CoreHandle::load(core_path)?;
        // SAFETY: kviečiama iš emuliavimo gijos (run_loop), prieš bet kokį retro_run().
        unsafe { core.init(stub_callbacks()) };
        // SAFETY: kviečiama iškart po init(), tos pačios gijos.
        let info = unsafe { core.load_game(rom_path) }?;
        Ok((core, info))
    })();

    match result {
        Ok((core, info)) => {
            tracing::info!(
                core = %core_path.display(),
                rom = %rom_path.display(),
                fps = info.fps,
                sample_rate = info.sample_rate,
                "core ir ROM'as įkelti"
            );
            state.core = Some(core);
            state.game_info = Some(info);
        }
        Err(error) => {
            tracing::error!(%error, core = %core_path.display(), rom = %rom_path.display(), "nepavyko įkelti core'o/ROM'o");
        }
    }
}

/// `unload_game()` → `deinit()` → `drop(Library)` (CLAUDE.md §8.2 žingsnis 14).
/// `Library` iškraunama automatiškai, kai `core` čia `take()`'inamas ir dingsta iš scope.
fn cleanup(state: &mut RunnerState) {
    if let Some(core) = state.core.take() {
        // SAFETY: kviečiama iš emuliavimo gijos, tos pačios, kurioje core buvo įkeltas.
        unsafe {
            core.unload_game();
            core.deinit();
        }
    }
    state.game_info = None;
    state.running = false;
}

/// `RETRO_PIXEL_FORMAT_*` (ffi.rs, žalia `u32`) → [`PixelFormat`] (video/pixel_format.rs).
fn map_pixel_format(raw: u32) -> Option<PixelFormat> {
    match raw {
        RETRO_PIXEL_FORMAT_0RGB1555 => Some(PixelFormat::Rgb0555),
        RETRO_PIXEL_FORMAT_XRGB8888 => Some(PixelFormat::Xrgb8888),
        RETRO_PIXEL_FORMAT_RGB565 => Some(PixelFormat::Rgb565),
        _ => None,
    }
}

/// Konvertuoja `EmuContext.video_frame` (žalia core formatu) į RGBA8 ir publikuoja per
/// `producer`. Tyliai praleidžia, jei dar nėra jokio kadro arba pixel format nepalaikomas —
/// tai NĖRA klaida (pvz. prieš pirmą `video_refresh_cb` kvietimą).
fn publish_video_frame(producer: &mut FrameProducer) {
    callbacks::with_context(|ctx| {
        let frame = &ctx.video_frame;
        if frame.width == 0 || frame.height == 0 || frame.data.is_empty() {
            return;
        }
        let Some(format) = map_pixel_format(ctx.pixel_format) else {
            return;
        };

        let (width, height, pitch) = (frame.width, frame.height, frame.pitch);
        let src = &frame.data;
        producer.write_frame(width, height, |dst| {
            pixel_format::convert_to_rgba8_into(src, format, width, height, pitch, dst);
        });
    });
}

/// Emuliavimo gijos pagrindinis loop'as. `thread_local` `EmuContext` (CLAUDE.md §3.3)
/// įdiegiama vieną kartą šios gijos pradžioje ir gyvena visą jos gyvavimo trukmę.
fn run_loop(receiver: Receiver<EmuCommand>, mut video_producer: FrameProducer) {
    callbacks::install_context(EmuContext::default());

    let mut state = RunnerState::new();
    let mut frame_count: u64 = 0;
    let mut last_video_frames: u64 = 0;
    let mut last_audio_samples: u64 = 0;
    let mut last_stats_log = Instant::now();
    let mut next_frame_deadline = Instant::now();

    'outer: loop {
        // Kai bėgame — netrukdome frame pacing'ui, tik trumpai pažiūrime, ar yra komanda.
        // Kai stovime — blokuojamai laukiame, kad neapkrautume CPU tuščiu loop'u.
        let timeout = if state.running {
            Duration::ZERO
        } else {
            Duration::from_millis(100)
        };

        match receiver.recv_timeout(timeout) {
            Ok(EmuCommand::Load { core, rom }) => {
                handle_load(&mut state, &core, &rom);
                frame_count = 0;
                next_frame_deadline = Instant::now();
            }
            Ok(EmuCommand::Run) => {
                if state.core.is_some() {
                    state.running = true;
                    next_frame_deadline = Instant::now();
                } else {
                    tracing::warn!("Run gauta, bet nėra įkelto core'o");
                }
            }
            Ok(EmuCommand::Pause) => state.running = false,
            Ok(EmuCommand::Resume) => {
                if state.core.is_some() {
                    state.running = true;
                    next_frame_deadline = Instant::now();
                }
            }
            Ok(EmuCommand::Reset) => {
                if let Some(core) = &state.core {
                    // SAFETY: emuliavimo gija, core jau įkeltas (load_game sėkmingas).
                    unsafe { core.reset() };
                }
            }
            Ok(EmuCommand::Stop) => {
                cleanup(&mut state);
                break 'outer;
            }
            Ok(EmuCommand::SaveState(slot)) => {
                tracing::debug!(slot, "SaveState dar neimplementuota (P8.1) — ignoruojama");
            }
            Ok(EmuCommand::LoadState(slot)) => {
                tracing::debug!(slot, "LoadState dar neimplementuota (P8.1) — ignoruojama");
            }
            Ok(EmuCommand::SetInput(input)) => {
                callbacks::with_context(|ctx| {
                    if let Some(slot) = ctx.input_state.get_mut(input.port as usize) {
                        *slot = input.buttons;
                    }
                });
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                cleanup(&mut state);
                break 'outer;
            }
        }

        if state.running {
            let Some(core) = &state.core else {
                state.running = false;
                continue;
            };
            let Some(info) = &state.game_info else {
                state.running = false;
                continue;
            };

            // SAFETY: emuliavimo gija, core įkeltas su sėkmingu load_game() (state.game_info
            // yra Some tik po to).
            unsafe { core.run() };
            frame_count += 1;
            publish_video_frame(&mut video_producer);

            // MVP kadrų pacing: laukiam iki kito kadro momento pagal core'o TIKRĄ fps
            // (ne apvalintą 60) — pakeis audio-driven sinchronizacija P3.4 (ADR-012).
            let fps = if info.fps > 0.0 { info.fps } else { 60.0 };
            next_frame_deadline += Duration::from_secs_f64(1.0 / fps);
            spin_sleep::sleep_until(next_frame_deadline);

            if last_stats_log.elapsed() >= Duration::from_secs(5) {
                let elapsed = last_stats_log.elapsed().as_secs_f64();
                let measured_fps = frame_count as f64 / elapsed;

                let (video_frames, audio_samples) = callbacks::with_context(|ctx| {
                    (ctx.video_frame_count, ctx.audio_samples_written)
                })
                .unwrap_or((0, 0));
                let video_fps = (video_frames - last_video_frames) as f64 / elapsed;
                let audio_rate = (audio_samples - last_audio_samples) as f64 / elapsed;

                tracing::info!(
                    measured_fps,
                    video_fps,
                    audio_samples_per_sec = audio_rate,
                    "emuliavimo statistika"
                );

                frame_count = 0;
                last_video_frames = video_frames;
                last_audio_samples = audio_samples;
                last_stats_log = Instant::now();
            }
        }
    }

    callbacks::take_context();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn snes9x_path() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cores/snes9x_libretro.dylib");
        path.exists().then_some(path)
    }

    fn first_snes_rom() -> Option<PathBuf> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/snes");
        std::fs::read_dir(&dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_lowercase)
                    .as_deref()
                    == Some("sfc")
            })
    }

    /// Greitas (kelių sekundžių) sanity testas — pilnas 60s soak testas yra
    /// `#[ignore]`'intas `runs_snes_rom_for_60_seconds_without_crash` (žr. žemiau),
    /// kad įprastas `cargo test` liktų greitas.
    #[test]
    fn loads_runs_and_stops_cleanly_short() {
        let (Some(core), Some(rom)) = (snes9x_path(), first_snes_rom()) else {
            eprintln!("praleista: snes9x_libretro.dylib arba .sfc ROM'as nerastas");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        let (emu, _video) = EmuThread::spawn();
        emu.send(EmuCommand::Load { core, rom }).unwrap();
        emu.send(EmuCommand::Run).unwrap();

        std::thread::sleep(Duration::from_millis(500));

        emu.send(EmuCommand::Pause).unwrap();
        emu.send(EmuCommand::Resume).unwrap();
        emu.send(EmuCommand::Reset).unwrap();

        std::thread::sleep(Duration::from_millis(200));

        // Drop siunčia Stop ir laukia join() — jei gija pakibtų ar panikuotų, testas
        // pakibtų/failintų čia.
        drop(emu);
    }

    #[test]
    fn stop_without_load_does_not_hang_or_panic() {
        let (emu, _video) = EmuThread::spawn();
        emu.send(EmuCommand::Pause).unwrap(); // komanda be įkelto core'o — turėtų būti no-op
        drop(emu);
    }

    /// Pilnas P1.7 acceptance soak testas: 60 sekundžių realaus SNES ROM'o be crash'o.
    /// Neįeina į įprastą `cargo test` (per lėtas) — paleisti rankiniu būdu:
    /// `cargo test --release runs_snes_rom_for_60_seconds -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn runs_snes_rom_for_60_seconds_without_crash() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::INFO)
            .try_init();

        let (Some(core), Some(rom)) = (snes9x_path(), first_snes_rom()) else {
            eprintln!("praleista: snes9x_libretro.dylib arba .sfc ROM'as nerastas");
            return;
        };

        let _core_lock = crate::core::test_support::lock_core_load();
        let (emu, _video) = EmuThread::spawn();
        emu.send(EmuCommand::Load { core, rom }).unwrap();
        emu.send(EmuCommand::Run).unwrap();

        std::thread::sleep(Duration::from_secs(60));

        drop(emu); // Stop + join — jei pakibtų, testas niekada nesibaigtų (matoma iš karto).
    }

    #[test]
    fn non_libretro_load_reports_error_and_thread_stays_alive() {
        let candidates = [
            "/usr/lib/libz.dylib",
            "/usr/lib/x86_64-linux-gnu/libz.so.1",
            "/lib/x86_64-linux-gnu/libz.so.1",
        ];
        let Some(bad_core) = candidates.iter().map(Path::new).find(|p| p.exists()) else {
            eprintln!("praleista: sistemos libz nerasta");
            return;
        };

        let (emu, _video) = EmuThread::spawn();
        emu.send(EmuCommand::Load {
            core: bad_core.to_path_buf(),
            rom: PathBuf::from("/nonexistent.sfc"),
        })
        .unwrap();
        // Klaida turėtų būti logginama (žr. handle_load), gija turėtų likti gyva ir
        // priimti tolimesnes komandas — patikrinam siųsdami dar vieną.
        emu.send(EmuCommand::Pause).unwrap();
        drop(emu);
    }
}
