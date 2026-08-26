// Vieninga klaidų apdorojimo vieta (MVP.md P9.3) — Tauri `invoke()` reject'ina su Rust
// `AppError`'io serializuota forma `{ kind, message }` (CLAUDE.md §6.1). `message` yra
// VIDINIS, LIETUVIŠKAS tekstas (kodo komentarai/klaidos — CLAUDE.md §1), tad NIEKADA
// nerodomas tiesiogiai vartotojui — TIK loginamas konsolėn debug'inimui. Vartotojui rodomas
// TIK kuruotas, ANGLIŠKAS tekstas iš žemiau esančio katalogo (CLAUDE.md §7.5: „Visas
// vartotojui matomas UI tekstas — angliškai").

import { toast } from "svelte-sonner";

/** `AppError.kind()`/`CoreError.kind()` (žr. crates/nullbyte-app/src/error.rs ir
 * crates/nullbyte-core/src/error.rs — `AppError::Core` DELEGUOJA į `CoreError::kind()`,
 * tad abiejų variantų kind'ai gali čia atsirasti) → kuruotas, veiksmingas anglų tekstas. */
const KIND_MESSAGES: Record<string, string> = {
  io: "A file operation failed. Check that the file or folder still exists and is accessible.",
  database: "A database error occurred. Try restarting Nullbyte.",
  network: "A network request failed. Check your internet connection and try again.",
  core_load:
    "This libretro core could not be loaded — it may be missing or corrupted. Check Settings → Cores.",
  api_version: "This core uses an incompatible libretro API version and can't be used.",
  rom_load: "The core rejected this ROM file — it may be corrupted or in an unsupported format.",
  missing_bios: "A required BIOS file is missing. Add it to the system files folder and try again.",
  unsupported_pixel_format: "This core requested a video format Nullbyte doesn't support yet.",
  save_state: "The save state couldn't be used — it may be from a different core version.",
  sram: "The in-game save file couldn't be read or written.",
  other: "Something went wrong.",
};

const FALLBACK_MESSAGE = KIND_MESSAGES.other;

interface BackendError {
  kind?: unknown;
  message?: unknown;
}

function isBackendError(error: unknown): error is BackendError {
  return typeof error === "object" && error !== null && ("kind" in error || "message" in error);
}

/** Kuruotas, angliškas tekstas BET KOKIAI klaidai — Rust `{kind, message}` (backend
 * komandos), JS `Error`, ar bet kas kita. Neatidėliotinai loginą pilną žalią klaidą
 * konsolėn (dev'inimui/bug report'ams), bet NIEKADA jos negrąžina/nerodo tiesiogiai. */
export function describeError(error: unknown): string {
  if (isBackendError(error) && typeof error.kind === "string") {
    console.error(`[${error.kind}]`, error.message ?? error);
    return KIND_MESSAGES[error.kind] ?? FALLBACK_MESSAGE;
  }
  console.error(error);
  return FALLBACK_MESSAGE;
}

/** Numatytasis būdas parodyti nepavykusią operaciją, kuri neturi savo dedikuotos vietos
 * UI (formos validacijos klaidoms geriau tinka inline tekstas prie lauko, ne toast). */
export function showErrorToast(error: unknown): void {
  toast.error(describeError(error));
}
