//! ROM bibliotekos skenavimas, hash'avimas, archyvų skaitymas (CLAUDE.md §3.1, Faza 5).
//!
//! `archive` prijungtas anksčiau (P1.6), nes ROM įkėlimui reikia archyvų skaitymo jau
//! Fazėje 1. `scanner` ir `hasher` — Fazėje 5 (P5.2–P5.3).

pub mod archive;
