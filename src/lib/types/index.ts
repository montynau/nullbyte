// Rankiniu būdu sinchronizuojama su Rust struct'ais, kurie kerta Tauri IPC.
// CLAUDE.md §7.3: keitus Rust pusę, TUOJ PAT atnaujink čia.

/** Atitinka `AppInfo` (src-tauri/src/lib.rs, komanda `get_app_info`). */
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
