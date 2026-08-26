// Globalus „vienas aktyvus preview" būvis (CLAUDE.md §7.1 runes store, P7.3) — `GameCard`/
// `VideoPreview` egzemplioriai dalinasi šiuo singleton'u, kad niekada negrotų 2 video vienu metu.

class VideoPreviewStore {
  activeGameId = $state<number | null>(null);
}

export const videoPreview = new VideoPreviewStore();
