<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import Play from "@lucide/svelte/icons/play";
  import Pause from "@lucide/svelte/icons/pause";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Heart from "@lucide/svelte/icons/heart";
  import ImageIcon from "@lucide/svelte/icons/image";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Volume2 from "@lucide/svelte/icons/volume-2";
  import Volume1 from "@lucide/svelte/icons/volume-1";
  import VolumeX from "@lucide/svelte/icons/volume-x";
  import Repeat from "@lucide/svelte/icons/repeat";
  import Repeat1 from "@lucide/svelte/icons/repeat-1";
  import PictureInPicture2 from "@lucide/svelte/icons/picture-in-picture-2";
  import { player, toggle, seek, setVolume, next, prev, upcoming, canPrev, canNext, toggleShuffle, cycleRepeat, toggleMute, fmt } from "$lib/player.svelte";
  import { library, coverOf, type Track } from "$lib/library.svelte";
  import { isFavorite, toggleFavorite } from "$lib/playlists.svelte";
  import QueuePopover from "$lib/QueuePopover.svelte";

  let { onopen }: { onopen: (t: Track, kind: "albums" | "artists") => void } = $props();

  let scrubbing = $state(false);
  let moved = false; // a press with no drag fires no change event, so it must end the scrub itself
  let showLeft = $state(localStorage.getItem("timeLeft") === "1");
  function toggleTimeLeft() {
    showLeft = !showLeft;
    localStorage.setItem("timeLeft", showLeft ? "1" : "0");
  }
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : player.pos);
  let current = $derived(library.tracks.find((t) => t.path === player.track));
  let title = $derived(current?.title ?? (player.track ? "—" : "Nothing Playing..."));
  let fav = $derived(isFavorite(player.track));

  let queueOpen = $state(false);
  let queueBox: HTMLElement;
  let queueBtn: HTMLButtonElement;
  function closeQueue() {
    queueOpen = false;
    queueBtn.focus();
  }

  // reactive variables for marquee animation
  let titleW = $state(0);
  let textW = $state(0);
  let overflow = $derived(textW > titleW);

  // cover for the playing track
  let cover = $state<string | null>(null);
  $effect(() => {
    const path = player.track;
    cover = null;
    if (path) coverOf(path).then((c) => player.track === path && (cover = c)); // ignore if the track changed meanwhile
  });
</script>

<!-- click outside the queue panel closes it -->
<svelte:window onpointerdown={(e) => queueOpen && !queueBox.contains(e.target as Node) && (queueOpen = false)} />

<footer>
  <div class="now">
    <!-- same size with or without art, so the bar keeps its height while a cover loads -->
    <button class="cover" aria-label="Open album" disabled={!current?.album} onclick={() => current && onopen(current, "albums")}>
      {#if cover}<img src={cover} alt="" />{:else}<ImageIcon size={32} strokeWidth={1.5} />{/if}
    </button>
    <div class="meta">
      <button class="title" class:fade={overflow} title={current ? title : undefined} disabled={!current?.album} onclick={() => current && onopen(current, "albums")} bind:clientWidth={titleW}>
        <div class="marquee" class:scrolling={overflow}>
          <span bind:clientWidth={textW}>{title}</span>
          {#if overflow}
            <span aria-hidden="true">{title}</span>
          {/if}
        </div>
      </button>
      <button class="artist cell" disabled={!current} onclick={() => current && onopen(current, "artists")}>{current?.artist ?? "No artist"}</button>
    </div>
    <button
      class="icon"
      class:on={fav}
      aria-pressed={fav}
      aria-label="Favorite"
      disabled={!player.track}
      onclick={() => player.track && toggleFavorite(player.track)}><Heart size={20} fill={fav ? "currentColor" : "none"} /></button
    >
  </div>

  <div class="center">
    <div class="progress">
      <span class="time">{player.track ? fmt(shownPos) : "—:—"}</span>
      <input
        class="seek"
        type="range"
        aria-label="Position"
        aria-valuetext="{fmt(shownPos)} of {fmt(player.duration)}"
        min="0"
        max={player.duration}
        step="0.5"
        value={shownPos}
        disabled={!player.track}
        onpointerdown={() => {
          scrubPos = player.pos;
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
          seek(+e.currentTarget.value);
          scrubbing = false;
        }}
      />
      <button class="time total" aria-label={showLeft ? "Show total time" : "Show time left"} disabled={!player.track} onclick={toggleTimeLeft}>
        {#if !player.track}—:—{:else if showLeft}-{fmt(Math.max(0, player.duration - shownPos))}{:else}{fmt(player.duration)}{/if}
      </button>
    </div>
    <div class="controls">
      <button class="icon end" class:on={player.shuffle} aria-pressed={player.shuffle} aria-label="Shuffle" onclick={toggleShuffle}>
        <Shuffle size={16} />
      </button>
      <div class="transport">
        <button class="icon" aria-label="Previous" onclick={prev} disabled={!canPrev()}><SkipBack size={20} fill="currentColor" /></button>
        <button class="play" aria-label={player.state === "playing" ? "Pause" : "Play"} onclick={toggle} disabled={!player.track}>
          {#if player.state === "playing"}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
        </button>
        <button class="icon" aria-label="Next" onclick={() => next()} disabled={!canNext()}><SkipForward size={20} fill="currentColor" /></button>
      </div>
      <button class="icon end" class:on={player.repeat !== "off"} aria-label="Repeat: {player.repeat}" onclick={cycleRepeat}>
        {#if player.repeat === "one"}<Repeat1 size={16} />{:else}<Repeat size={16} />{/if}
      </button>
    </div>
  </div>

  <div class="side">
    <button class="icon" aria-label="Open popout player" onclick={() => invoke("open_popout")}><PictureInPicture2 size={20} /></button>
    <div class="queue-box" bind:this={queueBox}>
      <button class="btn queue-btn" bind:this={queueBtn} aria-expanded={queueOpen} onclick={() => (queueOpen = !queueOpen)}
        ><ListMusic size={18} /> Queue · {upcoming()}</button
      >
      {#if queueOpen}<QueuePopover onclose={closeQueue} />{/if}
    </div>
    <div class="vol">
      <button class="icon" aria-label={player.volume ? "Mute" : "Unmute"} onclick={toggleMute}>
        {#if !player.volume}<VolumeX size={20} />{:else if player.volume < 0.5}<Volume1 size={20} />{:else}<Volume2 size={20} />{/if}
      </button>
      <input type="range" aria-label="Volume" min="0" max="1" step="0.01" value={player.volume} oninput={(e) => setVolume(+e.currentTarget.value)} />
    </div>
  </div>
</footer>

<style>
  footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: grid;
    /* the right side never shrinks below its controls, so it can't slide under the progress times */
    grid-template-columns: minmax(16rem, 1fr) minmax(12rem, 34rem) minmax(max-content, 1fr);
    gap: 16px;
    align-items: center;
    box-sizing: border-box;
    height: var(--bar-h);
    padding: 0 12px;
    background: var(--ground);
    border-top: 1px solid var(--rule);
  }
  .now {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
  }
  .cover {
    padding: 0;
    border: none;
    cursor: pointer;
    width: 80px;
    height: 80px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 4px;
    overflow: hidden;
    background: var(--surface);
    color: var(--faint);
  }
  .cover:disabled {
    cursor: default;
  }
  .cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .meta {
    min-width: 0;
    max-width: 22rem;
  }
  .title {
    all: unset;
    display: block;
    box-sizing: border-box;
    width: 100%;
    cursor: pointer;
    overflow: hidden;
    white-space: nowrap;
    font-size: 17px;
    font-weight: 500;
  }
  .title:disabled {
    cursor: default;
  }
  /* soft edge shows the title runs on; hovering holds it still to read */
  .title.fade {
    mask-image: linear-gradient(90deg, #000 88%, transparent);
  }
  .title:hover .marquee {
    animation-play-state: paused;
  }
  .artist {
    all: unset;
    display: block;
    box-sizing: border-box;
    max-width: 100%;
    cursor: pointer;
    margin-top: 4px;
    font-size: 14px;
    color: var(--muted);
  }
  .center {
    display: grid;
    gap: 10px;
  }
  /* shuffle and repeat sit under the ends of the seek bar, so previous, play and next stay on the centre line */
  .controls {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding-inline: calc(2.5rem + 8px); /* the width of a time plus its gap */
  }
  .controls .end {
    width: 32px;
    height: 32px;
    position: relative;
  }
  .controls .end:last-child {
    justify-self: end;
  }
  .controls .end.on::after {
    content: "";
    position: absolute;
    bottom: 2px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor; /* on means more than blue, so it reads without colour too */
  }
  .transport {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .transport .icon {
    width: 36px;
    height: 36px;
  }
  .play {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
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
  .total {
    all: unset;
    flex: none;
    min-width: 2.5rem;
    font-size: var(--fs-small);
    color: var(--muted);
    cursor: pointer;
  }
  .total:disabled {
    cursor: default;
  }
  .total:hover:not(:disabled),
  .artist:hover:not(:disabled) {
    color: var(--text);
    text-decoration: underline;
  }
  .total:focus-visible,
  .title:focus-visible,
  .artist:focus-visible {
    outline: 2px solid var(--accent);
  }
  .seek:disabled {
    opacity: 0.4;
  }
  input[type="range"] {
    accent-color: var(--accent); /* the progress is data, so it carries the blue */
  }
  .seek {
    flex: 1;
    min-width: 0;
  }
  .side {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 16px;
  }
  .queue-box {
    position: relative; /* the queue panel opens above this */
  }
  .queue-btn {
    height: 40px;
    padding: 0 16px;
    font-size: 14px;
  }
  .queue-btn[aria-expanded="true"] {
    background: var(--raised);
    border-color: var(--raised);
  }
  .vol {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
  }
  .vol input {
    width: clamp(4rem, 9vw, 9rem);
    accent-color: var(--muted); /* volume is a setting, not data: no blue */
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
  /* gap between the two copies, same on both so -50% lands on the second copy;
     margin, not padding, because clientWidth leaves margin out of the overflow check */
  .marquee.scrolling span {
    margin-right: 3rem;
  }
  /* the default window is 1000px wide: the title gets the room before the fixed-width centre does */
  @media (max-width: 1200px) {
    footer {
      grid-template-columns: minmax(16rem, 1.4fr) minmax(12rem, 1fr) minmax(max-content, 1fr);
    }
  }
  /* narrow windows (the app's minimum is 720px): a smaller cover leaves room for the title */
  @media (max-width: 900px) {
    .side {
      gap: 6px;
    }
    .queue-btn {
      padding: 0 10px;
    }
    .cover {
      width: 56px;
      height: 56px;
    }
    .title {
      font-size: 15px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .marquee.scrolling {
      animation: none;
    }
  }
  @keyframes marquee {
    from {
      transform: translateX(0);
    }
    to {
      transform: translateX(-50%);
    }
  }
</style>
