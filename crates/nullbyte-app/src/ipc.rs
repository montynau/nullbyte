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

    /// Švariai išjungia VISĄ `nullbyte-emu` procesą (P4.0.4) — naudoti normaliu atveju
    /// (vartotojas uždaro emuliatoriaus langą/aplikaciją), NE tik einamo žaidimo sustabdymui
    /// (tam skirta `send(EmuCommand::Stop)`, žr. `kill()` doc).
    ///
    /// `CommandChild` (tauri-plugin-shell 2.3.5 šaltinis) neturi eksplicitinio "uždaryk tik
    /// stdin" metodo — bet `self.child` numetus ČIA (funkcijos pabaigoje, `self` sunaudotas
    /// per value), jo `stdin_writer: PipeWriter` lauko `Drop` uždaro OS pipe write-end'ą, KO
    /// vaikas nežudomas signalu, o pats pastebi kaip stdin `EOF`
    /// (`nullbyte_emu::ipc::run_command_reader`) ir švariai išsijungia: unload'ina core'ą
    /// (`EmuThread::Drop` → `Stop` → `join()`) ir tik tada `process::exit()`
    /// (`nullbyte_emu::main` `user_event` handler'is). Tai IR YRA tas pats Unix pipe EOF
    /// mechanizmas, kuris jau saugo nuo orphan'ų netikėto tėvo mirimo atveju (CLAUDE.md §10)
    /// — čia tiesiog sąmoningai suveikia normaliu, ne `kill -9` atveju.
    pub fn shutdown_gracefully(self) {
        drop(self.child);
    }

    /// Vaiko proceso PID — naudoja `shutdown_gracefully_lets_child_exit_within_a_few_seconds`
    /// testas (žr. žemiau), kad realiai patikrintų, jog procesas išnyko iš OS proceso
    /// lentelės, ne tik kad `EmuClient` numestas.
    pub fn pid(&self) -> u32 {
        self.child.pid()
    }

    /// Priverstinai nutraukia `nullbyte-emu` procesą. Naudoti TIK kraštutiniu atveju (vaikas
    /// nereaguoja į `shutdown_gracefully()` per protingą laiką) — normalus VISO proceso
    /// išjungimas turėtų vykti per `shutdown_gracefully()`, o einamo žaidimo (ne viso
    /// proceso) sustabdymas — per `send(EmuCommand::Stop)`, kuris NEBAIGIA paties
    /// `nullbyte-emu` proceso (vartotojas gali įkelti kitą žaidimą tame pačiame lange, žr.
    /// `core::runner::run_loop`).
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
            // pats atsakingas išvalyti. Nuo P4.0.4 tam skirta `shutdown_gracefully()`
            // (žr. atskirą `shutdown_gracefully_lets_child_exit_within_a_few_seconds` testą
            // žemiau dėl PILNO acceptance patikrinimo — čia tik švarus cleanup).
            client.shutdown_gracefully();
        });
    }

    /// P4.0.4 acceptance: „Normalus žaidimo uždarymas švariai sustabdo vaiką be „zombie"
    /// proceso" — realiai patikrina (ne vien skaitant kodą), kad `shutdown_gracefully()`
    /// (stdin uždarymas → vaiko EOF → `nullbyte_emu` `user_event` → `event_loop.exit()` →
    /// `process::exit()`) baigia PATĮ OS procesą, ne tik numeta `EmuClient` rankeną. Naudoja
    /// `kill -0 <pid>` (POSIX: siunčia signalą 0 — tikrina proceso egzistavimą, nieko
    /// nežudo) kaip nepriklausomą, ne-`EmuClient`-vidinį patikrinimo šaltinį.
    ///
    /// `#[ignore]` dėl tos pačios priežasties kaip aukščiau (TIKRAS winit langas/wgpu/cpal).
    /// Paleisti rankiniu būdu:
    /// `cargo test --package nullbyte-app shutdown_gracefully_lets_child_exit -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn shutdown_gracefully_lets_child_exit_within_a_few_seconds() {
        tauri::async_runtime::block_on(async {
            let app = tauri::test::mock_builder()
                .plugin(tauri_plugin_shell::init())
                .build(tauri::test::mock_context(tauri::test::noop_assets()))
                .expect("mock Tauri app turėtų susikurti");

            let test_dir = std::env::temp_dir().join("nullbyte_shutdown_test");
            let client = EmuClient::spawn(
                &app.handle().clone(),
                &test_dir.join("system"),
                &test_dir.join("saves"),
            )
            .await
            .expect("EmuClient turėtų sėkmingai spawn'intis ir atlikti handshake'ą");

            let pid = client.pid();
            client.shutdown_gracefully();

            let mut still_alive = true;
            for _ in 0..50 {
                let status = std::process::Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .status();
                still_alive = matches!(status, Ok(s) if s.success());
                if !still_alive {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            assert!(
                !still_alive,
                "nullbyte-emu (pid {pid}) turėjo savaime išsijungti per stdin EOF \
                 (shutdown_gracefully) per ~5s, bet vis dar veikia"
            );
        });
    }

    /// P4.0.4 acceptance: „Vaiko crash'as (dirbtinis panic core'e) nenumuša tėvo proceso" —
    /// simuliuoja crash'ą realiu `kill -9` ant VAIKO (ne ant `EmuClient`, kuris čia lieka
    /// gyvas ir NEŽINO apie mirtį, kol negauna `CommandEvent::Terminated`) ir patikrina, kad
    /// `drain_loop` (žr. modulio doc) tai apdoroja be panic'o — pati asercija yra tai, kad
    /// šis testas apskritai baigiasi normaliai (ne crash'ina TESTO procesą), plius papildomas
    /// patikrinimas, kad `send()` po vaiko mirties grąžina `Err`, ne panic'ina.
    ///
    /// `#[ignore]` dėl tos pačios priežasties kaip aukščiau. Paleisti rankiniu būdu:
    /// `cargo test --package nullbyte-app parent_survives_child_crash -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn parent_survives_child_crash() {
        tauri::async_runtime::block_on(async {
            let app = tauri::test::mock_builder()
                .plugin(tauri_plugin_shell::init())
                .build(tauri::test::mock_context(tauri::test::noop_assets()))
                .expect("mock Tauri app turėtų susikurti");

            let test_dir = std::env::temp_dir().join("nullbyte_crash_test");
            let mut client = EmuClient::spawn(
                &app.handle().clone(),
                &test_dir.join("system"),
                &test_dir.join("saves"),
            )
            .await
            .expect("EmuClient turėtų sėkmingai spawn'intis ir atlikti handshake'ą");

            // `kill -9` TIESIOGIAI vaikui — `EmuClient`/`drain_loop` apie tai sužino TIK per
            // ateinantį `CommandEvent::Terminated`, ne sinchroniškai, kaip realaus core'o
            // panic'o atveju (core'as pats crash'intų vaiko PROCESĄ, ne EmuThread).
            let pid = client.pid();
            std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .status()
                .expect("kill -9 komanda turėtų pavykti paleisti");

            // Duodam `drain_loop`'ui laiko realiai gauti `Terminated` — testas šitą tašką
            // pasiekęs be panic'o JAU yra pagrindinė asercija (žr. doc aukščiau).
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Tėvo pusė (šis testo procesas) turėtų likti visiškai sveika — tolimesni
            // `EmuClient` metodų kvietimai grąžina klaidą, NE panic'ina, nes stdin pipe jau
            // negyvas (vaikas nužudytas).
            let send_result = client.send(EmuCommand::Stop);
            assert!(
                send_result.is_err(),
                "send() po vaiko kill -9 turėjo grąžinti Err (negyvas pipe), ne pavykti tyliai"
            );
        });
    }
}
