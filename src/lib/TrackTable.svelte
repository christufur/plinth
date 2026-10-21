<script lang="ts">
  import { player, play, fmt } from "$lib/player.svelte";
  import { artistOf, type Track } from "$lib/library.svelte";
  import { reorder } from "$lib/reorder";

  let {
    tracks,
    empty = "No tracks.",
    onmenu,
    onmove,
  }: {
    tracks: Track[];
    empty?: string;
    onmenu?: (t: Track, index: number) => void;
    onmove?: (from: number, to: number) => void; // set = playlist: ≡ handles, row numbers instead of track numbers
  } = $props();

  let paths = $derived(tracks.map((t) => t.path));
  // a right-click (or ctrl-click) can also fire a click, which played the song;
  // skip clicks until the next real pointer press or key press
  let menued = false;
</script>

<div class="row head" class:handles={onmove}>
  <span>#</span><span>Title</span><span>Artist</span><span>Album</span><span>Year</span><span>Time</span>
</div>
<!-- keyed by index: a playlist can hold the same path twice -->
{#each tracks as t, i (onmove ? i : t.path)}
  <div class="item" data-item>
    {#if onmove}
      <button class="handle" aria-label="Move {t.title} (Alt+Up/Down)" use:reorder={{ index: i, count: tracks.length, onmove }}>≡</button>
    {/if}
    <button
      class="row"
      class:active={t.path === player.track}
      onpointerdown={(e) => (menued = e.button !== 0 || e.ctrlKey)}
      onkeydown={() => (menued = false)}
      onclick={() => !menued && play(t.path, paths)}
      oncontextmenu={(e) => {
        menued = true;
        if (!onmenu) return;
        e.preventDefault();
        onmenu(t, i);
      }}
    >
      <span class="n">{player.state === "playing" && t.path === player.track ? "▶" : onmove ? i + 1 : (t.track_no ?? i + 1)}</span>
      <span class="cell">{t.title}</span>
      <span class="cell">{t.artist ?? artistOf(t)}</span>
      <span class="cell">{t.album ?? "—"}</span>
      <span class="n">{t.year ?? ""}</span>
      <span class="n">{t.secs ? fmt(t.secs) : ""}</span>
    </button>
  </div>
{:else}
  <p><em>{empty}</em></p>
{/each}

<style>
  .item {
    display: flex;
  }
  .handle {
    all: unset;
    flex: none;
    width: 1.5rem;
    text-align: center;
    cursor: grab;
    touch-action: none; /* keep pointer drags from scrolling the list */
    user-select: none;
  }
  .handles {
    padding-left: calc(1.5rem + 6px);
  }
</style>
