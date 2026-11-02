<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Heart from "@lucide/svelte/icons/heart";
  import ImageIcon from "@lucide/svelte/icons/image";
  import { player, playAt, playQueued, fmt } from "$lib/player.svelte";
  import { library, coverOf, type Track } from "$lib/library.svelte";
  import { isFavorite, toggleFavorite } from "$lib/playlists.svelte";
  import SeekBar from "$lib/SeekBar.svelte";
  import Transport from "$lib/Transport.svelte";
  import VolumeControl from "$lib/VolumeControl.svelte";

  const IDLE_AFTER = 10000;
  let idle = $state(false);
  let idleTimer: ReturnType<typeof setTimeout>;
  // the cover takes over while a song plays and nothing is touched; any input hands the screen back
  function wake() {
    idle = false;
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => (idle = player.state === "playing"), IDLE_AFTER);
  }
  $effect(() => {
    wake();
    return () => clearTimeout(idleTimer);
  });
  // paused from the keyboard or at a song's end: the controls should be there
  $effect(() => {
    if (player.state !== "playing") idle = false;
    else if (!idle) wake();
  });

  let { onback, onopen }: { onback: () => void; onopen: (t: Track, kind: "albums" | "artists") => void } = $props();

  let byPath = $derived(new Map(library.tracks.map((t) => [t.path, t])));
  let current = $derived(player.track ? byPath.get(player.track) : undefined);
  let fav = $derived(isFavorite(player.track));
  let details = $derived(
    [current?.album, current?.year, player.track?.split(".").pop()?.toUpperCase()].filter(Boolean).join(" · "),
  );
  // same order as the player: the queue first, then the rest of the list
  let upNext = $derived([
    ...player.queue.map((path, k) => ({ path, go: () => playQueued(k) })),
    ...player.context.slice(player.at + 1).map((path, k) => ({ path, go: () => playAt(player.at + 1 + k) })),
  ]);

  let cover = $state<string | null>(null);
  $effect(() => {
    const path = player.track;
    cover = null;
    if (path) coverOf(path).then((c) => player.track === path && (cover = c)); // ignore if the track changed meanwhile
  });
</script>

<svelte:window onpointermove={wake} onpointerdown={wake} onkeydown={wake} onwheel={wake} />

<!-- focus lands here, so a keyboard user starts on this screen without a ring on the first button -->
<!-- this screen is always dark: a tint light enough for light mode washed most covers out to off-white -->
<div class="backdrop" aria-hidden="true">
  {#if cover}<img class="glow" src={cover} alt="" />{/if}
</div>

<section class="now-playing" aria-label="Now Playing" tabindex="-1" {@attach (el: HTMLElement) => el.focus({ preventScroll: true })}>
  <button class="back" onclick={onback}><ArrowLeft size={14} /> Back to library</button>

  <div class="body">
    <div class="art">
      {#key cover}
        {#if cover}<img src={cover} alt="" />{:else}<ImageIcon size={72} strokeWidth={1} />{/if}
      {/key}
    </div>

    <div class="detail">
      <div class="heading">
        <div class="text">
          <h1 title={current?.title}>{current?.title ?? (player.track ? "—" : "Nothing Playing...")}</h1>
          <button class="link artist" disabled={!current} onclick={() => current && onopen(current, "artists")}>
            {current?.artist ?? "No artist"}
          </button>
          {#if details}
            <button class="link meta" disabled={!current?.album} onclick={() => current && onopen(current, "albums")}>{details}</button>
          {/if}
        </div>
      </div>

      <div class="playback">
        <SeekBar />
        <!-- volume shares this row, so the controls read as one line -->
        <div class="actions">
          <div class="left">
            <button
              class="icon"
              class:on={fav}
              aria-pressed={fav}
              aria-label="Favorite"
              disabled={!player.track}
              onclick={() => player.track && toggleFavorite(player.track)}><Heart size={20} fill={fav ? "currentColor" : "none"} /></button
            >
          </div>
          <Transport compact />
          <div class="right"><VolumeControl /></div>
        </div>
      </div>

      <h2>Up next</h2>
      <div class="list">
        <!-- keyed by index: the same song can be listed twice -->
        {#each upNext as item, i (i)}
          {@const t = byPath.get(item.path)}
          <button class="row" style:--cols="2.5rem 1fr 3.5rem" onclick={item.go}>
            <span class="num">{i + 2}</span>
            <span class="cell id">{t?.title ?? item.path.split(/[\\/]/).pop()}</span>
            <span class="num">{t?.secs ? fmt(t.secs) : "—"}</span>
          </button>
        {:else}
          <p class="empty">Nothing up next.</p>
        {/each}
      </div>
    </div>
  </div>
</section>

<!-- idle: only the cover, over a blurred copy of itself; the first input wakes it, so a stray click hits nothing -->
<div class="ambient" class:on={idle} aria-hidden="true">
  {#if cover}
    <img class="glow" src={cover} alt="" />
    <img class="hero" src={cover} alt="" />
  {:else}
    <ImageIcon size={120} strokeWidth={1} />
  {/if}
</div>

<style>
  .now-playing {
    display: flex;
    flex-direction: column;
    gap: 16px;
    box-sizing: border-box;
    height: 100%;
    padding-top: 20px;
    outline: none;
  }
  .back {
    all: unset;
    display: inline-flex;
    align-self: flex-start;
    align-items: center;
    gap: 6px;
    color: var(--muted);
    cursor: pointer;
  }
  .back:hover {
    color: var(--text);
  }
  .back:focus-visible,
  .link:focus-visible {
    outline: 2px solid var(--accent);
  }
  .body {
    flex: 1;
    display: grid;
    grid-template-columns: auto minmax(0, 640px);
    justify-content: center;
    gap: 40px;
    min-height: 0;
  }
  .art {
    display: grid;
    place-items: center;
    position: relative;
    width: clamp(180px, min(38vw, 100vh - 130px), 640px);
    aspect-ratio: 1;
    overflow: hidden;
    border-radius: 8px;
    background: var(--surface);
    color: var(--faint);
  }
  .art img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: arrive 200ms ease-out;
  }
  @keyframes arrive {
    from {
      opacity: 0;
    }
  }
  .detail {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }
  h1 {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2; /* the full title is in its tooltip */
    line-clamp: 2;
    font-size: 26px;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }
  .link {
    all: unset;
    display: block;
    max-width: 100%;
    cursor: pointer;
  }
  .link:disabled {
    cursor: default;
  }
  .link:hover:not(:disabled) {
    text-decoration: underline;
  }
  .artist {
    margin-top: 6px;
    font-size: 16px;
    font-weight: 500;
  }
  .meta {
    margin-top: 4px;
    color: var(--muted);
  }
  .playback {
    display: grid;
    gap: 10px;
    margin-top: 24px;
  }
  .actions {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 8px;
  }
  .right {
    display: flex;
    justify-content: flex-end;
  }
  .ambient {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: var(--ground);
    color: var(--faint);
    opacity: 0;
    pointer-events: none;
    transition: opacity 500ms ease;
  }
  .ambient.on {
    opacity: 1;
    pointer-events: auto;
    cursor: none;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: -1;
    overflow: hidden;
    background: #121212;
  }
  /* enough dark over the art that muted text holds 4.5:1 on a white cover */
  .backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    background: #0008;
  }
  .glow {
    position: absolute;
    inset: -10%;
    width: 120%;
    height: 120%;
    object-fit: cover;
    filter: blur(60px) saturate(1.5) brightness(0.6);
  }
  .hero {
    position: relative;
    width: min(70vh, 70vw);
    aspect-ratio: 1;
    object-fit: cover;
    border-radius: 8px;
    box-shadow: 0 24px 80px #0008;
    transform: scale(0.96);
    transition: transform 700ms ease;
  }
  .ambient.on .hero {
    transform: scale(1);
  }
  h2 {
    margin-top: 24px;
    margin-bottom: 8px;
    font-size: 16px;
  }
  .list {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-bottom: 20px;
  }
  /* the default window: the controls row needs the room more than the cover does */
  @media (max-width: 1100px) {
    .art {
      width: clamp(180px, min(30vw, 100vh - 130px), 640px);
    }
  }
  /* short windows: tighter, so Up next keeps a few rows */
  @media (max-height: 600px) {
    .playback,
    h2 {
      margin-top: 12px;
    }
    .now-playing {
      gap: 10px;
      padding-top: 12px;
    }
  }
  /* narrow windows: stack, and the whole screen scrolls in the page */
  @media (max-width: 900px) {
    .body {
      display: block;
    }
    .art {
      width: 160px;
      margin-bottom: 20px;
    }
    .list {
      overflow: visible;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .art img {
      animation: none;
    }
    .ambient,
    .hero {
      transition: none;
    }
  }
</style>
