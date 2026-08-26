//! Žaidimo paleidimo Tauri komandos (CLAUDE.md §6.3, MVP.md P9.1) — vienintelė vieta, kur
//! realiai sujungiami core'o pasirinkimas (P7.6 Nustatymai), žaidimo įrašas (P5.4 DB), ir
//! `nullbyte-emu` vaiko procesas (`crate::ipc::EmuClient`, P4.0.3).
//!
//! ADR-016: vienam žaidimo paleidimui — vienas vaiko procesas, savo langas/audio/gamepad'ai.
//! Šis modulis palaiko TIK VIENĄ veikiančią sesiją vienu metu (`AppState::emu_session`) —
//! antras `start_game()` kvietimas, kol pirmas dar veikia, grąžina aiškią klaidą (P9.3
//! filosofija — „aiškus pranešimas", ne tylus senos sesijos nutraukimas ar pakeitimas).
//! Kelių vienu metu veikiančių žaidimų palaikymas MVP.md apimtyje NENUMATYTAS.

use std::path::PathBuf;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, State};

use nullbyte_core::core::runner::EmuCommand;
use nullbyte_core::ipc::EmuStatus;

use crate::commands::settings::resolve_preferred_core_path;
use crate::db::games;
use crate::error::AppError;
use crate::ipc::EmuClient;
use crate::paths;
use crate::state::AppState;

/// UI „Žaisti" mygtukas — parenka core'ą (per Nustatymų → Cores preferenciją), paleidžia
/// `nullbyte-emu` vaiko procesą, siunčia `Load`+`Run`, ir LAUKIA REALAUS rezultato
/// (`EmuStatus::Loaded`/`Error`, ne tik to, kad komandos nusiuntimas per stdin pavyko — žr.
/// `crate::ipc` modulio doc #1) prieš grąžindama valdymą UI. Tai leidžia P9.1 acceptance
/// „Trūkstamas core → suprantamas pranešimas" REALIAI veikti klaidoms, kurios paaiškėja
/// TIK core'ui bandant įkelti ROM'ą (trūkstamas BIOS, blogas ROM formatas) — ne tik
/// akivaizdžioms spawn-metu klaidoms (trūkstamas sidecar binaras, blogas core'o failas
/// prieš pat tai gali sukelti ir spawn-metu klaidą per `EmuClient::spawn`, ir load-metu
/// klaidą per šį oneshot'ą — abu keliai grąžina `Err`, tiesiog skirtingu tašku).
#[tauri::command]
pub async fn start_game(app: AppHandle, id: i64) -> Result<(), AppError> {
    {
        let state = app.state::<AppState>();
        let session = state.emu_session.lock().expect("Mutex poisoned");
        if session.is_some() {
            return Err(AppError::Other(
                "žaidimas jau paleistas — pirma uždarykite jį, tada bandykite dar kartą"
                    .to_string(),
            ));
        }
    }

    let (rom_path, core_path, states_dir, sram_path, system_dir, save_dir) = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().expect("Mutex poisoned");
        let game = games::get_game(&conn, id)?
            .ok_or_else(|| AppError::Other(format!("žaidimas #{id} nerastas")))?;
        let platform_slug = games::get_platform_slug(&conn, game.platform_id)?
            .ok_or_else(|| AppError::Other(format!("žaidimo #{id} platforma nerasta DB")))?;
        let core_path = resolve_preferred_core_path(&conn, &platform_slug)?.ok_or_else(|| {
            AppError::Other(format!(
                "nėra pasirinkto core'o platformai „{platform_slug}\" — nueikite į Nustatymus \
                 → Cores ir pasirinkite core'ą šiai platformai"
            ))
        })?;
        (
            PathBuf::from(game.rom_path),
            PathBuf::from(core_path),
            paths::game_states_dir(id)?,
            paths::game_sram_path(id)?,
            state.system_dir.clone(),
            state.saves_dir.clone(),
        )
    };

    let started_at = Instant::now();
    let app_on_close = app.clone();
    let (mut client, load_rx) = EmuClient::spawn(&app, &system_dir, &save_dir, move || {
        // Kviečiama TIKSLIAI VIENĄ kartą, kai vaiko procesas PILNAI baigia darbą — žr.
        // `crate::ipc` modulio doc #3. `played_seconds` skaičiuojamas nuo `start_game`
        // pradžios, NE nuo `EmuStatus::Loaded` gavimo — spawn+handshake+core init laikas
        // (paprastai < 1s) čia neaktualus.
        let played_seconds = started_at.elapsed().as_secs() as i64;
        let state = app_on_close.state::<AppState>();
        *state.emu_session.lock().expect("Mutex poisoned") = None;
        if let Ok(conn) = state.db.lock() {
            if let Err(error) = games::record_play(&conn, id, played_seconds) {
                tracing::error!(%error, id, "nepavyko įrašyti žaidimo laiko uždarius");
            }
        }
        if let Err(error) = app_on_close.emit("game-closed", id) {
            tracing::warn!(%error, "nepavyko išsiųsti game-closed event'o");
        }
    })
    .await?;

    if let Err(error) = client.send(EmuCommand::Load {
        core: core_path,
        rom: rom_path,
        states_dir,
        sram_path,
    }) {
        client.shutdown_gracefully();
        return Err(error);
    }
    if let Err(error) = client.send(EmuCommand::Run) {
        client.shutdown_gracefully();
        return Err(error);
    }

    let status = load_rx.await.map_err(|_| {
        AppError::Other("nullbyte-emu nutrūko prieš atsakydamas į Load".to_string())
    })?;

    match status {
        EmuStatus::Loaded(_) => {
            let state = app.state::<AppState>();
            *state.emu_session.lock().expect("Mutex poisoned") = Some(client);
            Ok(())
        }
        EmuStatus::Error(error) => {
            client.shutdown_gracefully();
            Err(AppError::from(error))
        }
        // SAFETY-lygio pastaba (ne unsafe, bet analogiška griežtumo prasme): `drain_loop`
        // (crate::ipc) šį oneshot'ą pripildo TIK `Loaded`/`Error` atveju (žr. jo `match`) —
        // joks kitas `EmuStatus` variantas čia niekada neatkeliauja.
        _ => unreachable!("crate::ipc::drain_loop tik Loaded/Error siunčia į load_result"),
    }
}

/// Priverstinai uždaro veikiančią žaidimo sesiją (jei tokia yra) — atsarginis kelias, jei
/// vartotojas negali/nenori uždaryti paties `nullbyte-emu` lango (pvz. jis pakibo dėl
/// core'o klaidos, arba lango nematyti už kitų langų). `Ok(())` (ne klaida), jei nieko
/// nepaleista — idempotentiškas, saugu spausti kelis kartus iš eilės.
#[tauri::command]
pub fn stop_game(state: State<'_, AppState>) -> Result<(), AppError> {
    let client = state.emu_session.lock().expect("Mutex poisoned").take();
    if let Some(client) = client {
        client.shutdown_gracefully();
    }
    Ok(())
}

/// `true`, jei šiuo metu veikia žaidimo sesija — UI naudoja rodyti „Playing"/„Stop" būseną
/// vietoj „Play" mygtuko be poreikio klausytis event'ų vien šiam vienam patikrinimui.
#[tauri::command]
pub fn is_game_running(state: State<'_, AppState>) -> bool {
    state.emu_session.lock().expect("Mutex poisoned").is_some()
}
