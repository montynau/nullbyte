//! Įvesties Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į `input::gamepad`.

use tauri::{AppHandle, Emitter};

use crate::input::gamepad::GamepadEvent;

/// Prisijungimo/atsijungimo pranešimas UI — CLAUDE.md §7.3 (IPC struct'ai: camelCase).
/// Mygtukų/ašių įvykiai (P4.2/P4.3 valdymo mapping'ui) NEsiunčiami UI — tik prisijungimo
/// būvis (P4.1 „Ką daryti": „Prijungimo/atjungimo įvykiai → pranešk UI per Tauri event").
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct GamepadConnectionEvent {
    id: usize,
    connected: bool,
    name: Option<String>,
}

/// Paleidžia foninę giją, kuri seka `Receiver<GamepadEvent>` (P4.1) ir kiekvieną
/// prisijungimo/atsijungimo įvykį persiunčia į UI kaip Tauri `"gamepad-connection"` event'ą.
/// Mygtukų/ašių įvykiai tyliai praleidžiami — jiems dar nėra klausytojo (P4.2/P4.3).
#[allow(dead_code)] // kviesime iš lib.rs setup() P4.2+, kai UI turės ką su šiais event'ais daryti.
pub fn start_gamepad_pump(app: AppHandle, receiver: std::sync::mpsc::Receiver<GamepadEvent>) {
    std::thread::Builder::new()
        .name("nullbyte-gamepad-pump".to_string())
        .spawn(move || {
            for event in receiver {
                let payload = match event {
                    GamepadEvent::Connected { id, name } => GamepadConnectionEvent {
                        id,
                        connected: true,
                        name: Some(name),
                    },
                    GamepadEvent::Disconnected { id } => GamepadConnectionEvent {
                        id,
                        connected: false,
                        name: None,
                    },
                    GamepadEvent::ButtonChanged { .. } | GamepadEvent::AxisChanged { .. } => {
                        continue;
                    }
                };

                if let Err(error) = app.emit("gamepad-connection", payload) {
                    tracing::warn!(%error, "nepavyko išsiųsti gamepad-connection event'o");
                }
            }
            // `receiver` grąžino None — GamepadThread (taigi ir jo Sender) buvo numestas.
            tracing::debug!("gamepad pump gija baigė darbą (GamepadThread numestas)");
        })
        .expect("nepavyko sukurti gamepad pump gijos");
}
