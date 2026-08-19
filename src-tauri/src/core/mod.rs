//! libretro sluoksnis — core'ų įkėlimas, FFI, callback'ai, emuliavimo gija (CLAUDE.md §3.1, §8).
//!
//! Užpildoma Fazėje 1 (P1.1–P1.7): `ffi`, `loader`, `info`, `callbacks`, `environment`,
//! `runner`, `savestate`.

pub mod ffi;
pub mod loader;
