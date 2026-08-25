//! Hotkey'ai (P4.4, MVP.md lentelė).
//!
//! Kaip ir [`super::mapping`], `nullbyte-core` sąmoningai nepriklauso nuo `winit` (žr. tos
//! pačios pastabos `video::renderer` modulyje) — [`HotkeyKey`] yra lokalus, windowing-
//! biblioteka-agnostiškas enum'as, į kurį `nullbyte-emu` konvertuoja
//! `winit::keyboard::KeyCode` prieš kviesdamas [`resolve_hotkey`].
//!
//! `Space` (laikant → fast-forward) SĄMONINGAI NEĮTRAUKTAS į [`resolve_hotkey`] — visi kiti
//! hotkey'ai yra „paspaudimas = vienas veiksmas" (trigger), o `Space` yra „laikymas = būvis"
//! (kaip žaidimo mygtukas), su ATSKIRU press/release apdorojimu. Maišyti abu modelius į vieną
//! funkciją būtų klaidinga abstrakcija — `nullbyte-emu` tvarko `Space` tiesiogiai.

/// Minimalus, `winit`-nepriklausomas klavišų rinkinys — tik tie, kurie realiai naudojami
/// MVP.md P4.4 hotkey lentelėje (be `Space`, žr. modulio doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyKey {
    F1,
    F2,
    F4,
    F5,
    F6,
    F7,
    F8,
    F11,
    Escape,
    KeyR,
}

/// Veiksmas, kurį turi atlikti `nullbyte-emu` — ARBA nusiųsti `EmuCommand` emuliavimo gijai
/// (dauguma atvejų), ARBA atlikti grynai lango lygmens veiksmą (`ToggleFullscreen`), kurio
/// `nullbyte-core` net neturi kaip reprezentuoti (jis nežino apie `winit::window::Window`).
///
/// `TogglePause`/`ExitFullscreenOrLibrary` NĖRA tiesiogiai `EmuCommand`, nes reikalauja
/// KVIEČIANČIOSIOS PUSĖS BŪVIO (ar šiuo metu pauzė/fullscreen), kurio `nullbyte-core`
/// mapping'o funkcija neturi ir neturėtų turėti — ji lieka gryna, be jokio mutable būvio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    TogglePause,
    /// Rezervuotas „greito išsaugojimo" slot'as (0) — atskiras nuo numeruotų F5-F8 slot'ų
    /// (1..=4), kad vartotojas netyčia neperrašytų pavadinto save'o vienu klavišu.
    QuickSave,
    QuickLoad,
    /// `1..=4` — MVP.md lentelės F5-F8.
    SaveStateSlot(u8),
    LoadStateSlot(u8),
    ToggleFullscreen,
    /// MVP.md: „išeiti iš fullscreen / grįžti į biblioteką". Bibliotekos lango DAR NĖRA (P7
    /// UI nepradėta) — `nullbyte-emu` šiuo metu tai interpretuoja TIK kaip „išeiti iš
    /// fullscreen, jei jame esame", antroji dalis bus P7 darbas.
    ExitFullscreenOrLibrary,
    Reset,
}

/// `shift`/`primary_modifier` — jau IŠSPRĘSTA platformos modifikatoriaus reikšmė
/// (`primary_modifier` = Cmd macOS, Ctrl Linux/Windows — sprendžia kviečiantysis, žr.
/// `nullbyte-emu` `main.rs`), kad ši funkcija liktų platformai neutrali.
pub fn resolve_hotkey(key: HotkeyKey, shift: bool, primary_modifier: bool) -> Option<HotkeyAction> {
    match key {
        HotkeyKey::F1 => Some(HotkeyAction::TogglePause),
        HotkeyKey::F2 => Some(HotkeyAction::QuickSave),
        HotkeyKey::F4 => Some(HotkeyAction::QuickLoad),
        HotkeyKey::F5 if shift => Some(HotkeyAction::LoadStateSlot(1)),
        HotkeyKey::F5 => Some(HotkeyAction::SaveStateSlot(1)),
        HotkeyKey::F6 if shift => Some(HotkeyAction::LoadStateSlot(2)),
        HotkeyKey::F6 => Some(HotkeyAction::SaveStateSlot(2)),
        HotkeyKey::F7 if shift => Some(HotkeyAction::LoadStateSlot(3)),
        HotkeyKey::F7 => Some(HotkeyAction::SaveStateSlot(3)),
        HotkeyKey::F8 if shift => Some(HotkeyAction::LoadStateSlot(4)),
        HotkeyKey::F8 => Some(HotkeyAction::SaveStateSlot(4)),
        HotkeyKey::F11 => Some(HotkeyAction::ToggleFullscreen),
        HotkeyKey::Escape => Some(HotkeyAction::ExitFullscreenOrLibrary),
        HotkeyKey::KeyR if primary_modifier => Some(HotkeyAction::Reset),
        HotkeyKey::KeyR => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_function_keys_resolve_to_documented_actions() {
        assert_eq!(
            resolve_hotkey(HotkeyKey::F1, false, false),
            Some(HotkeyAction::TogglePause)
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F2, false, false),
            Some(HotkeyAction::QuickSave)
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F4, false, false),
            Some(HotkeyAction::QuickLoad)
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F11, false, false),
            Some(HotkeyAction::ToggleFullscreen)
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::Escape, false, false),
            Some(HotkeyAction::ExitFullscreenOrLibrary)
        );
    }

    #[test]
    fn f5_through_f8_save_without_shift_load_with_shift() {
        assert_eq!(
            resolve_hotkey(HotkeyKey::F5, false, false),
            Some(HotkeyAction::SaveStateSlot(1))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F5, true, false),
            Some(HotkeyAction::LoadStateSlot(1))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F6, false, false),
            Some(HotkeyAction::SaveStateSlot(2))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F6, true, false),
            Some(HotkeyAction::LoadStateSlot(2))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F7, false, false),
            Some(HotkeyAction::SaveStateSlot(3))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F7, true, false),
            Some(HotkeyAction::LoadStateSlot(3))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F8, false, false),
            Some(HotkeyAction::SaveStateSlot(4))
        );
        assert_eq!(
            resolve_hotkey(HotkeyKey::F8, true, false),
            Some(HotkeyAction::LoadStateSlot(4))
        );
    }

    #[test]
    fn reset_requires_primary_modifier_not_bare_r() {
        assert_eq!(resolve_hotkey(HotkeyKey::KeyR, false, false), None);
        assert_eq!(
            resolve_hotkey(HotkeyKey::KeyR, false, true),
            Some(HotkeyAction::Reset)
        );
        // Shift+R (be primary modifier) taip pat neturėtų suveikti Reset — tik primary_modifier.
        assert_eq!(resolve_hotkey(HotkeyKey::KeyR, true, false), None);
    }
}
