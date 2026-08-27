//! Lokalus HTTP serveris media katalogui (P7.3 real bug fix, ADR-041) — Tauri `asset://`
//! protokolas WebKitGTK (Linux) video/audio elementams nepatikimas: `<video>` reikalauja
//! HTTP Range palaikymo net pradiniam atkūrimo bandymui, kurio `asset://` tvarkytojas
//! nesuteikia (patikrinta realiai — GStreamer pats failą skaito be problemų, bet WebKit
//! atmeta „FormatError" per kelias milisekundes, PRIEŠ pasiekiant tinklo sluoksnį).
//!
//! Sprendimas: TIKRAS HTTP serveris `127.0.0.1` ant OS parinkto laisvo porto,
//! `tower_http::services::ServeDir` (Range užklausas apdoroja teisingai pagal HTTP spec).
//! Veikia VISOMS platformoms vienodai — supaprastina architektūrą, ne vien Linux apejimas.
//! Viršeliai/screenshot'ai/wheel'ai LIEKA ant `asset://` (`convertFileSrc`) — jiems Range
//! nereikalingas (paprastas `<img>` GET), tad nėra prasmės jų keisti.

use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;

use crate::error::AppError;

/// Sinchroniai pririša laisvą portą — greita (vien `bind()` syscall), tad netrukdo
/// `AppState::new()` likti sinchroniška. Grąžina PORTĄ IR PATĮ listener'į — pastarasis
/// perduodamas [`spawn`] TIK kai jau yra veikianti async runtime (Tauri `.setup()` metu),
/// nes `axum::serve` reikalauja `tokio::net::TcpListener`.
pub fn bind() -> Result<(u16, StdTcpListener), AppError> {
    let listener = StdTcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::Other(format!("nepavyko priristi media serverio porto: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::Other(format!("nepavyko gauti media serverio adreso: {e}")))?
        .port();
    Ok((port, listener))
}

/// Paleidžia serverį — NIEKADA negrąžina normaliai (veikia visą programos gyvavimo trukmę
/// atskiroje `tauri::async_runtime::spawn` užduotyje), klaidos tik loginamos, nes iki šio
/// momento portas jau grąžintas frontend'ui per [`bind`].
pub async fn spawn(listener: StdTcpListener, media_dir: PathBuf) {
    if let Err(error) = listener.set_nonblocking(true) {
        tracing::error!(%error, "nepavyko nustatyti media serverio listener'io į nonblocking");
        return;
    }
    let listener = match tokio::net::TcpListener::from_std(listener) {
        Ok(l) => l,
        Err(error) => {
            tracing::error!(%error, "nepavyko konvertuoti media serverio listener'io");
            return;
        }
    };

    let app = axum::Router::new().fallback_service(tower_http::services::ServeDir::new(media_dir));

    tracing::info!("media serveris paleistas");
    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "media serveris sustojo su klaida");
    }
}
