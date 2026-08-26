<script lang="ts">
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import { get } from "svelte/store";
  import GameCard from "./GameCard.svelte";
  import type { Game } from "$lib/types";

  let { games }: { games: Game[] } = $props();

  const ROW_HEIGHT = 220;
  const GAP = 16;
  // Placeholder'ių (dar nescrape'intų žaidimų, `coverWidth`/`coverHeight` = null) numatytoji
  // proporcija — jokia geresnė prielaida neįmanoma be tikrų matmenų.
  const DEFAULT_ASPECT = 3 / 4;

  let scrollElement: HTMLDivElement | undefined = $state();
  let containerWidth = $state(0);

  function aspectRatioFor(game: Game): number {
    if (game.coverWidth && game.coverHeight) {
      return game.coverWidth / game.coverHeight;
    }
    return DEFAULT_ASPECT;
  }

  interface PackedCard {
    game: Game;
    width: number;
  }

  // ADR-021: fiksuota AUKŠTIS (`ROW_HEIGHT`), plotis pagal TIKRĄ viršelio proporciją — vietoj
  // vienodo stulpelių tinklelio (P7.2), nes realūs viršeliai LABAI skiriasi tarp platformų
  // (PSX kvadratas, SNES platus, Genesis aukštas — patikrinta realiais matmenimis). Eilutės
  // pakuojamos „iš eilės kol tilpsta" (ragged-right), NE tobulai išlygintos per visą plotį —
  // vartotojas prašė TIK „fiksuota aukštis, plotis kad visas tilptu", ne edge-to-edge justify.
  const rows = $derived.by(() => {
    if (containerWidth <= 0) return [] as PackedCard[][];
    const packed: PackedCard[][] = [];
    let current: PackedCard[] = [];
    let currentWidth = 0;
    for (const game of games) {
      const width = ROW_HEIGHT * aspectRatioFor(game);
      const widthWithGap = current.length === 0 ? width : width + GAP;
      if (current.length > 0 && currentWidth + widthWithGap > containerWidth) {
        packed.push(current);
        current = [];
        currentWidth = 0;
      }
      current.push({ game, width });
      currentWidth += current.length === 1 ? width : width + GAP;
    }
    if (current.length > 0) packed.push(current);
    return packed;
  });

  // Tik pradinė reikšmė; realus sinchronizavimas vyksta žemiau per $effect + setOptions().
  // svelte-ignore state_referenced_locally
  const rowVirtualizer = createVirtualizer<HTMLDivElement, HTMLDivElement>({
    count: rows.length,
    getScrollElement: () => scrollElement ?? null,
    estimateSize: () => ROW_HEIGHT + GAP,
    overscan: 4,
  });

  // KRITIŠKA: `get()`, NE `$rowVirtualizer` — pastarasis būtų reaktyvus skaitymas, o
  // `setOptions()`/`measure()` PATYS priverčia store'ą pranešti apie pasikeitimą (measure
  // perskaičiuoja dydžius). `$`-prefiksas čia sukurtų begalinę kilpą: effect skaito store'ą →
  // store pasikeičia → effect vėl paleidžiamas → ir taip toliau (realiai patikrinta, žr.
  // MVP.md ADR-019 — Svelte `effect_update_depth_exceeded`).
  $effect(() => {
    const virtualizer = get(rowVirtualizer);
    virtualizer.setOptions({
      count: rows.length,
      estimateSize: () => ROW_HEIGHT + GAP,
    });
    virtualizer.measure();
  });
</script>

<div bind:this={scrollElement} bind:clientWidth={containerWidth} class="h-full overflow-y-auto p-4">
  <div style:height="{$rowVirtualizer.getTotalSize()}px" class="relative w-full">
    {#each $rowVirtualizer.getVirtualItems() as row (row.index)}
      <div
        class="absolute top-0 left-0 flex"
        style:height="{ROW_HEIGHT}px"
        style:transform="translateY({row.start}px)"
        style:gap="{GAP}px"
      >
        {#each rows[row.index] ?? [] as card (card.game.id)}
          <div style:width="{card.width}px" style:height="{ROW_HEIGHT}px" class="shrink-0">
            <GameCard game={card.game} />
          </div>
        {/each}
      </div>
    {/each}
  </div>
</div>
