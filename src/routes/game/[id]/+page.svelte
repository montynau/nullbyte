<script lang="ts">
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getGame, isGameRunning, scrapeGame, setFavorite, startGame } from "$lib/api";
  import { app } from "$lib/stores/app.svelte";
  import { platformAccentClass } from "$lib/utils/platforms";
  import { formatDate, formatFileSize, formatPlayTime } from "$lib/utils/format";
  import { describeError, showErrorToast } from "$lib/utils/errors";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
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
  let launching = $state(false);
  let running = $state(false);
  let launchError = $state<string | null>(null);

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

  // `nullbyte-emu` turi SAVO winit langą (ADR-016) — jis NEĮDĖTAS į šį Tauri langą, tad
  // vienintelis būdas šiam puslapiui sužinoti, kad sesija pasibaigė, yra "game-closed"
  // event'as (žr. `commands::emulator::start_game` doc). Atnaujina statistiką (last played/
  // play count/time), jei tai BUVO šis žaidimas — kitaip ignoruoja (vartotojas galėjo
  // paleisti, tada pereiti į kitą žaidimo puslapį, kol pirmasis dar veikė).
  $effect(() => {
    const unlisten = listen<number>("game-closed", (event) => {
      if (event.payload === gameId) {
        running = false;
        load(gameId);
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  });

  $effect(() => {
    isGameRunning().then((value) => {
      running = value;
    });
  });

  async function play() {
    if (!game || launching || running) return;
    launching = true;
    launchError = null;
    try {
      await startGame(game.id);
      running = true;
    } catch (error) {
      launchError = describeError(error);
    } finally {
      launching = false;
    }
  }

  async function toggleFavorite() {
    if (!game) return;
    const next = !game.favorite;
    try {
      await setFavorite(game.id, next);
      game = { ...game, favorite: next };
    } catch (error) {
      showErrorToast(error);
    }
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
    } catch (error) {
      showErrorToast(error);
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

<div class="relative h-full overflow-y-auto">
  <a
    href={resolve("/")}
    class="bg-background/80 absolute top-4 left-4 z-10 inline-flex size-8 items-center justify-center rounded-lg backdrop-blur"
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
        <img src={heroSrc} alt="" class="h-full w-full object-cover" />
      {/if}
      <div
        class="from-background absolute inset-0 bg-gradient-to-t via-transparent to-transparent"
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
        <Button onclick={play} disabled={launching || running}>
          <PlayIcon class={launching ? "size-4 animate-pulse" : "size-4"} />
          {#if running}
            Playing
          {:else if launching}
            Launching...
          {:else}
            Play
          {/if}
        </Button>

        <Button variant={game.favorite ? "default" : "outline"} onclick={toggleFavorite}>
          <StarIcon class={game.favorite ? "size-4 fill-current" : "size-4"} />
          {game.favorite ? "Favorited" : "Favorite"}
        </Button>

        <Button variant="outline" onclick={rescrape} disabled={scraping}>
          <RefreshCwIcon class={scraping ? "size-4 animate-spin" : "size-4"} />
          {scraping ? (scrapeStatusText ?? "Scraping...") : "Re-scrape"}
        </Button>
      </div>

      {#if launchError}
        <p class="text-destructive text-sm">{launchError}</p>
      {/if}

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
