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
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

use nullbyte_core::audio::output::{default_config, AudioOutput};
use nullbyte_core::audio::ring::AudioConsumer;
use nullbyte_core::core::runner::{EmuCommand, EmuThread, InputState};
use nullbyte_core::error::CoreError;
use nullbyte_core::input::gamepad::{GamepadEvent, GamepadThread};
use nullbyte_core::input::hotkeys::{resolve_hotkey, HotkeyAction, HotkeyKey};
use nullbyte_core::input::mapping::{
    default_gamepad_mapping, default_keyboard_mapping, joypad_bit, KeyboardKey,
};
use nullbyte_core::ipc::StatusWriter;
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

/// `system_dir`/`save_dir` — CLI argumentai `argv[1]`/`argv[2]` (`nullbyte-app` juos paduoda
/// sidecar spawn metu, žr. `nullbyte_app::ipc::EmuClient::spawn`, MVP.md P9.1). Standalone
/// dev paleidimui (`cargo run -p nullbyte-emu`, be `nullbyte-app`) trūkstami argumentai
/// pakeičiami `nullbyte-core/{bios,saves}` test fixture katalogais — tas pats principas kaip
/// `test_core_and_rom()` aukščiau.
fn resolve_system_and_save_dirs() -> (PathBuf, PathBuf) {
    let mut args = std::env::args().skip(1);
    let system_dir = args.next().map(PathBuf::from);
    let save_dir = args.next().map(PathBuf::from);

    let core_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nullbyte-core");
    (
        system_dir.unwrap_or_else(|| core_root.join("bios")),
        save_dir.unwrap_or_else(|| core_root.join("saves")),
    )
}

/// Atidaro cpal audio srautą, kurio šaltinis — `AudioConsumer` (P3.2/P3.4).
fn open_audio(mut consumer: AudioConsumer) -> Result<AudioOutput, CoreError> {
    AudioOutput::open(move |buf: &mut [f32], _channels: u16| {
        consumer.fill(buf);
    })
}

/// Winit vartotojo įvykis — VIENINTELĖ paskirtis: pranešti main gijai (per
/// [`EventLoopProxy`]), kad stdin skaitymo gija (kita gija, žr. `ipc::run_command_reader`)
/// gavo `EOF`, tad reikia švariai išsijungti (P4.0.4). `ActiveEventLoop` NETURI
/// `create_proxy()` (tik pati `EventLoop<T>`, PRIEŠ `run_app()`) — proxy sukuriamas `main()`
/// ir laikomas `App` lauke, kad `resumed()` galėtų jį klonuoti į stdin giją.
#[derive(Debug)]
enum EmuUserEvent {
    StdinClosed,
}

struct App {
    event_loop_proxy: EventLoopProxy<EmuUserEvent>,
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
    /// Laikoma gyva TIK tam, kad rašymo gija nebūtų nutraukta (žr. `nullbyte_core::ipc`
    /// modulio doc) — niekas tiesiogiai jos nekviečia po `resumed()`.
    _status_writer: Option<StatusWriter>,
    /// P4.2/P4.3: einamas mygtukų bitmask'as iš KIEKVIENO įvesties šaltinio ATSKIRAI (žr.
    /// `send_port_input` doc dėl KODĖL, ne vieno bendro lauko). Klaviatūra visada valdo TIK
    /// port'ą 0 (žaidėjas 1) — `core::callbacks::input_state_cb` jau palaiko iki 4 portų
    /// (P1.4/P1.5), bet klaviatūra fiziškai negali būti „antras žaidėjas" vienu metu su
    /// pirmuoju tame pačiame kompiuteryje, tad multiplayer sprendžiamas TIK per gamepad'us.
    keyboard_buttons: u16,
    /// P4.3: kiekvieno iš 4 portų gamepad'ų mygtukų bitmask'as (indeksas = port'o numeris).
    gamepad_port_buttons: [u16; 4],
    /// P4.3: `gilrs` gamepad ID → priskirtas port'as (0..4). Priskiriama PIRMO PRISIJUNGIMO
    /// eile — pirmas prisijungęs gamepad'as gauna port'ą 0 (dalinasi su klaviatūra, kaip
    /// „žaidėjas 1" abiem įvesties būdais), antras — port'ą 1, ir t.t. Atsijungus, port'as
    /// ATLAISVINAMAS kitam gamepad'ui (žr. `drain_gamepad_events`).
    gamepad_ports: std::collections::HashMap<usize, usize>,
    /// P4.4: dabartinė modifikatorių būsena (Shift/Ctrl/Alt/Cmd) — atnaujinama per
    /// `WindowEvent::ModifiersChanged` (winit NETEIKIA modifikatorių tiesiogiai
    /// `KeyEvent`'e, tai atskiras įvykis). Reikalinga F5-F8 vs Shift+F5-F8 ir
    /// Cmd/Ctrl+R atskyrimui.
    modifiers: ModifiersState,
    /// P4.4: `F1` (pauzė/tęsti) TOGGLE — `EmuCommand` neturi „koks dabar būvis" užklausos,
    /// tad `nullbyte-emu` pats laiko, ką paskutinį kartą nusiuntė.
    paused: bool,
}

impl App {
    fn new(event_loop_proxy: EventLoopProxy<EmuUserEvent>) -> Self {
        Self {
            event_loop_proxy,
            window: None,
            renderer: None,
            frame_consumer: None,
            emu_thread: None,
            _audio_output: None,
            gamepad_thread: None,
            gamepad_rx: None,
            _status_writer: None,
            keyboard_buttons: 0,
            gamepad_port_buttons: [0; 4],
            gamepad_ports: std::collections::HashMap::new(),
            modifiers: ModifiersState::empty(),
            paused: false,
        }
    }

    /// P4.2/P4.4: konvertuoja klavišo paspaudimą/atleidimą ARBA į hotkey veiksmą, ARBA į
    /// `EmuCommand::SetInput` port'ui 0 — NIEKADA abu (žr. MVP.md P4.4 acceptance
    /// „Nekonfliktuoja su žaidimo įvestimi": hotkey klavišai (`F1`-`F11`, `Esc`, `Cmd/Ctrl+R`)
    /// visiškai nesikerta su žaidimo klavišais (strėlės, `Z`/`X`/`A`/`S`, `Enter`/`Shift`),
    /// tad hotkey patikra PIRMIAU ir `return` užtikrina, kad viena netyčia neužgožtų kitos).
    /// `Space` (fast-forward) apdorojama ATSKIRAI — tai VIENINTELIS hotkey su press/release
    /// būviu, ne trigger'iu (žr. `hotkeys` modulio doc).
    ///
    /// `event.repeat` (klaviatūros OS-lygio pakartojimas laikant nuspaustą) praleidžiamas
    /// trigger-tipo hotkey'ams IR žaidimo mygtukams — bitas/veiksmas jau įvykdytas nuo pirmo
    /// paspaudimo, pakartotinis būtų arba no-op (žaidimo mygtukas), arba klaidingas
    /// pasikartojantis veiksmas (pvz. F1 kelis kartus per sekundę laikant nuspaustą).
    fn handle_keyboard(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(KeyCode::Space) = event.physical_key {
            if !event.repeat {
                self.send_emu_command(EmuCommand::SetFastForward(
                    event.state == ElementState::Pressed,
                ));
            }
            return;
        }

        if event.state == ElementState::Pressed && !event.repeat {
            if let Some(hotkey) = physical_key_to_hotkey(event.physical_key) {
                let primary_modifier = if cfg!(target_os = "macos") {
                    self.modifiers.super_key()
                } else {
                    self.modifiers.control_key()
                };
                if let Some(action) =
                    resolve_hotkey(hotkey, self.modifiers.shift_key(), primary_modifier)
                {
                    self.handle_hotkey_action(action);
                    return;
                }
            }
        }

        if event.repeat {
            return;
        }
        let Some(key) = physical_key_to_mapping_key(event.physical_key) else {
            return;
        };
        let Some(joypad_id) = default_keyboard_mapping(key) else {
            return;
        };
        let bit = joypad_bit(joypad_id);
        match event.state {
            ElementState::Pressed => self.keyboard_buttons |= bit,
            ElementState::Released => self.keyboard_buttons &= !bit,
        }
        self.send_port_input(0);
    }

    /// P4.3: priskiria kiekvieną prisijungusį gamepad'ą KITAM laisvam port'ui (0..4, žr.
    /// `gamepad_ports` doc), konvertuoja mygtukų įvykius į `EmuCommand::SetInput` teisingam
    /// port'ui. `try_recv()` — neblokuojantis, nes kviečiama kas `about_to_wait()` ciklą, ne
    /// dedikuotoje gijoje.
    fn drain_gamepad_events(&mut self) {
        // `take()`, NE `as_ref()` — reikia `&mut self` cikle (`assign_gamepad_port`), o
        // skolinimas iš `self.gamepad_rx` tam kliudytų (skolinimo tikrintuvas). `rx` grąžinama
        // atgal ciklo pabaigoje, nebent `Disconnected` (tada lieka `None`, kaip ir anksčiau).
        let Some(rx) = self.gamepad_rx.take() else {
            return;
        };
        let mut changed_ports: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut disconnected = false;
        loop {
            match rx.try_recv() {
                Ok(GamepadEvent::Connected { id, name }) => match self.assign_gamepad_port(id) {
                    Some(port) => {
                        tracing::info!(gamepad_id = id, name = %name, port, "gamepad prijungtas");
                    }
                    None => {
                        tracing::warn!(
                            gamepad_id = id,
                            name = %name,
                            "gamepad prijungtas, bet visi 4 portai jau užimti — ignoruojamas"
                        );
                    }
                },
                Ok(GamepadEvent::Disconnected { id }) => {
                    if let Some(port) = self.gamepad_ports.remove(&id) {
                        tracing::info!(gamepad_id = id, port, "gamepad atjungtas");
                        // Atlaisvinti port'ą — kitaip paskutinė žinoma bitmask'o reikšmė
                        // liktų „įstrigusi" (core'as manytų, kad mygtukas vis dar laikomas).
                        self.gamepad_port_buttons[port] = 0;
                        changed_ports.insert(port);
                    } else {
                        tracing::info!(
                            gamepad_id = id,
                            "gamepad atjungtas (nebuvo priskirtas port'ui)"
                        );
                    }
                }
                Ok(GamepadEvent::ButtonChanged {
                    id,
                    button,
                    pressed,
                }) => {
                    let (Some(&port), Some(joypad_id)) =
                        (self.gamepad_ports.get(&id), default_gamepad_mapping(button))
                    else {
                        continue;
                    };
                    let bit = joypad_bit(joypad_id);
                    if pressed {
                        self.gamepad_port_buttons[port] |= bit;
                    } else {
                        self.gamepad_port_buttons[port] &= !bit;
                    }
                    changed_ports.insert(port);
                }
                Ok(GamepadEvent::AxisChanged { .. }) => {
                    // Analoginės ašys → skaitmeninis D-pad ekvivalentas NEĮTRAUKTA MVP metu —
                    // MVP.md P4.2 „Ką daryti" prašo tik mygtukų mapping'o.
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }
        if !disconnected {
            self.gamepad_rx = Some(rx);
        }
        for port in changed_ports {
            self.send_port_input(port);
        }
    }

    /// Priskiria `gilrs` gamepad ID kitam laisvam port'ui (0..4) PIRMO PRISIJUNGIMO eile —
    /// žr. `gamepad_ports` lauko doc. `None`, jei visi 4 jau užimti (MVP libretro joypad
    /// portų limitas, žr. `core::callbacks::input_state_cb`).
    fn assign_gamepad_port(&mut self, id: usize) -> Option<usize> {
        if let Some(&existing) = self.gamepad_ports.get(&id) {
            return Some(existing); // Jau priskirtas (pvz. pakartotinis Connected įvykis).
        }
        let taken: std::collections::HashSet<usize> =
            self.gamepad_ports.values().copied().collect();
        let port = (0..4).find(|p| !taken.contains(p))?;
        self.gamepad_ports.insert(id, port);
        Some(port)
    }

    /// Siunčia SUJUNGTĄ (klaviatūra `|` gamepad, TIK port'ui 0 — žr. `keyboard_buttons` doc)
    /// bitmask'ą nurodytam port'ui. Du ATSKIRI laukai (`keyboard_buttons`/
    /// `gamepad_port_buttons`), sujungiami TIK siunčiant — jei būtų vienas bendras laukas su
    /// tiesioginiu set/clear iš abiejų šaltinių, vieno šaltinio mygtuko ATLEIDIMAS galėtų
    /// netyčia išvalyti bitą, kurį VIS DAR laiko nuspaudęs kitas šaltinis (pvz. laikai `UP`
    /// klaviatūra IR gamepad'u vienu metu, atleidi tik gamepad'ą — be šio atskyrimo
    /// klaviatūros paspaudimas irgi dingtų).
    fn send_port_input(&self, port: usize) {
        let Some(emu_thread) = &self.emu_thread else {
            return;
        };
        let gamepad_bits = self.gamepad_port_buttons[port];
        let buttons = if port == 0 {
            self.keyboard_buttons | gamepad_bits
        } else {
            gamepad_bits
        };
        if let Err(error) = emu_thread.send(EmuCommand::SetInput(InputState {
            port: port as u32,
            buttons,
        })) {
            tracing::warn!(%error, port, "nepavyko nusiųsti SetInput komandos");
        }
    }

    /// P4.4: vykdo [`HotkeyAction`] — ARBA nusiunčia `EmuCommand` (dauguma atvejų), ARBA
    /// atlieka lango lygmens veiksmą (`ToggleFullscreen`/`ExitFullscreenOrLibrary`), kurio
    /// `nullbyte-core` net neturi kaip reprezentuoti (žr. `HotkeyAction` doc).
    fn handle_hotkey_action(&mut self, action: HotkeyAction) {
        match action {
            HotkeyAction::TogglePause => {
                self.paused = !self.paused;
                let cmd = if self.paused {
                    EmuCommand::Pause
                } else {
                    EmuCommand::Resume
                };
                tracing::info!(paused = self.paused, "hotkey: pauzė/tęsti");
                self.send_emu_command(cmd);
            }
            HotkeyAction::QuickSave => {
                tracing::info!("hotkey: quick save");
                self.send_emu_command(EmuCommand::SaveState(0));
            }
            HotkeyAction::QuickLoad => {
                tracing::info!("hotkey: quick load");
                self.send_emu_command(EmuCommand::LoadState(0));
            }
            HotkeyAction::SaveStateSlot(slot) => {
                tracing::info!(slot, "hotkey: save state slot");
                self.send_emu_command(EmuCommand::SaveState(slot));
            }
            HotkeyAction::LoadStateSlot(slot) => {
                tracing::info!(slot, "hotkey: load state slot");
                self.send_emu_command(EmuCommand::LoadState(slot));
            }
            HotkeyAction::Reset => {
                tracing::info!("hotkey: reset");
                self.send_emu_command(EmuCommand::Reset);
            }
            HotkeyAction::ToggleFullscreen => self.toggle_fullscreen(),
            HotkeyAction::ExitFullscreenOrLibrary => {
                // „grįžti į biblioteką" dalis — P7 UI dar neegzistuoja (žr. `HotkeyAction`
                // doc), tad kol kas TIK išeina iš fullscreen, jei jame esame.
                if let Some(window) = &self.window {
                    if window.fullscreen().is_some() {
                        tracing::info!("hotkey: Esc — išeinama iš fullscreen");
                        window.set_fullscreen(None);
                    }
                }
            }
        }
    }

    fn toggle_fullscreen(&self) {
        let Some(window) = &self.window else {
            return;
        };
        if window.fullscreen().is_some() {
            tracing::info!("hotkey: F11 — išeinama iš fullscreen");
            window.set_fullscreen(None);
        } else {
            tracing::info!("hotkey: F11 — įjungiamas fullscreen");
            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }

    fn send_emu_command(&self, cmd: EmuCommand) {
        let Some(emu_thread) = &self.emu_thread else {
            return;
        };
        if let Err(error) = emu_thread.send(cmd) {
            tracing::warn!(%error, "nepavyko nusiųsti EmuCommand (hotkey)");
        }
    }
}

/// `nullbyte-core` sąmoningai nepriklauso nuo `winit` (žr. `mapping` modulio doc) — šis
/// konvertavimas gyvena `nullbyte-emu` pusėje, kur `winit` tipai jau yra.
fn physical_key_to_mapping_key(key: PhysicalKey) -> Option<KeyboardKey> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    match code {
        KeyCode::ArrowUp => Some(KeyboardKey::ArrowUp),
        KeyCode::ArrowDown => Some(KeyboardKey::ArrowDown),
        KeyCode::ArrowLeft => Some(KeyboardKey::ArrowLeft),
        KeyCode::ArrowRight => Some(KeyboardKey::ArrowRight),
        KeyCode::KeyZ => Some(KeyboardKey::KeyZ),
        KeyCode::KeyX => Some(KeyboardKey::KeyX),
        KeyCode::KeyA => Some(KeyboardKey::KeyA),
        KeyCode::KeyS => Some(KeyboardKey::KeyS),
        KeyCode::Enter => Some(KeyboardKey::Enter),
        KeyCode::ShiftRight => Some(KeyboardKey::ShiftRight),
        _ => None,
    }
}

/// `nullbyte-core` sąmoningai nepriklauso nuo `winit` (žr. `hotkeys` modulio doc) — tas pats
/// konvertavimo principas kaip [`physical_key_to_mapping_key`].
fn physical_key_to_hotkey(key: PhysicalKey) -> Option<HotkeyKey> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    match code {
        KeyCode::F1 => Some(HotkeyKey::F1),
        KeyCode::F2 => Some(HotkeyKey::F2),
        KeyCode::F4 => Some(HotkeyKey::F4),
        KeyCode::F5 => Some(HotkeyKey::F5),
        KeyCode::F6 => Some(HotkeyKey::F6),
        KeyCode::F7 => Some(HotkeyKey::F7),
        KeyCode::F8 => Some(HotkeyKey::F8),
        KeyCode::F11 => Some(HotkeyKey::F11),
        KeyCode::Escape => Some(HotkeyKey::Escape),
        KeyCode::KeyR => Some(HotkeyKey::KeyR),
        _ => None,
    }
}

impl ApplicationHandler<EmuUserEvent> for App {
    /// P4.0.4: `stdin` EOF (žr. `ipc::run_command_reader` iškvietimą `resumed()` žemiau) reiškia,
    /// kad tėvo procesas nutrūko (bet kokiu būdu, įskaitant `kill -9`) — Unix pipe semantika,
    /// NE PID pollinimas (CLAUDE.md §10). `event_loop.exit()` sukelia `run_app()` grąžinimą
    /// `main()`, kur `App` (taigi ir `self.emu_thread`) numetama — `EmuThread::Drop` PATS
    /// nusiunčia `Stop` ir laukia `join()`, tad core'as švariai atlaisvinamas prieš procesui
    /// pasibaigiant (žr. `exiting()` doc žemiau).
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EmuUserEvent) {
        match event {
            EmuUserEvent::StdinClosed => {
                tracing::info!("stdin EOF — tėvo procesas nutrūko, išsijungiame švariai (P4.0.4)");
                event_loop.exit();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            // `resumed()` gali suveikti pakartotinai (macOS aktyvacija, mobiliųjų suspend/
            // resume ciklas) — jau inicializuota, nekurk visko iš naujo.
            return;
        }
        event_loop.set_control_flow(ControlFlow::Poll);

        // Kaip anksti, kaip įmanoma — StatusWriter::spawn() PATS sinchroniškai parašo
        // IpcHello kaip pačią pirmą stdout eilutę (žr. nullbyte_core::ipc modulio doc).
        // Nepavykus (pvz. pipe jau uždarytas) — tęsiame BE status reportavimo, lygiai taip
        // pat, kaip audio nepavykimas žemiau netrukdo langui atsidaryti.
        let (status_writer, status_sender) = match StatusWriter::spawn(std::io::stdout()) {
            Ok((writer, sender)) => (Some(writer), Some(sender)),
            Err(error) => {
                tracing::warn!(%error, "IPC status writer nepavyko — tęsiame be status reportavimo");
                (None, None)
            }
        };

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

        let (system_dir, save_dir) = resolve_system_and_save_dirs();
        let (emu_thread, frame_consumer, audio_consumer) = EmuThread::spawn(
            device_sample_rate,
            device_channels,
            system_dir,
            save_dir,
            status_sender,
        );

        // Fono gija skaito stdin per `BufRead::lines()` (MVP.md P4.0.3 „Ką daryti") — tik
        // klonuota `Sender<EmuCommand>`, NE `&EmuThread`, kad nereikėtų 'static nuorodos
        // (žr. `EmuThread::command_sender()` doc).
        let command_sender = emu_thread.command_sender();
        let event_loop_proxy = self.event_loop_proxy.clone();
        std::thread::Builder::new()
            .name("nullbyte-emu-stdin".to_string())
            .spawn(move || {
                ipc::run_command_reader(std::io::stdin().lock(), command_sender);
                // `run_command_reader` grįžta TIK kai stdin baigėsi (EOF — tėvas nutrūko,
                // P4.0.4) arba įvyko neatstatoma skaitymo klaida — abiem atvejais tėvas
                // nebepasiekiamas, tad švariai baikime VISĄ procesą (žr. `user_event` doc).
                // `send_event` klaida reikštų, kad event loop JAU baigėsi — tada nėra ką
                // daugiau daryti, procesas ir taip užsidaro.
                let _ = event_loop_proxy.send_event(EmuUserEvent::StdinClosed);
            })
            .expect("nepavyko sukurti stdin skaitymo gijos");

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
        self._status_writer = status_writer;
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
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
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

    let mut builder = EventLoop::<EmuUserEvent>::with_user_event();
    // macOS: numatytai winit naudoja ActivationPolicy::Regular — vaiko procesas atsirastų
    // Dock'e kaip antra programa (CLAUDE.md §10).
    #[cfg(target_os = "macos")]
    builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = builder.build().expect("nepavyko sukurti winit event loop");

    let mut app = App::new(event_loop.create_proxy());
    event_loop
        .run_app(&mut app)
        .expect("winit event loop baigėsi klaida");
}
