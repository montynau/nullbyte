//! `nullbyte-emu` ↔ `nullbyte-app` IPC protokolo bendri tipai (CLAUDE.md §3.4/§10, ADR-016,
//! MVP.md P4.0.3). Transportas: NDJSON per stdin (`EmuCommand`, tėvas → vaikas) / stdout
//! (`EmuStatus`, vaikas → tėvas) — žr. `crates/nullbyte-emu/src/ipc.rs` (serveris) ir
//! `crates/nullbyte-app/src/ipc.rs` (klientas) realiam skaitymo/rašymo loop'ui.
//!
//! ## Protokolo versijos handshake
//!
//! Pati PIRMA eilutė ABIEM kryptimis PRIVALO būti [`IpcHello`] (NE `EmuCommand`/`EmuStatus`)
//! — apsauga nuo pasenusio sidecar binaro (build grandinė paprastai jį perstato prieš
//! kiekvieną `pnpm tauri dev`/`build`, žr. MVP.md P4.0.3 priešdarbio pastabą, bet rizika
//! nenulinė — pvz. rankiniu būdu paleidus seną `target/debug/nullbyte-emu` tiesiogiai).
//! Be handshake'o toks neatitikimas pasireikštų kaip nesuprantamos NDJSON parse klaidos giliai
//! protokolo viduryje, ne kaip aiškus „versijos nesutampa" pranešimas iškart prisijungus.
//!
//! `IpcHello` yra SĄMONINGAI ATSKIRAS tipas, ne `EmuCommand`/`EmuStatus` variantas — protokolo
//! lygmuo ([]versija") ir žaidimo valdymo/būvio lygmuo yra skirtingi rūpesčiai, o vienas
//! bendras tipas abiem kryptimis (ne du beveik identiški) reiškia, kad negalima atsitiktinai
//! pakeisti tik vienos pusės handshake formos.
//!
//! ## Backpressure (KRITIŠKAI SVARBU — žr. CLAUDE.md §3.2 taisyklę #4)
//!
//! [`StatusWriter`]/[`StatusSender`] gyvena ČIA (ne `nullbyte-emu`), nes `core::runner::
//! EmuThread` (`nullbyte-core`) turi pats juos naudoti tiesiogiai emuliavimo gijoje — jei jie
//! gyventų `nullbyte-emu`, priklausomybių kryptis būtų atvirkščia (`nullbyte-core` negali
//! priklausyti nuo crate'o, kuris PATS priklauso nuo `nullbyte-core`). Backpressure logika taip
//! pat yra grynai protokolo lygmens rūpestis (kaip saugiai pristatyti `EmuStatus`), ne
//! proceso-specifinis — tinka bet kuriam `EmuStatus` gamintojui, ne tik `nullbyte-emu` main.rs.
//!
//! OS pipe tarp vaiko `stdout` ir tėvo skaitymo pusės turi RIBOTĄ buferį (macOS ~64 KB). Jei
//! tėvas laikinai nustoja drenuoti (UI užimtas, `CommandEvent` receiver'is nepollinamas —
//! žr. `crates/nullbyte-app/src/ipc.rs` pastabą), pipe užsipildo, ir bet koks `write()` į jį
//! BLOKUOJA rašančią giją. Jei tas `write()` vyktų TIESIOGIAI emuliavimo gijoje
//! (`core::runner::run_loop`) arba winit main gijoje, emuliatorius SUSTOTŲ kartu su juo —
//! audio underrun'ai, kritę kadrai. Simptomas atrodytų kaip „retkarčiais traška, bet negaliu
//! pakartoti" — nepakartojama, apkrovos priklausoma klaida.
//!
//! Sprendimas: [`StatusWriter`] — DEDIKUOTA gija, VIENINTELĖ, kuri kada nors liečia stdout
//! (arba bet kurį kitą `Write`, žr. `spawn`). Emu gija ir winit main gija niekada nerašo
//! tiesiogiai — jos gauna [`StatusSender`] rankeną, kuri žinutes deda į RIBOTĄ
//! (`mpsc::sync_channel`) kanalą. Kai kanalas pilnas:
//! - [`EmuStatus::Stats`] — TYLIAI numetama per [`StatusSender::send_stats`] (`try_send`,
//!   niekada neblokuoja) — pasenusi telemetrijos reikšmė nekenkia.
//! - `Loaded`/`Error`/`Stopped` — per [`StatusSender::send_important`] NIEKADA nemetami;
//!   siuntėjas PALAUKIA (blokuojantis `send`), kol writer gija atlaisvins vietą. Šie įvykiai
//!   reti (po vieną–kelis per visą sesiją), tad kanalo talpa praktiškai visada suteikia daug
//!   atsargos.
//!
//! [`StatusSender::send_stats`] papildomai THROTTLE'ina iki [`STATS_MIN_INTERVAL`] (2–4 Hz) —
//! be throttle'o, kas-kadrą siuntimas reikštų 60 JSON eilučių per sekundę amžinai, dėl rodmens,
//! į kurį dažniausiai niekas nežiūri.
//!
//! [`StatusWriter::spawn`] PATS parašo [`IpcHello`] kaip pačią pirmą eilutę, PRIEŠ perduodamas
//! `writer` foninei gijai — handshake'o siuntimas struktūriškai garantuotas, ne kažkas, ką
//! caller'is turi prisiminti padaryti pats.

use std::cell::Cell;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::core::loader::LoadedGameInfo;
use crate::error::CoreError;

/// Didinamas KIEKVIENĄ kartą, kai keičiasi `EmuCommand`/`EmuStatus`/`IpcHello` laido formatas
/// (nauja privaloma reikšmė, pašalintas variantas, pervardytas laukas) — ne kiekvieną kartą,
/// kai pridedamas naujas OPCIONALUS/atgal-suderinamas variantas.
///
/// `2` (P8.1, 2026-08-26): `EmuCommand::Load` gavo naują PRIVALOMĄ `states_dir: PathBuf`
/// lauką (žr. `EmuCommand::Load` doc) — tai TIKSLIAI atvejis, kurį ši versija turi pagauti
/// (senas sidecar'as, gavęs naują lauką kaip trūkstamą, nesuprastų žinutės formato).
pub const IPC_PROTOCOL_VERSION: u32 = 2;

/// Protokolo versijos handshake — pirma žinutė, kurią kiekviena pusė siunčia SAVO, ir pirma
/// žinutė, kurią kiekviena pusė TIKISI gauti iš kitos. Abi pusės naudoja TĄ PATĮ tipą (žr.
/// modulio doc), tad `serde` laukų pakeitimas automatiškai galioja abiem kryptimis vienu metu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpcHello {
    pub protocol_version: u32,
}

impl IpcHello {
    /// Handshake žinutė su ŠIO build'o protokolo versija — siunčiama kaip pati pirma eilutė.
    pub fn current() -> Self {
        Self {
            protocol_version: IPC_PROTOCOL_VERSION,
        }
    }

    /// `true`, jei gautas handshake atitinka ŠIO build'o protokolo versiją. Nesutapimas —
    /// nesuderinamas sidecar binaras (žr. modulio doc); caller'is turėtų nutraukti ryšį su
    /// aiškiu pranešimu, NE bandyti tęsti su likusiu protokolu.
    pub fn is_compatible(&self) -> bool {
        self.protocol_version == IPC_PROTOCOL_VERSION
    }
}

/// Būvio pranešimai, siunčiami iš `nullbyte-emu` (vaikas) į `nullbyte-app` (tėvas) per stdout
/// (NDJSON, viena žinutė per eilutę — žr. modulio doc). `Error` neša [`CoreError`]
/// STRUKTŪRIŠKAI (ne suplokštintą `{kind, message}` eilutę) — leidžia tėvui atkurti visus
/// laukus (`path`/`expected`/`actual`/`bios_file` ir pan.) ir teisingai apgaubti į
/// `AppError::Core`, kurio `kind()` deleguoja į `CoreError::kind()` (žr.
/// `nullbyte-app::error::AppError` doc). Suplokštinimas vyksta TIK ties Tauri → frontend riba,
/// ne čia — priešingu atveju P4.0.1 metu pridėti konkretūs `CoreError` variantai (CoreLoad/
/// ApiVersion/RomLoad/MissingBios/UnsupportedPixelFormat) taptų bevertė taksonomija, mirštanti
/// ties šia IPC riba (P9.1/P9.3 reikalauja UI galėti šakotis pagal konkretų klaidos tipą).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EmuStatus {
    /// `EmuCommand::Load` sėkmingai užbaigtas — core'as ir ROM'as įkelti.
    Loaded(LoadedGameInfo),
    /// Bet kokia `CoreError` — core'o įkėlimo, ROM'o įkėlimo, save state ir pan. klaida.
    /// Gija LIEKA gyva (žr. `core::runner::handle_load` doc) — šis pranešimas NEREIŠKIA
    /// proceso pabaigos, tik VIENOS operacijos nesėkmę.
    Error(CoreError),
    /// Periodinė sveikatos informacija (post-MVP HUD/diagnostikai). `audio_buffer_occupancy`
    /// — tas pats `[0.0, 1.0]` occupancy, kurį `core::runner` jau naudoja audio-driven pacing'ui
    /// viduje (CLAUDE.md §8.5/§8.6, `audio::ring::AudioConsumer::occupancy()`); kiti laukai
    /// (frame timing ir pan.) pridedami tada, kai atsiras realus vartotojas UI pusėje —
    /// NEsuprojektuoti iš anksto (žr. CLAUDE.md „Ko nedaryti" §11 dėl spekuliatyvių abstrakcijų).
    Stats { audio_buffer_occupancy: f64 },
    /// `EmuCommand::Stop` užbaigtas švariai — core'as atlaisvintas (`unload_game` →
    /// `deinit` → `drop(Library)`, žr. CLAUDE.md §8.2 žingsnis 14).
    Stopped,
    /// `EmuCommand::SaveState` sėkmingai užbaigtas (P8.1). Kelio ATGAL siųsti NEREIKIA —
    /// tėvas jį gali išvesti pats iš `states_dir` (siųsto per `EmuCommand::Load`, žr. jo doc)
    /// ir `slot` — tai tik patvirtinimas, kad operacija realiai pavyko.
    StateSaved { slot: u8 },
    /// `EmuCommand::LoadState` sėkmingai užbaigtas (P8.1).
    StateLoaded { slot: u8 },
}

/// Kanalo talpa — kelios sekundės `Stats` žinučių esant throttle'ui + vietos retiems
/// `Loaded`/`Error`/`Stopped`, kad jie praktiškai niekada nereikalautų blokuoti siuntėjo
/// (žr. modulio doc).
const STATUS_CHANNEL_CAPACITY: usize = 32;

/// Minimalus intervalas tarp dviejų `Stats` siuntimų iš TO PATIES [`StatusSender`] —
/// ~3.3 Hz, MVP.md P4.0.3 reikalauja 2–4 Hz (žr. modulio doc).
const STATS_MIN_INTERVAL: Duration = Duration::from_millis(300);

/// Rankena, kurią gauna KIEKVIENA gija, norinti siųsti `EmuStatus` (emuliavimo gija, winit
/// main gija). `Clone`, kad kiekviena gija galėtų turėti savo kopiją; `Send`, kad galėtų
/// kirsti gijos ribą. NĖRA `Sync` (`Cell` throttle būviui) — kiekviena kopija naudojama iš
/// VIENOS gijos vienu metu, lygiai taip pat, kaip `EmuThread`/`AudioOutput` rankenos šiame
/// kodo bazėje.
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

    /// Best-effort variantas `send_important()` — TIK teardown keliams (šiuo metu vienintelis
    /// naudotojas: `core::runner::run_loop` siunčia `EmuStatus::Stopped` PER ŠITĄ metodą, ne
    /// `send_important()`). Skirtumas nuo `send_important()` yra kritiškas P4.0.4 orphan
    /// apsaugai: jei tėvas nutrūksta netikėtai (pvz. `kill -9`) ir stdout laikinai/visam nebe
    /// drenuojamas, blokuojantis `send()` teardown metu reikštų, kad VAIKAS PATS pakimba
    /// vietoj to, kad išeitų — tiksliai priešingai P4.0.4 tikslui („vaiko fono gija gauna EOF
    /// → švariai išsijungia pati"). `Loaded`/`Error` per `send_important()` LIEKA blokuojantys
    /// sąmoningai — jie siunčiami normalaus veikimo metu, kai tėvas AKTYVIAI drenuoja, tad
    /// blokavimas ten realiai nekyla (žr. modulio doc).
    pub fn send_best_effort(&self, status: EmuStatus) {
        let _ = self.tx.try_send(status);
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
    /// Rašo [`IpcHello`] kaip pačią pirmą eilutę (žr. modulio doc), tada paleidžia dedikuotą
    /// giją tolimesnėms `EmuStatus` žinutėms. Grąžina `io::Error`, jei net handshake'o
    /// nepavyko parašyti (pvz. pipe jau uždarytas prieš startuojant).
    pub fn spawn<W: Write + Send + 'static>(
        mut writer: W,
    ) -> std::io::Result<(Self, StatusSender)> {
        let mut hello_line = serde_json::to_string(&IpcHello::current())
            .expect("IpcHello serializacija neturėtų klysti (plokščias struct)");
        hello_line.push('\n');
        // VIENAS write_all() kvietimas (NE `writeln!`, kuris gali sukelti DU atskirus
        // write_all'us — turinį ir „\n" atskirai, priklausomai nuo fmt::Arguments
        // fragmentacijos) — svarbu testams su ribotos talpos/blokuojančiais writer'iais
        // (žr. testų GatedWriter), kur kiekvienas write() reikalauja atskiro leidimo.
        writer.write_all(hello_line.as_bytes())?;
        writer.flush()?;

        let (tx, rx) = mpsc::sync_channel(STATUS_CHANNEL_CAPACITY);

        let handle = std::thread::Builder::new()
            .name("nullbyte-emu-status-writer".to_string())
            .spawn(move || run_writer_loop(writer, rx))
            .expect("nepavyko sukurti status writer gijos");

        Ok((
            Self {
                handle: Some(handle),
            },
            StatusSender {
                tx,
                // Pirmas send_stats() kvietimas turi praeiti iš karto, ne laukti pilno
                // STATS_MIN_INTERVAL nuo proceso starto.
                last_stats_sent: Cell::new(Instant::now() - STATS_MIN_INTERVAL),
            },
        ))
    }
}

impl Drop for StatusWriter {
    /// Laukia, kol writer gija baigs darbą — tai įvyksta, kai VISOS [`StatusSender`] kopijos
    /// numestos (kanalas atsijungia, `for status in rx` grąžina). Normaliame proceso
    /// gyvavimo cikle `nullbyte-emu` baigiasi per `process::exit()` (P4.0.4), tad šis
    /// `join()` realiai reikšmingas tik testams/švariam ne-exit keliui.
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

    #[test]
    fn hello_roundtrips_and_detects_mismatch() {
        let json = serde_json::to_string(&IpcHello::current()).unwrap();
        let parsed: IpcHello = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_compatible());

        let stale = IpcHello {
            protocol_version: IPC_PROTOCOL_VERSION + 1,
        };
        assert!(!stale.is_compatible());
    }

    #[test]
    fn emu_status_error_carries_structured_core_error() {
        let status = EmuStatus::Error(CoreError::ApiVersion {
            path: std::path::PathBuf::from("/cores/old.dylib"),
            expected: 1,
            actual: 2,
        });
        let json = serde_json::to_string(&status).unwrap();
        let restored: EmuStatus = serde_json::from_str(&json).unwrap();
        match restored {
            EmuStatus::Error(err) => {
                assert_eq!(err.kind(), "api_version");
            }
            other => panic!("tikėtasi EmuStatus::Error, gauta {other:?}"),
        }
    }

    #[test]
    fn emu_status_loaded_roundtrips() {
        let status = EmuStatus::Loaded(LoadedGameInfo {
            fps: 60.098,
            sample_rate: 32040.0,
            base_width: 256,
            base_height: 224,
            max_width: 256,
            max_height: 239,
            aspect_ratio: -1.0,
        });
        let json = serde_json::to_string(&status).unwrap();
        let restored: EmuStatus = serde_json::from_str(&json).unwrap();
        match restored {
            EmuStatus::Loaded(info) => assert!((info.fps - 60.098).abs() < 1e-9),
            other => panic!("tikėtasi EmuStatus::Loaded, gauta {other:?}"),
        }
    }

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

    /// Nuskaito visas eilutes, PATIKRINA pirmąją kaip `IpcHello` (žr. `StatusWriter::spawn`
    /// doc — struktūriškai garantuota pirma eilutė), grąžina likusias kaip `EmuStatus`.
    fn parse_hello_then_statuses(bytes: &[u8]) -> Vec<EmuStatus> {
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        let mut lines = text.lines();
        let hello: IpcHello = serde_json::from_str(lines.next().expect("bent Hello eilutė"))
            .expect("pirma eilutė turi būti validus IpcHello");
        assert!(hello.is_compatible());
        lines
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

        // spawn() PATS sinchroniškai parašo IpcHello prieš grąžindamas — duodam leidimą
        // TUOJ PAT, kitaip šis kvietimas blokuotų testo giją.
        let _ = permit_tx.send(());
        let (status_writer, sender) = StatusWriter::spawn(writer).expect("spawn turėtų pavykti");

        // Writer gija dabar kabo `permits.recv()` viduje, laukdama SEKANČIOS žinutės (po
        // Hello) — kanalas vis dar tuščias. Užpildome jį TIESIOGIAI per privatų `tx`
        // (apeinant throttle'ą) — testas tame pačiame modulyje, privatus laukas pasiekiamas.
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
        // NE griežta `sent <= CAPACITY` — writer gija gali suspėti nuskaityti (bet dar ne
        // parašyti, nes blokuoja ties `write()`) VIENĄ eilutę dar besipildant kanalui,
        // atlaisvindama vieną papildomą vietą; tada `sent` gali siekti CAPACITY+1.
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
        drop(status_writer); // Drop laukia, kol writer gija baigs darbą.

        let messages = parse_hello_then_statuses(&written.lock().unwrap());
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

    /// P4.0.4 orphan apsaugos scenarijus: tėvas nutrūksta netikėtai (`kill -9`), stdout
    /// NIEKADA daugiau nebedrenuojamas. `send_best_effort(Stopped)` teardown metu PRIVALO
    /// grąžinti valdymą IŠKART, ne blokuoti — kitaip vaikas pats pakibtų vietoj to, kad
    /// švariai išeitų. Writer'iui SĄMONINGAI neduodama NĖ VIENO leidimo (net Hello rašymui
    /// per `spawn()`, kuris irgi vyksta per tą patį `GatedWriter` — permits kanalas lieka
    /// TUŠČIAS visą testą), imituojant „niekas niekada daugiau neskaitys" scenarijų.
    #[test]
    fn stopped_via_best_effort_never_blocks_even_when_writer_never_drains() {
        // Konstruojame kanalą TIESIOGIAI, apeidami `StatusWriter::spawn()` — jis PATS
        // blokuotų amžinai savo sinchroniniame Hello write'e, jei writer niekada nedrenuotų
        // (žr. modulio doc). Testuojame TIK `send_best_effort()` elgesį: `rx` čia niekada
        // nekviečiamas `.recv()`/`.try_recv()`, imituojant „niekas niekada nebeskaitys".
        let (tx, _rx_kept_alive_but_never_drained) = mpsc::sync_channel(STATUS_CHANNEL_CAPACITY);
        let sender = StatusSender {
            tx,
            last_stats_sent: Cell::new(Instant::now() - STATS_MIN_INTERVAL),
        };
        // Užpildome kanalą IKI PAT viršaus — jokios vietos, jokio writer'io kita galą
        // nuskaitančio (rx niekada nekviečiamas .recv()/.try_recv()).
        for i in 0..STATUS_CHANNEL_CAPACITY {
            sender
                .tx
                .try_send(EmuStatus::Stats {
                    audio_buffer_occupancy: i as f64,
                })
                .expect("kanalas turėtų priimti lygiai iki talpos");
        }
        assert!(matches!(
            sender.tx.try_send(EmuStatus::Stats {
                audio_buffer_occupancy: 0.0
            }),
            Err(TrySendError::Full(_))
        ));

        // Pati esmė: šis kvietimas PRIVALO grąžinti valdymą iškart (žr. modulio doc) —
        // jei `send_best_effort` viduje netyčia naudotų blokuojantį `send()`, šis testas
        // pakibtų (kaip ir realus vaikas pakibtų P4.0.4 scenarijuje).
        sender.send_best_effort(EmuStatus::Stopped);
        // Pasiekus čia — įrodyta, kad kvietimas negrįžtamai nepakibo (žr. testo doc).
    }

    #[test]
    fn send_stats_throttles_rapid_calls() {
        let (permit_tx, permit_rx) = mpsc::channel::<()>();
        let written = Arc::new(Mutex::new(Vec::new()));
        let writer = GatedWriter {
            permits: permit_rx,
            written: written.clone(),
        };
        let _ = permit_tx.send(()); // leidžia sinchroninį Hello rašymą spawn() viduje.
        let (status_writer, sender) = StatusWriter::spawn(writer).expect("spawn turėtų pavykti");

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

        let messages = parse_hello_then_statuses(&written.lock().unwrap());
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
        let _ = permit_tx.send(()); // Hello.
        let (status_writer, sender) = StatusWriter::spawn(writer).expect("spawn turėtų pavykti");

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

        assert_eq!(
            parse_hello_then_statuses(&written.lock().unwrap()).len(),
            50
        );
    }
}
