// Bendra aplikacijos info (versija, katalogai) — CLAUDE.md §7.1 Svelte 5 runes store.
// `mediaDir` reikalingas GameCard'ui sudaryti absoliutų viršelio kelią `convertFileSrc()`.

import { getAppInfo } from "$lib/api";
import type { AppInfo } from "$lib/types";

class AppStore {
  info = $state<AppInfo | null>(null);

  async load() {
    this.info = await getAppInfo();
  }
}

export const app = new AppStore();
