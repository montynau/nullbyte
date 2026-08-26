<script lang="ts">
  import { getAppInfo, getPreferredCores, listCores, setPreferredCores } from "$lib/api";
  import { library } from "$lib/stores/library.svelte";
  import * as Select from "$lib/components/ui/select/index.js";
  import type { CoreInfo, PlatformCorePreference } from "$lib/types";

  const NONE = "none";

  let cores = $state<CoreInfo[]>([]);
  let preferences = $state<PlatformCorePreference[]>([]);
  let coresDir = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    const [coresResult, preferencesResult, appInfo] = await Promise.all([
      listCores(),
      getPreferredCores(),
      getAppInfo(),
    ]);
    cores = coresResult;
    preferences = preferencesResult;
    coresDir = appInfo.coresDir;
    loading = false;
  }

  $effect(() => {
    load();
  });

  function preferredCorePath(platformSlug: string): string {
    return preferences.find((p) => p.platformSlug === platformSlug)?.corePath ?? NONE;
  }

  // Siūlo TIK core'us, kurių `validExtensions` sutampa su platformos plėtiniais — bet jei nė
  // vienas nesutampa (pvz. netikslus core .info failas, ar core dar nežino apie šią
  // platformą), rodom VISUS core'us vietoj tuščio sąrašo — geriau per daug pasirinkimų nei
  // paslėptas tinkamas core'as.
  function coresForPlatform(platformExtensions: string): CoreInfo[] {
    const wanted = platformExtensions.split(",").map((e) => e.trim().toLowerCase());
    const matching = cores.filter((c) =>
      c.validExtensions.some((e) => wanted.includes(e.toLowerCase())),
    );
    return matching.length > 0 ? matching : cores;
  }

  function coreLabel(core: CoreInfo): string {
    return core.version ? `${core.name} (${core.version})` : core.name;
  }

  async function setPreference(platformSlug: string, corePath: string) {
    preferences =
      corePath === NONE
        ? preferences.filter((p) => p.platformSlug !== platformSlug)
        : [
            ...preferences.filter((p) => p.platformSlug !== platformSlug),
            { platformSlug, corePath },
          ];
    await setPreferredCores(preferences);
  }
</script>

<div class="flex flex-col gap-6">
  <div class="border-border bg-muted/30 rounded-md border px-3 py-2 text-xs">
    <strong>Per-platform core selection is not applied to gameplay yet.</strong> It's saved for later
    — the emulator launch pipeline that would actually use it isn't built yet. The detected cores list
    below is accurate and live.
  </div>

  {#if loading}
    <p class="text-muted-foreground text-sm">Loading...</p>
  {:else}
    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-medium">Detected cores</h2>
      {#if cores.length === 0}
        <p class="text-muted-foreground text-sm">
          No cores found. Place libretro core files (<code>*_libretro.dylib</code> /
          <code>*_libretro.so</code>) in
          <span class="font-mono">{coresDir}</span>.
        </p>
      {:else}
        <ul class="flex flex-col gap-1">
          {#each cores as core (core.path)}
            <li class="border-border flex flex-col gap-0.5 rounded-md border px-3 py-2 text-sm">
              <div class="flex items-center justify-between">
                <span class="font-medium">{coreLabel(core)}</span>
                {#if core.systemName}
                  <span class="text-muted-foreground text-xs">{core.systemName}</span>
                {/if}
              </div>
              <span class="text-muted-foreground font-mono text-xs">
                {core.validExtensions.join(", ")}
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-medium">Preferred core per platform</h2>
      <div class="flex flex-col gap-1">
        {#each library.platforms as platform (platform.id)}
          {@const options = coresForPlatform(platform.extensions)}
          <div
            class="border-border flex items-center justify-between gap-3 rounded-md border px-3 py-1.5 text-sm"
          >
            <span>{platform.name}</span>
            <Select.Root
              type="single"
              value={preferredCorePath(platform.slug)}
              onValueChange={(value) => setPreference(platform.slug, value)}
            >
              <Select.Trigger class="h-8 w-56 text-sm">
                {preferredCorePath(platform.slug) === NONE
                  ? "None"
                  : (cores.find((c) => c.path === preferredCorePath(platform.slug))?.name ??
                    "Unknown core")}
              </Select.Trigger>
              <Select.Content>
                <Select.Item value={NONE} label="None" />
                {#each options as core (core.path)}
                  <Select.Item value={core.path} label={coreLabel(core)} />
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
        {/each}
      </div>
    </section>
  {/if}
</div>
