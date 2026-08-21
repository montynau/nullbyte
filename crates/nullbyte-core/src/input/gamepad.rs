//! Gamepad aptikimas ir įvykiai per `gilrs` (P4.1).
//!
//! `gilrs::Gilrs` nėra `Sync` — VISI kvietimai (event pump, `gamepad()` lookup'ai) turi
//! vykti iš TOS PAČIOS gijos. Todėl, kaip ir `core::runner::EmuThread`, čia naudojama
//! dedikuota gija: `Gilrs` sukuriamas ir gyvena joje visą laiką, o prisijungimo/atsijungimo
//! bei mygtukų/ašių įvykiai persiunčiami per kanalą kviečiančiajai pusei (`nullbyte-emu`
//! main.rs, žr. P4.0.2 — winit main gija juos nuskaito neblokuojančiai per `about_to_wait()`).

#![allow(dead_code)] // pilnai išnaudos P4.2/P4.3 (mapping, polling) — P4.1 metu naudoja testai.

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;

use gilrs::{Axis, Button, EventType, Gilrs, GilrsBuilder};

use crate::error::CoreError;

/// Analoginių ašių deadzone (P4.1 „Ką daryti": numatytoji 0.2). `gilrs` turi savo įmontuotą
/// deadzone filtrą (pagal kiekvieno valdiklio DB įrašą), bet MVP.md reikalauja FIKSUOTOS
/// 0.2 reikšmės visiems valdikliams — todėl įmontuoti filtrai išjungti
/// (`with_default_filters(false)`) ir deadzone taikomas rankiniu būdu žemiau.
pub const DEFAULT_DEADZONE: f32 = 0.2;

/// Kiek laiko `next_event_blocking` laukia prieš grąžindamas `None` — leidžia periodiškai
/// patikrinti, ar negauta stabdymo signalo, neapkraunant CPU busy-loop'u.
const POLL_TIMEOUT: Duration = Duration::from_millis(100);

/// Įvykis, siunčiamas iš gamepad gijos. `id` — `gilrs` vidinis identifikatorius, stabilus
/// per sesiją (bet gali pasikeisti perkrovus programą) — atvaizdavimas į loginį žaidėjo
/// portą yra P4.2/P4.3 darbas.
#[derive(Debug, Clone, PartialEq)]
pub enum GamepadEvent {
    Connected {
        id: usize,
        name: String,
    },
    Disconnected {
        id: usize,
    },
    ButtonChanged {
        id: usize,
        button: Button,
        pressed: bool,
    },
    /// `value` jau su pritaikytu deadzone (žr. [`DEFAULT_DEADZONE`]) — `0.0`, jei per silpnas
    /// signalas, kitaip perskaičiuotas į `[-1.0, 1.0]` diapazoną be staigaus šuolio ties riba.
    AxisChanged {
        id: usize,
        axis: Axis,
        value: f32,
    },
}

/// Rankena į veikiančią gamepad giją. `Drop` siunčia stabdymo signalą ir laukia `join()`.
pub struct GamepadThread {
    stop_tx: Sender<()>,
    handle: Option<JoinHandle<()>>,
}

impl GamepadThread {
    /// Paleidžia naują dedikuotą gamepad giją. Grąžina `Receiver<GamepadEvent>` —
    /// kviečiantis kodas jais dalinasi toliau (UI pranešimams, įvesties mapping'ui).
    pub fn spawn() -> Result<(Self, Receiver<GamepadEvent>), CoreError> {
        let gilrs = GilrsBuilder::new()
            .with_default_filters(false)
            .build()
            .map_err(|e| CoreError::Other(format!("nepavyko inicializuoti gilrs: {e}")))?;

        let (event_tx, event_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();

        // Jau prijungti valdikliai (aptinkami `build()` metu, be atskiro Connected įvykio) —
        // praneškime apie juos taip pat, kad caller'is iš karto matytų pilną vaizdą.
        for (id, gamepad) in gilrs.gamepads() {
            let _ = event_tx.send(GamepadEvent::Connected {
                id: id.into(),
                name: gamepad.name().to_string(),
            });
        }

        let handle = std::thread::Builder::new()
            .name("nullbyte-gamepad".to_string())
            .spawn(move || run_loop(gilrs, event_tx, stop_rx))
            .map_err(|e| CoreError::Other(format!("nepavyko sukurti gamepad gijos: {e}")))?;

        Ok((
            Self {
                stop_tx,
                handle: Some(handle),
            },
            event_rx,
        ))
    }
}

impl Drop for GamepadThread {
    fn drop(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// `[-1.0, -DEADZONE] ∪ [DEADZONE, 1.0]` → `[-1.0, 1.0]` (tolydus remap, ne tiesiog
/// atkirpimas — vengia staigaus šuolio nuo `0.0` prie `DEADZONE` peržengus ribą).
fn apply_deadzone(value: f32) -> f32 {
    if value.abs() < DEFAULT_DEADZONE {
        0.0
    } else {
        value.signum() * (value.abs() - DEFAULT_DEADZONE) / (1.0 - DEFAULT_DEADZONE)
    }
}

fn run_loop(mut gilrs: Gilrs, event_tx: Sender<GamepadEvent>, stop_rx: Receiver<()>) {
    loop {
        match stop_rx.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => return,
            Err(TryRecvError::Empty) => {}
        }

        while let Some(event) = gilrs.next_event_blocking(Some(POLL_TIMEOUT)) {
            let gamepad_event = match event.event {
                EventType::Connected => Some(GamepadEvent::Connected {
                    id: event.id.into(),
                    name: gilrs.gamepad(event.id).name().to_string(),
                }),
                EventType::Disconnected => Some(GamepadEvent::Disconnected {
                    id: event.id.into(),
                }),
                EventType::ButtonPressed(button, _) => Some(GamepadEvent::ButtonChanged {
                    id: event.id.into(),
                    button,
                    pressed: true,
                }),
                EventType::ButtonReleased(button, _) => Some(GamepadEvent::ButtonChanged {
                    id: event.id.into(),
                    button,
                    pressed: false,
                }),
                EventType::AxisChanged(axis, value, _) => Some(GamepadEvent::AxisChanged {
                    id: event.id.into(),
                    axis,
                    value: apply_deadzone(value),
                }),
                _ => None,
            };

            if let Some(ev) = gamepad_event {
                // Receiver'is dingęs reiškia, kad caller'is nebesidomi — tyliai baigiame giją.
                if event_tx.send(ev).is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadzone_zeroes_small_values() {
        assert_eq!(apply_deadzone(0.0), 0.0);
        assert_eq!(apply_deadzone(0.1), 0.0);
        assert_eq!(apply_deadzone(-0.19), 0.0);
    }

    #[test]
    fn deadzone_remaps_large_values_continuously() {
        // Tiksliai ties riba — turėtų būti ~0.0 (tolydumas, ne šuolis).
        let at_edge = apply_deadzone(DEFAULT_DEADZONE);
        assert!(at_edge.abs() < 1e-6, "at_edge={at_edge}");

        // Pilna verte (1.0) turėtų likti 1.0 (viršutinė riba nekinta).
        let at_max = apply_deadzone(1.0);
        assert!((at_max - 1.0).abs() < 1e-6, "at_max={at_max}");

        // Neigiama pusė veidrodinė.
        let at_min = apply_deadzone(-1.0);
        assert!((at_min + 1.0).abs() < 1e-6, "at_min={at_min}");
    }

    #[test]
    fn deadzone_is_monotonic() {
        let mut previous = apply_deadzone(0.0);
        let mut steps = 0;
        let mut value = 0.0f32;
        while value <= 1.0 {
            let mapped = apply_deadzone(value);
            assert!(
                mapped >= previous,
                "apply_deadzone turėtų būti monotoniškai nemažėjanti [0,1] intervale: {value} -> {mapped} < {previous}"
            );
            previous = mapped;
            value += 0.01;
            steps += 1;
        }
        assert!(steps > 50);
    }

    /// Greitas sanity testas: `Gilrs` inicializavimasis nepanikuoja net be jokio prijungto
    /// valdiklio (CI/headless aplinkoje). Realaus aptikimo (Xbox/DualShock/8BitDo) testas —
    /// rankinis, žr. MVP.md P4.1 acceptance.
    #[test]
    fn spawn_does_not_panic_without_any_gamepad() {
        match GamepadThread::spawn() {
            Ok((thread, _events)) => drop(thread),
            Err(error) => {
                eprintln!("praleista: gilrs neinicializavo šioje aplinkoje ({error})");
            }
        }
    }
}
