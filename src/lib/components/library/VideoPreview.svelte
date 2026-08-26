<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { videoPreview } from "$lib/stores/videoPreview.svelte";
  import type { Game } from "$lib/types";

  const HOVER_DELAY_MS = 300;

  let { game, mediaDir }: { game: Game; mediaDir: string | null } = $props();

  const src = $derived(
    game.videoPath && mediaDir ? convertFileSrc(`${mediaDir}/${game.videoPath}`) : null,
  );
  const active = $derived(videoPreview.activeGameId === game.id);

  let hoverTimer: ReturnType<typeof setTimeout> | undefined;
  let videoEl: HTMLVideoElement | undefined = $state();
  let ready = $state(false);

  function handlePointerEnter() {
    if (!src) return;
    clearTimeout(hoverTimer);
    hoverTimer = setTimeout(() => {
      videoPreview.activeGameId = game.id;
    }, HOVER_DELAY_MS);
  }

  function deactivate() {
    clearTimeout(hoverTimer);
    ready = false;
    if (videoPreview.activeGameId === game.id) {
      videoPreview.activeGameId = null;
    }
  }

  // Groja tik kol `active` — kito kortelės tampa aktyvia (globalus singleton) automatiškai
  // šitą padaro `false`, todėl `<video>` žemiau nustoja egzistuoti (žr. #if) ir šis cleanup
  // paleidžiamas, atlaisvindamas dekoderio resursus (P7.3 acceptance: „atmintis nekyla").
  $effect(() => {
    if (!active) return;
    const el = videoEl;
    if (!el) return;
    el.play().catch(() => {});
    return () => {
      el.pause();
      el.currentTime = 0;
      el.removeAttribute("src");
      el.load();
    };
  });

  $effect(() => {
    return () => clearTimeout(hoverTimer);
  });
</script>

<div
  class="absolute inset-0"
  role="presentation"
  onpointerenter={handlePointerEnter}
  onpointerleave={deactivate}
>
  {#if active && src}
    <video
      bind:this={videoEl}
      {src}
      muted
      loop
      playsinline
      preload="none"
      class="absolute inset-0 h-full w-full object-cover transition-opacity duration-200"
      class:opacity-100={ready}
      class:opacity-0={!ready}
      onloadeddata={() => (ready = true)}
    ></video>
  {/if}
</div>
