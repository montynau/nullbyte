//! Tauri komandų sluoksnis — plonas, jokios verslo logikos (CLAUDE.md §3.1, §6.3).
//!
//! `emulator` (P2.3, Tauri langas be webview + lokalus `Renderer`/`EmuThread`) pašalintas
//! P4.0.3 metu — ADR-016 (P4.0.x) perkėlė vaizdą/garsą/emuliaciją į atskirą `nullbyte-emu`
//! vaiko procesą; realus žaidimo paleidimo srautas (per `crate::ipc::EmuClient`) — P9.1.
//! `settings` — dar neužpildyta, liks vėlesnei fazei.

pub mod input;
pub mod library;
pub mod scraper;
