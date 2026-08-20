//! `nullbyte-emu` IPC serveris — stdout (`EmuStatus`) rašymas (CLAUDE.md §3.4/§10,
//! MVP.md P4.0.3). Stdin skaitymo pusė (`EmuCommand` parsinimas, EOF → shutdown, P4.0.4)
//! — atskiras, dar neparašytas žingsnis; šis failas kol kas apima TIK rašymo pusę.
//!
//! ## Backpressure (KRITIŠKAI SVARBU — žr. CLAUDE.md §3.2 taisyklę #4)
//!
//! OS pipe tarp vaiko `stdout` ir tėvo skaitymo pusės turi RIBOTĄ buferį (macOS ~64 KB).
//! Jei tėvas laikinai nustoja drenuoti (UI užimtas, `CommandEvent` receiver'is
//! nepollinamas — žr. `crates/nullbyte-app/src/ipc.rs` pastabą), pipe užsipildo, ir bet
//! koks `write()` į jį BLOKUOJA rašančią giją, kol tėvas vėl pradės skaityti. Jei tas
//! `write()` vyktų TIESIOGIAI emuliavimo gijoje (`core::runner::run_loop`) arba winit main
//! gijoje (`about_to_wait()`), emuliatorius SUSTOTŲ kartu su juo — audio underrun'ai,
//! kritę kadrai. Simptomas atrodytų kaip „retkarčiais traška, bet negaliu pakartoti" —
//! nepakartojama, tik apkrovos priklausoma klaida, nes priežastis (tėvo pusės UI
//! užimtumas) niekaip nesusijusi su emuliacijos kodu, kur simptomas pasireiškia.
//!
//! Sprendimas: [`StatusWriter`] — DEDIKUOTA gija, VIENINTELĖ, kuri kada nors liečia
//! stdout. ĖMU gija ir winit main gija niekada nerašo tiesiogiai — jos gauna
//! [`StatusSender`] rankeną, kuri žinutes deda į RIBOTĄ (bounded, `mpsc::sync_channel`)
//! kanalą. Kai kanalas pilnas:
//! - [`EmuStatus::Stats`] — TYLIAI numetama per [`StatusSender::send_stats`]
//!   (`try_send`, niekada neblokuoja) — pasenusi telemetrijos reikšmė nekenkia.
//! - `Loaded`/`Error`/`Stopped` — per [`StatusSender::send_important`] NIEKADA nemetami;
//!   siuntėjas PALAUKIA (blokuojantis `send`), kol writer gija atlaisvins vietą. Šie
//!   įvykiai reti (po vieną–kelis per visą sesiją, ne kas kadrą), tad kanalo talpa
//!   (žr. [`STATUS_CHANNEL_CAPACITY`]) praktiškai visada suteikia daug atsargos — realiam
//!   blokavimui reikėtų, kad tėvas ignoruotų stdout MINUTES, ne sekundes.
//!
//! [`StatusSender::send_stats`] papildomai THROTTLE'ina iki [`STATS_MIN_INTERVAL`]
//! (2–4 Hz) — be throttle'o, kas-kadrą siuntimas reikštų 60 JSON eilučių per sekundę
//! amžinai, dėl rodmens, į kurį dažniausiai niekas nežiūri.
//!
//! Stdin skaitymo pusė (kito žingsnio darbas) prijungs `StatusSender` prie `EmuThread`
//! (per `core::runner`) ir winit `App` — kol to nėra, `main.rs` šio modulio dar nenaudoja.

#![allow(dead_code)] // prijungs stdin skaitymo pusė / EmuThread integracija (kitas žingsnis)

use std::cell::Cell;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use nullbyte_core::ipc::EmuStatus;

/// Kanalo talpa — kelios sekundės `Stats` žinučių esant throttle'ui + vietos retiems
/// `Loaded`/`Error`/`Stopped`, kad jie praktiškai niekada nereikalautų blokuoti siuntėjo
/// (žr. modulio doc).
const STATUS_CHANNEL_CAPACITY: usize = 32;

/// Minimalus intervalas tarp dviejų `Stats` siuntimų iš TO PATIES [`StatusSender`] —
/// ~3.3 Hz, MVP.md P4.0.3 reikalauja 2–4 Hz (žr. modulio doc).
const STATS_MIN_INTERVAL: Duration = Duration::from_millis(300);

/// Rankena, kurią gauna KIEKVIENA gija, norinti siųsti `EmuStatus` (emuliavimo gija, winit
/// main gija). `Clone`, kad kiekviena gija galėtų turėti savo kopiją; `Send`, kad
/// galėtų kirsti gijos ribą persiunčiant į `EmuThread::spawn`-stiliaus closure'ą. NĖRA
/// `Sync` (`Cell` throttle būviui) — kiekviena kopija naudojama iš VIENOS gijos vienu metu,
/// lygiai taip pat, kaip `EmuThread`/`AudioOutput` rankenos šiame kodo bazėje.
#[derive(Clone)]
pub struct StatusSender {
    tx: SyncSender<EmuStatus>,
    last_stats_sent: Cell<Instant>,
}

impl StatusSender {
    /// `Loaded`/`Error`/`Stopped` — PRIVALO pasiekti tėvą, žr. modulio doc dėl kodėl
    /// blokuojantis `send` čia saugus. Kanalo atsijungimas (writer gija baigė darbą, pvz.
    /// stdout uždarytas) tyliai ignoruojamas — nėra ko daugiau daryti šiame taške.
    pub fn send_important(&self, status: EmuStatus) {
        debug_assert!(
            !matches!(status, EmuStatus::Stats { .. }),
            "send_important() negauna Stats — naudok send_stats() (throttle + drop-on-full)"
        );
        let _ = self.tx.send(status);
    }

    /// Telemetrija — throttled iki [`STATS_MIN_INTERVAL`], TYLIAI numetama, jei kanalas
    /// pilnas. NIEKADA neblokuoja siuntėjo (žr. modulio doc).
    pub fn send_stats(&self, audio_buffer_occupancy: f64) {
        if self.last_stats_sent.get().elapsed() < STATS_MIN_INTERVAL {
            return;
        }
        self.last_stats_sent.set(Instant::now());

        match self.tx.try_send(EmuStatus::Stats {
            audio_buffer_occupancy,
        }) {
            Ok(()) | Err(TrySendError::Disconnected(_)) => {}
            Err(TrySendError::Full(_)) => {
                tracing::debug!(
                    "EmuStatus::Stats numesta — kanalas pilnas (tėvas laikinai nedrenuoja)"
                );
            }
        }
    }
}

/// Dedikuota rašymo gija (žr. modulio doc). `spawn` generic per `Write + Send + 'static`,
/// kad testai galėtų paduoti kontroliuojamą fake writer'į vietoj tikro `stdout`.
pub struct StatusWriter {
    handle: Option<JoinHandle<()>>,
}

impl StatusWriter {
    pub fn spawn<W: Write + Send + 'static>(writer: W) -> (Self, StatusSender) {
        let (tx, rx) = mpsc::sync_channel(STATUS_CHANNEL_CAPACITY);

        let handle = std::thread::Builder::new()
            .name("nullbyte-emu-status-writer".to_string())
            .spawn(move || run_writer_loop(writer, rx))
            .expect("nepavyko sukurti status writer gijos");

        (
            Self {
                handle: Some(handle),
            },
            StatusSender {
                tx,
                // Pirmas send_stats() kvietimas turi praeiti iš karto, ne laukti pilno
                // STATS_MIN_INTERVAL nuo proceso starto.
                last_stats_sent: Cell::new(Instant::now() - STATS_MIN_INTERVAL),
            },
        )
    }
}

impl Drop for StatusWriter {
    /// Laukia, kol writer gija baigs darbą — tai įvyksta, kai VISOS [`StatusSender`]
    /// kopijos numestos (kanalas atsijungia, `for status in rx` grąžina). Normaliame
    /// proceso gyvavimo cikle `nullbyte-emu` baigiasi per `process::exit()` (P4.0.4), tad
    /// šis `join()` realiai reikšmingas tik testams/švariam ne-exit keliui.
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_writer_loop<W: Write>(mut writer: W, rx: Receiver<EmuStatus>) {
    for status in rx {
        let mut line = match serde_json::to_string(&status) {
            Ok(line) => line,
            Err(error) => {
                // Serializacijos klaida NESULAUŽO proceso (P4.0.3 acceptance) — praleidžiam
                // šią vieną žinutę, gija tęsia darbą.
                tracing::error!(%error, "EmuStatus serializacijos klaida — eilutė praleista");
                continue;
            }
        };
        line.push('\n');

        if let Err(error) = writer.write_all(line.as_bytes()) {
            tracing::error!(%error, "nepavyko rašyti EmuStatus į stdout — writer gija baigia darbą");
            return;
        }
        if let Err(error) = writer.flush() {
            tracing::error!(%error, "nepavyko flush'inti stdout");
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Testinis writer'is, kuris NEDRENUOJA (blokuoja `write`), kol testas eksplicitiškai
    /// neatsiunčia leidimo per `permits` — imituoja tėvą, kuris nustojo skaityti stdout
    /// (žr. modulio doc dėl pipe backpressure).
    struct GatedWriter {
        permits: Receiver<()>,
        written: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for GatedWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            // Blokuoja, kol testas duoda leidimą — imituoja pilną OS pipe.
            let _ = self.permits.recv();
            self.written.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn parse_lines(bytes: &[u8]) -> Vec<EmuStatus> {
        String::from_utf8(bytes.to_vec())
            .unwrap()
            .lines()
            .map(|line| {
                serde_json::from_str(line).expect("kiekviena eilutė turi būti validus JSON")
            })
            .collect()
    }

    #[test]
    fn stats_dropped_under_backpressure_but_important_always_delivered() {
        let (permit_tx, permit_rx) = mpsc::channel::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let writer = GatedWriter {
            permits: permit_rx,
            written: written.clone(),
        };

        let (status_writer, sender) = StatusWriter::spawn(writer);

        // Writer gija dabar kabo `permits.recv()` viduje, laukdama PIRMOS žinutės — kanalas
        // vis dar tuščias. Užpildome jį TIESIOGIAI per privatų `tx` (apeinant throttle'ą,
        // kuris kitaip leistų tik vieną send_stats() kvietimą iš karto po kito) — testas
        // yra tame pačiame modulyje, tad privatus laukas pasiekiamas.
        let mut sent = 0;
        let mut dropped = 0;
        for i in 0..(STATUS_CHANNEL_CAPACITY * 3) {
            match sender.tx.try_send(EmuStatus::Stats {
                audio_buffer_occupancy: i as f64,
            }) {
                Ok(()) => sent += 1,
                Err(TrySendError::Full(_)) => dropped += 1,
                Err(TrySendError::Disconnected(_)) => panic!("kanalas neturėtų būti uždarytas"),
            }
        }
        assert!(
            dropped > 0,
            "bent dalis Stats turėjo būti numesta, kai kanalas pilnas (sent={sent})"
        );
        // NE `sent <= STATUS_CHANNEL_CAPACITY` — writer gija gali suspėti nuskaityti (bet
        // dar ne užrašyti, nes blokuoja ties pirmu `write()`) VIENĄ eilutę dar besipildant
        // kanalui, atlaisvindama vieną papildomą vietą; tada `sent` gali siekti CAPACITY+1.
        // Tai NĖRA klaida — writer'io pusėje realiai parašomų eilučių skaičius (tikrinama
        // žemiau) vis tiek tiksliai atitinka tai, kas pateko į kanalą.
        assert!(sent <= STATUS_CHANNEL_CAPACITY + 1, "sent={sent}");

        // Kritinis įvykis PO to, kai kanalas jau pilnas Stats — send_important() blokuoja,
        // tad siunčiam iš atskiros gijos, kad testas pats neužstrigtų.
        let sender_for_thread = sender.clone();
        let important_thread = std::thread::spawn(move || {
            sender_for_thread.send_important(EmuStatus::Stopped);
        });

        // Dabar leidžiam writer gijai drenuoti VISKĄ — pakanka leidimų kiekvienam
        // pasiektam elementui (Stats, kurie realiai pateko į kanalą, + Stopped).
        for _ in 0..(sent + 1) {
            let _ = permit_tx.send(());
        }

        important_thread.join().unwrap();
        drop(sender);
        drop(status_writer); // Drop laukia, kol writer gija baigs darbą (kanalas tuščias+uždarytas).

        let messages = parse_lines(&written.lock().unwrap());
        assert_eq!(
            messages.len(),
            sent + 1,
            "turėjo būti parašyta lygiai tiek, kiek pateko į kanalą, plius Stopped"
        );
        assert!(
            matches!(messages.last(), Some(EmuStatus::Stopped)),
            "Stopped turėjo pasiekti writer'į NEPRIKLAUSOMAI nuo numestų Stats, gauta: {:?}",
            messages.last()
        );
    }

    #[test]
    fn send_stats_throttles_rapid_calls() {
        let (permit_tx, permit_rx) = mpsc::channel::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let writer = GatedWriter {
            permits: permit_rx,
            written: written.clone(),
        };
        let (status_writer, sender) = StatusWriter::spawn(writer);

        // Iškart po sukūrimo pirmas kvietimas turėtų praeiti (žr. StatusWriter::spawn).
        sender.send_stats(1.0);
        // Antras, tuoj pat po pirmo, turėtų būti throttle'intas (praleistas), NE numestas
        // dėl pilno kanalo — kanalas beveik tuščias šiame taške.
        sender.send_stats(2.0);
        sender.send_stats(3.0);

        drop(sender);
        // Vienas leidimas — tiek žinučių turėtų realiai pasiekti kanalą.
        let _ = permit_tx.send(());
        drop(status_writer);

        let messages = parse_lines(&written.lock().unwrap());
        assert_eq!(
            messages.len(),
            1,
            "throttle turėjo praleisti antrą/trečią kvietimą, gauta: {messages:?}"
        );
    }

    #[test]
    fn serialization_never_panics_and_writer_thread_survives_many_messages() {
        let (permit_tx, permit_rx) = mpsc::channel::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let writer = GatedWriter {
            permits: permit_rx,
            written: written.clone(),
        };
        let (status_writer, sender) = StatusWriter::spawn(writer);

        std::thread::spawn(move || {
            for _ in 0..50 {
                let _ = permit_tx.send(());
            }
        });

        for _ in 0..50 {
            sender.send_important(EmuStatus::Stopped);
        }
        drop(sender);
        drop(status_writer);

        assert_eq!(parse_lines(&written.lock().unwrap()).len(), 50);
    }
}
