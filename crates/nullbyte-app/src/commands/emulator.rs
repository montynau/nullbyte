//! Žaidimo paleidimo Tauri komandos (CLAUDE.md §6.3, MVP.md P9.1) — vienintelė vieta, kur
//! realiai sujungiami core'o pasirinkimas (P7.6 Nustatymai), žaidimo įrašas (P5.4 DB), ir
//! `nullbyte-emu` vaiko procesas (`crate::ipc::EmuClient`, P4.0.3).
//!
//! ADR-016: vienam žaidimo paleidimui — vienas vaiko procesas, savo langas/audio/gamepad'ai.
//! Šis modulis palaiko TIK VIENĄ veikiančią sesiją vienu metu (`AppState::emu_session`) —
//! antras `start_game()` kvietimas, kol pirmas dar veikia, grąžina aiškią klaidą (P9.3
//! filosofija — „aiškus pranešimas", ne tylus senos sesijos nutraukimas ar pakeitimas).
//! Kelių vienu metu veikiančių žaidimų palaikymas MVP.md apimtyje NENUMATYTAS.
//!
//! Nuo P8.1 UI sluoksnio: `start_game`'o `on_status` callback'as (žr. `crate::ipc` modulio
//! doc #2) į DB įrašo kiekvieną `EmuStatus::StateSaved` (F5-F8 hotkey'ų, MVP.md P4.4,
//! rezultatas) — `commands::savestate` tada šiuos įrašus tik SKAITO/TRINA, pati DB rašymo
//! atsakomybė lieka ČIA, kur yra visas reikiamas kontekstas (`game_id`, `states_dir`, core
//! pavadinimas/versija).

use std::path::PathBuf;
use std::time::Instant;

use tauri::{AppHandle, Emitter, Manager, State};

use nullbyte_core::core::info::load_core_info;
use nullbyte_core::core::runner::EmuCommand;
use nullbyte_core::ipc::EmuStatus;

use crate::commands::settings::resolve_preferred_core_path;
use crate::db::{games, save_states};
use crate::error::AppError;
use crate::ipc::EmuClient;
use crate::paths;
use crate::state::{AppState, RunningSession};

/// Nustatyto core-mismatch duomenys — žr. [`detect_core_mismatch`].
struct CoreMismatch {
    saved_core_name: String,
    saved_core_version: String,
}

/// Grynas palyginimas (jokio `AppHandle`, jokio side-effect'o) — testuojama be pilno Tauri
/// app'o, ta pati technika kaip `commands::settings::resolve_preferred_core_path` ir
/// `commands::savestate::delete_save_state_impl`. `None`, jei slot'as tuščias ARBA core
/// sutampa; kitaip — `Some` su IŠSAUGOTU (ne dabartiniu, tas jau žinomas kviečiančiajam)
/// core'o pavadinimu/versija.
fn detect_core_mismatch(
    conn: &rusqlite::Connection,
    game_id: i64,
    slot: u8,
    current_core_name: &str,
    current_core_version: &str,
) -> Result<Option<CoreMismatch>, AppError> {
    let Some(existing) = save_states::get_save_state(conn, game_id, slot as i64)? else {
        return Ok(None);
    };
    if existing.core_name == current_core_name && existing.core_version == current_core_version {
        return Ok(None);
    }
    Ok(Some(CoreMismatch {
        saved_core_name: existing.core_name,
        saved_core_version: existing.core_version,
    }))
}

/// P8.1 „Kito core state → įspėjimas, ne crash" (CLAUDE.md §8.7, MVP.md P8.1 acceptance) —
/// plonas sluoksnis virš [`detect_core_mismatch`], kuris IR SIUNČIA `"save-state-core-
/// mismatch"` event'ą į UI. Kviečiantysis VIS TIEK siunčia `LoadState` NEPAISANT rezultato
/// (žr. CLAUDE.md: „įkeliant — perspėk vartotoją, jei nesutampa" — ne „atsisakyk bandyti").
fn warn_on_core_mismatch(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    game_id: i64,
    slot: u8,
    current_core_name: &str,
    current_core_version: &str,
) {
    let Ok(Some(mismatch)) =
        detect_core_mismatch(conn, game_id, slot, current_core_name, current_core_version)
    else {
        return;
    };
    if let Err(error) = app.emit(
        "save-state-core-mismatch",
        serde_json::json!({
            "gameId": game_id,
            "slot": slot,
            "savedCoreName": mismatch.saved_core_name,
            "savedCoreVersion": mismatch.saved_core_version,
            "currentCoreName": current_core_name,
            "currentCoreVersion": current_core_version,
        }),
    ) {
        tracing::warn!(%error, "nepavyko išsiųsti save-state-core-mismatch event'o");
    }
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// UI „Žaisti" mygtukas — parenka core'ą (per Nustatymų → Cores preferenciją), paleidžia
/// `nullbyte-emu` vaiko procesą, siunčia `Load`+`Run`, ir LAUKIA REALAUS rezultato
/// (`EmuStatus::Loaded`/`Error`, ne tik to, kad komandos nusiuntimas per stdin pavyko — žr.
/// `crate::ipc` modulio doc #1) prieš grąžindama valdymą UI. Tai leidžia P9.1 acceptance
/// „Trūkstamas core → suprantamas pranešimas" REALIAI veikti klaidoms, kurios paaiškėja
/// TIK core'ui bandant įkelti ROM'ą (trūkstamas BIOS, blogas ROM formatas) — ne tik
/// akivaizdžioms spawn-metu klaidoms (trūkstamas sidecar binaras, blogas core'o failas
/// prieš pat tai gali sukelti ir spawn-metu klaidą per `EmuClient::spawn`, ir load-metu
/// klaidą per šį oneshot'ą — abu keliai grąžina `Err`, tiesiog skirtingu tašku).
///
/// `load_slot` (P8.1 UI sluoksnis) — jei `Some`, iškart po sėkmingo `Loaded` nusiunčiama
/// `EmuCommand::LoadState(slot)`, kad paspaudus „Load" ant konkretaus save state'o žaidimo
/// detalių puslapyje, žaidimas IŠKART pasileistų nuo TO taško, ne tuščio pradinio ekrano.
#[tauri::command]
pub async fn start_game(app: AppHandle, id: i64, load_slot: Option<u8>) -> Result<(), AppError> {
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

    // P8.1 UI sluoksnis: core'o pavadinimas/versija reikalingi save state DB įrašams
    // (`save_states.core_name`/`core_version`, core-mismatch įspėjimui ateityje) —
    // TIK vieno core'o info, NE visas `cores_dir` skenavimas (žr. `load_core_info` doc).
    let core_info = load_core_info(&core_path)?;
    let core_name = core_info.name;
    let core_version = core_info.version;

    let started_at = Instant::now();
    let app_on_close = app.clone();
    let app_on_status = app.clone();
    let states_dir_for_status = states_dir.clone();
    let game_id = id;
    // Klonuojama, nes žemiau esantis `move` closure PERIMA (ne pasiskolina) `core_name`/
    // `core_version` — jie dar reikalingi ŠIAME (`start_game`) scope'e po `spawn()`
    // grąžinimo: `RunningSession` lauke IR core-mismatch palyginimui prieš `LoadState`.
    let core_name_for_status = core_name.clone();
    let core_version_for_status = core_version.clone();
    let (mut client, load_rx) = EmuClient::spawn(
        &app,
        &system_dir,
        &save_dir,
        move |status| {
            // Kviečiama kiekvienam VĖLESNIAM (po pirmo Loaded/Error) statusui — žr.
            // `crate::ipc` modulio doc #2.
            match status {
                EmuStatus::StateSaved { slot } => {
                    let path = states_dir_for_status.join(format!("{slot}.state"));
                    let thumb_path = states_dir_for_status.join(format!("{slot}.png"));
                    // Preview'o nepavykimas NĖRA save state'o klaida (žr. `savestate::
                    // save_state` doc) — failas gali tiesiog neegzistuoti, tada DB
                    // `thumb_path` lieka `NULL`, ne klaidingas kelias.
                    let thumb_path_str = thumb_path
                        .exists()
                        .then(|| thumb_path.to_string_lossy().into_owned());
                    let app_state = app_on_status.state::<AppState>();
                    let upsert_result = app_state.db.lock().map_err(|_| ()).and_then(|conn| {
                        save_states::upsert_save_state(
                            &conn,
                            game_id,
                            slot as i64,
                            &path.to_string_lossy(),
                            thumb_path_str.as_deref(),
                            &core_name_for_status,
                            &core_version_for_status,
                            unix_now(),
                        )
                        .map_err(|_| ())
                    });
                    if upsert_result.is_err() {
                        tracing::error!(game_id, slot, "nepavyko įrašyti save state į DB");
                    }
                    if let Err(error) = app_on_status.emit("save-states-changed", game_id) {
                        tracing::warn!(%error, "nepavyko išsiųsti save-states-changed event'o");
                    }
                }
                EmuStatus::StateLoaded { slot } => {
                    if let Err(error) = app_on_status.emit(
                        "save-state-loaded",
                        serde_json::json!({ "gameId": game_id, "slot": slot }),
                    ) {
                        tracing::warn!(%error, "nepavyko išsiųsti save-state-loaded event'o");
                    }
                }
                EmuStatus::Error(error) => {
                    let payload = serde_json::to_value(AppError::from(error)).ok();
                    if let Some(payload) = payload {
                        if let Err(error) = app_on_status.emit("game-error", payload) {
                            tracing::warn!(%error, "nepavyko išsiųsti game-error event'o");
                        }
                    }
                }
                EmuStatus::Loaded(_) | EmuStatus::Stats { .. } | EmuStatus::Stopped => {}
            }
        },
        move || {
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
        },
    )
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
            if let Some(slot) = load_slot {
                if let Ok(conn) = state.db.lock() {
                    warn_on_core_mismatch(&app, &conn, id, slot, &core_name, &core_version);
                }
                if let Err(error) = client.send(EmuCommand::LoadState(slot)) {
                    tracing::warn!(
                        %error,
                        slot,
                        "nepavyko nusiųsti pradinio LoadState po sėkmingo paleidimo"
                    );
                }
            }
            *state.emu_session.lock().expect("Mutex poisoned") = Some(RunningSession {
                client,
                game_id: id,
                core_name,
                core_version,
            });
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
    let session = state.emu_session.lock().expect("Mutex poisoned").take();
    if let Some(session) = session {
        session.client.shutdown_gracefully();
    }
    Ok(())
}

/// `true`, jei šiuo metu veikia žaidimo sesija — UI naudoja rodyti „Playing"/„Stop" būseną
/// vietoj „Play" mygtuko be poreikio klausytis event'ų vien šiam vienam patikrinimui.
#[tauri::command]
pub fn is_game_running(state: State<'_, AppState>) -> bool {
    state.emu_session.lock().expect("Mutex poisoned").is_some()
}

/// Nusiunčia `EmuCommand::LoadState` VEIKIANČIAI sesijai — P8.1 UI sluoksnis naudoja, kai
/// vartotojas žaidimo detalių puslapyje paspaudžia „Load" ant save state'o TAM PAČIAM
/// žaidimui, kuris ŠIUO METU JAU VEIKIA (nereikia naujo paleidimo per `start_game`, kuris
/// šiuo atveju grąžintų „jau paleista" klaidą — žr. jo doc). Rezultatas (pavyko/ne) ateina
/// ASINCHRONIŠKAI per `"save-state-loaded"`/`"game-error"` event'us (žr. `start_game`
/// `on_status`), ne šio kvietimo grąžinamą reikšmę — TIK komandos NUSIUNTIMAS patvirtinamas
/// čia, tiksliai taip pat, kaip veikia F5-F8 hotkey'ai `nullbyte-emu` viduje.
///
/// Prieš siųsdama, tyliai palygina veikiančio core'o pavadinimą/versiją su
/// `save_states.core_name`/`core_version` (žr. `warn_on_core_mismatch`) ir emituoja
/// `"save-state-core-mismatch"`, jei nesutampa — **bet SIUNČIA `LoadState` NEPAISANT
/// rezultato** (CLAUDE.md §8.7: „perspėk vartotoją, jei nesutampa", ne „atsisakyk").
#[tauri::command]
pub fn load_state_now(
    app: AppHandle,
    state: State<'_, AppState>,
    slot: u8,
) -> Result<(), AppError> {
    let mut session = state.emu_session.lock().expect("Mutex poisoned");
    let Some(session) = session.as_mut() else {
        return Err(AppError::Other(
            "joks žaidimas šiuo metu nepaleistas".to_string(),
        ));
    };
    if let Ok(conn) = state.db.lock() {
        warn_on_core_mismatch(
            &app,
            &conn,
            session.game_id,
            slot,
            &session.core_name,
            &session.core_version,
        );
    }
    session.client.send(EmuCommand::LoadState(slot))
}

/// Veikiančio žaidimo `id`, arba `None` — P8.1 UI sluoksnis naudoja žaidimo detalių puslapyje
/// atskirti „šis KONKRETUS žaidimas dabar veikia" (rodyti „Playing", leisti „Load" tiesiogiai
/// siųsti į veikiančią sesiją be naujo paleidimo) nuo „veikia KAŽKAS KITAS" (rodyti tik
/// bendrą „žaidimas jau paleistas" — žr. `start_game` doc).
#[tauri::command]
pub fn get_running_game_id(state: State<'_, AppState>) -> Option<i64> {
    state
        .emu_session
        .lock()
        .expect("Mutex poisoned")
        .as_ref()
        .map(|session| session.game_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::migrations::MIGRATIONS
            .iter()
            .for_each(|(_, sql)| conn.execute_batch(sql).unwrap());
        conn
    }

    fn insert_test_game(conn: &Connection) -> i64 {
        let platform_id: i64 = conn
            .query_row("SELECT id FROM platforms WHERE slug = 'snes'", [], |row| {
                row.get(0)
            })
            .unwrap();
        conn.execute(
            "INSERT INTO games (platform_id, title, sort_title, rom_path, rom_size, added_at, file_mtime)
             VALUES (?1, 'Test Game', 'test game', '/roms/test.sfc', 1024, 0, 0)",
            [platform_id],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn detect_core_mismatch_is_none_for_an_empty_slot() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);
        assert!(detect_core_mismatch(&conn, game_id, 1, "Snes9x", "1.63")
            .unwrap()
            .is_none());
    }

    #[test]
    fn detect_core_mismatch_is_none_when_core_name_and_version_match() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);
        save_states::upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "1.63",
            1000,
        )
        .unwrap();

        assert!(detect_core_mismatch(&conn, game_id, 1, "Snes9x", "1.63")
            .unwrap()
            .is_none());
    }

    #[test]
    fn detect_core_mismatch_fires_on_version_change_but_not_name_change() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);
        save_states::upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "Snes9x",
            "1.62.3",
            1000,
        )
        .unwrap();

        let mismatch = detect_core_mismatch(&conn, game_id, 1, "Snes9x", "1.63")
            .unwrap()
            .expect("skirtinga versija turėjo sukelti mismatch'ą");
        assert_eq!(mismatch.saved_core_name, "Snes9x");
        assert_eq!(mismatch.saved_core_version, "1.62.3");
    }

    #[test]
    fn detect_core_mismatch_fires_on_different_core_entirely() {
        let conn = open_test_db();
        let game_id = insert_test_game(&conn);
        save_states::upsert_save_state(
            &conn,
            game_id,
            1,
            "/states/1.state",
            None,
            "bsnes",
            "115",
            1000,
        )
        .unwrap();

        let mismatch = detect_core_mismatch(&conn, game_id, 1, "Snes9x", "1.63")
            .unwrap()
            .expect("skirtingas core'as turėjo sukelti mismatch'ą");
        assert_eq!(mismatch.saved_core_name, "bsnes");
    }
}
