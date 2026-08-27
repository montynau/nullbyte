//! Aplikacijos globalus būvis, laikomas kaip Tauri managed state.
//!
//! Laiko išspręstus duomenų katalogus, DB ryšį, ir (nuo P9.1) `nullbyte-emu` vaiko proceso
//! rankeną (`emu_session`, žr. jo doc) — žr. `crate::commands` modulio doc dėl P2.3-eros
//! lokalaus `Renderer`/`EmuThread` pašalinimo P4.0.3 metu (ADR-016 juos perkėlė į atskirą
//! procesą, kurio gyvavimo ciklą nuo P9.1 valdo `commands::emulator`).

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::db::migrations;
use crate::error::AppError;
use crate::paths;

/// Veikiančios žaidimo sesijos rankena — `EmuClient` PLIUS `game_id`, kurio pačiam
/// `EmuClient`/`crate::ipc` NIEKADA nereikia (ADR-016 „DB-oblivious" — vaikas jo nežino),
/// bet kurio REIKIA čia, tėvo pusėje, kad `EmuStatus::StateSaved`/`StateLoaded` (P8.1 UI
/// sluoksnis, `commands::emulator::start_game`'o `on_status`) žinotų, KURIAM žaidimui
/// priklauso gautas `slot`, ir kad `commands::savestate` galėtų patikrinti, ar konkretus
/// žaidimas ŠIUO METU veikia (pvz. ar leisti tiesiogiai siųsti `SaveState` be paleidimo).
pub struct RunningSession {
    pub client: crate::ipc::EmuClient,
    pub game_id: i64,
    /// Šiuo metu veikiančio core'o pavadinimas/versija — `commands::emulator::load_state_now`
    /// naudoja lyginti su `save_states.core_name`/`core_version` (P8.1 core-mismatch
    /// įspėjimas, ADR-028 pastaba), nes tas kelias (skirtingai nuo `start_game`) neturi
    /// prieigos prie ką tik išspręsto `core_info` — sesija jau egzistuoja.
    pub core_name: String,
    pub core_version: String,
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub cores_dir: PathBuf,
    pub system_dir: PathBuf,
    pub saves_dir: PathBuf,
    pub states_dir: PathBuf,
    pub media_dir: PathBuf,
    pub db_path: PathBuf,
    /// `rusqlite::Connection` NĖRA `Sync` (CLAUDE.md §10 „SQLite") — `Mutex<Connection>` MVP
    /// metu pakanka (vienas ryšys, ne pool'as; `r2d2_sqlite` — post-MVP, jei tikrai reikės).
    pub db: Mutex<Connection>,
    /// Bendras `reqwest::Client` ScreenScraper užklausoms (P6.4) — vienas klientas visai
    /// programos gyvavimo trukmei, kad būtų pakartotinai naudojami TCP/TLS ryšiai, ne kuriami
    /// nauji kiekvienam `scrape_game`/`scrape_library` kvietimui.
    pub scraper_client: reqwest::Client,
    /// Semaforas + žinomas `maxthreads` (P6.2) — TURI IŠGYVENTI tarp atskirų
    /// `scrape_game`/`scrape_library` kvietimų, kad kartą sužinotas realus `maxthreads`
    /// nebūtų pamirštas kiekvieną kartą pradedant nuo numatytosios „1".
    pub rate_limiter: crate::scraper::rate_limit::RateLimiter,
    /// Šiuo metu vykstančio `scrape_library` atšaukimo žetonas — `None`, kai joks scraping'as
    /// nevyksta. `commands::scraper::cancel_scrape` jį randa čia (P6.4 acceptance:
    /// „Atšaukimas veikia iškart").
    pub scrape_cancellation: Mutex<Option<tokio_util::sync::CancellationToken>>,
    /// Paskutinė ŽINOMA (iš gyvo, ne cache'uoto, atsakymo) likusi dienos kvota — `None`, kol
    /// šią sesiją dar niekas nescrape'inta. Sąmoningai NĖRA gaunama specialiu „patikrink
    /// kvotą" API kvietimu (CLAUDE.md §9.3 „niekada neskenuok/nešvaistyk kvotos be reikalo") —
    /// atnaujinama TIK kaip `scrape_game`/`scrape_library` šalutinis produktas
    /// (`commands::scraper`). P7.6 Settings ekranas ją rodo pasyviai.
    pub last_quota: Mutex<Option<crate::commands::scraper::QuotaSnapshot>>,
    /// P9.1: veikiančio `nullbyte-emu` vaiko proceso rankena — `None`, kai joks žaidimas
    /// nepaleistas. ADR-016: vienam žaidimo paleidimui tenka VIENAS vaiko procesas, tad
    /// šioje aplikacijoje vienu metu gali veikti tik VIENAS žaidimas (žr.
    /// `commands::emulator::start_game` doc dėl KODĖL — antras `start_game` kvietimas, kol
    /// šis `Some`, grąžina aiškią klaidą, o ne tyliai pakeičia/nutraukia esamą sesiją).
    pub emu_session: Mutex<Option<RunningSession>>,
    /// P7.3 real bug fix (ADR-041) — lokalaus media HTTP serverio portas, jau pririštas
    /// (`media_server::bind()`), bet DAR NEPALEISTAS (tam reikia async runtime, žr.
    /// `media_server_listener` doc). Frontend'as jį gauna per `get_app_info` ir konstruoja
    /// video URL kaip `http://127.0.0.1:{port}/...`, ne per `convertFileSrc`.
    pub media_server_port: u16,
    /// Pririštas (bet dar nepaleistas) TCP listener — laikinai laikomas čia, nes
    /// `AppState::new()` kviečiamas PRIEŠ Tauri `.setup()` (taigi ir prieš bet kokią async
    /// runtime), o `axum::serve` reikalauja `tokio::net::TcpListener`. `.setup()` closure
    /// jį `.take()`'ina ir perduoda `media_server::spawn()`. `None` po to — VISADA (niekas
    /// kitas šio lauko nenaudoja pakartotinai).
    pub media_server_listener: Mutex<Option<std::net::TcpListener>>,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let db_path = paths::db_path()?;
        let db = migrations::open_and_migrate(&db_path)?;
        let (media_server_port, media_server_listener) = crate::media_server::bind()?;

        Ok(Self {
            data_dir: paths::data_dir()?,
            cores_dir: paths::cores_dir()?,
            system_dir: paths::system_dir()?,
            saves_dir: paths::saves_dir()?,
            states_dir: paths::states_dir()?,
            media_dir: paths::media_dir()?,
            db_path,
            db: Mutex::new(db),
            scraper_client: reqwest::Client::new(),
            rate_limiter: crate::scraper::rate_limit::RateLimiter::new(),
            scrape_cancellation: Mutex::new(None),
            last_quota: Mutex::new(None),
            emu_session: Mutex::new(None),
            media_server_port,
            media_server_listener: Mutex::new(Some(media_server_listener)),
        })
    }
}
