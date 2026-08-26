//! SQLite duomenų bazė — migracijos, modeliai, žaidimų CRUD (CLAUDE.md §3.1, Faza 5).
//!
//! Užpildoma Fazėje 5 (P5.1–P5.4): `migrations`, `models`, `games`, `settings`.

pub mod games;
pub mod migrations;
pub mod models;
pub mod rom_directories;
pub mod save_states;
pub mod settings;
