<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { resolve } from "$app/paths";
  import { app } from "$lib/stores/app.svelte";
  import { platformAccentClass } from "$lib/utils/platforms";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import VideoPreview from "./VideoPreview.svelte";
  import type { Game } from "$lib/types";

  let { game }: { game: Game } = $props();

  let imgFailed = $state(false);

  const coverSrc = $derived(
    game.coverPath && app.info ? convertFileSrc(`${app.info.mediaDir}/${game.coverPath}`) : null,
  );
  const showPlaceholder = $derived(!coverSrc || imgFailed);
</script>

<a
  href={resolve("/game/[id]", { id: String(game.id) })}
  class="border-border bg-card group relative flex h-full w-full flex-col overflow-hidden rounded-lg border transition-transform duration-150 hover:-translate-y-1 hover:shadow-lg"
>
  {#if showPlaceholder}
    <div
      class={`flex h-full w-full flex-col items-center justify-center gap-2 p-3 text-center opacity-25 ${platformAccentClass(game.platformId)}`}
    >
      <Gamepad2Icon class="text-foreground size-8" />
    </div>
    <span
      class="text-foreground/80 pointer-events-none absolute inset-x-0 top-1/2 line-clamp-3 -translate-y-1/2 px-3 text-center text-xs font-medium"
    >
      {game.title}
    </span>
  {:else}
    <img
      src={coverSrc}
      alt={game.title}
      loading="lazy"
      class="h-full w-full object-cover"
      onerror={() => (imgFailed = true)}
    />
  {/if}

  <VideoPreview {game} mediaServerPort={app.info?.mediaServerPort ?? null} />

  <div
    class="pointer-events-none absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/85 to-transparent p-2 pt-6 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
  >
    <p class="truncate text-xs font-medium text-white">{game.title}</p>
  </div>
</a>
