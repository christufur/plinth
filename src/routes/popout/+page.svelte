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
  import Maximize2 from "@lucide/svelte/icons/maximize-2";
  import Heart from "@lucide/svelte/icons/heart";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Repeat from "@lucide/svelte/icons/repeat";
  import Repeat1 from "@lucide/svelte/icons/repeat-1";
  import Volume2 from "@lucide/svelte/icons/volume-2";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fmt } from "$lib/queue";

  // this window must never import player.svelte or library.svelte: a second copy would skip a track at every song end
  let song = $state({
    path: null as string | null,
    title: "",
    artist: "",
    canPrev: false,
    canNext: false,
    favorite: false,
    shuffle: false,
    repeat: "off",
    volume: 1,
  });
  let volumeOpen = $state(false);
  // the title scrolls only when it doesn't fit, as in the player bar
  let titleW = $state(0);
  let textW = $state(0);
  let overflow = $derived(textW > titleW);
  const send = (cmd: string) => emit("popout_cmd", cmd);
  const close = () => getCurrentWindow().close(); // main comes back when this window goes
  let playState = $state("stopped");
  let pos = $state(0);
  let duration = $state(0);

  let scrubbing = $state(false);
  let moved = false; // a press with no drag fires no change event, so it must end the scrub itself
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

<!-- no title bar: the cover and empty areas drag the window, its edges and the grip resize it -->
<main class:tinted={cover} data-tauri-drag-region>
  <!-- the blank part of the window takes the song's colour; dark, so the text reads on any art -->
  {#if cover}<div class="backdrop" aria-hidden="true"><img src={cover} alt="" /></div>{/if}
  <div class="cover" data-tauri-drag-region>
    {#key cover}
      {#if cover}<img src={cover} alt="" />{:else}<ImageIcon size={24} strokeWidth={1.5} />{/if}
    {/key}
  </div>

  <div class="panel" data-tauri-drag-region>
    <div class="info" data-tauri-drag-region>
      <div class="meta">
        <button class="title" class:fade={overflow} title={song.title} onclick={close} bind:clientWidth={titleW}>
          <span class="marquee" class:scrolling={overflow}>
            <span bind:clientWidth={textW}>{song.title || "Nothing Playing..."}</span>
            {#if overflow}<span aria-hidden="true">{song.title}</span>{/if}
          </span>
        </button>
        <button class="artist" onclick={close}>{song.artist || "No artist"}</button>
      </div>
    </div>

    <div class="controls">
      <button class="icon extra" class:on={song.shuffle} aria-pressed={song.shuffle} aria-label="Shuffle" onclick={() => send("shuffle")}>
        <Shuffle size={16} />
      </button>
      <div class="transport">
        <span class="balance"></span>
        <button class="icon prev" aria-label="Previous" onclick={() => send("prev")} disabled={!song.canPrev}
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
        <button class="icon" aria-label="Next" onclick={() => send("next")} disabled={!song.canNext}
          ><SkipForward size={18} fill="currentColor" /></button
        >
        <button
          class="icon like"
          class:on={song.favorite}
          aria-pressed={song.favorite}
          aria-label="Favorite"
          disabled={!song.path}
          onclick={() => send("favorite")}><Heart size={16} fill={song.favorite ? "currentColor" : "none"} /></button
        >
      </div>
      <button class="icon extra" class:on={song.repeat !== "off"} aria-label="Repeat: {song.repeat}" onclick={() => send("repeat")}>
        {#if song.repeat === "one"}<Repeat1 size={16} />{:else}<Repeat size={16} />{/if}
      </button>
    </div>
  </div>

  <div class="progress" style:--done="{duration ? (shownPos / duration) * 100 : 0}%">
    <span class="time start">{fmt(shownPos)}</span>
    <input
      class="seek"
      type="range"
      aria-label="Position"
      aria-valuetext="{fmt(shownPos)} of {fmt(duration)}"
      min="0"
      max={duration}
      step="0.5"
      value={shownPos}
      disabled={!song.path}
      onpointerdown={() => {
        scrubPos = pos;
        scrubbing = true;
        moved = false;
      }}
      onpointerup={() => {
        if (!moved) scrubbing = false;
      }}
      oninput={(e) => {
        scrubPos = +e.currentTarget.value;
        moved = true;
      }}
      onchange={(e) => {
        pos = +e.currentTarget.value; // optimistic; Rust confirms on next tick
        invoke("seek", { secs: pos });
        scrubbing = false;
      }}
    />
    <span class="time end">{fmt(duration)}</span>
  </div>

  <div class="corner">
    <div class="volume">
      <button class="icon small" aria-label="Volume" aria-expanded={volumeOpen} onclick={() => (volumeOpen = !volumeOpen)}><Volume2 size={14} /></button>
      {#if volumeOpen}
        <input type="range" aria-label="Volume level" min="0" max="1" step="0.01" value={song.volume} oninput={(e) => emit("popout_volume", +e.currentTarget.value)} />
      {/if}
    </div>
    <button class="icon small" aria-label="Open main window" onclick={close}><Maximize2 size={14} /></button>
    <button class="icon small" aria-label="Close popout player" onclick={close}><X size={14} /></button>
  </div>
  <button class="icon grip" aria-label="Resize popout player" onpointerdown={() => getCurrentWindow().startResizeDragging("SouthEast")}><Grip size={12} /></button>
</main>

<style>
  /* three layouts by window height. Strip is the default: art at full height, the player beside it */
  main {
    position: relative;
    display: grid;
    grid-template-columns: min(100vh, 40vw) minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
    background: var(--ground);
  }
  .tinted {
    color-scheme: dark;
    color: var(--text);
  }
  .backdrop {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none; /* drags land on the window underneath */
  }
  .backdrop img {
    position: absolute;
    inset: -30%;
    width: 160%;
    height: 160%;
    object-fit: cover;
    filter: blur(24px) saturate(1.5) brightness(0.6);
    animation: arrive 300ms ease-out;
  }
  .backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    background: #0008; /* keeps muted text at 4.5:1 on a white cover */
  }
  .cover {
    position: relative;
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
  /* out of the grid: inside it, height: 100% has no fixed height to resolve against, so the art kept its own shape */
  .cover img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: arrive 150ms ease-out;
  }
  @keyframes arrive {
    from {
      opacity: 0;
    }
  }
  .panel {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    min-width: 0;
    padding: 4px 10px 8px 12px;
  }
  .info {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }
  .meta {
    flex: 1;
    min-width: 0;
    padding-right: 56px; /* clears the window buttons */
  }
  .title,
  .artist {
    all: unset;
    display: block;
    box-sizing: border-box;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
  }
  .title {
    font-size: 14px;
    font-weight: 500;
  }
  .artist {
    font-size: var(--fs-small);
    color: var(--muted);
  }
  .artist:hover {
    text-decoration: underline;
  }
  /* soft edge shows the title runs on; hovering holds it still to read */
  .title.fade {
    text-overflow: clip;
    mask-image: linear-gradient(90deg, #000 88%, transparent);
  }
  .marquee {
    display: inline-block;
    width: max-content;
  }
  .marquee.scrolling {
    animation: marquee 10s linear infinite;
  }
  /* inline spans report clientWidth 0, so the text width needs inline-block */
  .marquee span {
    display: inline-block;
  }
  /* margin, not padding: clientWidth leaves margin out of the overflow check */
  .marquee.scrolling span {
    margin-right: 3rem;
  }
  .title:hover .marquee {
    animation-play-state: paused;
  }
  @keyframes marquee {
    to {
      transform: translateX(-50%);
    }
  }
  .title:focus-visible,
  .artist:focus-visible {
    outline: 2px solid var(--accent);
  }
  .balance {
    width: 32px; /* mirrors the heart, so play stays on the centre line */
  }
  .controls {
    display: flex;
    justify-content: center;
    align-items: center;
  }
  .transport {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .controls .icon {
    width: 32px;
    height: 32px;
  }
  .extra {
    display: none;
  }
  .play {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    margin: 0 4px;
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
  /* a thin line along the bottom edge that thickens under the pointer */
  .progress {
    position: absolute;
    left: min(100vh, 40vw);
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
  }
  .time {
    display: none;
  }
  .seek {
    flex: 1;
    min-width: 0;
    height: 10px;
    margin: 0;
    appearance: none;
    background: none;
    cursor: pointer;
  }
  .seek:disabled {
    cursor: default;
  }
  .seek::-webkit-slider-runnable-track {
    height: 3px;
    background: linear-gradient(90deg, var(--accent) var(--done), var(--raised) 0);
    transition: height 120ms;
  }
  .seek:hover::-webkit-slider-runnable-track {
    height: 5px;
  }
  .seek::-webkit-slider-thumb {
    appearance: none;
    width: 0;
    height: 0;
  }
  .corner {
    position: absolute;
    top: 2px;
    right: 2px;
    display: flex;
    align-items: center;
  }
  .small {
    width: 24px;
    height: 24px;
  }
  .volume {
    display: none;
    align-items: center;
  }
  .volume input {
    width: 72px;
    accent-color: var(--accent);
  }
  .grip {
    position: absolute;
    right: 1px;
    bottom: 3px;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    opacity: 0;
  }
  .corner,
  .grip {
    transition: opacity 120ms 1.5s;
  }
  .corner {
    opacity: 0;
  }
  main:hover .corner,
  main:focus-within .corner,
  main:hover .grip {
    opacity: 1;
    transition-delay: 0s;
  }

  /* Bar: the slimmest shape keeps only what you reach for */
  @media (max-height: 71px) {
    .artist,
    .like,
    .balance,
    .prev,
    .progress {
      display: none;
    }
    .panel {
      flex-direction: row;
      align-items: center;
      gap: 8px;
      padding: 0 56px 0 10px;
    }
    .info {
      flex: 1;
    }
    .meta {
      padding-right: 0;
    }
  }

  /* short strip, down to where Bar takes over: everything stays, just tighter, so nothing is cut */
  @media (min-height: 72px) and (max-height: 103px) {
    .panel {
      gap: 0;
      padding: 2px 10px 6px 12px;
    }
    .title {
      font-size: 13px;
      line-height: 1.2;
    }
    .artist {
      font-size: 11px;
      line-height: 1.2;
    }
    .controls .icon {
      width: 28px;
      height: 28px;
    }
    .balance {
      width: 28px;
    }
    .play {
      width: 28px;
      height: 28px;
    }
  }

  /* narrow strip: the row keeps the heart and drops its balance, then previous */
  @media (max-width: 340px) and (max-height: 180px), (max-width: 280px) {
    .balance {
      display: none;
    }
  }
  @media (max-width: 280px) and (max-height: 180px) {
    .prev {
      display: none;
    }
  }

  /* Art: dragged taller, the cover fills the window and the player sits on it */
  @media (min-height: 181px) {
    main {
      grid-template-columns: minmax(0, 1fr);
    }
    .cover {
      position: absolute;
      inset: 0;
    }
    .panel {
      position: relative;
      align-self: end;
      gap: 6px;
      padding: 56px 14px 36px;
      background: linear-gradient(transparent, #000c);
      color: #fff;
    }
    .meta {
      padding-right: 0;
    }
    .artist,
    .time,
    .controls .icon,
    .corner .icon {
      color: #fff;
    }
    .extra {
      display: grid;
    }
    .extra.on {
      color: var(--accent);
    }
    .controls {
      justify-content: space-between;
      opacity: 0;
      transition: opacity 120ms 1.5s;
    }
    main:hover .controls,
    main:focus-within .controls {
      opacity: 1;
      transition-delay: 0s;
    }
    .play {
      background: #fff;
      color: #000;
    }
    .progress {
      left: 0;
      bottom: 8px;
      gap: 8px;
      padding: 0 14px;
    }
    .time {
      display: block;
      min-width: 2.5rem;
      font-size: var(--fs-small);
      opacity: 0;
      transition: opacity 120ms 1.5s;
    }
    .time.end {
      text-align: right;
    }
    main:hover .time {
      opacity: 1;
      transition-delay: 0s;
    }
    .seek::-webkit-slider-runnable-track {
      background: linear-gradient(90deg, #fff var(--done), #fff5 0);
    }
    .corner .icon {
      background: #0006;
    }
    .volume {
      display: flex;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .cover img,
    .backdrop img,
    .marquee.scrolling {
      animation: none;
    }
    .corner,
    .grip,
    .controls,
    .time {
      transition: none !important;
    }
  }
</style>
