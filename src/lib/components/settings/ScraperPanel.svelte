<script lang="ts">
  import {
    clearScraperCredentials,
    getScraperQuota,
    getScraperStatus,
    setScraperCredentials,
  } from "$lib/api";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { describeError } from "$lib/utils/errors";
  import type { QuotaSnapshot, ScraperCredentialStatus } from "$lib/types";

  // Atitinka `REGION_PRIORITY`/media tipų lentelę CLAUDE.md §9.2 ir
  // `crates/nullbyte-app/src/scraper/screenscraper.rs` — ŠIUO METU hardkodinta Rust pusėje,
  // tad čia tik INFORMACINIS rodinys (P7.6 v1), ne redaguojama forma. Jei kada taps
  // konfigūruojama per `settings` KV lentelę — šis sąrašas turi būti pakeistas realiu
  // backend kvietimu.
  const REGION_PRIORITY = ["wor", "eu", "us", "jp", "ss"];
  const MEDIA_TYPES = [
    { type: "box-2D", usage: "Cover (library grid, detail page)" },
    { type: "ss", usage: "Screenshot (detail page)" },
    { type: "wheel", usage: "Logo overlay" },
    { type: "video-normalized", usage: "Hover preview (preferred)" },
    { type: "video", usage: "Hover preview (fallback)" },
  ];

  let status = $state<ScraperCredentialStatus | null>(null);
  let quota = $state<QuotaSnapshot | null>(null);
  let loading = $state(true);

  let editing = $state(false);
  let devIdInput = $state("");
  let devPasswordInput = $state("");
  let ssidInput = $state("");
  let sspasswordInput = $state("");
  let saving = $state(false);
  let saveError = $state<string | null>(null);

  async function loadStatus() {
    loading = true;
    const [statusResult, quotaResult] = await Promise.all([getScraperStatus(), getScraperQuota()]);
    status = statusResult;
    quota = quotaResult;
    loading = false;
  }

  // Abu kvietimai pasyvūs — jokio HTTP į ScreenScraper (žr. Rust pusės
  // `get_scraper_status`/`get_scraper_quota` doc), tad saugu kviesti kiekvieną kartą atidarius
  // šią panelę.
  $effect(() => {
    loadStatus();
  });

  function startEditing() {
    devIdInput = "";
    devPasswordInput = "";
    ssidInput = "";
    sspasswordInput = "";
    saveError = null;
    editing = true;
  }

  async function save() {
    saving = true;
    saveError = null;
    try {
      await setScraperCredentials(
        devIdInput,
        devPasswordInput,
        ssidInput || null,
        sspasswordInput || null,
      );
      editing = false;
      await loadStatus();
    } catch (error) {
      saveError = describeError(error);
    } finally {
      saving = false;
    }
  }

  async function clearOverride() {
    saving = true;
    saveError = null;
    try {
      await clearScraperCredentials();
      await loadStatus();
    } catch (error) {
      saveError = describeError(error);
    } finally {
      saving = false;
    }
  }

  function relativeTime(unixSeconds: number): string {
    const deltaSeconds = Math.max(0, Math.floor(Date.now() / 1000) - unixSeconds);
    if (deltaSeconds < 60) return "just now";
    const minutes = Math.floor(deltaSeconds / 60);
    if (minutes < 60) return `${minutes} min ago`;
    const hours = Math.floor(minutes / 60);
    return `${hours} h ago`;
  }
</script>

<div class="flex flex-col gap-6">
  <section class="flex flex-col gap-3">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-medium">Credentials</h2>
      {#if status && !editing}
        <Button variant="outline" size="sm" onclick={startEditing}>Edit</Button>
      {/if}
    </div>

    {#if loading}
      <p class="text-muted-foreground text-sm">Loading...</p>
    {:else if status && !editing}
      <div class="border-border flex flex-col gap-2 rounded-md border px-3 py-2 text-sm">
        <div class="flex items-center justify-between">
          <span>Developer credentials</span>
          {#if status.devCredentialsConfigured}
            <span class="text-xs">Configured ({status.devIdMasked})</span>
          {:else}
            <span class="text-destructive text-xs">Not configured</span>
          {/if}
        </div>
        <div class="flex items-center justify-between">
          <span>User login (higher quota)</span>
          {#if status.userLoginConfigured}
            <span class="text-xs">Configured</span>
          {:else}
            <span class="text-muted-foreground text-xs">Not configured</span>
          {/if}
        </div>
        {#if status.overridden}
          <div class="flex items-center justify-between pt-1">
            <span class="text-muted-foreground text-xs">Saved in Settings (overrides .env)</span>
            <Button variant="ghost" size="sm" disabled={saving} onclick={clearOverride}>
              Clear override
            </Button>
          </div>
        {/if}
      </div>
      {#if !status.devCredentialsConfigured}
        <p class="text-muted-foreground text-xs">
          No credentials found in Settings or .env. Set them above, or set SCREENSCRAPER_DEV_ID /
          SCREENSCRAPER_DEV_PASSWORD in .env and restart the app.
        </p>
      {/if}
    {:else if editing}
      <form
        class="border-border flex flex-col gap-3 rounded-md border px-3 py-3"
        onsubmit={(event) => {
          event.preventDefault();
          save();
        }}
      >
        <label class="flex flex-col gap-1 text-xs">
          Dev ID
          <Input bind:value={devIdInput} placeholder="required" required autocomplete="off" />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          Dev password
          <Input
            type="password"
            bind:value={devPasswordInput}
            placeholder="required"
            required
            autocomplete="off"
          />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          User login (optional, higher quota)
          <Input bind:value={ssidInput} placeholder="optional" autocomplete="off" />
        </label>
        <label class="flex flex-col gap-1 text-xs">
          User password (optional)
          <Input
            type="password"
            bind:value={sspasswordInput}
            placeholder="optional"
            autocomplete="off"
          />
        </label>
        {#if saveError}
          <p class="text-destructive text-xs">{saveError}</p>
        {/if}
        <div class="flex items-center gap-2">
          <Button type="submit" size="sm" disabled={saving}>
            {saving ? "Saving..." : "Save"}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            disabled={saving}
            onclick={() => (editing = false)}
          >
            Cancel
          </Button>
        </div>
      </form>
    {/if}
  </section>

  <section class="flex flex-col gap-2">
    <h2 class="text-sm font-medium">Quota</h2>
    {#if quota}
      <p class="text-sm">
        <span class="font-medium">{quota.quotaLeft.toLocaleString()}</span> requests left today
        <span class="text-muted-foreground text-xs">(checked {relativeTime(quota.checkedAt)})</span>
      </p>
    {:else}
      <p class="text-muted-foreground text-sm">
        Not checked yet this session — run a scrape to see the current quota.
      </p>
    {/if}
  </section>

  <section class="flex flex-col gap-2">
    <h2 class="text-sm font-medium">Region priority</h2>
    <p class="text-muted-foreground text-xs">
      Used to pick a name, date and media when a game has data from multiple regions.
    </p>
    <div class="flex flex-wrap gap-1.5">
      {#each REGION_PRIORITY as region, i (region)}
        <span class="border-border rounded-md border px-2 py-0.5 text-xs uppercase">
          {i + 1}. {region}
        </span>
      {/each}
    </div>
  </section>

  <section class="flex flex-col gap-2">
    <h2 class="text-sm font-medium">Media types</h2>
    <ul class="flex flex-col gap-1">
      {#each MEDIA_TYPES as media (media.type)}
        <li
          class="border-border flex items-center justify-between rounded-md border px-3 py-1.5 text-sm"
        >
          <span class="font-mono text-xs">{media.type}</span>
          <span class="text-muted-foreground text-xs">{media.usage}</span>
        </li>
      {/each}
    </ul>
  </section>
</div>
