//! Scraping orkestracijos Tauri komandos (CLAUDE.md §6.3, MVP.md P6.4) — plonas
//! `Channel`/`CancellationToken` laidas aplink `scraper::{scrape_single_game,
//! scrape_pending_games}`. Visa tikroji logika gyvena `scraper::mod` (Tauri-nepriklausoma,
//! žr. jos modulio doc), kaip ir `library::scanner`/`commands::library` pora.

use tauri::ipc::Channel;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::error::AppError;
use crate::scraper::screenscraper::ScreenScraperCredentials;
use crate::scraper::{self, ScrapeContext, ScrapeProgress, ScrapeSummary};
use crate::state::AppState;

fn forward_progress(channel: Channel<ScrapeProgress>) -> impl FnMut(ScrapeProgress) {
    move |update| {
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
    let credentials = ScreenScraperCredentials::from_env()?;
    let ctx = ScrapeContext {
        client: &state.scraper_client,
        credentials: &credentials,
        limiter: &state.rate_limiter,
        db: &state.db,
        media_dir: &state.media_dir,
    };

    scraper::scrape_single_game(&ctx, id, forward_progress(progress)).await
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
    let credentials = ScreenScraperCredentials::from_env()?;
    let ctx = ScrapeContext {
        client: &state.scraper_client,
        credentials: &credentials,
        limiter: &state.rate_limiter,
        db: &state.db,
        media_dir: &state.media_dir,
    };

    let cancel = CancellationToken::new();
    *state.scrape_cancellation.lock().expect("Mutex poisoned") = Some(cancel.clone());

    let result =
        scraper::scrape_pending_games(&ctx, platform_id, &cancel, forward_progress(progress)).await;

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
