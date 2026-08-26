// Bendra aplikacijos info (versija, katalogai) — CLAUDE.md §7.1 Svelte 5 runes store.
// `mediaDir` reikalingas GameCard'ui sudaryti absoliutų viršelio kelią `convertFileSrc()`.

import { getAppInfo } from "$lib/api";
import { showErrorToast } from "$lib/utils/errors";
import type { AppInfo } from "$lib/types";

class AppStore {
  info = $state<AppInfo | null>(null);

  async load() {
    try {
      this.info = await getAppInfo();
    } catch (error) {
      showErrorToast(error);
    }
  }
}

export const app = new AppStore();
