//! Emuliatoriaus Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į
//! `video::renderer`.

use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::state::AppState;
use crate::video::renderer::Renderer;

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

    let renderer = Renderer::new(window.clone())?;

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
