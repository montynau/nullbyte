//! Įvesties Tauri komandos (CLAUDE.md §6.3) — plonas sluoksnis, deleguoja į `input::gamepad`.
//!
//! `get_input_mapping`/`set_input_mapping`/`reset_input_mapping` (P7.6 Input panelė) —
//! **TIK persistencija**, dar NEVEIKIA realiame žaidime. Realus mygtukų mapping'as (žr.
//! `nullbyte_core::input::mapping`) šiuo metu HARDKODINTAS `nullbyte-emu/src/main.rs`, be
//! jokio IPC kanalo jam perduoti — o realus žaidimo paleidimo srautas per
//! `crate::ipc::EmuClient` yra P9.1, DAR NEĮGYVENDINTA. Šios komandos tiesiog leidžia
//! vartotojui iš anksto susikonfigūruoti norimą mapping'ą Settings ekrane, kad jis būtų
//! paruoštas, kai P9.1 sujungs UI pasirinkimą su realiu vaiko procesu.

use tauri::{AppHandle, Emitter, State};

use nullbyte_core::input::gamepad::GamepadEvent;

use crate::db::settings;
use crate::error::AppError;
use crate::state::AppState;

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

/// Vienas RetroPad mygtuko priskyrimas — CLAUDE.md §7.3 (IPC struct'ai: camelCase).
/// `keyboardKey` — naršyklės `KeyboardEvent.code` reikšmė (pvz. `"ArrowUp"`, `"KeyZ"`).
/// `gamepadButton` — naršyklės Gamepad API `button` indeksas standartiniame mapping'e.
/// Abu `None`, jei tas RetroPad mygtukas dar nepriskirtas jokiam fiziniam įvesties šaltiniui.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputBinding {
    pub retropad_button: String,
    pub keyboard_key: Option<String>,
    pub gamepad_button: Option<u32>,
}

const INPUT_MAPPING_KEY: &str = "input.mapping";

/// Numatytieji priskyrimai — RetroPad mygtukų sąrašas/eiliškumas tas pats kaip
/// `nullbyte_core::input::mapping::RETRO_DEVICE_ID_JOYPAD_*` (be `Mode`/nenaudojamų).
/// Klaviatūros numatytosios reikšmės ATITINKA ŠIUO METU REALIAI VEIKIANTĮ hardkodintą
/// `default_keyboard_mapping` (`ArrowUp`/`Z`/`X`/`A`/`S`/`Enter`/`ShiftRight`) — kad UI iš
/// karto rodytų teisingą būvį, o ne tuščią lentelę. `L`/`R`/`L2`/`R2`/`L3`/`R3` klaviatūroje
/// numatytai nepriskirti (tas pats kaip realiame mapping'e). Gamepad numatytieji VISI
/// tušti — standartinio (gilrs) mapping'o nekartojame kaip Gamepad API button indeksų
/// spėjimo, nes jie NĖRA tas pats API/numeravimas; vartotojas priskiria pats, jei nori.
fn default_bindings() -> Vec<InputBinding> {
    let entries: &[(&str, Option<&str>)] = &[
        ("up", Some("ArrowUp")),
        ("down", Some("ArrowDown")),
        ("left", Some("ArrowLeft")),
        ("right", Some("ArrowRight")),
        ("b", Some("KeyZ")),
        ("a", Some("KeyX")),
        ("y", Some("KeyA")),
        ("x", Some("KeyS")),
        ("l", None),
        ("r", None),
        ("l2", None),
        ("r2", None),
        ("l3", None),
        ("r3", None),
        ("select", Some("ShiftRight")),
        ("start", Some("Enter")),
    ];
    entries
        .iter()
        .map(|(button, key)| InputBinding {
            retropad_button: button.to_string(),
            keyboard_key: key.map(str::to_string),
            gamepad_button: None,
        })
        .collect()
}

/// Grąžina išsaugotą mapping'ą, arba [`default_bindings`], jei vartotojas dar nieko
/// nekeitė (žr. modulio doc dėl KODĖL tai TIK persistencija).
#[tauri::command]
pub fn get_input_mapping(state: State<'_, AppState>) -> Result<Vec<InputBinding>, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    match settings::get(&conn, INPUT_MAPPING_KEY)? {
        Some(json) => serde_json::from_str(&json)
            .map_err(|error| AppError::Other(format!("sugadintas input.mapping JSON: {error}"))),
        None => Ok(default_bindings()),
    }
}

#[tauri::command]
pub fn set_input_mapping(
    state: State<'_, AppState>,
    bindings: Vec<InputBinding>,
) -> Result<(), AppError> {
    let json = serde_json::to_string(&bindings)
        .map_err(|error| AppError::Other(format!("nepavyko serializuoti mapping'o: {error}")))?;
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, INPUT_MAPPING_KEY, &json)
}

/// Ištrina UI išsaugotą override'ą — kitas `get_input_mapping` kvietimas vėl grąžins
/// [`default_bindings`].
#[tauri::command]
pub fn reset_input_mapping(state: State<'_, AppState>) -> Result<(), AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::delete(&conn, INPUT_MAPPING_KEY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_cover_all_sixteen_retropad_buttons_uniquely() {
        let bindings = default_bindings();
        assert_eq!(bindings.len(), 16);
        let unique: std::collections::HashSet<_> = bindings
            .iter()
            .map(|b| b.retropad_button.as_str())
            .collect();
        assert_eq!(
            unique.len(),
            16,
            "retropad_button reikšmės turi būti unikalios"
        );
    }

    /// Numatytosios klaviatūros reikšmės turi ATITIKTI dabar realiai veikiantį
    /// `nullbyte_core::input::mapping::default_keyboard_mapping` — kitaip UI rodytų
    /// klaidingą pradinį būvį (žr. modulio doc).
    #[test]
    fn default_bindings_match_the_real_hardcoded_keyboard_mapping() {
        let bindings = default_bindings();
        let find = |button: &str| {
            bindings
                .iter()
                .find(|b| b.retropad_button == button)
                .and_then(|b| b.keyboard_key.as_deref())
        };
        assert_eq!(find("up"), Some("ArrowUp"));
        assert_eq!(find("down"), Some("ArrowDown"));
        assert_eq!(find("left"), Some("ArrowLeft"));
        assert_eq!(find("right"), Some("ArrowRight"));
        assert_eq!(find("b"), Some("KeyZ"));
        assert_eq!(find("a"), Some("KeyX"));
        assert_eq!(find("y"), Some("KeyA"));
        assert_eq!(find("x"), Some("KeyS"));
        assert_eq!(find("start"), Some("Enter"));
        assert_eq!(find("select"), Some("ShiftRight"));
        assert_eq!(find("l"), None);
        assert_eq!(find("r"), None);
    }

    #[test]
    fn bindings_roundtrip_through_json_like_settings_storage_would() {
        let original = default_bindings();
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Vec<InputBinding> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), original.len());
        assert_eq!(parsed[0].retropad_button, original[0].retropad_button);
    }
}
