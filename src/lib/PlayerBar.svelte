<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { player, toggle, seek, setVolume, next, prev, queueIdx, fmt } from "$lib/player.svelte";
  import { library, artistOf } from "$lib/library.svelte";

  let scrubbing = $state(false);
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : player.pos);
  let idx = $derived(queueIdx());
  let current = $derived(library.tracks.find((t) => t.path === player.track));

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
  <button onclick={prev} disabled={idx <= 0}>⏮</button>
  <button onclick={toggle} disabled={!player.track}>
    {player.state === "playing" ? "⏸" : "▶"}
  </button>
  <button onclick={next} disabled={idx < 0 || idx >= player.queue.length - 1}
    >⏭</button
  >
  {#if cover}<img class="cover" src={cover} alt="" />{:else}<div class="cover empty" aria-hidden="true">♪</div>{/if}
  <div class="title" bind:clientWidth={titleW}>
    <div class = "marquee" class:scrolling={overflow}>
      <span bind:clientWidth={textW}>{current?.title ?? "Nothing playing"}</span>
      {#if overflow}
        <span aria-hidden="true">{current?.title ?? "Nothing playing"}</span>
      {/if}
    </div>
    <div class="artist">{current ? (current.artist ?? artistOf(current)) : ""}</div>
  </div>
  <input
    type="range"
    aria-label="Seek"
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
  <span class="time">{fmt(shownPos)} / {fmt(player.duration)}</span>
  <span class="n">Vol</span>
  <input
    type="range"
    aria-label="Volume"
    class="volume"
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
  /* same box when a song has no cover, so the title doesn't jump sideways */
  .cover.empty {
    display: grid;
    place-items: center;
    box-sizing: border-box;
    border: 1px solid var(--line);
    background: var(--line);
    font-size: 2rem;
    opacity: 0.6;
  }
  .title > .marquee {
    font-weight: 600;
    font-size: 1.1rem;
  }
  .artist {
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0.7;
  }
  .time {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .volume {
    width: 7rem;
  }
  button {
    width: 2.5rem;
    height: 2.5rem;
    font-size: 1.5rem;
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
