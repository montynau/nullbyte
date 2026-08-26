<script lang="ts">
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getGame, scrapeGame, setFavorite } from "$lib/api";
  import { app } from "$lib/stores/app.svelte";
  import { platformAccentClass } from "$lib/utils/platforms";
  import { formatDate, formatFileSize, formatPlayTime } from "$lib/utils/format";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import PlayIcon from "@lucide/svelte/icons/play";
  import StarIcon from "@lucide/svelte/icons/star";
  import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import type { Game } from "$lib/types";

  const gameId = $derived(Number(page.params.id));

  let game = $state<Game | null>(null);
  let notFound = $state(false);
  let scraping = $state(false);
  let scrapeStatusText = $state<string | null>(null);

  async function load(id: number) {
    game = null;
    notFound = false;
    const result = await getGame(id);
    if (!result) {
      notFound = true;
      return;
    }
    game = result;
  }

  $effect(() => {
    load(gameId);
  });

  async function toggleFavorite() {
    if (!game) return;
    const next = !game.favorite;
    await setFavorite(game.id, next);
    game = { ...game, favorite: next };
  }

  async function rescrape() {
    if (!game || scraping) return;
    scraping = true;
    scrapeStatusText = "Scraping...";
    try {
      await scrapeGame(game.id, (progress) => {
        scrapeStatusText = progress.status;
      });
      await load(game.id);
    } finally {
      scraping = false;
      scrapeStatusText = null;
    }
  }

  const heroSrc = $derived(
    game?.screenshotPath && app.info
      ? convertFileSrc(`${app.info.mediaDir}/${game.screenshotPath}`)
      : null,
  );
  const wheelSrc = $derived(
    game?.wheelPath && app.info ? convertFileSrc(`${app.info.mediaDir}/${game.wheelPath}`) : null,
  );

  const metaRows = $derived(
    game
      ? [
          { label: "Developer", value: game.developer },
          { label: "Publisher", value: game.publisher },
          { label: "Genre", value: game.genre },
          { label: "Players", value: game.players != null ? String(game.players) : null },
          { label: "Release date", value: game.releaseDate },
          { label: "Region", value: game.region },
          { label: "Rating", value: game.rating != null ? `${game.rating.toFixed(1)} / 20` : null },
        ].filter((row) => row.value)
      : [],
  );
</script>

<div class="h-full overflow-y-auto">
  <a
    href={resolve("/")}
    class="bg-background/80 fixed top-4 left-4 z-10 inline-flex size-8 items-center justify-center rounded-lg backdrop-blur"
  >
    <ArrowLeftIcon class="size-4" />
  </a>

  {#if notFound}
    <div class="text-muted-foreground p-8 text-center text-sm">Game not found.</div>
  {:else if !game}
    <div class="text-muted-foreground p-8 text-sm">Loading...</div>
  {:else}
    <div
      class={`relative h-56 w-full overflow-hidden ${heroSrc ? "" : platformAccentClass(game.platformId)}`}
    >
      {#if heroSrc}
        <img src={heroSrc} alt="" class="h-full w-full scale-110 object-cover opacity-50 blur-md" />
      {/if}
      <div
        class="from-background via-background/70 absolute inset-0 bg-gradient-to-t to-transparent"
      ></div>
      <div class="absolute inset-x-0 bottom-0 flex items-end gap-4 p-6">
        {#if wheelSrc}
          <img
            src={wheelSrc}
            alt={game.title}
            class="max-h-24 max-w-xs object-contain drop-shadow-lg"
          />
        {:else}
          <h1 class="text-3xl font-bold drop-shadow-lg">{game.title}</h1>
        {/if}
      </div>
    </div>

    <div class="flex flex-col gap-6 p-6">
      <div class="flex flex-wrap items-center gap-2">
        <Tooltip.Provider>
          <Tooltip.Root>
            <Tooltip.Trigger>
              <Button disabled>
                <PlayIcon class="size-4" />
                Play
              </Button>
            </Tooltip.Trigger>
            <Tooltip.Content>Game launching — coming soon (P9.1)</Tooltip.Content>
          </Tooltip.Root>
        </Tooltip.Provider>

        <Button variant={game.favorite ? "default" : "outline"} onclick={toggleFavorite}>
          <StarIcon class={game.favorite ? "size-4 fill-current" : "size-4"} />
          {game.favorite ? "Favorited" : "Favorite"}
        </Button>

        <Button variant="outline" onclick={rescrape} disabled={scraping}>
          <RefreshCwIcon class={scraping ? "size-4 animate-spin" : "size-4"} />
          {scraping ? (scrapeStatusText ?? "Scraping...") : "Re-scrape"}
        </Button>
      </div>

      {#if game.description}
        <p class="text-foreground/90 max-w-3xl text-sm leading-relaxed">{game.description}</p>
      {/if}

      {#if metaRows.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each metaRows as row (row.label)}
            <Badge variant="secondary">{row.label}: {row.value}</Badge>
          {/each}
        </div>
      {/if}

      <div class="border-border grid grid-cols-2 gap-4 border-t pt-6 text-sm sm:grid-cols-4">
        <div>
          <p class="text-muted-foreground text-xs">Last played</p>
          <p class="font-medium">{formatDate(game.lastPlayed)}</p>
        </div>
        <div>
          <p class="text-muted-foreground text-xs">Play count</p>
          <p class="font-medium">{game.playCount}</p>
        </div>
        <div>
          <p class="text-muted-foreground text-xs">Play time</p>
          <p class="font-medium">{formatPlayTime(game.playTimeSeconds)}</p>
        </div>
        <div>
          <p class="text-muted-foreground text-xs">ROM size</p>
          <p class="font-medium">{formatFileSize(game.romSize)}</p>
        </div>
      </div>

      <div class="border-border border-t pt-6">
        <h2 class="mb-2 text-sm font-medium">Save states</h2>
        <p class="text-muted-foreground text-sm">No save states yet — coming in P8.1.</p>
      </div>
    </div>
  {/if}
</div>
