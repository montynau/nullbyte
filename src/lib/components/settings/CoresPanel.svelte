<script lang="ts">
  import {
    getAppInfo,
    getCorePriority,
    getPreferredCores,
    listCores,
    setPreferredCores,
  } from "$lib/api";
  import { library } from "$lib/stores/library.svelte";
  import { showErrorToast } from "$lib/utils/errors";
  import * as Select from "$lib/components/ui/select/index.js";
  import type { CoreInfo, PlatformCorePreference } from "$lib/types";

  const NONE = "none";

  let cores = $state<CoreInfo[]>([]);
  let preferences = $state<PlatformCorePreference[]>([]);
  let corePriority = $state<Record<string, string[]>>({});
  let coresDir = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      const [coresResult, preferencesResult, priorityResult, appInfo] = await Promise.all([
        listCores(),
        getPreferredCores(),
        getCorePriority(),
        getAppInfo(),
      ]);
      cores = coresResult;
      preferences = preferencesResult;
      corePriority = priorityResult;
      coresDir = appInfo.coresDir;
    } catch (error) {
      showErrorToast(error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  function preferredCorePath(platformSlug: string): string {
    return preferences.find((p) => p.platformSlug === platformSlug)?.corePath ?? NONE;
  }

  // Filtruojama pagal kuruotą `supportedPlatforms` (backend `known_core_platforms`, ADR-024),
  // NE pagal `validExtensions` sutapimą — patikrinta REALIAIS core'ais, kad extension'ų
  // sutapimas duoda klaidingus teigiamus rezultatus. TIK VERIFIKUOTI atitikimai (core'as
  // eksplicitiškai nurodytas kaip palaikantis šią platformą) laikomi „turi core'ą" — jei jų
  // yra bent vienas, „None" IŠVIS nerodoma kaip pasirinkimas (vartotojo prašymas). Nežinomi
  // core'ai (`supportedPlatforms === null`) visada pridedami kaip papildomas, aiškiai
  // paženklintas „· unverified" pasirinkimas — niekada nepaslepiami, bet niekada NEI
  // laikomi „turi core'ą" požymiu.
  function verifiedCores(platformSlug: string): CoreInfo[] {
    return cores.filter((c) => c.supportedPlatforms?.includes(platformSlug));
  }

  function unverifiedCores(): CoreInfo[] {
    return cores.filter((c) => c.supportedPlatforms === null);
  }

  function optionsForPlatform(platformSlug: string): CoreInfo[] {
    return [...verifiedCores(platformSlug), ...unverifiedCores()].sort((a, b) =>
      a.name.localeCompare(b.name),
    );
  }

  function showNoneFor(platformSlug: string): boolean {
    return verifiedCores(platformSlug).length === 0;
  }

  // Pirmas `corePriority[slug]` sąraše nurodytas core'as, kuris REALIAI rastas `cores_dir` —
  // naudojama automatiniam pasiūlymui (žr. `$effect` žemiau).
  function recommendedCorePath(platformSlug: string): string | null {
    const order = corePriority[platformSlug] ?? [];
    for (const name of order) {
      const match = cores.find((c) => c.name === name);
      if (match) return match.path;
    }
    return null;
  }

  const sortedCores = $derived([...cores].sort((a, b) => a.name.localeCompare(b.name)));

  function coreLabel(core: CoreInfo): string {
    const base = core.version ? `${core.name} (${core.version})` : core.name;
    return core.supportedPlatforms === null ? `${base} · unverified` : base;
  }

  // Automatiškai priskiria rekomenduojamą core'ą KIEKVIENAI platformai, kurios vartotojas dar
  // NELIETĖ (nėra `preferences` įraše) — vartotojo prašymas: „galima iškart uždėti jei randa
  // rekomenduojamą... nenorėčiau viską nuo 0 suvedinėti". Reaktyvus (ne vienkartinis `load()`
  // viduje), nes `library.platforms` užsipildo ASINCHRONIŠKAI iš atskiro store'o (`+layout.svelte`)
  // — gali dar būti tuščias, kai šio komponento `load()` baigiasi. Idempotentiškas: antrą kartą
  // paleidus (po `preferences` pasikeitimo) `next.some(...)` jau ras visus ką tik pridėtus
  // įrašus, `changed` liks `false`, jokios begalinės kilpos.
  $effect(() => {
    if (loading || cores.length === 0 || library.platforms.length === 0) return;

    const next = [...preferences];
    let changed = false;
    for (const platform of library.platforms) {
      if (next.some((p) => p.platformSlug === platform.slug)) continue;
      const recommended = recommendedCorePath(platform.slug);
      if (recommended) {
        next.push({ platformSlug: platform.slug, corePath: recommended });
        changed = true;
      }
    }
    if (changed) {
      preferences = next;
      setPreferredCores(preferences).catch(showErrorToast);
    }
  });

  async function setPreference(platformSlug: string, corePath: string) {
    preferences =
      corePath === NONE
        ? preferences.filter((p) => p.platformSlug !== platformSlug)
        : [
            ...preferences.filter((p) => p.platformSlug !== platformSlug),
            { platformSlug, corePath },
          ];
    try {
      await setPreferredCores(preferences);
    } catch (error) {
      showErrorToast(error);
    }
  }
</script>

<div class="flex flex-col gap-6">
  <div class="border-border bg-muted/30 rounded-md border px-3 py-2 text-xs">
    The core you pick here is what launches when you press Play. A recommended core is picked
    automatically for each platform that has one — change it any time.
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
          {#each sortedCores as core (core.path)}
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
          {@const options = optionsForPlatform(platform.slug)}
          {@const currentPath = preferredCorePath(platform.slug)}
          {@const currentCore = cores.find((c) => c.path === currentPath)}
          <div
            class="border-border flex items-center justify-between gap-3 rounded-md border px-3 py-1.5 text-sm"
          >
            <span>{platform.name}</span>
            <Select.Root
              type="single"
              value={currentPath}
              onValueChange={(value) => setPreference(platform.slug, value)}
            >
              <Select.Trigger class="h-8 w-56 text-sm">
                {currentPath === NONE
                  ? "None"
                  : currentCore
                    ? coreLabel(currentCore)
                    : "Unknown core"}
              </Select.Trigger>
              <Select.Content>
                {#if showNoneFor(platform.slug)}
                  <Select.Item value={NONE} label="None" />
                {/if}
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
