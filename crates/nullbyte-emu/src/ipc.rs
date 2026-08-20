//! `nullbyte-emu` IPC serveris — stdin (`EmuCommand`) skaitymas (CLAUDE.md §3.4/§10,
//! MVP.md P4.0.3). Stdout rašymo pusė (`StatusWriter`/`StatusSender`, backpressure) gyvena
//! `nullbyte_core::ipc` — `core::runner::EmuThread` naudoja ją tiesiogiai emuliavimo gijoje,
//! tad ji negalėjo gyventi šiame crate'e (žr. `nullbyte_core::ipc` modulio doc dėl
//! priklausomybių krypties).
//!
//! Handshake: [`nullbyte_core::ipc::IpcHello`] PATI PIRMA eilutė ABIEM kryptimis. Tėvo pusė
//! (`StatusWriter::spawn`, `nullbyte_core::ipc`) savo `Hello` parašo struktūriškai
//! garantuotai; ČIA (skaitymo pusėje) validuojame TĖVO atsiųstą `Hello` PRIEŠ apdorodami bet
//! kokią `EmuCommand` eilutę — nesutapimas reiškia pasenusį sidecar binarą (žr. MVP.md P4.0.3
//! priešdarbio pastabą apie build grandinę).

use std::io::BufRead;
use std::sync::mpsc::Sender;

use nullbyte_core::core::runner::EmuCommand;
use nullbyte_core::ipc::{IpcHello, IPC_PROTOCOL_VERSION};

/// Skaito `reader` per `BufRead::lines()`. Pirma eilutė PRIVALO būti [`IpcHello`] — jei
/// trūksta arba nesutampa versija, gija iškart baigia darbą (klaida logina, kodėl). Kiekviena
/// TOLESNĖ eilutė parsinama kaip [`EmuCommand`] ir persiunčiama per `command_sender`
/// (klonuota `EmuThread` vidinio kanalo siuntėja — žr. `EmuThread::command_sender()`, NE
/// visa `EmuThread` rankena, kad šiai funkcijai nereikėtų `'static` nuorodos gyvenimo trukmės
/// galvosūkio).
///
/// Blogos `EmuCommand` eilutės NESULAUŽO proceso (P4.0.3 acceptance: „Serializacijos klaida
/// NESULAUŽO nei vieno proceso") — praleidžiamos su `tracing::error!`. Stdin `EOF` (tėvo
/// procesas nutrūko — P4.0.4) grąžina normaliai; caller'is atsakingas už tolimesnį shutdown.
pub fn run_command_reader<R: BufRead>(reader: R, command_sender: Sender<EmuCommand>) {
    let mut lines = reader.lines();

    match lines.next() {
        Some(Ok(line)) => match serde_json::from_str::<IpcHello>(&line) {
            Ok(hello) if hello.is_compatible() => {
                tracing::info!(
                    protocol_version = hello.protocol_version,
                    "IPC handshake OK"
                );
            }
            Ok(hello) => {
                tracing::error!(
                    got = hello.protocol_version,
                    expected = IPC_PROTOCOL_VERSION,
                    "IPC protokolo versija nesutampa — sidecar binaras pasenęs? \
                     paleisk `pnpm run build:sidecar`"
                );
                return;
            }
            Err(error) => {
                tracing::error!(
                    %error,
                    %line,
                    "pirma stdin eilutė turėjo būti IpcHello, gauta kažkas kito"
                );
                return;
            }
        },
        Some(Err(error)) => {
            tracing::error!(%error, "nepavyko perskaityti pirmos stdin eilutės");
            return;
        }
        None => {
            tracing::warn!("stdin EOF prieš gaunant IpcHello — tėvas nieko neatsiuntė");
            return;
        }
    }

    for line in lines {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                tracing::error!(%error, "stdin skaitymo klaida — skaitymo gija baigia darbą");
                return;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<EmuCommand>(&line) {
            Ok(cmd) => {
                if command_sender.send(cmd).is_err() {
                    tracing::error!("EmuThread nebeveikia — skaitymo gija baigia darbą");
                    return;
                }
            }
            Err(error) => {
                tracing::error!(%error, %line, "EmuCommand parse klaida — eilutė praleista");
            }
        }
    }

    tracing::info!("stdin EOF — tėvo procesas nutrūko");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::mpsc;

    fn hello_line() -> String {
        let mut s = serde_json::to_string(&IpcHello::current()).unwrap();
        s.push('\n');
        s
    }

    #[test]
    fn forwards_valid_commands_after_hello() {
        let mut input = hello_line();
        input.push_str(&serde_json::to_string(&EmuCommand::Run).unwrap());
        input.push('\n');
        input.push_str(&serde_json::to_string(&EmuCommand::Stop).unwrap());
        input.push('\n');

        let (tx, rx) = mpsc::channel();
        run_command_reader(Cursor::new(input), tx);

        let received: Vec<EmuCommand> = rx.try_iter().collect();
        assert_eq!(received.len(), 2);
        assert!(matches!(received[0], EmuCommand::Run));
        assert!(matches!(received[1], EmuCommand::Stop));
    }

    /// P4.0.3 acceptance: „Serializacijos klaida NESULAUŽO nei vieno proceso" — bloga
    /// eilutė turi būti praleista, ne sustabdyti visą skaitymo giją.
    #[test]
    fn bad_command_line_is_skipped_not_fatal() {
        let mut input = hello_line();
        input.push_str("{ šitas JSON sąmoningai sugadintas\n");
        input.push_str(&serde_json::to_string(&EmuCommand::Stop).unwrap());
        input.push('\n');

        let (tx, rx) = mpsc::channel();
        run_command_reader(Cursor::new(input), tx);

        let received: Vec<EmuCommand> = rx.try_iter().collect();
        assert_eq!(
            received.len(),
            1,
            "blogas JSON turėjo būti praleistas, geras — persiųstas toliau"
        );
        assert!(matches!(received[0], EmuCommand::Stop));
    }

    #[test]
    fn missing_hello_stops_before_any_command_processed() {
        let mut input = serde_json::to_string(&EmuCommand::Run).unwrap();
        input.push('\n');

        let (tx, rx) = mpsc::channel();
        run_command_reader(Cursor::new(input), tx);

        assert!(
            rx.try_recv().is_err(),
            "be Hello pirmoje eilutėje, jokia komanda neturėjo būti apdorota"
        );
    }

    #[test]
    fn incompatible_hello_version_stops_before_any_command_processed() {
        let mut input = serde_json::to_string(&IpcHello {
            protocol_version: IPC_PROTOCOL_VERSION + 1,
        })
        .unwrap();
        input.push('\n');
        input.push_str(&serde_json::to_string(&EmuCommand::Run).unwrap());
        input.push('\n');

        let (tx, rx) = mpsc::channel();
        run_command_reader(Cursor::new(input), tx);

        assert!(
            rx.try_recv().is_err(),
            "nesuderinama protokolo versija turėjo sustabdyti PRIEŠ apdorojant komandas"
        );
    }
}
