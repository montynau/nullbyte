//! Fizinio mygtuko/klavišo → `RETRO_DEVICE_ID_JOYPAD_*` mapping'as (P4.2).
//!
//! **Kodėl VIENA lentelė visiems gamepad'ams, ne per-brand'inė:** `gilrs` PATS abstrahuoja
//! tarp gamintojų per SDL's `GameControllerDB` — tiek Xbox, tiek DualShock, tiek 8BitDo
//! valdikliai atkeliauja kaip tas pats [`gilrs::Button`] enum'as (`South`/`East`/`North`/
//! `West` ir t.t., pagal FIZINĘ poziciją, ne pagal gamintojo etiketę). Per-brand'inės lentelės
//! (Xbox `A` → ..., DualShock `Cross` → ...) būtų grynas dubliavimas — `gilrs` tai jau
//! išsprendė P4.1 metu (žr. `input::gamepad` modulio doc).
//!
//! **Kodėl `RETRO_DEVICE_ID_JOYPAD_A`/`_B` atrodo „sukeisti":** libretro joypad ID pavadinimai
//! kilę iš SNES valdiklio išdėstymo, kur `B` yra APAČIOJE, `A` — DEŠINĖJE (MVP.md P4.2 „Ką
//! daryti"). Standartinio (Xbox/DS4 tipo) valdiklio APATINIS mygtukas (Xbox `A`, DS4 „Cross")
//! FIZIŠKAI atitinka SNES `B` poziciją, ne SNES `A`. Ši lentelė mapina pagal FIZINĘ poziciją
//! (`gilrs::Button::South`/`East`/`North`/`West`), tad automatiškai gerbia šį persidengimą —
//! tai IR YRA standartinis RetroArch „RetroPad" numatytasis mapping'as, ne Nullbyte'o išradimas.

use gilrs::{Axis, Button};

use crate::core::ffi::{
    RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_DOWN,
    RETRO_DEVICE_ID_JOYPAD_L, RETRO_DEVICE_ID_JOYPAD_L2, RETRO_DEVICE_ID_JOYPAD_L3,
    RETRO_DEVICE_ID_JOYPAD_LEFT, RETRO_DEVICE_ID_JOYPAD_R, RETRO_DEVICE_ID_JOYPAD_R2,
    RETRO_DEVICE_ID_JOYPAD_R3, RETRO_DEVICE_ID_JOYPAD_RIGHT, RETRO_DEVICE_ID_JOYPAD_SELECT,
    RETRO_DEVICE_ID_JOYPAD_START, RETRO_DEVICE_ID_JOYPAD_UP, RETRO_DEVICE_ID_JOYPAD_X,
    RETRO_DEVICE_ID_JOYPAD_Y,
};

/// Numatytasis gamepad mapping'as (fizinė pozicija → libretro joypad ID). `None` — mygtukas
/// sąmoningai neatvaizduojamas (pvz. `Mode`/gamintojo logotipo mygtukas, `C`/`Z` — retai kada
/// egzistuoja standartiniuose Xbox/DS/8BitDo valdikliuose).
///
/// P5.1 (SQLite schema) DAR NEEGZISTUOJA — „mapping'as saugomas DB" (MVP.md P4.2) atidėtas
/// (žr. P4.2 ADR-016 pastabą MVP.md faile); iki tol tai VIENINTELIS, hardkodintas mapping'as.
pub fn default_gamepad_mapping(button: Button) -> Option<u32> {
    match button {
        Button::South => Some(RETRO_DEVICE_ID_JOYPAD_B),
        Button::East => Some(RETRO_DEVICE_ID_JOYPAD_A),
        Button::West => Some(RETRO_DEVICE_ID_JOYPAD_Y),
        Button::North => Some(RETRO_DEVICE_ID_JOYPAD_X),
        Button::LeftTrigger => Some(RETRO_DEVICE_ID_JOYPAD_L),
        Button::LeftTrigger2 => Some(RETRO_DEVICE_ID_JOYPAD_L2),
        Button::RightTrigger => Some(RETRO_DEVICE_ID_JOYPAD_R),
        Button::RightTrigger2 => Some(RETRO_DEVICE_ID_JOYPAD_R2),
        Button::Select => Some(RETRO_DEVICE_ID_JOYPAD_SELECT),
        Button::Start => Some(RETRO_DEVICE_ID_JOYPAD_START),
        Button::LeftThumb => Some(RETRO_DEVICE_ID_JOYPAD_L3),
        Button::RightThumb => Some(RETRO_DEVICE_ID_JOYPAD_R3),
        Button::DPadUp => Some(RETRO_DEVICE_ID_JOYPAD_UP),
        Button::DPadDown => Some(RETRO_DEVICE_ID_JOYPAD_DOWN),
        Button::DPadLeft => Some(RETRO_DEVICE_ID_JOYPAD_LEFT),
        Button::DPadRight => Some(RETRO_DEVICE_ID_JOYPAD_RIGHT),
        Button::C | Button::Z | Button::Mode | Button::Unknown => None,
    }
}

/// D-pad kaip ANALOGINĖ ašis (`Axis::DPadX`/`Axis::DPadY`), NE atskiri `Button::DPad*` —
/// realiu hardware'u patikrinta (2026-08-26, Xbox Wireless Controller, macOS): šis valdiklis
/// D-pad'ą siunčia IŠIMTINAI kaip `AxisChanged` (švarios `-1.0`/`0.0`/`1.0` reikšmės, be
/// analoginio triukšmo — funkciškai skaitmeninis „hat switch", tiesiog kitu gilrs API keliu
/// nei `Button::DPad*`), NIEKADA kaip `ButtonChanged`. Be šios funkcijos toks valdiklis
/// turėtų VISIŠKAI neveikiantį D-pad'ą realiame žaidime (patvirtinta prieš pridedant šią
/// funkciją — `AxisChanged` anksčiau buvo sąmoningai ignoruojamas, žr. git istoriją).
///
/// Grąžina `(teigiamos_krypties_id, neigiamos_krypties_id)` — kviečiančioji pusė
/// (`nullbyte-emu` `drain_gamepad_events`) sprendžia, kurį bitą įjungti/išjungti pagal ašies
/// ženklą su `AXIS_DPAD_THRESHOLD` slenksčiu (skaitmeninis paversimas, NE analoginis judesio
/// jautrumas — RetroPad D-pad pats savaime skaitmeninis).
pub fn dpad_axis_ids(axis: Axis) -> Option<(u32, u32)> {
    match axis {
        Axis::DPadX => Some((RETRO_DEVICE_ID_JOYPAD_RIGHT, RETRO_DEVICE_ID_JOYPAD_LEFT)),
        // Empiriškai patikrinta reikšmių ženklas (NE prielaida): paspaudus D-pad VIRŠŲ šis
        // valdiklis siunčia `DPadY = +1.0`, APAČIĄ — `-1.0`. Tai priešinga standartinei
        // ekrano/analoginio stiko Y ašies konvencijai (kur +Y dažnai reiškia žemyn), tad
        // ženklas ČIA fiksuotas pagal REALIAI stebėtą elgesį, ne bendrą prielaidą.
        Axis::DPadY => Some((RETRO_DEVICE_ID_JOYPAD_UP, RETRO_DEVICE_ID_JOYPAD_DOWN)),
        _ => None,
    }
}

/// Slenkstis, virš kurio `dpad_axis_ids` ašies reikšmė laikoma „paspausta" — analoginio
/// triukšmo apsauga, nors realiu hardware'u pastebėtos reikšmės visada buvo švarios
/// `-1.0`/`0.0`/`1.0` (žr. `dpad_axis_ids` doc).
pub const AXIS_DPAD_THRESHOLD: f32 = 0.5;

/// Minimalus, windowing-biblioteka-agnostiškas klavišų rinkinys (MVP.md P4.2 „Klaviatūros
/// numatytieji: strėlės + Z/X/A/S + Enter/Shift"). `nullbyte-core` SĄMONINGAI nepriklauso nuo
/// `winit` (žr. `video::renderer` modulio doc — tas pats principas per `raw_window_handle`
/// abstrakciją) — `nullbyte-emu` konvertuoja `winit::keyboard::KeyCode` į šį enum'ą prieš
/// kviesdamas [`default_keyboard_mapping`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardKey {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    KeyZ,
    KeyX,
    KeyA,
    KeyS,
    Enter,
    ShiftRight,
}

/// Numatytasis klaviatūros mapping'as — tas pats fizinis-pozicija principas kaip
/// [`default_gamepad_mapping`]: `Z`/`X`/`A`/`S` atitinka apatinį/dešinį/kairį/viršutinį
/// veido mygtuką (standartinis RetroArch klaviatūros numatytasis mapping'as).
pub fn default_keyboard_mapping(key: KeyboardKey) -> Option<u32> {
    match key {
        KeyboardKey::ArrowUp => Some(RETRO_DEVICE_ID_JOYPAD_UP),
        KeyboardKey::ArrowDown => Some(RETRO_DEVICE_ID_JOYPAD_DOWN),
        KeyboardKey::ArrowLeft => Some(RETRO_DEVICE_ID_JOYPAD_LEFT),
        KeyboardKey::ArrowRight => Some(RETRO_DEVICE_ID_JOYPAD_RIGHT),
        KeyboardKey::KeyZ => Some(RETRO_DEVICE_ID_JOYPAD_B),
        KeyboardKey::KeyX => Some(RETRO_DEVICE_ID_JOYPAD_A),
        KeyboardKey::KeyA => Some(RETRO_DEVICE_ID_JOYPAD_Y),
        KeyboardKey::KeyS => Some(RETRO_DEVICE_ID_JOYPAD_X),
        KeyboardKey::Enter => Some(RETRO_DEVICE_ID_JOYPAD_START),
        KeyboardKey::ShiftRight => Some(RETRO_DEVICE_ID_JOYPAD_SELECT),
    }
}

/// `RETRO_DEVICE_ID_JOYPAD_*` (0..=15) → atitinkamas bitas `EmuContext.input_state`
/// bitmask'e (`core::callbacks` doc: „bitas N = RETRO_DEVICE_ID_JOYPAD_N"). Naudoja
/// `nullbyte-emu`, kad iš atskirų mygtukų paspaudimo/atleidimo įvykių sudarytų pilną `u16`
/// bitmask'ą, kurį reikia siųsti per [`crate::core::runner::EmuCommand::SetInput`] (jis
/// PAKEIČIA visą porto reikšmę, ne pavienį bitą — žr. `runner.rs` `SetInput` apdorojimą).
pub fn joypad_bit(id: u32) -> u16 {
    debug_assert!(
        id < 16,
        "RETRO_DEVICE_ID_JOYPAD_* turėtų būti 0..16, gauta {id}"
    );
    1u16 << id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamepad_face_buttons_respect_snes_layout_swap() {
        // MVP.md P4.2: fizinis APATINIS mygtukas (South — Xbox „A", DS4 „Cross") → SNES B,
        // NE SNES A — tai IR YRA visos šios pataisos esmė.
        assert_eq!(
            default_gamepad_mapping(Button::South),
            Some(RETRO_DEVICE_ID_JOYPAD_B)
        );
        assert_eq!(
            default_gamepad_mapping(Button::East),
            Some(RETRO_DEVICE_ID_JOYPAD_A)
        );
        assert_eq!(
            default_gamepad_mapping(Button::West),
            Some(RETRO_DEVICE_ID_JOYPAD_Y)
        );
        assert_eq!(
            default_gamepad_mapping(Button::North),
            Some(RETRO_DEVICE_ID_JOYPAD_X)
        );
    }

    #[test]
    fn gamepad_dpad_and_shoulders_map_directly() {
        assert_eq!(
            default_gamepad_mapping(Button::DPadUp),
            Some(RETRO_DEVICE_ID_JOYPAD_UP)
        );
        assert_eq!(
            default_gamepad_mapping(Button::DPadDown),
            Some(RETRO_DEVICE_ID_JOYPAD_DOWN)
        );
        assert_eq!(
            default_gamepad_mapping(Button::DPadLeft),
            Some(RETRO_DEVICE_ID_JOYPAD_LEFT)
        );
        assert_eq!(
            default_gamepad_mapping(Button::DPadRight),
            Some(RETRO_DEVICE_ID_JOYPAD_RIGHT)
        );
        assert_eq!(
            default_gamepad_mapping(Button::LeftTrigger),
            Some(RETRO_DEVICE_ID_JOYPAD_L)
        );
        assert_eq!(
            default_gamepad_mapping(Button::RightTrigger),
            Some(RETRO_DEVICE_ID_JOYPAD_R)
        );
    }

    #[test]
    fn gamepad_unmapped_buttons_return_none() {
        assert_eq!(default_gamepad_mapping(Button::Mode), None);
        assert_eq!(default_gamepad_mapping(Button::Unknown), None);
        assert_eq!(default_gamepad_mapping(Button::C), None);
        assert_eq!(default_gamepad_mapping(Button::Z), None);
    }

    #[test]
    fn keyboard_face_buttons_match_zxas_convention() {
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::KeyZ),
            Some(RETRO_DEVICE_ID_JOYPAD_B)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::KeyX),
            Some(RETRO_DEVICE_ID_JOYPAD_A)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::KeyA),
            Some(RETRO_DEVICE_ID_JOYPAD_Y)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::KeyS),
            Some(RETRO_DEVICE_ID_JOYPAD_X)
        );
    }

    #[test]
    fn keyboard_arrows_and_menu_keys_map_directly() {
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::ArrowUp),
            Some(RETRO_DEVICE_ID_JOYPAD_UP)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::ArrowDown),
            Some(RETRO_DEVICE_ID_JOYPAD_DOWN)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::ArrowLeft),
            Some(RETRO_DEVICE_ID_JOYPAD_LEFT)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::ArrowRight),
            Some(RETRO_DEVICE_ID_JOYPAD_RIGHT)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::Enter),
            Some(RETRO_DEVICE_ID_JOYPAD_START)
        );
        assert_eq!(
            default_keyboard_mapping(KeyboardKey::ShiftRight),
            Some(RETRO_DEVICE_ID_JOYPAD_SELECT)
        );
    }

    #[test]
    fn joypad_bit_matches_documented_bit_position() {
        assert_eq!(joypad_bit(RETRO_DEVICE_ID_JOYPAD_B), 0b1);
        assert_eq!(joypad_bit(RETRO_DEVICE_ID_JOYPAD_Y), 0b10);
        assert_eq!(joypad_bit(RETRO_DEVICE_ID_JOYPAD_R3), 1u16 << 15);
    }

    /// Realiu hardware'u patikrintas ženklas (2026-08-26, Xbox Wireless Controller, macOS) —
    /// žr. `dpad_axis_ids` doc. `DPadY = +1.0` → UP, `-1.0` → DOWN (priešinga standartinei
    /// ekrano Y konvencijai — TYČIA, ne klaida).
    #[test]
    fn dpad_axis_ids_match_empirically_observed_sign_convention() {
        assert_eq!(
            dpad_axis_ids(Axis::DPadX),
            Some((RETRO_DEVICE_ID_JOYPAD_RIGHT, RETRO_DEVICE_ID_JOYPAD_LEFT))
        );
        assert_eq!(
            dpad_axis_ids(Axis::DPadY),
            Some((RETRO_DEVICE_ID_JOYPAD_UP, RETRO_DEVICE_ID_JOYPAD_DOWN))
        );
    }

    #[test]
    fn non_dpad_axes_are_not_mapped() {
        assert_eq!(dpad_axis_ids(Axis::LeftStickX), None);
        assert_eq!(dpad_axis_ids(Axis::RightStickY), None);
    }
}
