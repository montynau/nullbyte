<script lang="ts">
  import { resolve } from "$app/paths";
  import PathsPanel from "$lib/components/settings/PathsPanel.svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
</script>

<div class="h-full overflow-y-auto p-6">
  <div class="mb-6 flex items-center gap-3">
    <a
      href={resolve("/")}
      class="border-border hover:bg-muted inline-flex size-8 items-center justify-center rounded-lg border"
    >
      <ArrowLeftIcon class="size-4" />
    </a>
    <h1 class="text-lg font-semibold">Settings</h1>
  </div>

  <div class="max-w-2xl">
    <PathsPanel />
  </div>
</div>
