<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    addRomDirectory,
    cancelScrape,
    listRomDirectories,
    removeRomDirectory,
    scanLibrary,
    scrapeLibrary,
  } from "$lib/api";
  import { library } from "$lib/stores/library.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress";
  import * as Select from "$lib/components/ui/select/index.js";
  import FolderPlusIcon from "@lucide/svelte/icons/folder-plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import type {
    RomDirectory,
    ScanProgress,
    ScanSummary,
    ScrapeProgress,
    ScrapeSummary,
  } from "$lib/types";

  const AUTO_DETECT = "auto";

  let directories = $state<RomDirectory[]>([]);
  let loadingDirs = $state(true);
  // Select veikia su string reikšmėmis — "auto" žymi `platformId: null` (automatinis
  // nustatymas pagal plėtinį), kitaip platformos `id` kaip tekstas.
  let pendingPlatformId = $state(AUTO_DETECT);

  let scanning = $state(false);
  let scanProgress = $state<ScanProgress | null>(null);
  let scanSummary = $state<ScanSummary | null>(null);

  let scraping = $state(false);
  let scrapeProgress = $state<ScrapeProgress | null>(null);
  let scrapeSummary = $state<ScrapeSummary | null>(null);

  async function loadDirectories() {
    loadingDirs = true;
    try {
      directories = await listRomDirectories();
    } finally {
      loadingDirs = false;
    }
  }

  $effect(() => {
    loadDirectories();
  });

  async function pickDirectory() {
    const selected = await open({ directory: true, multiple: false });
    if (!selected || Array.isArray(selected)) return;
    const platformId = pendingPlatformId === AUTO_DETECT ? null : Number(pendingPlatformId);
    await addRomDirectory(selected, true, platformId);
    await loadDirectories();
  }

  async function removeDirectory(id: number) {
    await removeRomDirectory(id);
    await loadDirectories();
  }

  function platformName(platformId: number | null): string {
    if (platformId == null) return "Auto-detect";
    return library.platforms.find((p) => p.id === platformId)?.name ?? `#${platformId}`;
  }

  async function runScan() {
    if (scanning) return;
    scanning = true;
    scanSummary = null;
    scanProgress = null;
    try {
      scanSummary = await scanLibrary((progress) => {
        scanProgress = progress;
      });
      await library.loadGames();
      await library.loadPlatforms();
    } finally {
      scanning = false;
      scanProgress = null;
    }
  }

  async function runScrapeLibrary() {
    if (scraping) return;
    scraping = true;
    scrapeSummary = null;
    scrapeProgress = null;
    try {
      scrapeSummary = await scrapeLibrary(null, (progress) => {
        scrapeProgress = progress;
      });
      await library.loadGames();
    } finally {
      scraping = false;
      scrapeProgress = null;
    }
  }

  const scanPercent = $derived(
    scanProgress && scanProgress.total > 0 ? (scanProgress.current / scanProgress.total) * 100 : 0,
  );
  const scrapePercent = $derived(
    scrapeProgress && scrapeProgress.total > 0
      ? (scrapeProgress.current / scrapeProgress.total) * 100
      : 0,
  );
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-3">
    <div class="flex items-center justify-between gap-2">
      <h2 class="text-sm font-medium">ROM directories</h2>
      <div class="flex items-center gap-2">
        <Select.Root type="single" bind:value={pendingPlatformId}>
          <Select.Trigger class="h-8 w-40 text-sm">
            {platformName(pendingPlatformId === AUTO_DETECT ? null : Number(pendingPlatformId))}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value={AUTO_DETECT} label="Auto-detect" />
            {#each library.platforms as platform (platform.id)}
              <Select.Item value={String(platform.id)} label={platform.name} />
            {/each}
          </Select.Content>
        </Select.Root>
        <Button size="sm" onclick={pickDirectory}>
          <FolderPlusIcon class="size-4" />
          Add directory
        </Button>
      </div>
    </div>

    {#if loadingDirs}
      <p class="text-muted-foreground text-sm">Loading...</p>
    {:else if directories.length === 0}
      <p class="text-muted-foreground text-sm">No ROM directories added yet.</p>
    {:else}
      <ul class="flex flex-col gap-1">
        {#each directories as dir (dir.id)}
          <li
            class="border-border flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm"
          >
            <div class="flex min-w-0 flex-col">
              <span class="truncate">{dir.path}</span>
              <span class="text-muted-foreground text-xs">{platformName(dir.platformId)}</span>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onclick={() => removeDirectory(dir.id)}
              aria-label="Remove directory"
            >
              <Trash2Icon class="size-4" />
            </Button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="flex flex-col gap-2">
    <Button onclick={runScan} disabled={scanning || directories.length === 0} class="w-fit">
      {scanning ? "Scanning..." : "Scan library"}
    </Button>
    {#if scanning}
      <Progress value={scanPercent} />
      {#if scanProgress}
        <p class="text-muted-foreground truncate text-xs">{scanProgress.currentFile}</p>
      {/if}
    {/if}
    {#if scanSummary}
      <p class="text-muted-foreground text-xs">
        Added {scanSummary.added}, updated {scanSummary.updated}, removed {scanSummary.removed},
        unchanged {scanSummary.unchanged}, skipped {scanSummary.skippedUnknownExtension}.
      </p>
    {/if}
  </section>

  <section class="flex flex-col gap-2">
    <div class="flex items-center gap-2">
      <Button onclick={runScrapeLibrary} disabled={scraping} variant="outline">
        {scraping ? "Scraping..." : "Scrape library metadata"}
      </Button>
      {#if scraping}
        <Button variant="ghost" size="sm" onclick={cancelScrape}>Cancel</Button>
      {/if}
    </div>
    {#if scraping}
      <Progress value={scrapePercent} />
      {#if scrapeProgress}
        <p class="text-muted-foreground truncate text-xs">
          {scrapeProgress.title} — {scrapeProgress.status}
          {#if scrapeProgress.quotaLeft != null}
            · quota left: {scrapeProgress.quotaLeft}
          {/if}
        </p>
      {/if}
    {/if}
    {#if scrapeSummary}
      <p class="text-muted-foreground text-xs">
        Found {scrapeSummary.found}, not found {scrapeSummary.notFound}, errors
        {scrapeSummary.errored}{scrapeSummary.cancelled ? " (cancelled)" : ""}.
      </p>
    {/if}
  </section>
</div>
