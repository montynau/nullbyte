<script lang="ts">
  import * as Command from "$lib/components/ui/command/index.js";
  import { library } from "$lib/stores/library.svelte";
  import LibraryIcon from "@lucide/svelte/icons/library";
  import StarIcon from "@lucide/svelte/icons/star";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";

  let open = $state(false);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      open = !open;
    }
  }

  function pick(action: () => void) {
    action();
    open = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Command.Dialog bind:open title="Command Palette" description="Quick navigation across the library">
  <Command.Input placeholder="Search views or platforms..." />
  <Command.List>
    <Command.Empty>No results found.</Command.Empty>
    <Command.Group heading="Views">
      <Command.Item onSelect={() => pick(() => library.selectAll())}>
        <LibraryIcon class="size-4" />
        All Games
      </Command.Item>
      <Command.Item onSelect={() => pick(() => library.selectFavorites())}>
        <StarIcon class="size-4" />
        Favorites
      </Command.Item>
      <Command.Item onSelect={() => pick(() => library.selectRecentlyPlayed())}>
        <ClockIcon class="size-4" />
        Recently Played
      </Command.Item>
    </Command.Group>
    {#if library.platforms.length > 0}
      <Command.Group heading="Platforms">
        {#each library.platforms as platform (platform.id)}
          <Command.Item onSelect={() => pick(() => library.selectPlatform(platform.id))}>
            <Gamepad2Icon class="size-4" />
            {platform.name}
            <Command.Shortcut>{platform.gameCount}</Command.Shortcut>
          </Command.Item>
        {/each}
      </Command.Group>
    {/if}
  </Command.List>
</Command.Dialog>
