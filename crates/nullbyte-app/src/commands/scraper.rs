//! Scraping orkestracijos Tauri komandos (CLAUDE.md §6.3, MVP.md P6.4) — plonas
//! `Channel`/`CancellationToken` laidas aplink `scraper::{scrape_single_game,
//! scrape_pending_games}`. Visa tikroji logika gyvena `scraper::mod` (Tauri-nepriklausoma,
//! žr. jos modulio doc), kaip ir `library::scanner`/`commands::library` pora.
//!
//! P7.6 (Settings ekranas) pridėjo `get_scraper_status`/`get_scraper_quota` — pasyvūs
//! „skaityk esamą būvį" kvietimai, NEKVIEČIA ScreenScraper API (žr. `QuotaSnapshot` doc dėl
//! priežasties).

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::db::settings;
use crate::error::AppError;
use crate::scraper::screenscraper::ScreenScraperCredentials;
use crate::scraper::{self, ScrapeContext, ScrapeProgress, ScrapeSummary};
use crate::state::AppState;

/// Paskutinė žinoma likusi dienos kvota — žr. `AppState.last_quota` doc dėl KODĖL tai
/// pasyvus, ne aktyviai užklausiamas laukas.
#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    pub quota_left: i64,
    /// Unix sekundės, kada šis skaičius buvo gautas iš GYVO ScreenScraper atsakymo.
    pub checked_at: i64,
}

fn forward_progress(
    channel: Channel<ScrapeProgress>,
    last_quota: &Mutex<Option<QuotaSnapshot>>,
) -> impl FnMut(ScrapeProgress) + '_ {
    move |update| {
        if let Some(quota_left) = update.quota_left {
            let checked_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or(0);
            *last_quota.lock().expect("Mutex poisoned") = Some(QuotaSnapshot {
                quota_left,
                checked_at,
            });
        }
        if let Err(error) = channel.send(update) {
            tracing::warn!(%error, "nepavyko išsiųsti scrape progreso į UI");
        }
    }
}

/// Vieno žaidimo scraping'as — MVP.md P6.4 `scrape_game(id)`.
#[tauri::command]
pub async fn scrape_game(
    state: State<'_, AppState>,
    id: i64,
    progress: Channel<ScrapeProgress>,
) -> Result<ScrapeSummary, AppError> {
    let credentials = {
        let conn = state.db.lock().expect("Mutex poisoned");
        ScreenScraperCredentials::load(&conn)?
    };
    let ctx = ScrapeContext {
        client: &state.scraper_client,
        credentials: &credentials,
        limiter: &state.rate_limiter,
        db: &state.db,
        media_dir: &state.media_dir,
    };

    scraper::scrape_single_game(&ctx, id, forward_progress(progress, &state.last_quota)).await
}

/// Visos `scrape_status = 'pending'` eilės scraping'as — MVP.md P6.4
/// `scrape_library(platform_id?)`. **Niekada nekviečiama automatiškai** (MVP.md „Ką daryti") —
/// tik vartotojui paspaudus P7 UI mygtuką.
#[tauri::command]
pub async fn scrape_library(
    state: State<'_, AppState>,
    platform_id: Option<i64>,
    progress: Channel<ScrapeProgress>,
) -> Result<ScrapeSummary, AppError> {
    let credentials = {
        let conn = state.db.lock().expect("Mutex poisoned");
        ScreenScraperCredentials::load(&conn)?
    };
    let ctx = ScrapeContext {
        client: &state.scraper_client,
        credentials: &credentials,
        limiter: &state.rate_limiter,
        db: &state.db,
        media_dir: &state.media_dir,
    };

    let cancel = CancellationToken::new();
    *state.scrape_cancellation.lock().expect("Mutex poisoned") = Some(cancel.clone());

    let result = scraper::scrape_pending_games(
        &ctx,
        platform_id,
        &cancel,
        forward_progress(progress, &state.last_quota),
    )
    .await;

    // Baigėsi (sėkmingai, klaidingai ar atšaukta) — nebėra ką atšaukti šiuo žetonu.
    *state.scrape_cancellation.lock().expect("Mutex poisoned") = None;

    result
}

/// Atšaukia šiuo metu vykstantį `scrape_library` — tyliai nieko nedaro, jei joks scraping'as
/// nevyksta (MVP.md P6.4 acceptance: „Atšaukimas veikia iškart").
#[tauri::command]
pub fn cancel_scrape(state: State<'_, AppState>) {
    if let Some(token) = state
        .scrape_cancellation
        .lock()
        .expect("Mutex poisoned")
        .as_ref()
    {
        token.cancel();
    }
}

/// ScreenScraper kredencialų konfigūracijos būvis — MVP.md P7.6 Settings „Scraper" panelė.
/// **Niekada** negrąžina slaptažodžių ar pilno `devid`/`ssid` — tik tiek, kad UI galėtų
/// parodyti „sukonfigūruota / ne" be jautrių duomenų atskleidimo.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScraperCredentialStatus {
    pub dev_credentials_configured: bool,
    /// Pvz. `"ab••••"` — tik pirmi du `devid` simboliai, orientacijai (kad vartotojas
    /// atpažintų KURIE kredencialai naudojami), niekada visas ID.
    pub dev_id_masked: Option<String>,
    pub user_login_configured: bool,
    /// `true`, jei `settings` lentelėje yra UI įvestas `devid` (žr.
    /// `ScreenScraperCredentials::load` doc dėl pirmenybės prieš `.env`) — UI tai naudoja
    /// spręsdama, ar rodyti „Clear override" mygtuką.
    pub overridden: bool,
}

fn mask_dev_id(devid: &str) -> String {
    let visible: String = devid.chars().take(2).collect();
    format!("{visible}••••")
}

/// Skaito kredencialus (žr. `ScreenScraperCredentials::load` — `settings` lentelė TURI
/// PIRMENYBĘ prieš `.env`) ir grąžina TIK jų konfigūracijos būvį — pati komanda NEDARO
/// jokio HTTP kvietimo į ScreenScraper (CLAUDE.md §9.3 „niekada neskenuok/nešvaistyk kvotos
/// be reikalo" — vien Settings ekrano atidarymas neturi kainuoti dienos kvotos).
#[tauri::command]
pub fn get_scraper_status(state: State<'_, AppState>) -> Result<ScraperCredentialStatus, AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    let overridden = settings::get(&conn, ScreenScraperCredentials::KEY_DEV_ID)?
        .is_some_and(|value| !value.is_empty());

    Ok(match ScreenScraperCredentials::load(&conn) {
        Ok(credentials) => ScraperCredentialStatus {
            dev_credentials_configured: true,
            dev_id_masked: Some(mask_dev_id(&credentials.devid)),
            user_login_configured: credentials.ssid.is_some(),
            overridden,
        },
        Err(_) => ScraperCredentialStatus {
            dev_credentials_configured: false,
            dev_id_masked: None,
            user_login_configured: false,
            overridden,
        },
    })
}

/// Įrašo kredencialus, redaguotus P7.6 Scraper panelėje, į `settings` lentelę — nuo šiol jie
/// TURI PIRMENYBĘ prieš `.env` (žr. `ScreenScraperCredentials::load`). `ssid`/`sspassword`
/// neprivalomi (CLAUDE.md §9.3): tuščia arba praleista reikšmė IŠTRINA anksčiau įrašytą
/// override'ą, ne įrašo tuščią eilutę.
#[tauri::command]
pub fn set_scraper_credentials(
    state: State<'_, AppState>,
    dev_id: String,
    dev_password: String,
    ssid: Option<String>,
    sspassword: Option<String>,
) -> Result<(), AppError> {
    if dev_id.trim().is_empty() || dev_password.trim().is_empty() {
        return Err(AppError::Other(
            "Dev ID ir Dev Password yra privalomi".to_string(),
        ));
    }

    let conn = state.db.lock().expect("Mutex poisoned");
    settings::set(&conn, ScreenScraperCredentials::KEY_DEV_ID, dev_id.trim())?;
    settings::set(
        &conn,
        ScreenScraperCredentials::KEY_DEV_PASSWORD,
        dev_password.trim(),
    )?;

    match ssid.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => {
            settings::set(&conn, ScreenScraperCredentials::KEY_SSID, value)?
        }
        _ => settings::delete(&conn, ScreenScraperCredentials::KEY_SSID)?,
    }
    match sspassword.as_deref().map(str::trim) {
        Some(value) if !value.is_empty() => {
            settings::set(&conn, ScreenScraperCredentials::KEY_SSPASSWORD, value)?
        }
        _ => settings::delete(&conn, ScreenScraperCredentials::KEY_SSPASSWORD)?,
    }

    Ok(())
}

/// Pašalina VISUS UI įrašytus kredencialų override'us — grąžina prie `.env` (jei jame kas
/// nors yra) arba prie „nesukonfigūruota" būvio.
#[tauri::command]
pub fn clear_scraper_credentials(state: State<'_, AppState>) -> Result<(), AppError> {
    let conn = state.db.lock().expect("Mutex poisoned");
    settings::delete(&conn, ScreenScraperCredentials::KEY_DEV_ID)?;
    settings::delete(&conn, ScreenScraperCredentials::KEY_DEV_PASSWORD)?;
    settings::delete(&conn, ScreenScraperCredentials::KEY_SSID)?;
    settings::delete(&conn, ScreenScraperCredentials::KEY_SSPASSWORD)?;
    Ok(())
}

/// Paskutinė šią sesiją žinoma kvota — `None`, jei dar niekas nescrape'inta (žr.
/// `AppState.last_quota` doc).
#[tauri::command]
pub fn get_scraper_quota(state: State<'_, AppState>) -> Option<QuotaSnapshot> {
    *state.last_quota.lock().expect("Mutex poisoned")
}
