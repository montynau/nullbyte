//! `nullbyte-app` IPC klientas — `nullbyte-emu` sidecar spawn'inimas per `tauri-plugin-shell`,
//! `EmuCommand` siuntimas, `EmuStatus` skaitymas (CLAUDE.md §3.4, ADR-016, MVP.md P4.0.3).
//!
//! ## `CommandEvent` receiver'is PRIVALO būti drenuojamas VISADA (KRITIŠKAI SVARBU)
//!
//! `tauri-plugin-shell`'io PATIES vidinis `CommandEvent` kanalas (`Command::spawn()`
//! implementacija, tauri-plugin-shell 2.3.5 šaltinis) turi talpą **1**. Jei `EmuClient`
//! naudotojas (UI komandos) nedelsdamas nedrenuoja `Receiver<CommandEvent>`, ne tik VAIKO
//! `StatusWriter` gija (žr. `nullbyte_core::ipc` modulio doc) — PATI `tauri-plugin-shell`
//! vidinė stdout skaitymo užduotis užsiblokuoja bandydama `tx.send(...).await` į pilną
//! kanalą, kas savo ruožtu reiškia, kad OS pipe nebedrenuojamas, o VAIKAS pradeda kabinti
//! ties `write()`. Backpressure grandinė gali prasidėti NE tik `kill -9` scenarijuje, bet
//! bet kada, kai UI tiesiog "nieko nedaro" su gaunamais įvykiais.
//!
//! Sprendimas: [`EmuClient::spawn`] IŠKART (dar prieš grąžindama valdymą caller'iui) paleidžia
//! `tauri::async_runtime::spawn`'inta foninę užduotį, kuri drenuoja `Receiver<CommandEvent>`
//! VISADA, nepriklausomai nuo to, ar UI kada nors paprašys `EmuStatus`. Šiame etape (P4.0.3)
//! kiekviena `EmuStatus` žinutė tik loginama — `app.emit(...)` UI event'ams yra P9.1 darbas,
//! kai frontend'as turės klausytoją (tas pats principas kaip P4.1 `gamepad-connection`).

#![allow(dead_code)] // prijungs P9.1 (žaidimo paleidimo Tauri komandų sluoksnis)

use tauri::async_runtime::Receiver;
use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use nullbyte_core::core::runner::EmuCommand;
use nullbyte_core::ipc::{EmuStatus, IpcHello, IPC_PROTOCOL_VERSION};

use crate::error::AppError;

/// Rankena į veikiantį `nullbyte-emu` sidecar procesą.
pub struct EmuClient {
    child: CommandChild,
}

impl EmuClient {
    /// Paleidžia `nullbyte-emu` sidecar'ą, atlieka protokolo versijos handshake'ą (žr.
    /// `nullbyte_core::ipc` modulio doc — pati pirma eilutė abiem kryptimis yra [`IpcHello`]),
    /// ir IŠKART paleidžia foninę VISADA-drenuojančią užduotį (žr. modulio doc).
    ///
    /// Handshake'as sąmoningai NUOSEKLUS (siunčiam SAVO Hello, laukiam VAIKO Hello), o
    /// foninė drenavimo užduotis paleidžiama TIK POTO — kad handshake'o klaidos (versijos
    /// nesutapimas, vaikas nepasileido) būtų grąžintos KAIP `Result`, ne tyliai praryjamos
    /// fono užduoties viduje.
    ///
    /// Generic per `R: tauri::Runtime`, NE fiksuota `Wry` — leidžia testams naudoti
    /// `tauri::test::MockRuntime` vietoj tikro `AppHandle`.
    ///
    /// `system_dir`/`save_dir` (`AppState::system_dir`/`saves_dir`, `crate::paths`) paduodami
    /// kaip CLI argumentai (`argv[1]`/`argv[2]`) — vaikas juos naudoja
    /// `GET_SYSTEM_DIRECTORY`/`GET_SAVE_DIRECTORY` core callback'ams (CLAUDE.md §8.3). Būtina:
    /// kai kurie core'ai (pvz. MAME) besąlygiškai dereferencina šią rodyklę, tad trūkstamas
    /// arba `NULL` kelias sukelia segfault, ne gražią klaidą — žr. `nullbyte_core::core::runner`
    /// `make_initial_context` doc.
    pub async fn spawn<R: tauri::Runtime>(
        app: &AppHandle<R>,
        system_dir: &std::path::Path,
        save_dir: &std::path::Path,
    ) -> Result<Self, AppError> {
        let sidecar = app
            .shell()
            .sidecar("nullbyte-emu")
            .map_err(|e| AppError::Other(format!("nepavyko paruošti nullbyte-emu sidecar: {e}")))?
            .args([
                system_dir.to_string_lossy().into_owned(),
                save_dir.to_string_lossy().into_owned(),
            ]);

        let (mut rx, mut child) = sidecar
            .spawn()
            .map_err(|e| AppError::Other(format!("nepavyko paleisti nullbyte-emu: {e}")))?;

        let mut hello_line = serde_json::to_string(&IpcHello::current())
            .expect("IpcHello serializacija neturėtų klysti (plokščias struct)");
        hello_line.push('\n');
        child
            .write(hello_line.as_bytes())
            .map_err(|e| AppError::Other(format!("nepavyko nusiųsti Hello vaikui: {e}")))?;

        let hello = Self::read_hello(&mut rx).await?;
        if !hello.is_compatible() {
            let _ = child.kill();
            return Err(AppError::Other(format!(
                "nullbyte-emu protokolo versija nesutampa (gauta {}, tikimasi {IPC_PROTOCOL_VERSION}) \
                 — sidecar binaras pasenęs? paleisk `pnpm run build:sidecar`",
                hello.protocol_version
            )));
        }
        tracing::info!(
            protocol_version = hello.protocol_version,
            "nullbyte-emu IPC handshake OK"
        );

        // KRITIŠKAI SVARBU: paleidžiama ČIA, iškart po sėkmingo handshake'o — žr. modulio doc.
        tauri::async_runtime::spawn(Self::drain_loop(rx));

        Ok(Self { child })
    }

    /// Laukia PIRMOS `CommandEvent::Stdout` eilutės ir parsina ją kaip [`IpcHello`] (žr.
    /// modulio doc — vaikas savo Hello parašo struktūriškai garantuotai kaip pačią pirmą
    /// eilutę, `StatusWriter::spawn()`, nullbyte-core). Bet koks kitas įvykis PRIEŠ tai
    /// (stderr log'ai, klaidos, netikėtas nutrūkimas) — apdorojamas atitinkamai, ne
    /// ignoruojamas tyliai.
    async fn read_hello(rx: &mut Receiver<CommandEvent>) -> Result<IpcHello, AppError> {
        loop {
            match rx.recv().await {
                Some(CommandEvent::Stdout(bytes)) => {
                    let line = String::from_utf8_lossy(&bytes);
                    return serde_json::from_str(&line).map_err(|e| {
                        AppError::Other(format!(
                            "pirma nullbyte-emu eilutė turėjo būti IpcHello, gauta kažkas kito \
                             ({e}): {line}"
                        ))
                    });
                }
                Some(CommandEvent::Stderr(bytes)) => {
                    tracing::debug!(
                        line = %String::from_utf8_lossy(&bytes),
                        "nullbyte-emu stderr (prieš handshake'ą)"
                    );
                }
                Some(CommandEvent::Error(error)) => {
                    return Err(AppError::Other(format!(
                        "nullbyte-emu IPC klaida prieš handshake'ą: {error}"
                    )));
                }
                Some(CommandEvent::Terminated(payload)) => {
                    return Err(AppError::Other(format!(
                        "nullbyte-emu užsidarė prieš atsiųsdamas Hello (kodas {:?})",
                        payload.code
                    )));
                }
                None => {
                    return Err(AppError::Other(
                        "nullbyte-emu stdout užsidarė prieš atsiųsdamas Hello".to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    /// VISADA veikianti drenavimo užduotis — žr. modulio doc KODĖL ji negali laukti UI
    /// veiksmo. Kiekviena `EmuStatus` eilutė šiame etape tik loginama.
    async fn drain_loop(mut rx: Receiver<CommandEvent>) {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(bytes) => {
                    let line = String::from_utf8_lossy(&bytes);
                    match serde_json::from_str::<EmuStatus>(&line) {
                        Ok(status) => tracing::info!(?status, "EmuStatus gautas iš nullbyte-emu"),
                        Err(error) => {
                            // P4.0.3 acceptance: serializacijos klaida NESULAUŽO proceso —
                            // praleidžiam šią vieną eilutę, gija tęsia darbą.
                            tracing::error!(
                                %error,
                                %line,
                                "EmuStatus parse klaida — eilutė praleista"
                            );
                        }
                    }
                }
                CommandEvent::Stderr(bytes) => {
                    tracing::debug!(line = %String::from_utf8_lossy(&bytes), "nullbyte-emu stderr");
                }
                CommandEvent::Error(error) => {
                    tracing::error!(%error, "nullbyte-emu IPC klaida");
                }
                CommandEvent::Terminated(payload) => {
                    tracing::info!(code = ?payload.code, "nullbyte-emu procesas baigėsi");
                    break;
                }
                _ => {}
            }
        }
        tracing::debug!("nullbyte-emu CommandEvent drenavimo užduotis baigė darbą");
    }

    /// Siunčia [`EmuCommand`] vaikui per stdin (viena NDJSON eilutė).
    pub fn send(&mut self, cmd: EmuCommand) -> Result<(), AppError> {
        let mut line = serde_json::to_string(&cmd)
            .map_err(|e| AppError::Other(format!("EmuCommand serializacijos klaida: {e}")))?;
        line.push('\n');
        self.child
            .write(line.as_bytes())
            .map_err(|e| AppError::Other(format!("nepavyko nusiųsti EmuCommand: {e}")))
    }

    /// Priverstinai nutraukia `nullbyte-emu` procesą. Naudoti TIK kraštutiniu atveju —
    /// normalus žaidimo sustabdymas vyksta per `send(EmuCommand::Stop)`, kuris NEBAIGIA
    /// paties `nullbyte-emu` proceso (tik einamą žaidimą — vartotojas gali įkelti kitą
    /// žaidimą tame pačiame lange, žr. `core::runner::run_loop`). Švarus VISO proceso
    /// išjungimas (stdin uždarymas → vaiko EOF → savaiminis `process::exit()`) — P4.0.4,
    /// dar neįgyvendinta; iki tol `CommandChild` numesti BE `kill()` paliktų vaiką kaip
    /// orphan procesą (`CommandChild` neturi `Drop`, kuris jį automatiškai nutrauktų —
    /// patikrinta prieš tauri-plugin-shell 2.3.5 šaltinį, ADR-016 sąmoningai NEpasitiki
    /// automatiniu vaiko mirimu, žr. CLAUDE.md §10).
    pub fn kill(self) -> Result<(), AppError> {
        self.child
            .kill()
            .map_err(|e| AppError::Other(format!("nepavyko nutraukti nullbyte-emu proceso: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Realus e2e testas — paleidžia TIKRĄ `nullbyte-emu` sidecar procesą per
    /// `tauri-plugin-shell`, atlieka handshake'ą, gauna `EmuStatus::Loaded` (P4.0.2 test
    /// hook'o auto-load), siunčia `Stop`. Patikrinta rankiniu būdu realiu `nullbyte-app`
    /// paleidimu prieš rašant šį testą — pilnas ciklas (Hello → Loaded → Stop → Stopped)
    /// praėjo per tikrą sidecar transportą.
    ///
    /// `#[ignore]`: `nullbyte-emu` sukuria TIKRĄ winit langą + wgpu Surface + cpal audio
    /// srautą (P4.0.2) — headless CI runner'yje (ypač `ubuntu-latest`, be X11/Wayland) tai
    /// gali nepavykti arba elgtis nenuspėjamai (ta pati priežastis kaip CLAUDE.md §10
    /// P2.3/P2.5/P3.1 Linux apribojimai). Taip pat reikalauja jau sugeneruoto sidecar
    /// binaro (`pnpm run build:sidecar`). Paleisti rankiniu būdu:
    /// `cargo test --package nullbyte-app emu_client_spawns_real_sidecar -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn emu_client_spawns_real_sidecar_and_completes_handshake() {
        tauri::async_runtime::block_on(async {
            let app = tauri::test::mock_builder()
                .plugin(tauri_plugin_shell::init())
                .build(tauri::test::mock_context(tauri::test::noop_assets()))
                .expect("mock Tauri app turėtų susikurti");

            let test_dir = std::env::temp_dir().join("nullbyte_emu_client_test");
            let mut client = EmuClient::spawn(
                &app.handle().clone(),
                &test_dir.join("system"),
                &test_dir.join("saves"),
            )
            .await
            .expect("EmuClient turėtų sėkmingai spawn'intis ir atlikti handshake'ą");

            client
                .send(EmuCommand::Stop)
                .expect("Stop komanda turėtų nusisiųsti");

            // Duodam drain_loop'ui laiko realiai gauti Stopped prieš testui pasibaigiant —
            // vien log'o matomumui (--nocapture), ne assertion'ui (drain_loop klaidų
            // pati nemeta, jos tik loginamos, žr. modulio doc).
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // `Stop` NEBAIGIA nullbyte-emu proceso (žr. `EmuClient::kill` doc) — testas
            // pats atsakingas išvalyti, kad nepaliktų orphan proceso (P4.0.4 švaraus
            // proceso gyvavimo ciklo dar nėra).
            client.kill().expect("kill() turėtų pavykti");
        });
    }
}
