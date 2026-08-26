<script lang="ts">
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import GameCard from "./GameCard.svelte";
  import type { Game } from "$lib/types";

  let { games }: { games: Game[] } = $props();

  const CARD_MIN_WIDTH = 160;
  const GAP = 16;

  let scrollElement: HTMLDivElement | undefined = $state();
  let containerWidth = $state(0);

  const columns = $derived(
    Math.max(1, Math.floor((containerWidth + GAP) / (CARD_MIN_WIDTH + GAP))),
  );
  const cardWidth = $derived(
    columns > 0 && containerWidth > 0
      ? (containerWidth - GAP * (columns - 1)) / columns
      : CARD_MIN_WIDTH,
  );
  const cardHeight = $derived(cardWidth * (4 / 3));
  const rowCount = $derived(Math.ceil(games.length / columns));
  const columnIndexes = $derived(Array.from({ length: columns }, (_, i) => i));

  // Tik pradinė reikšmė; realus sinchronizavimas vyksta žemiau per $effect + setOptions()
  // kaskart pasikeitus rowCount/cardHeight.
  // svelte-ignore state_referenced_locally
  const rowVirtualizer = createVirtualizer<HTMLDivElement, HTMLDivElement>({
    count: rowCount,
    getScrollElement: () => scrollElement ?? null,
    estimateSize: () => cardHeight + GAP,
    overscan: 4,
  });

  $effect(() => {
    $rowVirtualizer.setOptions({
      count: rowCount,
      estimateSize: () => cardHeight + GAP,
    });
    $rowVirtualizer.measure();
  });
</script>

<div bind:this={scrollElement} bind:clientWidth={containerWidth} class="h-full overflow-y-auto p-4">
  <div style:height="{$rowVirtualizer.getTotalSize()}px" class="relative w-full">
    {#each $rowVirtualizer.getVirtualItems() as row (row.index)}
      <div
        class="absolute top-0 left-0 flex w-full"
        style:height="{row.size}px"
        style:transform="translateY({row.start}px)"
        style:gap="{GAP}px"
      >
        {#each columnIndexes as col (col)}
          {@const game = games[row.index * columns + col]}
          {#if game}
            <div style:width="{cardWidth}px">
              <GameCard {game} />
            </div>
          {/if}
        {/each}
      </div>
    {/each}
  </div>
</div>
