<script lang="ts">
  import { getVideoSettings, setVideoSettings } from "$lib/api";
  import { showErrorToast } from "$lib/utils/errors";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Switch } from "$lib/components/ui/switch";
  import type { VideoSettings } from "$lib/types";

  let settings = $state<VideoSettings | null>(null);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      settings = await getVideoSettings();
    } catch (error) {
      showErrorToast(error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  async function update(partial: Partial<VideoSettings>) {
    if (!settings) return;
    settings = { ...settings, ...partial };
    try {
      await setVideoSettings(settings);
    } catch (error) {
      showErrorToast(error);
    }
  }
</script>

<div class="flex flex-col gap-6">
  <div class="border-border bg-muted/30 rounded-md border px-3 py-2 text-xs">
    <strong>Not applied to gameplay yet.</strong> Filter and scaling are saved and the renderer already
    supports both internally, but there's no way to send this choice to a running game yet. Vsync and
    start-fullscreen have no engine hook at all today, saved for later regardless.
  </div>

  {#if loading || !settings}
    <p class="text-muted-foreground text-sm">Loading...</p>
  {:else}
    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-medium">Texture filter</h2>
      <Select.Root
        type="single"
        value={settings.filter}
        onValueChange={(value) => update({ filter: value })}
      >
        <Select.Trigger class="h-8 w-52 text-sm">
          {settings.filter === "nearest" ? "Nearest (pixel-perfect)" : "Linear (smoothed)"}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="nearest" label="Nearest (pixel-perfect)" />
          <Select.Item value="linear" label="Linear (smoothed)" />
        </Select.Content>
      </Select.Root>
    </section>

    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-medium">Scaling</h2>
      <Select.Root
        type="single"
        value={settings.scaleMode}
        onValueChange={(value) => update({ scaleMode: value })}
      >
        <Select.Trigger class="h-8 w-52 text-sm">
          {settings.scaleMode === "fit" ? "Fit window" : "Integer scale"}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="fit" label="Fit window" />
          <Select.Item value="integer" label="Integer scale" />
        </Select.Content>
      </Select.Root>
    </section>

    <section class="border-border flex items-center justify-between rounded-md border px-3 py-2">
      <div>
        <h2 class="text-sm font-medium">Vsync</h2>
        <p class="text-muted-foreground text-xs">Reduces screen tearing.</p>
      </div>
      <Switch checked={settings.vsync} onCheckedChange={(checked) => update({ vsync: checked })} />
    </section>

    <section class="border-border flex items-center justify-between rounded-md border px-3 py-2">
      <div>
        <h2 class="text-sm font-medium">Start fullscreen</h2>
        <p class="text-muted-foreground text-xs">Launch games in fullscreen by default.</p>
      </div>
      <Switch
        checked={settings.startFullscreen}
        onCheckedChange={(checked) => update({ startFullscreen: checked })}
      />
    </section>
  {/if}
</div>
