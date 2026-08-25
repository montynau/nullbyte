//! Media (viršeliai/screenshot/wheel/video) atsisiuntimas ir cache'avimas (MVP.md P6.3,
//! CLAUDE.md §9.2 media tipai, §9.4 cache struktūra).
//!
//! Sąmoningai priima `&[Media]` (žaliavinis API tipas iš `types.rs`), NE `screenscraper::Jeu`
//! ar `GameMetadata` — media.rs nežino NIEKO apie ScreenScraper užklausos formavimą ar kitus
//! `jeu` laukus, tik apie tai, KAIP iš duotų media įrašų pasirinkti geriausią kiekvienam tipui
//! ir saugiai atsisiųsti. Kas paduoda `medias` (P6.4 orkestracija) — atskiro sluoksnio reikalas.

#![allow(dead_code)] // pilnai išnaudos P6.4 (scraping orkestracija)

use std::path::Path;

use tokio::io::AsyncWriteExt;

use crate::error::AppError;
use crate::scraper::types::Media;

/// Regionų prioritetas media pasirinkimui — TAS PATS sąrašas kaip `screenscraper.rs` teksto
/// laukams (CLAUDE.md §9.2), bet čia taikomas atskirai, nes media pasirinkimas ir teksto
/// laukų pasirinkimas yra du nepriklausomi sprendimai dėl to paties `jeu` atsakymo.
const REGION_PRIORITY: &[&str] = &["wor", "eu", "us", "jp", "ss"];

/// MVP.md P6.3 „Ką daryti": „Video dydžio limitas (numatytasis 10 MB) — didesnius praleisk".
const MAX_VIDEO_BYTES: u64 = 10 * 1024 * 1024;

/// Santykiniai keliai (nuo `media_dir()`) atsisiųstiems failams — TOKIE PATYS keliai kaip
/// saugomi `games` lentelės `cover_path`/`screenshot_path`/`wheel_path`/`video_path`
/// stulpeliuose (CLAUDE.md §9.4: „DB laiko tik santykinius kelius").
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaPaths {
    pub cover_path: Option<String>,
    pub screenshot_path: Option<String>,
    pub wheel_path: Option<String>,
    pub video_path: Option<String>,
}

/// Atsisiunčia visus keturis media tipus vienam žaidimui. NIEKADA negrąžina `Err` dėl
/// pavienio media įrašo problemos (sugadinta nuoroda, nutrūkęs ryšys, per didelis failas) —
/// tokie atvejai tyliai virsta `None` tam laukui (žr. `download_one`), nes vieno viršelio
/// nepavykimas neturėtų sužlugdyti viso žaidimo scraping'o. `Err` grąžinamas TIK jei pats
/// `media_dir` katalogas nesukuriamas — tai reiškia realią, visiems tipams bendrą problemą
/// (teisės, diskas pilnas), verta sustabdyti.
pub async fn download_game_media(
    client: &reqwest::Client,
    media_dir: &Path,
    game_id: i64,
    medias: &[Media],
) -> Result<MediaPaths, AppError> {
    tokio::fs::create_dir_all(media_dir).await?;

    let cover_path = download_one(
        client,
        media_dir,
        "covers",
        game_id,
        pick_best(medias, "box-2D"),
        None,
    )
    .await;
    let screenshot_path = download_one(
        client,
        media_dir,
        "screenshots",
        game_id,
        pick_best(medias, "ss"),
        None,
    )
    .await;
    let wheel_path = download_one(
        client,
        media_dir,
        "wheels",
        game_id,
        pick_best(medias, "wheel"),
        None,
    )
    .await;
    let video_path = download_one(
        client,
        media_dir,
        "videos",
        game_id,
        pick_video(medias),
        Some(MAX_VIDEO_BYTES),
    )
    .await;

    Ok(MediaPaths {
        cover_path,
        screenshot_path,
        wheel_path,
        video_path,
    })
}

/// Geriausias `media_type` įrašas pagal regiono prioritetą. Įrašai be `region` (dažnai
/// `wheel`/`video`) laimi TIK jei NĖRA nė vieno prioriteto sąraše esančio regiono varianto —
/// tada imamas tiesiog pirmas rastas.
fn pick_best<'a>(medias: &'a [Media], media_type: &str) -> Option<&'a Media> {
    let candidates: Vec<&Media> = medias
        .iter()
        .filter(|m| m.media_type == media_type)
        .collect();

    for region in REGION_PRIORITY {
        if let Some(found) = candidates
            .iter()
            .find(|m| m.region.as_deref() == Some(*region))
        {
            return Some(found);
        }
    }

    candidates.into_iter().next()
}

/// `video-normalized` pirmenybė prieš `video` (CLAUDE.md §9.2: „mažesnis, vienodesnis").
fn pick_video(medias: &[Media]) -> Option<&Media> {
    pick_best(medias, "video-normalized").or_else(|| pick_best(medias, "video"))
}

/// Atsisiunčia VIENĄ media įrašą. `None` grąžina TYLIAI (ne `Err`) bet kokiu iš šių atvejų:
/// nėra kandidato, HTTP klaida, tinklo klaida, viršytas `size_limit`. Sėkmės atveju grąžina
/// SANTYKINĮ kelią (`{subdir}/{game_id}.{ext}`), tinkamą tiesiogiai rašyti į DB.
async fn download_one(
    client: &reqwest::Client,
    media_dir: &Path,
    subdir: &str,
    game_id: i64,
    media: Option<&Media>,
    size_limit: Option<u64>,
) -> Option<String> {
    let media = media?;
    let extension = extension_for(media);
    let relative_path = format!("{subdir}/{game_id}.{extension}");
    let final_path = media_dir.join(&relative_path);

    // MVP.md P6.3 acceptance: „Pakartotinis scraping'as nesiunčia to paties dar kartą".
    if final_path
        .metadata()
        .map(|meta| meta.len() > 0)
        .unwrap_or(false)
    {
        return Some(relative_path);
    }

    match try_download(
        client,
        media_dir,
        subdir,
        &final_path,
        &media.url,
        size_limit,
    )
    .await
    {
        Ok(()) => Some(relative_path),
        Err(error) => {
            tracing::warn!(url = %media.url, %error, "media atsisiuntimas nepavyko, praleidžiama");
            None
        }
    }
}

/// Rašo į `{final_path}.tmp`, tada `rename` — MVP.md P6.3 acceptance: „Nutrūkęs atsisiuntimas
/// nepalieka sugadinto failo". `rename` tame pačiame failų sistemos taške (POSIX) yra
/// atominis — arba visas failas, arba jokio.
async fn try_download(
    client: &reqwest::Client,
    media_dir: &Path,
    subdir: &str,
    final_path: &Path,
    url: &str,
    size_limit: Option<u64>,
) -> Result<(), AppError> {
    tokio::fs::create_dir_all(media_dir.join(subdir)).await?;

    let tmp_path = final_path.with_extension(format!(
        "{}.tmp",
        final_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin")
    ));

    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(AppError::from)?
        .error_for_status()
        .map_err(AppError::from)?;

    let mut file = tokio::fs::File::create(&tmp_path).await?;
    let mut total: u64 = 0;

    while let Some(chunk) = response.chunk().await.map_err(AppError::from)? {
        total += chunk.len() as u64;
        if let Some(limit) = size_limit {
            if total > limit {
                drop(file);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(AppError::Other(format!(
                    "media per didelis ({total} > {limit} baitų limitas)"
                )));
            }
        }
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);

    if total == 0 {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(AppError::Other(
            "media atsakymas tuščias (0 baitų)".to_string(),
        ));
    }

    tokio::fs::rename(&tmp_path, final_path).await?;
    Ok(())
}

/// Plėtinys iš `Media.format` (kai API jį duoda), kitaip iš URL kelio (nuėmus query string'ą —
/// jis dažnai turi devid/devpassword parametrus, žr. `types.rs` modulio doc), kitaip
/// numatytoji reikšmė pagal tipą.
fn extension_for(media: &Media) -> String {
    if let Some(format) = &media.format {
        return format.to_lowercase();
    }

    let path_only = media.url.split('?').next().unwrap_or(&media.url);
    Path::new(path_only)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| default_extension(&media.media_type).to_string())
}

fn default_extension(media_type: &str) -> &'static str {
    match media_type {
        "video" | "video-normalized" => "mp4",
        _ => "png",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::net::TcpListener;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn temp_media_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("nullbyte_media_test_{}_{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn media(media_type: &str, region: Option<&str>, url: &str, format: Option<&str>) -> Media {
        Media {
            media_type: media_type.to_string(),
            region: region.map(str::to_string),
            url: url.to_string(),
            format: format.map(str::to_string),
        }
    }

    /// Minimalus vietinis HTTP serveris VIENAI užklausai — testuoja realų tinklo/failų I/O
    /// (rašymas į `.tmp` + `rename`) BE priklausomybės nuo realaus ScreenScraper media
    /// hosto (URL su realiais kredencialais NIEKADA negali atsidurti test kode, žr.
    /// `types.rs` modulio doc — realus media URL turi devid/devpassword atviru tekstu).
    async fn serve_once(body: Vec<u8>) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(header.as_bytes()).await.unwrap();
            socket.write_all(&body).await.unwrap();
            socket.shutdown().await.unwrap();
        });
        (format!("http://{addr}/media.bin"), handle)
    }

    #[test]
    fn pick_best_prefers_eu_over_us_jp_ss() {
        let medias = vec![
            media("box-2D", Some("ss"), "u1", None),
            media("box-2D", Some("us"), "u2", None),
            media("box-2D", Some("eu"), "u3", None),
            media("ss", Some("us"), "u4", None), // kitas tipas — neturi trukdyti.
        ];
        let best = pick_best(&medias, "box-2D").unwrap();
        assert_eq!(best.url, "u3");
    }

    #[test]
    fn pick_best_falls_back_to_first_when_no_priority_region_present() {
        let medias = vec![media("wheel", Some("br"), "u1", None)];
        let best = pick_best(&medias, "wheel").unwrap();
        assert_eq!(best.url, "u1");
    }

    #[test]
    fn pick_best_returns_none_when_type_absent() {
        let medias = vec![media("box-2D", Some("eu"), "u1", None)];
        assert!(pick_best(&medias, "wheel").is_none());
    }

    #[test]
    fn pick_video_prefers_normalized_over_plain() {
        let medias = vec![
            media("video", Some("eu"), "plain", None),
            media("video-normalized", Some("eu"), "normalized", None),
        ];
        assert_eq!(pick_video(&medias).unwrap().url, "normalized");
    }

    #[test]
    fn pick_video_falls_back_to_plain_when_normalized_absent() {
        let medias = vec![media("video", Some("eu"), "plain", None)];
        assert_eq!(pick_video(&medias).unwrap().url, "plain");
    }

    #[test]
    fn extension_prefers_format_field_over_url() {
        let m = media(
            "box-2D",
            None,
            "https://x.invalid/file.jpg?x=1",
            Some("png"),
        );
        assert_eq!(extension_for(&m), "png");
    }

    #[test]
    fn extension_falls_back_to_url_path_ignoring_query_string() {
        let m = media(
            "box-2D",
            None,
            "https://x.invalid/file.JPG?devid=secret",
            None,
        );
        assert_eq!(extension_for(&m), "jpg");
    }

    #[test]
    fn extension_falls_back_to_type_default_when_nothing_else_available() {
        let m = media("video", None, "https://x.invalid/no-extension-here", None);
        assert_eq!(extension_for(&m), "mp4");
    }

    #[tokio::test]
    async fn successful_download_writes_final_file_via_tmp_rename() {
        let dir = temp_media_dir();
        let (url, handle) = serve_once(b"fake-cover-bytes".to_vec()).await;
        let client = reqwest::Client::new();
        let m = media("box-2D", Some("eu"), &url, Some("png"));

        let result = download_one(&client, &dir, "covers", 42, Some(&m), None).await;
        handle.await.unwrap();

        assert_eq!(result, Some("covers/42.png".to_string()));
        assert_eq!(
            std::fs::read(dir.join("covers/42.png")).unwrap(),
            b"fake-cover-bytes"
        );
        assert!(!dir.join("covers/42.png.tmp").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn oversized_media_is_skipped_and_leaves_no_files() {
        let dir = temp_media_dir();
        let (url, handle) = serve_once(vec![0u8; 2048]).await;
        let client = reqwest::Client::new();
        let m = media("video", None, &url, Some("mp4"));

        let result = download_one(&client, &dir, "videos", 7, Some(&m), Some(1024)).await;
        handle.await.unwrap();

        assert_eq!(result, None);
        assert!(!dir.join("videos/7.mp4").exists());
        assert!(!dir.join("videos/7.mp4.tmp").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    /// MVP.md P6.3 acceptance: „Pakartotinis scraping'as nesiunčia to paties dar kartą" —
    /// URL sąmoningai NEPASIEKIAMAS (1 uostas — rezervuotas, niekas ten nesiklauso). Jei
    /// kodas VIS TIEK bandytų siųsti, gautume `None` (klaida), ne `Some` su nepakitusiu
    /// turiniu — tai įrodo, kad tinklas apskritai nebuvo liestas.
    #[tokio::test]
    async fn existing_nonempty_file_is_skipped_without_new_request() {
        let dir = temp_media_dir();
        std::fs::create_dir_all(dir.join("covers")).unwrap();
        std::fs::write(dir.join("covers/9.png"), b"already-here").unwrap();
        let client = reqwest::Client::new();
        let m = media(
            "box-2D",
            None,
            "http://127.0.0.1:1/unreachable.png",
            Some("png"),
        );

        let result = download_one(&client, &dir, "covers", 9, Some(&m), None).await;

        assert_eq!(result, Some("covers/9.png".to_string()));
        assert_eq!(
            std::fs::read(dir.join("covers/9.png")).unwrap(),
            b"already-here"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn missing_media_slot_returns_none_without_any_request() {
        let dir = temp_media_dir();
        let client = reqwest::Client::new();
        let result = download_one(&client, &dir, "wheels", 1, None, None).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn download_game_media_maps_each_type_to_its_own_subdirectory() {
        let dir = temp_media_dir();
        let (cover_url, cover_handle) = serve_once(b"cover".to_vec()).await;
        let (video_url, video_handle) = serve_once(b"video".to_vec()).await;
        let client = reqwest::Client::new();

        let medias = vec![
            media("box-2D", Some("eu"), &cover_url, Some("png")),
            media("video-normalized", Some("eu"), &video_url, Some("mp4")),
        ];

        let paths = download_game_media(&client, &dir, 100, &medias)
            .await
            .unwrap();
        cover_handle.await.unwrap();
        video_handle.await.unwrap();

        assert_eq!(paths.cover_path.as_deref(), Some("covers/100.png"));
        assert_eq!(paths.video_path.as_deref(), Some("videos/100.mp4"));
        assert_eq!(paths.screenshot_path, None);
        assert_eq!(paths.wheel_path, None);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// REALUS tinklo kvietimas — atsisiunčia TIKRĄ viršelį iš ScreenScraper. Sąmoningai
    /// NEnaudoja `screenscraper::lookup_game()` (jis grąžina TIK `GameMetadata`, be `medias`
    /// lauko — media.rs sąmoningai atskirtas nuo `Jeu`, žr. modulio doc), o daro savo TIESIOGINĮ
    /// `jeuInfos.php` kvietimą, kad gautų `jeu.medias`. **KRITIŠKAI SVARBU:** realaus atsakymo
    /// `medias[].url` turi devid/devpassword ATVIRU TEKSTU (žr. `types.rs` modulio doc) — jokiu
    /// būdu NEGALIMA šio URL užrašyti kaip konstantą test kode; jis gaunamas TIK vykdymo metu.
    /// `#[ignore]`: priklauso nuo tinklo IR realių `.env` kredencialų. Paleisti rankiniu būdu:
    /// `cargo test -p nullbyte-app real_cover_downloads_from_live_screenscraper_response -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_cover_downloads_from_live_screenscraper_response() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let credentials = crate::scraper::screenscraper::ScreenScraperCredentials::from_env()
                .expect(".env turėtų turėti SCREENSCRAPER_DEV_ID/DEV_PASSWORD");
            let client = reqwest::Client::new();

            let query = [
                ("devid", credentials.devid.as_str()),
                ("devpassword", credentials.devpassword.as_str()),
                ("softname", "Nullbyte-test"),
                ("output", "json"),
                ("crc", "AD2CBF9C"),
                ("romnom", "Super Metroid.sfc"),
                ("systemeid", "4"),
            ];
            let body = client
                .get("https://www.screenscraper.fr/api2/jeuInfos.php")
                .query(&query)
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            let parsed: crate::scraper::types::JeuInfosResponse =
                serde_json::from_str(&body).expect("tikėtasi galiojančio JSON");
            let medias = parsed
                .response
                .expect("tikėtasi Found Super Metroid CRC'ui")
                .jeu
                .medias
                .into_vec();
            assert!(
                medias.iter().any(|m| m.media_type == "box-2D"),
                "tikėtasi bent vieno box-2D viršelio realiame atsakyme"
            );

            let dir = temp_media_dir();
            let result = download_game_media(&client, &dir, 999_999, &medias)
                .await
                .unwrap();

            eprintln!("atsisiųsti keliai: {result:?}");
            let cover_relative = result.cover_path.expect("tikėtasi atsisiųsto viršelio");
            let cover_bytes = std::fs::read(dir.join(&cover_relative)).unwrap();
            assert!(!cover_bytes.is_empty());
            assert!(!dir.join(format!("{cover_relative}.tmp")).exists());

            std::fs::remove_dir_all(&dir).ok();
        });
    }
}
