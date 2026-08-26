<script lang="ts">
  import { resolve } from "$app/paths";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import CoresPanel from "$lib/components/settings/CoresPanel.svelte";
  import InputPanel from "$lib/components/settings/InputPanel.svelte";
  import PathsPanel from "$lib/components/settings/PathsPanel.svelte";
  import ScraperPanel from "$lib/components/settings/ScraperPanel.svelte";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";

  let activeTab = $state("paths");
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

  <div class="max-w-3xl">
    <Tabs.Root bind:value={activeTab}>
      <Tabs.List class="mb-6">
        <Tabs.Trigger value="paths">Paths</Tabs.Trigger>
        <Tabs.Trigger value="cores">Cores</Tabs.Trigger>
        <Tabs.Trigger value="video">Video</Tabs.Trigger>
        <Tabs.Trigger value="audio">Audio</Tabs.Trigger>
        <Tabs.Trigger value="input">Input</Tabs.Trigger>
        <Tabs.Trigger value="scraper">Scraper</Tabs.Trigger>
      </Tabs.List>

      <Tabs.Content value="paths">
        <PathsPanel />
      </Tabs.Content>
      <Tabs.Content value="cores">
        <CoresPanel />
      </Tabs.Content>
      <Tabs.Content value="video">
        <p class="text-muted-foreground text-sm">Video settings — coming soon.</p>
      </Tabs.Content>
      <Tabs.Content value="audio">
        <p class="text-muted-foreground text-sm">Audio settings — coming soon.</p>
      </Tabs.Content>
      <Tabs.Content value="input">
        <InputPanel />
      </Tabs.Content>
      <Tabs.Content value="scraper">
        <ScraperPanel />
      </Tabs.Content>
    </Tabs.Root>
  </div>
</div>
