// Tipizuoti Tauri `invoke` wrapper'iai (CLAUDE.md §7.3) — vienas failas per domeną,
// kai jų prisikaups daugiau; kol kas viskas telpa čia.

import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  Game,
  GameFilter,
  PlatformSummary,
  ScrapeProgress,
  ScrapeSummary,
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
