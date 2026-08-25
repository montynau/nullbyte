//! ROM bibliotekos skenavimas, hash'avimas (CLAUDE.md §3.1, Faza 5).
//!
//! Archyvų skaitymas (`archive`) persikėlė į `nullbyte-core` (P4.0.1, ADR-016) — jo reikia
//! IR `core::loader` (ROM įkėlimui), IR čia; kadangi `nullbyte-core` negali priklausyti nuo
//! `nullbyte-app`, jis gyvena bendrame crate'e.

pub mod hasher;
pub mod scanner;
