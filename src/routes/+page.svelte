<script lang="ts">
  import { library } from "$lib/stores/library.svelte";

  $effect(() => {
    // Priklauso nuo library.filter — persikrauna kaskart pasikeitus (Sidebar/TopBar/paletė).
    void library.filter;
    library.loadGames();
  });
</script>

<div class="p-4">
  {#if library.loading}
    <p class="text-muted-foreground text-sm">Kraunama...</p>
  {:else if library.error}
    <p class="text-destructive text-sm">{library.error}</p>
  {:else if library.games.length === 0}
    <p class="text-muted-foreground text-sm">Žaidimų nerasta. Pridėk ROM katalogą nustatymuose.</p>
  {:else}
    <!-- Laikinas sąrašas — P7.2 pakeis tikru virtualizuotu grid'u su viršeliais. -->
    <ul class="flex flex-col gap-1">
      {#each library.games as game (game.id)}
        <li class="hover:bg-accent rounded-md px-2 py-1.5 text-sm">{game.title}</li>
      {/each}
    </ul>
  {/if}
</div>
