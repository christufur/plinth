<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { emit, listen } from "@tauri-apps/api/event";
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import Play from "@lucide/svelte/icons/play";
  import Pause from "@lucide/svelte/icons/pause";
  import ImageIcon from "@lucide/svelte/icons/image";
  import Grip from "@lucide/svelte/icons/grip";
  import X from "@lucide/svelte/icons/x";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fmt } from "$lib/queue";

  // this window must never import player.svelte or library.svelte: a second copy would skip a track at every song end
  let song = $state({ path: null as string | null, title: "", artist: "", canPrev: false, canNext: false });
  let playState = $state("stopped");
  let pos = $state(0);
  let duration = $state(0);

  let scrubbing = $state(false);
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : pos);

  let cover = $state<string | null>(null);
  $effect(() => {
    const path = song.path;
    cover = null;
    if (path) invoke<string | null>("cover_art", { path }).then((c) => song.path === path && (cover = c)); // ignore if the track changed meanwhile
  });

  // "" drops the inline value, so app.css follows the system
  const applyTheme = () => (document.documentElement.style.colorScheme = localStorage.getItem("theme") ?? "");
  applyTheme();

  onMount(() => {
    const listeners = Promise.all([
      listen<typeof song>("popout_state", (e) => void Object.assign(song, e.payload)),
      listen<string>("state_changed", (e) => (playState = e.payload)),
      listen<[number, number]>("position", (e) => ([pos, duration] = e.payload)),
      listen("track_changed", () => (pos = 0)),
      // Rust sends no state_changed when a track runs out
      listen("track_ended", () => {
        playState = "stopped";
        pos = 0;
      }),
    ]);
    listeners.then(() => emit("popout_ready"));
    return () => void listeners.then((fns) => fns.forEach((unlisten) => unlisten()));
  });
</script>

<svelte:window onstorage={applyTheme} />

<!-- the window has no title bar: the empty areas drag it, its edges resize it, and the X brings main back -->
<main data-tauri-drag-region>
  <div class="cover" data-tauri-drag-region>
    {#if cover}<img src={cover} alt="" />{:else}<ImageIcon size={24} strokeWidth={1.5} />{/if}
  </div>

  <div class="panel" data-tauri-drag-region>
    <div class="meta" data-tauri-drag-region>
      <div class="title cell" data-tauri-drag-region>{song.title || "Nothing Playing..."}</div>
      <div class="artist cell" data-tauri-drag-region>{song.artist || "No artist"}</div>
    </div>

    <div class="controls">
      <button class="icon" aria-label="Previous" onclick={() => emit("popout_cmd", "prev")} disabled={!song.canPrev}
        ><SkipBack size={18} fill="currentColor" /></button
      >
      <button
        class="play"
        aria-label={playState === "playing" ? "Pause" : "Play"}
        onclick={() => invoke(playState === "playing" ? "pause" : "resume")}
        disabled={!song.path}
      >
        {#if playState === "playing"}<Pause size={16} fill="currentColor" />{:else}<Play size={16} fill="currentColor" />{/if}
      </button>
      <button class="icon" aria-label="Next" onclick={() => emit("popout_cmd", "next")} disabled={!song.canNext}
        ><SkipForward size={18} fill="currentColor" /></button
      >
    </div>

    <div class="progress">
      <span class="time">{fmt(shownPos)}</span>
      <input
        class="seek"
        type="range"
        aria-label="Position"
        min="0"
        max={duration}
        step="0.5"
        value={shownPos}
        disabled={!song.path}
        onpointerdown={() => {
          scrubPos = pos;
          scrubbing = true;
        }}
        oninput={(e) => (scrubPos = +e.currentTarget.value)}
        onchange={(e) => {
          pos = +e.currentTarget.value; // optimistic; Rust confirms on next tick
          invoke("seek", { secs: pos });
          scrubbing = false;
        }}
      />
      <span class="time">{fmt(duration)}</span>
    </div>
  </div>

  <button class="icon close" aria-label="Close popout player" onclick={() => getCurrentWindow().close()}><X size={14} /></button>
  <button
    class="icon grip"
    aria-label="Resize popout player"
    onpointerdown={() => getCurrentWindow().startResizeDragging("SouthEast")}><Grip size={12} /></button
  >
</main>

<style>
  /* compact strip: art on the left at the window's full height, the player beside it */
  main {
    position: relative;
    display: grid;
    grid-template-columns: min(100vh, 40vw) minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
  }
  .cover {
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--surface);
    color: var(--faint);
  }
  .cover :global(svg),
  .cover img {
    pointer-events: none; /* so a drag starting on the art still moves the window */
  }
  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .panel {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    min-width: 0;
    padding: 4px 12px;
  }
  .meta {
    min-width: 0;
    padding-right: 20px; /* clears the close button */
  }
  .title {
    font-size: 14px;
    font-weight: 500;
  }
  .artist {
    font-size: var(--fs-small);
    color: var(--muted);
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .controls .icon {
    width: 32px;
    height: 32px;
  }
  .grip {
    position: absolute;
    right: 2px;
    bottom: 2px;
    width: 18px;
    height: 18px;
    cursor: nwse-resize;
  }
  .close {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 24px;
    height: 24px;
  }
  .play {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    margin: 0 6px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: var(--primary);
    color: var(--on-primary);
    cursor: pointer;
  }
  .play:hover:not(:disabled) {
    transform: scale(1.05);
  }
  .play:disabled {
    opacity: 0.35;
    cursor: default;
  }
  .progress {
    padding-right: 10px; /* clears the resize grip */
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .time {
    flex: none;
    min-width: 2.5rem; /* m:ss and mm:ss keep the slider still; longer times just widen it */
    font-size: var(--fs-small);
    color: var(--muted);
  }
  .time:first-child {
    text-align: right;
  }
  .seek {
    flex: 1;
    min-width: 0;
    margin: 0;
    accent-color: var(--accent);
  }
</style>
