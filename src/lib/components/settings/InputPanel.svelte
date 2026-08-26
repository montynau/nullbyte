<script lang="ts">
  import { getInputMapping, resetInputMapping, setInputMapping } from "$lib/api";
  import { showErrorToast } from "$lib/utils/errors";
  import { Button } from "$lib/components/ui/button";
  import type { InputBinding } from "$lib/types";

  // Rodymo tvarka/etiketės — atitinka `RETRO_DEVICE_ID_JOYPAD_*` grupavimą, NE `default_bindings()`
  // (Rust pusėje) masyvo eiliškumą (kuris irgi tas pats, bet čia eksplicitiška, kad UI grupavimas
  // liktų stabilus net jei backend sąrašo tvarka pasikeistų).
  const LAYOUT: { button: string; label: string }[] = [
    { button: "up", label: "Up" },
    { button: "down", label: "Down" },
    { button: "left", label: "Left" },
    { button: "right", label: "Right" },
    { button: "a", label: "A" },
    { button: "b", label: "B" },
    { button: "x", label: "X" },
    { button: "y", label: "Y" },
    { button: "l", label: "L" },
    { button: "r", label: "R" },
    { button: "l2", label: "L2" },
    { button: "r2", label: "R2" },
    { button: "l3", label: "L3 (stick click)" },
    { button: "r3", label: "R3 (stick click)" },
    { button: "select", label: "Select" },
    { button: "start", label: "Start" },
  ];

  let bindings = $state<InputBinding[]>([]);
  let loading = $state(true);
  let listening = $state<{ button: string; kind: "keyboard" | "gamepad" } | null>(null);

  async function load() {
    loading = true;
    try {
      bindings = await getInputMapping();
    } catch (error) {
      showErrorToast(error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    load();
  });

  // Švarus sustabdymas, jei vartotojas išnavigavo iš šio ekrano PER VIDURĮ listening'o —
  // kitaip `window` keydown klausytojas/`requestAnimationFrame` liktų kabėti po komponento
  // sunaikinimo.
  $effect(() => {
    return () => cancelListening();
  });

  function bindingFor(button: string): InputBinding | undefined {
    return bindings.find((b) => b.retropadButton === button);
  }

  async function save() {
    try {
      await setInputMapping(bindings);
    } catch (error) {
      showErrorToast(error);
    }
  }

  // Vienas keydown klausytojas veikia PER ABU listening tipus — klaviatūros listening'ui jis
  // pagauna PATĮ klavišą, gamepad listening'ui jis pagauna TIK `Escape` (kitus klavišus
  // ignoruoja, praleisdamas juos toliau — vartotojas gali laisvai judėti pele/kitur, kol
  // laukiama gamepad paspaudimo).
  function startListening(button: string, kind: "keyboard" | "gamepad") {
    listening = { button, kind };
    window.addEventListener("keydown", onKeydown, { capture: true });
    if (kind === "gamepad") {
      gamepadBaseline = snapshotPressedButtons();
      gamepadPollHandle = requestAnimationFrame(pollGamepad);
    }
    // Apsauga nuo amžinai kabančio „listening" būvio, jei vartotojas tiesiog nusprendžia
    // niekur nebespausti (pvz. nutolsta nuo klaviatūros/gamepad'o).
    listenTimeout = window.setTimeout(cancelListening, 10_000);
  }

  function cancelListening() {
    window.removeEventListener("keydown", onKeydown, { capture: true });
    if (gamepadPollHandle != null) {
      cancelAnimationFrame(gamepadPollHandle);
      gamepadPollHandle = null;
    }
    if (listenTimeout != null) {
      clearTimeout(listenTimeout);
      listenTimeout = null;
    }
    listening = null;
  }

  function onKeydown(event: KeyboardEvent) {
    if (!listening) return;
    if (event.code === "Escape") {
      event.preventDefault();
      cancelListening();
      return;
    }
    if (listening.kind === "keyboard") {
      event.preventDefault();
      event.stopPropagation();
      applyCapture(listening.button, { keyboardKey: event.code });
      cancelListening();
    }
  }

  // Plikas masyvas, NE Map/Set — čia tyčia NEreaktyvi (šablone nenaudojama), tik trumpalaikė
  // bookkeeping'o būsena vieno listening'o metu.
  let gamepadBaseline: { padIndex: number; pressed: number[] }[] = [];
  let gamepadPollHandle: number | null = null;
  let listenTimeout: number | null = null;

  function snapshotPressedButtons(): { padIndex: number; pressed: number[] }[] {
    const snapshot: { padIndex: number; pressed: number[] }[] = [];
    for (const pad of navigator.getGamepads()) {
      if (!pad) continue;
      const pressed = pad.buttons.flatMap((b, index) => (b.pressed ? [index] : []));
      snapshot.push({ padIndex: pad.index, pressed });
    }
    return snapshot;
  }

  function pollGamepad() {
    if (!listening || listening.kind !== "gamepad") return;
    for (const pad of navigator.getGamepads()) {
      if (!pad) continue;
      const baseline = gamepadBaseline.find((entry) => entry.padIndex === pad.index)?.pressed ?? [];
      const newlyPressed = pad.buttons.findIndex(
        (b, index) => b.pressed && !baseline.includes(index),
      );
      if (newlyPressed !== -1) {
        applyCapture(listening.button, { gamepadButton: newlyPressed });
        cancelListening();
        return;
      }
    }
    gamepadPollHandle = requestAnimationFrame(pollGamepad);
  }

  function applyCapture(
    button: string,
    value: { keyboardKey: string } | { gamepadButton: number },
  ) {
    bindings = bindings.map((b) => (b.retropadButton === button ? { ...b, ...value } : b));
    save();
  }

  function clearBinding(button: string, kind: "keyboard" | "gamepad") {
    bindings = bindings.map((b) =>
      b.retropadButton === button
        ? kind === "keyboard"
          ? { ...b, keyboardKey: null }
          : { ...b, gamepadButton: null }
        : b,
    );
    save();
  }

  async function resetAll() {
    cancelListening();
    try {
      await resetInputMapping();
      await load();
    } catch (error) {
      showErrorToast(error);
    }
  }
</script>

<div class="flex flex-col gap-4">
  <div class="border-border bg-muted/30 rounded-md border px-3 py-2 text-xs">
    <strong>Not applied to gameplay yet.</strong> These bindings are saved for later — the emulator launch
    pipeline that would actually use them isn't built yet, so games still use the fixed default controls
    for now.
  </div>

  <div class="flex items-center justify-between">
    <h2 class="text-sm font-medium">Controls</h2>
    <Button variant="ghost" size="sm" onclick={resetAll}>Reset to defaults</Button>
  </div>

  {#if loading}
    <p class="text-muted-foreground text-sm">Loading...</p>
  {:else}
    <div class="flex flex-col gap-1">
      <div class="text-muted-foreground flex items-center gap-3 px-3 text-xs font-medium">
        <span class="w-32 shrink-0"></span>
        <span class="flex-1">Keyboard</span>
        <span class="flex-1">Gamepad</span>
      </div>
      {#each LAYOUT as row (row.button)}
        {@const binding = bindingFor(row.button)}
        <div class="border-border flex items-center gap-3 rounded-md border px-3 py-1.5 text-sm">
          <span class="w-32 shrink-0">{row.label}</span>

          <div class="flex flex-1 items-center gap-1.5">
            {#if listening?.button === row.button && listening.kind === "keyboard"}
              <span class="text-muted-foreground w-28 shrink-0 truncate text-xs"
                >Press a key...</span
              >
            {:else}
              <span class="w-28 shrink-0 truncate font-mono text-xs">
                {binding?.keyboardKey ?? "Not set"}
              </span>
            {/if}
            {#if listening?.button === row.button && listening.kind === "keyboard"}
              <Button variant="ghost" size="sm" onclick={cancelListening}>Cancel</Button>
            {:else}
              <Button
                variant="outline"
                size="sm"
                onclick={() => startListening(row.button, "keyboard")}
              >
                Rebind
              </Button>
              <Button
                variant="ghost"
                size="sm"
                disabled={!binding?.keyboardKey}
                class={binding?.keyboardKey ? "" : "invisible"}
                onclick={() => clearBinding(row.button, "keyboard")}
              >
                Clear
              </Button>
            {/if}
          </div>

          <div class="flex flex-1 items-center gap-1.5">
            {#if listening?.button === row.button && listening.kind === "gamepad"}
              <span class="text-muted-foreground w-28 shrink-0 truncate text-xs"
                >Press a button...</span
              >
            {:else}
              <span class="w-28 shrink-0 truncate font-mono text-xs">
                {binding?.gamepadButton != null ? `Button ${binding.gamepadButton}` : "Not set"}
              </span>
            {/if}
            {#if listening?.button === row.button && listening.kind === "gamepad"}
              <Button variant="ghost" size="sm" onclick={cancelListening}>Cancel</Button>
            {:else}
              <Button
                variant="outline"
                size="sm"
                onclick={() => startListening(row.button, "gamepad")}
              >
                Rebind
              </Button>
              <Button
                variant="ghost"
                size="sm"
                disabled={binding?.gamepadButton == null}
                class={binding?.gamepadButton != null ? "" : "invisible"}
                onclick={() => clearBinding(row.button, "gamepad")}
              >
                Clear
              </Button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
