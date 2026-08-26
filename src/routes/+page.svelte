<script lang="ts">
  import { library } from "$lib/stores/library.svelte";
  import GameGrid from "$lib/components/library/GameGrid.svelte";
  import { Skeleton } from "$lib/components/ui/skeleton";

  $effect(() => {
    // Priklauso nuo library.filter — persikrauna kaskart pasikeitus (Sidebar/TopBar/paletė).
    void library.filter;
    library.loadGames();
  });
</script>

{#if library.loading && library.games.length === 0}
  <div class="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-4 p-4">
    {#each [...Array(18).keys()] as i (i)}
      <Skeleton class="aspect-[3/4] w-full" />
    {/each}
  </div>
{:else if library.error}
  <p class="text-destructive p-4 text-sm">{library.error}</p>
{:else if library.games.length === 0}
  <p class="text-muted-foreground p-4 text-sm">No games found. Add a ROM directory in settings.</p>
{:else}
  <GameGrid games={library.games} />
{/if}
