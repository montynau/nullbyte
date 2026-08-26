<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import CommandPalette from "$lib/components/layout/CommandPalette.svelte";
  import { Toaster } from "$lib/components/ui/sonner";
  import { library } from "$lib/stores/library.svelte";
  import { app } from "$lib/stores/app.svelte";
  import { showErrorToast } from "$lib/utils/errors";

  let { children } = $props();

  onMount(() => {
    library.loadPlatforms();
    app.load();

    // Klaidos, kurios atsiranda PO sėkmingo žaidimo paleidimo (pvz. save state klaida
    // per hotkey) — žr. `crate::ipc` modulio doc #2 (`nullbyte-app`). Mount'inama ČIA, ne
    // žaidimo detalių puslapyje, nes vartotojas iki tada gali būti jau grįžęs į biblioteką,
    // kol `nullbyte-emu` langas tebeveikia — toast'as turi pasiekti nepriklausomai nuo to,
    // kuriame puslapyje vartotojas šiuo metu yra.
    const unlisten = listen("game-error", (event) => {
      showErrorToast(event.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
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
<Toaster theme="dark" />
