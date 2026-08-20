//! Bendras branduolys tarp `nullbyte-emu` (vaiko procesas, vykdo emuliaciją) ir `nullbyte-app`
//! (Tauri tėvas, IPC tipų bendrinimui) — žr. CLAUDE.md §4, ADR-016.

pub mod archive;
pub mod audio;
pub mod core;
pub mod error;
pub mod input;
pub mod ipc;
pub mod video;
