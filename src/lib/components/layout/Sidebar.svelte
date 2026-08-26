<script lang="ts">
  import { library } from "$lib/stores/library.svelte";
  import LibraryIcon from "@lucide/svelte/icons/library";
  import StarIcon from "@lucide/svelte/icons/star";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";

  const isAllActive = $derived(
    library.filter.platformId == null &&
      !library.filter.favoritesOnly &&
      library.filter.sort !== "lastPlayed",
  );
  const isFavoritesActive = $derived(!!library.filter.favoritesOnly);
  const isRecentActive = $derived(
    library.filter.sort === "lastPlayed" && !library.filter.favoritesOnly,
  );
</script>

<nav
  aria-label="Library navigation"
  class="border-sidebar-border bg-sidebar text-sidebar-foreground flex h-full w-56 shrink-0 flex-col gap-4 overflow-y-auto border-r p-3"
>
  <ul class="flex flex-col gap-0.5">
    <li>
      <button
        type="button"
        onclick={() => library.selectAll()}
        aria-current={isAllActive}
        class="hover:bg-sidebar-accent aria-[current=true]:bg-sidebar-accent aria-[current=true]:text-sidebar-primary flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
      >
        <LibraryIcon class="size-4" />
        <span class="flex-1 text-left">All</span>
        <span class="text-muted-foreground text-xs tabular-nums">{library.totalGameCount}</span>
      </button>
    </li>
    <li>
      <button
        type="button"
        onclick={() => library.selectFavorites()}
        aria-current={isFavoritesActive}
        class="hover:bg-sidebar-accent aria-[current=true]:bg-sidebar-accent aria-[current=true]:text-sidebar-primary flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
      >
        <StarIcon class="size-4" />
        <span class="flex-1 text-left">Favorites</span>
      </button>
    </li>
    <li>
      <button
        type="button"
        onclick={() => library.selectRecentlyPlayed()}
        aria-current={isRecentActive}
        class="hover:bg-sidebar-accent aria-[current=true]:bg-sidebar-accent aria-[current=true]:text-sidebar-primary flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
      >
        <ClockIcon class="size-4" />
        <span class="flex-1 text-left">Recently Played</span>
      </button>
    </li>
  </ul>

  {#if library.platforms.length > 0}
    <div class="flex flex-col gap-0.5">
      <h2 class="text-muted-foreground px-2 pb-1 text-xs font-medium tracking-wide uppercase">
        Platforms
      </h2>
      <ul class="flex flex-col gap-0.5">
        {#each library.platforms as platform (platform.id)}
          <li>
            <button
              type="button"
              onclick={() => library.selectPlatform(platform.id)}
              aria-current={library.filter.platformId === platform.id}
              class="hover:bg-sidebar-accent aria-[current=true]:bg-sidebar-accent aria-[current=true]:text-sidebar-primary flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors"
            >
              <Gamepad2Icon class="size-4" />
              <span class="flex-1 truncate text-left">{platform.name}</span>
              <span class="text-muted-foreground text-xs tabular-nums">{platform.gameCount}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</nav>
