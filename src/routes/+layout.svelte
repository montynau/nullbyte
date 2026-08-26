<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import CommandPalette from "$lib/components/layout/CommandPalette.svelte";
  import { library } from "$lib/stores/library.svelte";
  import { app } from "$lib/stores/app.svelte";

  let { children } = $props();

  onMount(() => {
    library.loadPlatforms();
    app.load();
  });
</script>

<div class="bg-background flex h-screen w-screen overflow-hidden">
  <Sidebar />
  <div class="flex min-w-0 flex-1 flex-col">
    <TopBar />
    <main class="min-h-0 flex-1 overflow-hidden">
      {@render children()}
    </main>
  </div>
</div>

<CommandPalette />
