//! libretro sluoksnis — core'ų įkėlimas, FFI, callback'ai, emuliavimo gija (CLAUDE.md §3.1, §8).
//!
//! Užpildoma Fazėje 1 (P1.1–P1.7): `ffi`, `loader`, `info`, `callbacks`, `environment`,
//! `runner`, `savestate`.

pub mod callbacks;
pub mod environment;
pub mod ffi;
pub mod info;
pub mod loader;
pub mod runner;
pub mod savestate;
pub mod sram;

// Testų-tik pagalbinė medžiaga bendra `core::*` moduliams.
#[cfg(test)]
pub(crate) mod test_support {
    //! Bet kuris testas, kuris `dlopen`'ina tikrą libretro core'ą (`CoreHandle::load` arba
    //! `EmuThread::spawn` + `Load`), PRIVALO paimti šį užraktą prieš tai. Priežastis —
    //! CLAUDE.md §3.2 taisyklė #2: procese vienu metu gali būti įkeltas tik VIENAS core, nes
    //! kai kurie core'ai turi globalų (ne thread-local) būvį. Lygiagretus `cargo test` be šito
    //! užrakto realiai sukelia SIGSEGV, kai du testai vienu metu inicializuoja tą patį core'ą.
    use std::sync::Mutex;

    pub static CORE_LOAD_LOCK: Mutex<()> = Mutex::new(());

    pub fn lock_core_load() -> std::sync::MutexGuard<'static, ()> {
        CORE_LOAD_LOCK
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}
