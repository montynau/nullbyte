// Formatavimo pagalbinės funkcijos žaidimo detalių puslapiui (P7.4) ir vėlesniam UI darbui.

export function formatPlayTime(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m`;
  return "< 1m";
}

export function formatDate(unixSeconds: number | null): string {
  if (!unixSeconds) return "Never";
  return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

export function formatFileSize(bytes: number): string {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

/** Rust `AppError` serializuojasi kaip `{ kind, message }` (CLAUDE.md §6.1) — Tauri `invoke()`
 * reject'ina su ta pačia forma. Naudinga bet kuriam `catch` blokui, kuris rodo klaidą UI. */
export function errorMessage(error: unknown): string {
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}
