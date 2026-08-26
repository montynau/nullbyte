<script lang="ts">
  import { resolve } from "$app/paths";
  import { library } from "$lib/stores/library.svelte";
  import GameGrid from "$lib/components/library/GameGrid.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import { Button } from "$lib/components/ui/button";

  $effect(() => {
    // Priklauso nuo library.filter — persikrauna kaskart pasikeitus (Sidebar/TopBar/paletė).
    void library.filter;
    library.loadGames();
  });

  // P9.3: skirtingos tuščios būsenos priežastys — ar TAI dėl siauro filtro (galima
  // išvalyti), ar apskritai biblioteka be žaidimų (reikia pridėti ROM katalogą). Abi
  // situacijos VIENODAI turi `games.length === 0`, tad reikia atskirti pačiam filtrui.
  const filterActive = $derived(
    Boolean(library.filter.search) ||
      library.filter.platformId != null ||
      library.filter.favoritesOnly,
  );
</script>

{#if library.loading && library.games.length === 0}
  <div class="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-4 p-4">
    {#each [...Array(18).keys()] as i (i)}
      <Skeleton class="aspect-[3/4] w-full" />
    {/each}
  </div>
{:else if library.error}
  <p class="text-destructive p-4 text-sm">{library.error}</p>
{:else if library.games.length === 0 && filterActive}
  <div class="flex flex-col items-center gap-3 p-8 text-center">
    <p class="text-muted-foreground text-sm">No games match this filter.</p>
    <Button variant="outline" size="sm" onclick={() => library.selectAll()}>Clear filter</Button>
  </div>
{:else if library.games.length === 0}
  <div class="flex flex-col items-center gap-3 p-8 text-center">
    <p class="text-muted-foreground text-sm">
      No games yet. Add a ROM directory and scan your library to get started.
    </p>
    <Button size="sm" href={resolve("/settings")}>Go to Settings</Button>
  </div>
{:else}
  <GameGrid games={library.games} />
{/if}
