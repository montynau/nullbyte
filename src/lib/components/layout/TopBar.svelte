<script lang="ts">
  import { library } from "$lib/stores/library.svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import SearchIcon from "@lucide/svelte/icons/search";
  import SettingsIcon from "@lucide/svelte/icons/settings";

  const SORT_OPTIONS: { value: string; label: string }[] = [
    { value: "title-asc", label: "Pavadinimas (A–Z)" },
    { value: "title-desc", label: "Pavadinimas (Z–A)" },
    { value: "lastPlayed-desc", label: "Neseniai žaisti" },
    { value: "addedAt-desc", label: "Naujausiai pridėti" },
  ];

  const sortValue = $derived(`${library.filter.sort}-${library.filter.sortDirection}`);
  const sortLabel = $derived(
    SORT_OPTIONS.find((o) => o.value === sortValue)?.label ?? "Rūšiavimas",
  );

  function handleSortChange(value: string | undefined) {
    if (!value) return;
    const [sort, direction] = value.split("-") as [
      "title" | "lastPlayed" | "addedAt",
      "asc" | "desc",
    ];
    library.setSort(sort, direction);
  }
</script>

<header class="border-border bg-background flex h-12 shrink-0 items-center gap-3 border-b px-4">
  <div class="relative max-w-sm flex-1">
    <SearchIcon
      class="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2"
    />
    <Input
      type="text"
      placeholder="Ieškoti bibliotekoje..."
      value={library.filter.search ?? ""}
      oninput={(e) => library.setSearch(e.currentTarget.value)}
      class="h-8 pl-8"
    />
  </div>

  <Select.Root type="single" value={sortValue} onValueChange={handleSortChange}>
    <Select.Trigger class="h-8 w-44 text-sm">
      {sortLabel}
    </Select.Trigger>
    <Select.Content>
      {#each SORT_OPTIONS as option (option.value)}
        <Select.Item value={option.value} label={option.label} />
      {/each}
    </Select.Content>
  </Select.Root>

  <div class="flex-1"></div>

  <Tooltip.Provider>
    <Tooltip.Root>
      <Tooltip.Trigger>
        <Button variant="ghost" size="icon" disabled aria-label="Nustatymai">
          <SettingsIcon class="size-4" />
        </Button>
      </Tooltip.Trigger>
      <Tooltip.Content>Nustatymai — netrukus (P7.6)</Tooltip.Content>
    </Tooltip.Root>
  </Tooltip.Provider>
</header>
