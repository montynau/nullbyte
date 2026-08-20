//! cpal garso išvestis, resampling, dynamic rate control (CLAUDE.md §3.1, §8.6).
//!
//! Užpildoma Fazėje 3 (P3.1–P3.4): `output`, `ring`, `resampler`.

pub mod output;
pub mod resampler;
pub mod ring;
