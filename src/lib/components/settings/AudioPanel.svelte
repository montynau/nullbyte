<script lang="ts">
  import { getAudioSettings, listAudioDevices, setAudioSettings } from "$lib/api";
  import { showErrorToast } from "$lib/utils/errors";
  import * as Select from "$lib/components/ui/select/index.js";
  import { Slider } from "$lib/components/ui/slider";
  import type { AudioSettings } from "$lib/types";

  const SYSTEM_DEFAULT = "__default__";

  let settings = $state<AudioSettings | null>(null);
  let devices = $state<string[]>([]);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      const [settingsResult, devicesResult] = await Promise.all([
        getAudioSettings(),
        listAudioDevices(),
      ]);
      settings = settingsResult;
      devices = devicesResult;
    } catch (error) {
      showErrorToast(error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  async function update(partial: Partial<AudioSettings>) {
    if (!settings) return;
    settings = { ...settings, ...partial };
    try {
      await setAudioSettings(settings);
    } catch (error) {
      showErrorToast(error);
    }
  }
</script>

<div class="flex flex-col gap-6">
  <div class="border-border bg-muted/30 rounded-md border px-3 py-2 text-xs">
    <strong>Not applied to gameplay yet.</strong> Device/volume/buffer size are saved, but audio output
    has no way to use a chosen device, apply volume, or resize its buffer today — that needs new engine
    work, not just the emulator launch pipeline (P9.1). The device list below is real and live.
  </div>

  {#if loading || !settings}
    <p class="text-muted-foreground text-sm">Loading...</p>
  {:else}
    <section class="flex flex-col gap-2">
      <h2 class="text-sm font-medium">Output device</h2>
      <Select.Root
        type="single"
        value={settings.device ?? SYSTEM_DEFAULT}
        onValueChange={(value) => update({ device: value === SYSTEM_DEFAULT ? null : value })}
      >
        <Select.Trigger class="h-8 w-64 text-sm">
          {settings.device ?? "System default"}
        </Select.Trigger>
        <Select.Content>
          <Select.Item value={SYSTEM_DEFAULT} label="System default" />
          {#each devices as device (device)}
            <Select.Item value={device} label={device} />
          {/each}
        </Select.Content>
      </Select.Root>
      {#if devices.length === 0}
        <p class="text-muted-foreground text-xs">No output devices detected.</p>
      {/if}
    </section>

    <section class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-medium">Volume</h2>
        <span class="text-muted-foreground text-xs">{Math.round(settings.volume * 100)}%</span>
      </div>
      <Slider
        type="single"
        value={settings.volume * 100}
        min={0}
        max={100}
        step={1}
        onValueChange={(value) => update({ volume: value / 100 })}
      />
    </section>

    <section class="flex flex-col gap-2">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-medium">Buffer size</h2>
        <span class="text-muted-foreground text-xs">{settings.bufferMs} ms</span>
      </div>
      <Slider
        type="single"
        value={settings.bufferMs}
        min={20}
        max={200}
        step={10}
        onValueChange={(value) => update({ bufferMs: value })}
      />
      <p class="text-muted-foreground text-xs">Lower = less latency, higher = fewer crackles.</p>
    </section>
  {/if}
</div>
