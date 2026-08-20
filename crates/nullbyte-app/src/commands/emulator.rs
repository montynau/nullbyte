//! Emuliatoriaus Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į
//! `video::renderer` / `core::runner`.

use tauri::{AppHandle, Manager};

use nullbyte_core::audio::output::AudioOutput;
use nullbyte_core::audio::ring::AudioConsumer;
use nullbyte_core::video::frame_buffer::{FrameConsumer, VideoFrameData};
use nullbyte_core::video::renderer::Renderer;

use crate::error::AppError;
use crate::state::AppState;

/// Atidaro atskirą langą (be webview) emuliatoriaus vaizdui ir inicializuoja wgpu Surface
/// (P2.3). Pilnas žaidimo paleidimo srautas (ROM parinkimas, core'o įkėlimas ir pan.) —
/// P9.1; ši komanda kol kas tik įrodo, kad langas + wgpu mechanizmas veikia.
///
/// Surface kūrimas PRIVALO vykti main gijoje (macOS/Metal reikalavimas, CLAUDE.md §10).
/// Kadangi Tauri komandų handleriai gali būti kviečiami iš bet kurios gijos, naudojame
/// `run_on_main_thread` + kanalą, kad tai garantuotume nepriklausomai nuo iškvietimo gijos.
#[tauri::command]
pub fn open_emulator_window(app: AppHandle) -> Result<(), AppError> {
    let (tx, rx) = std::sync::mpsc::channel();
    let app_for_main_thread = app.clone();

    app.run_on_main_thread(move || {
        let result = create_window_and_renderer(&app_for_main_thread);
        // Klaida čia reikštų, kad rx pusė jau nustojo laukti (pvz. caller'is dingo) —
        // nėra ko daugiau daryti, nei tyliai ignoruoti.
        let _ = tx.send(result);
    })
    .map_err(|e| AppError::Other(format!("nepavyko planuoti main gijos užduoties: {e}")))?;

    rx.recv()
        .map_err(|_| AppError::Other("main gijos užduotis nutrūko be atsakymo".to_string()))?
}

/// # Safety motyvacija
/// Kviečiama TIK per `run_on_main_thread` iš [`open_emulator_window`] — `Renderer::new()`
/// viduje kuriamas wgpu Surface, kuris macOS/Metal atveju panikuotų ne main gijoje.
fn create_window_and_renderer(app: &AppHandle) -> Result<(), AppError> {
    if app.get_window("emulator").is_some() {
        tracing::info!("emuliatoriaus langas jau atidarytas — nekuriame antro");
        return Ok(());
    }

    let window = tauri::window::WindowBuilder::new(app, "emulator")
        .title("Nullbyte — Emuliatorius")
        .inner_size(800.0, 600.0)
        .build()
        .map_err(|e| AppError::Other(format!("nepavyko sukurti emuliatoriaus lango: {e}")))?;

    let size = window
        .inner_size()
        .map_err(|e| AppError::Other(format!("nepavyko gauti lango dydžio: {e}")))?;
    let renderer = Renderer::new(window.clone(), (size.width, size.height))?;

    {
        let state = app.state::<AppState>();
        let mut guard = state
            .renderer
            .lock()
            .map_err(|_| AppError::Other("renderer mutex užstrigęs (poisoned)".to_string()))?;
        *guard = Some(renderer);
    }

    let app_for_resize = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Resized(size) = event {
            let state = app_for_resize.state::<AppState>();
            let lock_result = state.renderer.lock();
            if let Ok(mut guard) = lock_result {
                if let Some(renderer) = guard.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
        }
    });

    tracing::info!("emuliatoriaus langas + wgpu Renderer sėkmingai sukurti");
    Ok(())
}

/// Perjungia emuliatoriaus langą į/iš fullscreen (P2.5). Klavišų susiejimas (`F11`,
/// `Cmd+Ctrl+F`, `Esc`) — P4.2 (`input/keyboard.rs`), kai bus klaviatūros įvesties
/// sluoksnis; Tauri `Window` be webview nesiunčia klaviatūros `WindowEvent`'ų šioje API
/// versijoje, tad realaus klavišo paspaudimo čia dar nėra kam pagauti. Ši komanda —
/// mechanizmas, kurį P4.2/P7.x UI galės kviesti tiesiogiai per `invoke`.
#[tauri::command]
pub fn toggle_emulator_fullscreen(app: AppHandle) -> Result<bool, AppError> {
    let window = app
        .get_window("emulator")
        .ok_or_else(|| AppError::Other("emuliatoriaus langas neatidarytas".to_string()))?;

    let is_fullscreen = window
        .is_fullscreen()
        .map_err(|e| AppError::Other(format!("nepavyko sužinoti fullscreen būvio: {e}")))?;
    window
        .set_fullscreen(!is_fullscreen)
        .map_err(|e| AppError::Other(format!("nepavyko perjungti fullscreen: {e}")))?;

    tracing::info!(
        fullscreen = !is_fullscreen,
        "emuliatoriaus langas perjungtas"
    );
    Ok(!is_fullscreen)
}

/// Paleidžia foninę „frame pump" giją, kuri seka `FrameConsumer` (P2.2) ir kiekvieną naują
/// emuliatoriaus kadrą nupiešia per `Renderer` (P2.4). Kadro duomenys kopijuojami (klonuojami)
/// prieš perduodant į `run_on_main_thread`, nes `Surface`/`render()` operacijos PRIVALO vykti
/// main gijoje (CLAUDE.md §10), o `FrameConsumer` skolinys negali kirsti gijos ribos.
///
/// Gija veikia tol, kol egzistuoja procesas — jei atitinkama `EmuThread` sustabdoma,
/// `consumer.update()` tiesiog visada grąžina `false` ir gija tyliai laukia (1ms intervalais),
/// beveik nenaudodama CPU. Švaraus gijos sustabdymo mechanizmas — post-MVP patobulinimas.
// Kviesime iš `start_game` komandos P9.1 — kol žaidimo paleidimo srautas neapjungtas,
// funkcija lieka nenaudota infrastruktūra (jau patikrinta P2.4 laikinu verifikacijos hook'u).
#[allow(dead_code)]
pub fn start_frame_pump(app: AppHandle, mut consumer: FrameConsumer) {
    std::thread::Builder::new()
        .name("nullbyte-frame-pump".to_string())
        .spawn(move || loop {
            if !consumer.update() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }

            let frame = consumer.current();
            let owned = VideoFrameData {
                width: frame.width,
                height: frame.height,
                aspect_ratio: frame.aspect_ratio,
                generation: frame.generation,
                data: frame.data.clone(),
            };

            let app_for_render = app.clone();
            let _ = app.run_on_main_thread(move || {
                let state = app_for_render.state::<AppState>();
                let Ok(mut guard) = state.renderer.lock() else {
                    return;
                };
                if let Some(renderer) = guard.as_mut() {
                    renderer.upload_frame(&owned);
                    if let Err(error) = renderer.render() {
                        tracing::warn!(%error, "renderer.render() klaida");
                    }
                }
            });
        })
        .expect("nepavyko sukurti frame pump gijos");
}

/// Atidaro realų cpal audio srautą, kurio šaltinis — `AudioConsumer` (P3.2/P3.4).
///
/// **Kodėl dedikuota gija, ne `AppState`:** `cpal::Stream` (macOS CoreAudio backend'e)
/// NĖRA `Send` (viduje laiko `Box<dyn FnMut()>` property listener'į) — jo negalima laikyti
/// `Mutex<Option<AudioOutput>>` lauke, nes Tauri managed state reikalauja `Send + Sync`
/// (kompiliavimo klaida patikrinta P3.4 metu). Sprendimas — ta pati technika kaip
/// `core::runner::EmuThread`: dedikuota gija sukuria IR laiko `AudioOutput` savo pačios
/// stack'e visą gyvavimo trukmę, niekada neperduodama jo per gijos ribą.
#[allow(dead_code)]
pub fn start_audio_pump(mut consumer: AudioConsumer) -> Result<(), AppError> {
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();

    std::thread::Builder::new()
        .name("nullbyte-audio-pump".to_string())
        .spawn(move || {
            let output = AudioOutput::open(move |buf: &mut [f32], _channels: u16| {
                consumer.fill(buf);
            });
            match output {
                Ok(_output) => {
                    let _ = ready_tx.send(Ok(()));
                    // `output` (taigi ir `cpal::Stream`) gyvena čia amžinai — gija tiesiog
                    // "parkuojasi", kol procesas veikia. Švaraus sustabdymo mechanizmas
                    // (analogiškas EmuThread Drop+join) — post-MVP, kai turėsime realų
                    // žaidimo pabaigos srautą (P9.1).
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(3600));
                    }
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            }
        })
        .expect("nepavyko sukurti audio pump gijos");

    ready_rx
        .recv()
        .map_err(|_| AppError::Other("audio pump gija nutrūko be atsakymo".to_string()))?
        .map_err(AppError::from)
}
