<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import CirclePlay from "@lucide/svelte/icons/circle-play";
  import CirclePause from "@lucide/svelte/icons/circle-pause";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Heart from "@lucide/svelte/icons/heart";
  import { player, toggle, seek, setVolume, next, prev, queueIdx, toggleShuffle, fmt } from "$lib/player.svelte";
  import { library } from "$lib/library.svelte";
  import { isFavorite, toggleFavorite } from "$lib/playlists.svelte";

  let scrubbing = $state(false);
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : player.pos);
  let idx = $derived(queueIdx());
  let current = $derived(library.tracks.find((t) => t.path === player.track));
  let fav = $derived(isFavorite(player.track));

  // reactive variables for marquee animation
  let titleW = $state(0);
  let textW = $state(0);
  let overflow = $derived(textW > titleW);

  // cover for the playing track; cached per path so skipping back doesn't re-read the file
  const covers = new Map<string, string | null>();
  let cover = $state<string | null>(null);
  $effect(() => {
    const path = player.track;
    if (!path) return void (cover = null);
    if (covers.has(path)) return void (cover = covers.get(path)!);
    cover = null;
    invoke<string | null>("cover_art", { path }).then((c) => {
      covers.set(path, c);
      if (player.track === path) cover = c; // ignore if the track changed meanwhile
    });
  });
</script>

<footer>
  <button class="icon" aria-label="Previous" onclick={prev} disabled={idx <= 0}><SkipBack size={24} /></button>
  <button class="icon play" aria-label={player.state === "playing" ? "Pause" : "Play"} onclick={toggle} disabled={!player.track}>
    {#if player.state === "playing"}<CirclePause size={36} />{:else}<CirclePlay size={36} />{/if}
  </button>
  <button class="icon" aria-label="Next" onclick={next} disabled={idx < 0 || idx >= player.queue.length - 1}><SkipForward size={24} /></button>
  <button class="icon" class:on={player.shuffle} aria-pressed={player.shuffle} aria-label="Shuffle" onclick={toggleShuffle}>
    <Shuffle size={20} />
  </button>
  <button
    class="icon"
    class:on={fav}
    aria-pressed={fav}
    aria-label="Favorite"
    disabled={!player.track}
    onclick={() => player.track && toggleFavorite(player.track)}><Heart size={20} fill={fav ? "currentColor" : "none"} /></button
  >
  {#if cover}<img class="cover" src={cover} alt="" />{/if}
  <div class="title" bind:clientWidth={titleW}>
    <div class = "marquee" class:scrolling={overflow}>
      <span bind:clientWidth={textW}>{current?.title ?? "—"}</span>
      {#if overflow}
        <span aria-hidden="true">{current?.title ?? "—"}</span>
      {/if}
    </div>
  </div>
  <input
    type="range"
    min="0"
    max={player.duration}
    step="0.5"
    value={shownPos}
    onpointerdown={() => {
      scrubPos = player.pos;
      scrubbing = true;
    }}
    oninput={(e) => (scrubPos = +e.currentTarget.value)}
    onchange={(e) => {
      seek(+e.currentTarget.value);
      scrubbing = false;
    }}
  />
  <span>{fmt(shownPos)} / {fmt(player.duration)}</span>
  <input
    type="range"
    min="0"
    max="1"
    step="0.01"
    value={player.volume}
    oninput={(e) => setVolume(+e.currentTarget.value)}
  />
</footer>

<style>
  footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 2rem 2rem;
    background: Canvas;
    border-top: 1px solid GrayText;
  }
  .cover {
    width: 5rem;
    height: 5rem;
    object-fit: cover;
    border-radius: 4px;
    flex: none;
  }
  .play {
    width: 3rem;
    height: 3rem;
  }
  .title {
    flex: 0 0 12rem;
    overflow: hidden;
    white-space: nowrap;
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
  @media (prefers-reduced-motion: reduce) {
    .marquee.scrolling {
      animation: none;
    }
  }
  @keyframes marquee {
    from{
      transform: translateX(0);
    }
    to {
      transform: translateX(-50%);
    }
  }
  footer input[type="range"]:first-of-type {
    flex: 1;
  }
</style>
