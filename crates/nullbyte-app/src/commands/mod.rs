//! Tauri komandų sluoksnis — plonas, jokios verslo logikos (CLAUDE.md §3.1, §6.3).
//!
//! Senasis `emulator` (P2.3, Tauri langas be webview + lokalus `Renderer`/`EmuThread`)
//! pašalintas P4.0.3 metu — ADR-016 (P4.0.x) perkėlė vaizdą/garsą/emuliaciją į atskirą
//! `nullbyte-emu` vaiko procesą. Nuo P9.1 `emulator` modulis SUGRĮŽO nauju pavidalu —
//! `start_game`/`stop_game`/`is_game_running`, orkestruojantys `crate::ipc::EmuClient` (žr.
//! jo doc).
//! `settings` — nuo P7.6 Cores panelės turi `list_cores`/`get_preferred_cores`/
//! `set_preferred_cores` (nuo P9.1 realiai naudojama `start_game` core'o pasirinkimui per
//! `resolve_preferred_core_path`); video/audio nustatymų PRITAIKYMAS realiam žaidimui vis
//! dar tik persistencija — reikalauja NAUJŲ `EmuCommand` variantų, kurių dar nėra (atskiras
//! darbas nuo paties paleidimo srauto, žr. `settings::VideoSettings`/`AudioSettings` doc).

pub mod emulator;
pub mod input;
pub mod library;
pub mod savestate;
pub mod scraper;
pub mod settings;
