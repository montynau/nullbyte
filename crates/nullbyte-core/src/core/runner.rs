//! Emuliavimo gija: dedikuota gija su komandų kanalu, `retro_run()` loop, kadrų pacing
//! (CLAUDE.md §3.2, §8.2, §8.5–§8.6, P1.7/P3.4). Nuo P2.4 kiekvieno kadro pabaigoje
//! konvertuoja `EmuContext.video_frame` į RGBA8 ([`pixel_format`]) ir publikuoja per
//! [`frame_buffer::FrameProducer`] — UI/render gija skaito per grąžintą `FrameConsumer`.
//! Nuo P3.4 taip pat perleidžia `EmuContext.audio_buffer` per [`AudioResampler`] į
//! [`audio_ring::AudioProducer`] IR naudoja jo occupancy kaip pagrindinį kadrų pacing
//! mechanizmą (audio-driven sync, CLAUDE.md §8.5) — garso plokštė tampa laikrodžiu.
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

use crate::audio::resampler::AudioResampler;
use crate::audio::ring::{self as audio_ring, AudioConsumer, AudioProducer};
use crate::core::callbacks::{self, EmuContext};
use crate::core::ffi::{
    RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
};
use crate::core::loader::{CoreHandle, LoadedGameInfo, RetroCallbacks};
use crate::error::CoreError;
use crate::ipc::{EmuStatus, StatusSender};
use crate::video::frame_buffer::{self, FrameConsumer, FrameProducer};
use crate::video::pixel_format::{self, PixelFormat};

/// Kai audio ring buferis pasiekia šią occupancy dalį — sustabdome kadrų generavimą ir
/// laukiame, kol consumer'is (audio aparatūra) jį nusausins (CLAUDE.md §8.5 audio-driven
/// sync). SĄMONINGAI arti tikslinio ~50% (P3.4 dinaminio rate control tikslas), NE toli virš
/// jo — kiekvienas `retro_run()` kadras įrašo apytiksliai vieną „chunk'ą" (P3.3
/// `AudioResampler` per vieną `process()` kvietimą sugeneruoja ~1 kadro audio, kuris gali
/// siekti ~8% viso ring buferio talpos). Su TOLIMA riba (pvz. 0.9) emuliavimo gija bėga
/// NEATSKĖTA (jokio delsimo tarp kadrų) tol, kol occupancy pasieks ribą, po to STAIGA
/// „prasiveržia" per ją ir overrun'ina — patikrinta empiriškai P3.4 verifikacijos metu
/// (occupancy pasiekė 0.91, overrun augo pastoviai). Artima riba priverčia throttle'ą
/// suveikti KIEKVIENĄ kadrą, kai occupancy artėja prie tikslo — tai IR YRA audio-driven
/// pacing (kadrų sparta pritaikoma prie consumer'io nusausinimo greičio), ne šalutinis efektas.
const BUFFER_HIGH_WATERMARK: f64 = 0.6;
/// Kiek laukti, kol vėl patikrinti, ar audio ring buferyje atsirado vietos (throttled
/// pacing) — pakankamai trumpai, kad nebūtų girdimo delsimo, bet ne busy-spin.
const THROTTLE_POLL_INTERVAL: Duration = Duration::from_millis(1);

/// Vieno porto įvestis (`RETRO_DEVICE_JOYPAD` bitmask). Kol P4.x mapping'as neparašytas,
/// tai minimali reprezentacija tiesiogiai atitinkanti `EmuContext.input_state`.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputState {
    pub port: u32,
    pub buttons: u16,
}

/// Komandos, siunčiamos į emuliavimo giją per kanalą. Nuo P4.0.3 taip pat kerta
/// `nullbyte-app` → `nullbyte-emu` IPC ribą (NDJSON per stdin, žr. `crate::ipc`) — TIK
/// žaidimo valdymo komandos; protokolo versijos handshake yra ATSKIRAS, bendras
/// `crate::ipc::IpcHello` tipas (siunčiamas kaip pati pirma eilutė abiem kryptimis), ne šio
/// enum'o variantas — nemaišo protokolo lygmens su žaidimo valdymo lygmeniu.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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
    /// Fast-forward (P3.4): `true` — bėga CPU pilnu greičiu, be audio-driven pacing'o, ir
    /// meta (nepublikuoja) audio sample'us, kaip nurodyta MVP.md P3.4 „Ką daryti".
    SetFastForward(bool),
}

/// Rankena į veikiančią emuliavimo giją. `Drop` siunčia `Stop` ir laukia, kol gija baigs
/// darbą — švarus sustabdymas garantuotas net jei caller'is pamiršta jį padaryti pats.
pub struct EmuThread {
    sender: Sender<EmuCommand>,
    handle: Option<JoinHandle<()>>,
}

impl EmuThread {
    /// Paleidžia naują dedikuotą emuliavimo giją. Grąžina [`FrameConsumer`] (UI/render gija
    /// per jį gauna kiekvieną naują nupieštą kadrą, P2.4) ir [`AudioConsumer`] (audio
    /// callback per jį gauna resample'intus sample'us, P3.4).
    ///
    /// `device_sample_rate`/`device_channels` — REALAUS garso išvesties įrenginio
    /// konfigūracija (žr. `audio::output::default_config()`) — reikalinga iš anksto, kad
    /// ring buferis būtų teisingo dydžio ir kad `handle_load` žinotų, į kokį rate
    /// resample'inti kiekvieno naujo core'o garsą.
    ///
    /// `status_sender` — `None` (dauguma testų, kuriems IPC nerūpi) reiškia „nesiųsk jokių
    /// `EmuStatus` pranešimų"; `Some(...)` (realus `nullbyte-emu` paleidimas) — gija siųs
    /// `Loaded`/`Error`/`Stats`/`Stopped` per jį (žr. `crate::ipc` modulio doc dėl
    /// backpressure). `EmuThread` PATS nekuria `StatusWriter`/stdout ryšio — tik naudoja
    /// jau paruoštą rankeną, kad `core::runner` liktų nepriklausomas nuo proceso/IPC detalių.
    pub fn spawn(
        device_sample_rate: u32,
        device_channels: u16,
        status_sender: Option<StatusSender>,
    ) -> (Self, FrameConsumer, AudioConsumer) {
        let (sender, receiver) = mpsc::channel();
        let (video_producer, video_consumer) = frame_buffer::new();
        let ring_capacity = audio_ring::recommended_capacity(
            device_sample_rate,
            device_channels,
            crate::audio::output::TARGET_LATENCY_MS,
        );
        let (audio_producer, audio_consumer) = audio_ring::new(ring_capacity);

        let handle = std::thread::Builder::new()
            .name("nullbyte-emu".to_string())
            .spawn(move || {
                run_loop(
                    receiver,
                    video_producer,
                    audio_producer,
                    device_sample_rate,
                    device_channels,
                    status_sender,
                )
            })
            .expect("nepavyko sukurti emuliavimo gijos");

        (
            Self {
                sender,
                handle: Some(handle),
            },
            video_consumer,
            audio_consumer,
        )
    }

    /// Klonuota vidinio komandų kanalo siuntėja — naudoja `nullbyte-emu`'s stdin skaitymo
    /// gija (`ipc::run_command_reader`), kuri persiunčia `EmuCommand`'us iš tėvo be
    /// poreikio laikyti `&'static EmuThread` nuorodą kitoje gijoje (`Sender` yra `Clone` +
    /// `Send` + `'static` pats savaime).
    pub fn command_sender(&self) -> Sender<EmuCommand> {
        self.sender.clone()
    }

    /// Siunčia komandą į emuliavimo giją. Klaida reiškia, kad gija jau baigė darbą.
    pub fn send(&self, cmd: EmuCommand) -> Result<(), CoreError> {
        self.sender
            .send(cmd)
            .map_err(|_| CoreError::Other("emuliavimo gija nebeveikia".to_string()))
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
    /// P3.4: `true` kai fast-forward įjungtas — jokio audio-driven throttle'o, audio
    /// sample'ai meta (nepublikuojami).
    fast_forward: bool,
    /// Kiekvieno naujo core'o garso resampler'is — perkuriamas `handle_load`, nes core rate
    /// (`info.sample_rate`) gali skirtis tarp ROM'ų. `None`, kol nieko neįkelta arba
    /// resampler'io kūrimas nepavyko (žr. `handle_load`).
    resampler: Option<AudioResampler>,
    /// Realaus garso įrenginio konfigūracija — konstanta visai gijos gyvavimo trukmei
    /// (nustatoma `EmuThread::spawn()` metu).
    device_sample_rate: u32,
    device_channels: u16,
}

impl RunnerState {
    fn new(device_sample_rate: u32, device_channels: u16) -> Self {
        Self {
            core: None,
            game_info: None,
            running: false,
            fast_forward: false,
            resampler: None,
            device_sample_rate,
            device_channels,
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
fn handle_load(
    state: &mut RunnerState,
    core_path: &std::path::Path,
    rom_path: &std::path::Path,
    status_sender: Option<&StatusSender>,
) {
    cleanup(state);

    let result = (|| -> Result<(CoreHandle, LoadedGameInfo), CoreError> {
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

            // P3.4: kiekvienas core'as/ROM'as gali turėti skirtingą sample rate (žr.
            // CLAUDE.md §8.5 SNES/Genesis/GBA pavyzdžius) — resampler'is kuriamas iš naujo.
            state.resampler = match AudioResampler::new(
                info.sample_rate,
                f64::from(state.device_sample_rate),
                state.device_channels as usize,
            ) {
                Ok(resampler) => Some(resampler),
                Err(error) => {
                    tracing::error!(
                        %error,
                        core_sample_rate = info.sample_rate,
                        device_sample_rate = state.device_sample_rate,
                        "nepavyko sukurti audio resampler'io — garso nebus šiai sesijai"
                    );
                    None
                }
            };

            if let Some(sender) = status_sender {
                sender.send_important(EmuStatus::Loaded(info.clone()));
            }
            state.core = Some(core);
            state.game_info = Some(info);
        }
        Err(error) => {
            tracing::error!(%error, core = %core_path.display(), rom = %rom_path.display(), "nepavyko įkelti core'o/ROM'o");
            if let Some(sender) = status_sender {
                sender.send_important(EmuStatus::Error(error));
            }
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
    state.resampler = None;
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
///
/// `aspect_ratio` — core'o `av_info.geometry.aspect_ratio` (P2.5); Renderer'is naudoja
/// `width / height`, jei čia `<= 0.0`.
fn publish_video_frame(producer: &mut FrameProducer, aspect_ratio: f32) {
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
        producer.write_frame(width, height, aspect_ratio, |dst| {
            pixel_format::convert_to_rgba8_into(src, format, width, height, pitch, dst);
        });
    });
}

/// Ištraukia šio kadro žalius (core sample rate'u) audio sample'us iš `EmuContext`,
/// perleidžia per resampler'į ir publikuoja į `producer` (P3.4). Fast-forward metu (arba
/// jei resampler'io nėra) sample'ai tik ištraukiami (drain'inami) ir IŠMETAMI — CLAUDE.md
/// §8.6/MVP.md P3.4 „Ką daryti": „Fast-forward režimas: išjunk rate control, mesk audio
/// sample'us". Po sėkmingo publikavimo koreguoja resampling ratio pagal ring occupancy.
fn process_audio_frame(
    state: &mut RunnerState,
    producer: &mut AudioProducer,
    scratch: &mut Vec<i16>,
) {
    callbacks::with_context(|ctx| {
        scratch.clear();
        scratch.extend_from_slice(&ctx.audio_buffer);
        ctx.audio_buffer.clear();
    });

    if state.fast_forward || scratch.is_empty() {
        return;
    }

    let Some(resampler) = &mut state.resampler else {
        return;
    };

    let push_result = resampler
        .process(scratch)
        .map(|resampled| producer.push_samples(resampled));
    if let Err(error) = push_result {
        tracing::error!(%error, "audio resampling nepavyko");
        return;
    }

    // Dinaminis rate control (CLAUDE.md §8.6): centruoja occupancy apie ~50%, kad ring
    // buferis niekada nedreiftų į 0% (traškesiai) ar 100% (overrun).
    let occupancy = producer.occupancy();
    let deviation = (occupancy - 0.5) * 2.0;
    if let Err(error) = resampler.adjust_ratio(deviation) {
        tracing::debug!(%error, occupancy, "nepavyko koreguoti resampling ratio");
    }
}

/// Emuliavimo gijos pagrindinis loop'as. `thread_local` `EmuContext` (CLAUDE.md §3.3)
/// įdiegiama vieną kartą šios gijos pradžioje ir gyvena visą jos gyvavimo trukmę.
fn run_loop(
    receiver: Receiver<EmuCommand>,
    mut video_producer: FrameProducer,
    mut audio_producer: AudioProducer,
    device_sample_rate: u32,
    device_channels: u16,
    status_sender: Option<StatusSender>,
) {
    callbacks::install_context(EmuContext::default());

    let mut state = RunnerState::new(device_sample_rate, device_channels);
    let mut frame_count: u64 = 0;
    let mut last_video_frames: u64 = 0;
    let mut last_audio_samples: u64 = 0;
    let mut last_stats_log = Instant::now();
    let mut next_frame_deadline = Instant::now();
    let mut audio_scratch: Vec<i16> = Vec::new();

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
                handle_load(&mut state, &core, &rom, status_sender.as_ref());
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
            Ok(EmuCommand::SetFastForward(enabled)) => {
                state.fast_forward = enabled;
                tracing::debug!(enabled, "fast-forward režimas pakeistas");
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
            // Kopijuojame reikšmes ANKSTI — `info` skolinys iš `state.game_info` turi
            // pasibaigti prieš `process_audio_frame(&mut state, ...)` žemiau.
            let aspect_ratio = info.aspect_ratio;
            let fps = info.fps;

            // Audio-driven pacing (P3.4, CLAUDE.md §8.5): kai NE fast-forward ir yra veikiantis
            // audio pipeline'as, nebėginėjame naujo kadro, kol ring buferis beveik pilnas —
            // laukiame, kol consumer'is (real-time audio aparatūra) jį nusausins. Tai IR yra
            // laikrodis — jokio fiksuoto sleep'o nebereikia šiuo atveju.
            if !state.fast_forward
                && state.resampler.is_some()
                && audio_producer.occupancy() >= BUFFER_HIGH_WATERMARK
            {
                std::thread::sleep(THROTTLE_POLL_INTERVAL);
                continue;
            }

            // SAFETY: emuliavimo gija, core įkeltas su sėkmingu load_game() (state.game_info
            // yra Some tik po to).
            unsafe { core.run() };
            frame_count += 1;
            publish_video_frame(&mut video_producer, aspect_ratio);
            process_audio_frame(&mut state, &mut audio_producer, &mut audio_scratch);

            // `send_stats` throttle'ina viduje (žr. crate::ipc modulio doc) — saugu kviesti
            // kas kadrą, realiai išeis ~2-4 Hz.
            if let Some(sender) = &status_sender {
                sender.send_stats(audio_producer.occupancy());
            }

            if state.fast_forward {
                // Fast-forward: jokio laukimo — bėgam CPU pilnu greičiu (P3.4 „Ką daryti").
            } else if state.resampler.is_none() {
                // Atsarginis fiksuotas P1.7 pacing'as — tik jei audio pipeline'as neveikia
                // (pvz. resampler'io kūrimas nepavyko). Apsauga nuo nekontroliuojamo CPU
                // spin'o, jei dėl kokios nors priežasties audio-driven pacing negalimas.
                let fps = if fps > 0.0 { fps } else { 60.0 };
                next_frame_deadline += Duration::from_secs_f64(1.0 / fps);
                spin_sleep::sleep_until(next_frame_deadline);
            }
            // else: audio-driven pacing jau atliktas aukščiau (occupancy patikra prieš
            // bėgant šį kadrą) — jokio papildomo laukimo nereikia.

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
                    audio_occupancy = audio_producer.occupancy(),
                    audio_overrun_count = audio_producer.overrun_count(),
                    "emuliavimo statistika"
                );

                frame_count = 0;
                last_video_frames = video_frames;
                last_audio_samples = audio_samples;
                last_stats_log = Instant::now();
            }
        }
    }

    // Abu `break 'outer` keliai (Stop, stdin/kanalo Disconnected) jau kvietė cleanup() —
    // Stopped siunčiamas VIENĄ kartą čia, DRY vietoj dubliavimo abiejose šakose.
    if let Some(sender) = &status_sender {
        sender.send_important(EmuStatus::Stopped);
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
        let (emu, _video, _audio) = EmuThread::spawn(48000, 2, None);
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
        let (emu, _video, _audio) = EmuThread::spawn(48000, 2, None);
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
        let (emu, _video, _audio) = EmuThread::spawn(48000, 2, None);
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

        let (emu, _video, _audio) = EmuThread::spawn(48000, 2, None);
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
