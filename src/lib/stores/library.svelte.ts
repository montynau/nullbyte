// Bibliotekos būvis (Svelte 5 runes, CLAUDE.md §7.1) — dalinamasi tarp Sidebar, TopBar,
// CommandPalette ir bibliotekos grid'o (P7.2).

import { listGames, listPlatforms } from "$lib/api";
import type { Game, GameFilter, PlatformSummary, SortDirection, SortField } from "$lib/types";

const DEFAULT_FILTER: GameFilter = {
  platformId: null,
  search: null,
  favoritesOnly: false,
  sort: "title",
  sortDirection: "asc",
  page: 0,
  pageSize: 200,
};

class LibraryStore {
  platforms = $state<PlatformSummary[]>([]);
  games = $state<Game[]>([]);
  filter = $state<GameFilter>({ ...DEFAULT_FILTER });
  loading = $state(false);
  error = $state<string | null>(null);

  totalGameCount = $derived(this.platforms.reduce((sum, p) => sum + p.gameCount, 0));

  async loadPlatforms() {
    this.platforms = await listPlatforms();
  }

  async loadGames() {
    this.loading = true;
    this.error = null;
    try {
      this.games = await listGames(this.filter);
    } catch (e) {
      this.error = e instanceof Error ? e.message : String(e);
    } finally {
      this.loading = false;
    }
  }

  setSearch(search: string) {
    this.filter = { ...this.filter, search: search.trim() || null, page: 0 };
  }

  selectAll() {
    this.filter = { ...DEFAULT_FILTER };
  }

  selectPlatform(platformId: number) {
    this.filter = { ...DEFAULT_FILTER, platformId };
  }

  selectFavorites() {
    this.filter = { ...DEFAULT_FILTER, favoritesOnly: true };
  }

  selectRecentlyPlayed() {
    this.filter = { ...DEFAULT_FILTER, sort: "lastPlayed", sortDirection: "desc" };
  }

  setSort(sort: SortField, sortDirection: SortDirection) {
    this.filter = { ...this.filter, sort, sortDirection, page: 0 };
  }
}

export const library = new LibraryStore();
