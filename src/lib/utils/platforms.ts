// Platformų vizualiniai metaduomenys — spalvos TIK per Tailwind `chart-*` CSS kintamuosius
// (CLAUDE.md §7.5: jokių hardcode'intų hex reikšmių komponentuose).

const ACCENT_CLASSES = ["bg-chart-1", "bg-chart-2", "bg-chart-3", "bg-chart-4", "bg-chart-5"];

/** Deterministinė (pagal `platformId`) fono spalva placeholder'iui be viršelio. */
export function platformAccentClass(platformId: number): string {
  return ACCENT_CLASSES[platformId % ACCENT_CLASSES.length];
}
