// Tipizuoti Tauri `invoke` wrapper'iai (CLAUDE.md §7.3) — vienas failas per domeną,
// kai jų prisikaups daugiau; kol kas viskas telpa čia.

import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  AudioSettings,
  CoreInfo,
  Game,
  GameFilter,
  InputBinding,
  PlatformCorePreference,
  PlatformSummary,
  QuotaSnapshot,
  RomDirectory,
  ScanProgress,
  ScanSummary,
  ScraperCredentialStatus,
  ScrapeProgress,
  ScrapeSummary,
  VideoSettings,
} from "$lib/types";

export function getAppInfo(): Promise<AppInfo> {
  return invoke("get_app_info");
}

export function listGames(filter: GameFilter): Promise<Game[]> {
  return invoke("list_games", { filter });
}

export function getGame(id: number): Promise<Game | null> {
  return invoke("get_game", { id });
}

export function setFavorite(id: number, favorite: boolean): Promise<void> {
  return invoke("set_favorite", { id, favorite });
}

export function recordPlay(id: number, seconds: number): Promise<void> {
  return invoke("record_play", { id, seconds });
}

export function listPlatforms(): Promise<PlatformSummary[]> {
  return invoke("list_platforms");
}

export function scrapeGame(
  id: number,
  onProgress: (progress: ScrapeProgress) => void,
): Promise<ScrapeSummary> {
  const channel = new Channel<ScrapeProgress>();
  channel.onmessage = onProgress;
  return invoke("scrape_game", { id, progress: channel });
}

export function scrapeLibrary(
  platformId: number | null,
  onProgress: (progress: ScrapeProgress) => void,
): Promise<ScrapeSummary> {
  const channel = new Channel<ScrapeProgress>();
  channel.onmessage = onProgress;
  return invoke("scrape_library", { platformId, progress: channel });
}

export function cancelScrape(): Promise<void> {
  return invoke("cancel_scrape");
}

export function listRomDirectories(): Promise<RomDirectory[]> {
  return invoke("list_rom_directories");
}

export function addRomDirectory(
  path: string,
  recursive: boolean,
  platformId: number | null,
): Promise<RomDirectory> {
  return invoke("add_rom_directory", { path, recursive, platformId });
}

export function removeRomDirectory(id: number): Promise<void> {
  return invoke("remove_rom_directory", { id });
}

export function scanLibrary(onProgress: (progress: ScanProgress) => void): Promise<ScanSummary> {
  const channel = new Channel<ScanProgress>();
  channel.onmessage = onProgress;
  return invoke("scan_library", { progress: channel });
}

export function getScraperStatus(): Promise<ScraperCredentialStatus> {
  return invoke("get_scraper_status");
}

export function getScraperQuota(): Promise<QuotaSnapshot | null> {
  return invoke("get_scraper_quota");
}

export function setScraperCredentials(
  devId: string,
  devPassword: string,
  ssid: string | null,
  sspassword: string | null,
): Promise<void> {
  return invoke("set_scraper_credentials", { devId, devPassword, ssid, sspassword });
}

export function clearScraperCredentials(): Promise<void> {
  return invoke("clear_scraper_credentials");
}

export function getInputMapping(): Promise<InputBinding[]> {
  return invoke("get_input_mapping");
}

export function setInputMapping(bindings: InputBinding[]): Promise<void> {
  return invoke("set_input_mapping", { bindings });
}

export function resetInputMapping(): Promise<void> {
  return invoke("reset_input_mapping");
}

export function listCores(): Promise<CoreInfo[]> {
  return invoke("list_cores");
}

export function getPreferredCores(): Promise<PlatformCorePreference[]> {
  return invoke("get_preferred_cores");
}

export function setPreferredCores(preferences: PlatformCorePreference[]): Promise<void> {
  return invoke("set_preferred_cores", { preferences });
}

/** Platformos `slug` -> rekomenduojamų core'o pavadinimų tvarka (pirmas rastas laimi). */
export function getCorePriority(): Promise<Record<string, string[]>> {
  return invoke("get_core_priority");
}

export function getVideoSettings(): Promise<VideoSettings> {
  return invoke("get_video_settings");
}

export function setVideoSettings(value: VideoSettings): Promise<void> {
  return invoke("set_video_settings", { value });
}

export function getAudioSettings(): Promise<AudioSettings> {
  return invoke("get_audio_settings");
}

export function setAudioSettings(value: AudioSettings): Promise<void> {
  return invoke("set_audio_settings", { value });
}

export function listAudioDevices(): Promise<string[]> {
  return invoke("list_audio_devices");
}

/** Paleidžia žaidimą — LAUKIA, kol `nullbyte-emu` realiai patvirtina (arba atmeta) `Load`
 * (MVP.md P9.1), tad `await` trukmė apima ne tik proceso spawn'inimą, bet ir core/ROM
 * įkėlimą. Klaida (`AppError` `{kind, message}` forma) — nueik per `errorMessage()`
 * ($lib/utils/format) prieš rodant vartotojui. */
export function startGame(id: number): Promise<void> {
  return invoke("start_game", { id });
}

export function stopGame(): Promise<void> {
  return invoke("stop_game");
}

export function isGameRunning(): Promise<boolean> {
  return invoke("is_game_running");
}
