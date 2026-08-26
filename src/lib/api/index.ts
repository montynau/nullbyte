// Tipizuoti Tauri `invoke` wrapper'iai (CLAUDE.md §7.3) — vienas failas per domeną,
// kai jų prisikaups daugiau; kol kas viskas telpa čia.

import { invoke } from "@tauri-apps/api/core";
import type { AppInfo, Game, GameFilter, PlatformSummary } from "$lib/types";

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
