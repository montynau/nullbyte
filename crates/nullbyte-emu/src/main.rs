//! `nullbyte-emu` — vaiko procesas: winit langas, wgpu vaizdas, cpal garsas, emuliavimo gija
//! (CLAUDE.md §3.4, ADR-016, MVP.md P4.0.2).
//!
//! P4.0.2 etape core/ROM kelias HARDKODINTAS testams (`test_core_and_rom` žemiau) — realų IPC
//! `Load` srautą per `nullbyte-app` atneš P4.0.3. Tai tas pats verifikacijos hook'o principas,
//! kokį naudojo P1.7/P2.4/P3.4 fazių testai prieš atsirandant komandų sluoksniui.
//!
//! **`stdout` PRIKLAUSO P4.0.3 IPC protokolui** — šis procesas niekada į jį nerašo (CLAUDE.md
//! §10). `init_tracing()` nukreipia visus logus į `stderr`.

mod ipc;

use std::path::PathBuf;
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use nullbyte_core::audio::output::{default_config, AudioOutput};
use nullbyte_core::audio::ring::AudioConsumer;
use nullbyte_core::core::runner::{EmuCommand, EmuThread};
use nullbyte_core::error::CoreError;
use nullbyte_core::input::gamepad::{GamepadEvent, GamepadThread};
use nullbyte_core::video::frame_buffer::{FrameConsumer, VideoFrameData};
use nullbyte_core::video::renderer::Renderer;

fn init_tracing() {
    // KRITIŠKAI SVARBU: niekada į stdout (CLAUDE.md §10) — numatytasis tracing_subscriber
    // writer'is rašo į stdout, tad čia jis EKSPLICITIŠKAI nukreipiamas į stderr.
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// P4.0.2 laikinas verifikacijos hook'as. Skenuoja tuos pačius `nullbyte-core` test fixture
/// katalogus, kuriuos naudoja `core::loader` testai (`crates/nullbyte-core/{cores,roms}/`,
/// `.gitignore`'inti) — jei jų lokaliai nėra (CI, švari mašina), langas atsidaro be žaidimo,
/// be crash'o, nepriklausomai nuo konkretaus ROM'o pavadinimo kataloge.
fn test_core_and_rom() -> Option<(PathBuf, PathBuf)> {
    let core_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core");
    let core = core_root.join("cores/snes9x_libretro.dylib");
    if !core.exists() {
        return None;
    }
    let roms_dir = core_root.join("roms/snes");
    let rom = std::fs::read_dir(&roms_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(str::to_lowercase)
                .as_deref()
                == Some("sfc")
        })?;
    Some((core, rom))
}

/// Atidaro cpal audio srautą, kurio šaltinis — `AudioConsumer` (P3.2/P3.4).
fn open_audio(mut consumer: AudioConsumer) -> Result<AudioOutput, CoreError> {
    AudioOutput::open(move |buf: &mut [f32], _channels: u16| {
        consumer.fill(buf);
    })
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    frame_consumer: Option<FrameConsumer>,
    emu_thread: Option<EmuThread>,
    /// `cpal::Stream` (macOS CoreAudio backend'e) NĖRA `Send` — anksčiau (Tauri managed
    /// state) tai reikalavo dedikuotos „parkuojančios" gijos vien tam, kad apeitų Tauri
    /// `Send + Sync` reikalavimą (žr. senojo `commands/emulator.rs::start_audio_pump`
    /// doc komentarą). `App` gyvena TIK winit main gijoje ir niekada nekerta gijos ribos,
    /// tad `AudioOutput` čia tiesiog laukas — dedikuota gija ADR-016 architektūroje
    /// nebereikalinga.
    _audio_output: Option<AudioOutput>,
    /// `GamepadThread` (P4.1) — dedikuota gija LIEKA (ne architektūros klaida): `gilrs-core`
    /// macOS backend'as PATS viduje sukuria savo „gilrs" giją su CFRunLoop
    /// (`gilrs-core-0.6.8/src/platform/macos/gamepad.rs::spawn_thread`) ir HID įvykius
    /// persiunčia per `mpsc::channel` — `next_event_blocking`/`try_recv` tiesiog skaito iš
    /// kanalo, nepriklausomai nuo to, kurioje gijoje kviečiami. Todėl kviečiančiosios pusės
    /// run loop'as neturi jokios reikšmės HID pristatymui; winit main gija čia tik kanalo
    /// vartotoja (`about_to_wait`), lygiai taip pat, kaip būtų bet kurioje kitoje gijoje.
    gamepad_thread: Option<GamepadThread>,
    gamepad_rx: Option<std::sync::mpsc::Receiver<GamepadEvent>>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            frame_consumer: None,
            emu_thread: None,
            _audio_output: None,
            gamepad_thread: None,
            gamepad_rx: None,
        }
    }

    /// P4.0.2 acceptance: įrodo, kad winit `KeyboardInput` REALIAI pasiekia procesą — su
    /// Tauri `Window` tai buvo neįmanoma (ADR-016, CLAUDE.md §10). Pilnas fizinio klavišo →
    /// `RETRO_DEVICE_ID_JOYPAD_*` mapping'as yra atskira užduotis (P4.2); čia tik paprastas
    /// test mapping'as (strėlė → log'as), kuris įrodo, kad įvykiai ateina.
    fn handle_keyboard(&self, event: KeyEvent) {
        if event.state != ElementState::Pressed || event.repeat {
            return;
        }
        let label = match event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp) => Some("UP"),
            PhysicalKey::Code(KeyCode::ArrowDown) => Some("DOWN"),
            PhysicalKey::Code(KeyCode::ArrowLeft) => Some("LEFT"),
            PhysicalKey::Code(KeyCode::ArrowRight) => Some("RIGHT"),
            _ => None,
        };
        if let Some(label) = label {
            tracing::info!(button = label, "klaviatūros test mapping: strėlė paspausta");
        }
    }

    /// P4.0.2 acceptance (simetriškas klaviatūrai): įrodo, kad `GamepadThread` įvykiai
    /// REALIAI pasiekia `nullbyte-emu`. Pilnas mapping'as (fizinis mygtukas →
    /// `RETRO_DEVICE_ID_JOYPAD_*`) — P4.2, kol kas tik log'as. `try_recv()` — neblokuojantis,
    /// nes kviečiama kas `about_to_wait()` ciklą, ne dedikuotoje gijoje.
    fn drain_gamepad_events(&mut self) {
        let Some(rx) = self.gamepad_rx.as_ref() else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(GamepadEvent::Connected { id, name }) => {
                    tracing::info!(gamepad_id = id, name = %name, "gamepad prijungtas");
                }
                Ok(GamepadEvent::Disconnected { id }) => {
                    tracing::info!(gamepad_id = id, "gamepad atjungtas");
                }
                Ok(GamepadEvent::ButtonChanged {
                    id,
                    button,
                    pressed: true,
                }) => {
                    tracing::info!(gamepad_id = id, button = ?button, "gamepad mygtukas paspaustas");
                }
                Ok(GamepadEvent::ButtonChanged { pressed: false, .. }) => {
                    // Atleidimo įvykiai kol kas neloginami — simetriška klaviatūros
                    // handle_keyboard() elgesiui (žr. aukščiau).
                }
                Ok(GamepadEvent::AxisChanged { .. }) => {
                    // Per triukšminga log'inti kiekvieną ašies pokytį — mapping'as (P4.2)
                    // juos naudos tiesiogiai, be log'o.
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.gamepad_rx = None;
                    break;
                }
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            // `resumed()` gali suveikti pakartotinai (macOS aktyvacija, mobiliųjų suspend/
            // resume ciklas) — jau inicializuota, nekurk visko iš naujo.
            return;
        }
        event_loop.set_control_flow(ControlFlow::Poll);

        let attributes = Window::default_attributes()
            .with_title("Nullbyte — Emuliatorius")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));
        let window = match event_loop.create_window(attributes) {
            Ok(w) => Arc::new(w),
            Err(error) => {
                tracing::error!(%error, "nepavyko sukurti winit lango");
                event_loop.exit();
                return;
            }
        };

        let size = window.inner_size();
        let mut renderer = match Renderer::new(Arc::clone(&window), (size.width, size.height)) {
            Ok(r) => r,
            Err(error) => {
                tracing::error!(%error, "nepavyko sukurti wgpu Renderer");
                event_loop.exit();
                return;
            }
        };
        // Pirmas render() iškart — juodas fonas, kol dar neįkeltas nė vienas kadras (žr.
        // Renderer::render() doc), kad langas nerodytų neinicializuoto Surface turinio.
        if let Err(error) = renderer.render() {
            tracing::warn!(%error, "pradinis renderer.render() klaida");
        }

        let (device_sample_rate, device_channels) = match default_config() {
            Ok(cfg) => cfg,
            Err(error) => {
                tracing::error!(%error, "nepavyko gauti audio įrenginio numatytosios konfigūracijos");
                event_loop.exit();
                return;
            }
        };

        let (emu_thread, frame_consumer, audio_consumer) =
            EmuThread::spawn(device_sample_rate, device_channels);

        let audio_output = match open_audio(audio_consumer) {
            Ok(output) => Some(output),
            Err(error) => {
                tracing::warn!(%error, "audio išvestis nepavyko — tęsiame be garso");
                None
            }
        };

        let (gamepad_thread, gamepad_rx) = match GamepadThread::spawn() {
            Ok((thread, rx)) => (Some(thread), Some(rx)),
            Err(error) => {
                tracing::warn!(%error, "gamepad gija nepavyko — tęsiame be gamepad įvesties");
                (None, None)
            }
        };

        if let Some((core, rom)) = test_core_and_rom() {
            tracing::info!(
                core = %core.display(),
                rom = %rom.display(),
                "P4.0.2 test hook: kraunam core+ROM"
            );
            if let Err(error) = emu_thread.send(EmuCommand::Load { core, rom }) {
                tracing::error!(%error, "nepavyko nusiųsti Load komandos emuliavimo gijai");
            }
        } else {
            tracing::warn!(
                "nerasta nullbyte-core test fixture core/ROM (crates/nullbyte-core/cores|roms) \
                 — langas atsidaro be žaidimo"
            );
        }

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.frame_consumer = Some(frame_consumer);
        self.emu_thread = Some(emu_thread);
        self._audio_output = audio_output;
        self.gamepad_thread = gamepad_thread;
        self.gamepad_rx = gamepad_rx;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("langas uždaromas");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard(event);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.drain_gamepad_events();

        let Some(consumer) = self.frame_consumer.as_mut() else {
            return;
        };
        if !consumer.update() {
            return;
        }

        let frame = consumer.current();
        let owned = VideoFrameData {
            width: frame.width,
            height: frame.height,
            aspect_ratio: frame.aspect_ratio,
            generation: frame.generation,
            data: frame.data.clone(),
        };

        if let Some(renderer) = self.renderer.as_mut() {
            renderer.upload_frame(&owned);
            if let Err(error) = renderer.render() {
                tracing::warn!(%error, "renderer.render() klaida");
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        tracing::info!("nullbyte-emu išsijungia");
        // `self.emu_thread`/`self._audio_output` Drop čia (App drop'inama iškart po
        // run_app() grąžinimo) švariai sustabdo emuliavimo giją ir audio srautą.
    }
}

fn main() {
    init_tracing();

    let mut builder = EventLoop::builder();
    // macOS: numatytai winit naudoja ActivationPolicy::Regular — vaiko procesas atsirastų
    // Dock'e kaip antra programa (CLAUDE.md §10).
    #[cfg(target_os = "macos")]
    builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = builder.build().expect("nepavyko sukurti winit event loop");

    let mut app = App::new();
    event_loop
        .run_app(&mut app)
        .expect("winit event loop baigėsi klaida");
}
