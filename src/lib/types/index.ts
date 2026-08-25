// Rankiniu būdu sinchronizuojama su Rust struct'ais, kurie kerta Tauri IPC.
// CLAUDE.md §7.3: keitus Rust pusę, TUOJ PAT atnaujink čia.

/** Atitinka `AppInfo` (crates/nullbyte-app/src/lib.rs, komanda `get_app_info`). */
export interface AppInfo {
  version: string;
  platform: string;
  dataDir: string;
  coresDir: string;
  systemDir: string;
  savesDir: string;
  statesDir: string;
  mediaDir: string;
  dbPath: string;
}

/** Atitinka `Platform` (crates/nullbyte-app/src/db/models.rs). */
export interface Platform {
  id: number;
  slug: string;
  name: string;
  screenscraperId: number | null;
  /** Kableliais atskirti plėtiniai, be taško (pvz. `"sfc,smc,fig"`). */
  extensions: string;
}

/** Atitinka `Game` (crates/nullbyte-app/src/db/models.rs). */
export interface Game {
  id: number;
  platformId: number;
  title: string;
  sortTitle: string;
  /** ABSOLIUTUS kelias (žr. Rust pusės doc komentarą dėl KODĖL, ne santykinis). */
  romPath: string;
  romSize: number;
  archiveInner: string | null;
  crc32: string | null;
  md5: string | null;
  sha1: string | null;
  description: string | null;
  developer: string | null;
  publisher: string | null;
  genre: string | null;
  players: number | null;
  releaseDate: string | null;
  rating: number | null;
  region: string | null;
  coverPath: string | null;
  screenshotPath: string | null;
  wheelPath: string | null;
  videoPath: string | null;
  /** `"pending" | "ok" | "notfound" | "error"`. */
  scrapeStatus: string;
  scrapedAt: number | null;
  lastPlayed: number | null;
  playCount: number;
  playTimeSeconds: number;
  favorite: boolean;
  addedAt: number;
  fileMtime: number;
}

/** Atitinka `PlatformSummary` (crates/nullbyte-app/src/db/games.rs) — `#[serde(flatten)]`
 * sujungia `Platform` laukus su `gameCount` į vieną plokščią objektą. */
export interface PlatformSummary extends Platform {
  gameCount: number;
}

/** Atitinka `SortField` (crates/nullbyte-app/src/db/games.rs). */
export type SortField = "title" | "lastPlayed" | "addedAt";

/** Atitinka `SortDirection` (crates/nullbyte-app/src/db/games.rs). */
export type SortDirection = "asc" | "desc";

/** Atitinka `GameFilter` (crates/nullbyte-app/src/db/games.rs, komanda `list_games`). Visi
 * laukai neprivalomi TS pusėje irgi — Rust `#[serde(default)]` juos užpildo, jei praleisti. */
export interface GameFilter {
  platformId?: number | null;
  search?: string | null;
  favoritesOnly?: boolean;
  sort?: SortField;
  sortDirection?: SortDirection;
  page?: number;
  pageSize?: number;
}
