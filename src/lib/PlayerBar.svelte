<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Heart from "@lucide/svelte/icons/heart";
  import ImageIcon from "@lucide/svelte/icons/image";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import PictureInPicture2 from "@lucide/svelte/icons/picture-in-picture-2";
  import { player, upcoming } from "$lib/player.svelte";
  import { library, coverOf, type Track } from "$lib/library.svelte";
  import { isFavorite, toggleFavorite } from "$lib/playlists.svelte";
  import QueuePopover from "$lib/QueuePopover.svelte";
  import SeekBar from "$lib/SeekBar.svelte";
  import Transport from "$lib/Transport.svelte";
  import VolumeControl from "$lib/VolumeControl.svelte";

  let { onopen, onnowplaying }: { onopen: (t: Track, kind: "albums" | "artists") => void; onnowplaying: () => void } = $props();

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
    <button class="cover" aria-label="Open Now Playing" disabled={!player.track} onclick={onnowplaying}>
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
    <SeekBar />
    <Transport />
  </div>

  <div class="side">
    <button class="icon" aria-label="Open popout player" onclick={() => invoke("open_popout")}><PictureInPicture2 size={20} /></button>
    <div class="queue-box" bind:this={queueBox}>
      <button class="btn queue-btn" bind:this={queueBtn} aria-expanded={queueOpen} onclick={() => (queueOpen = !queueOpen)}
        ><ListMusic size={18} /> Queue · {upcoming()}</button
      >
      {#if queueOpen}<QueuePopover onclose={closeQueue} />{/if}
    </div>
    <VolumeControl />
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
  .artist:hover:not(:disabled) {
    color: var(--text);
    text-decoration: underline;
  }
  .title:focus-visible,
  .artist:focus-visible {
    outline: 2px solid var(--accent);
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
