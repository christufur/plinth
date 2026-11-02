<script lang="ts">
  import { player, play, fmt } from "$lib/player.svelte";
  import { artistOf, sortBy, type Track } from "$lib/library.svelte";
  import { reorder } from "$lib/reorder";
  import AudioLines from "@lucide/svelte/icons/audio-lines";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import ArrowUp from "@lucide/svelte/icons/arrow-up";
  import ArrowDown from "@lucide/svelte/icons/arrow-down";

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

  const columns: [string, (t: Track) => string | number | null, boolean][] = [
    ["Title", (t) => t.title, false],
    ["Artist", (t) => t.artist ?? artistOf(t), false],
    ["Album", (t) => t.album, false],
    ["Year", (t) => t.year, true],
    ["Time", (t) => t.secs || null, true],
  ];
  let sort = $state<{ col: number; dir: "asc" | "desc" } | null>(null);
  // first click ascending, second descending, third back to the list's own order
  function sortOn(col: number) {
    if (sort?.col !== col) sort = { col, dir: "asc" };
    else sort = sort.dir === "asc" ? { col, dir: "desc" } : null;
  }
  // a playlist's order is its drag order, so it never sorts
  let shown = $derived(sort && !onmove ? sortBy(tracks, columns[sort.col][1], sort.dir) : tracks);
  let paths = $derived(shown.map((t) => t.path));
  // a right-click (or ctrl-click) can also fire a click, which played the song;
  // skip clicks until the next real pointer press or key press
  let menued = false;
</script>

<div class="row head" class:handles={onmove}>
  <span class="num">#</span>
  {#each columns as [label, , isNum], c}
    {#if onmove}
      <span class:num={isNum}>{label}</span>
    {:else}
      <span class:num={isNum} role="columnheader" aria-sort={sort?.col === c ? (sort.dir === "asc" ? "ascending" : "descending") : "none"}>
        <button class="sort" class:num={isNum} onclick={() => sortOn(c)}>
          {label}
          {#if sort?.col === c}{#if sort.dir === "asc"}<ArrowUp size={12} />{:else}<ArrowDown size={12} />{/if}{/if}
        </button>
      </span>
    {/if}
  {/each}
</div>
<!-- keyed by index: a playlist can hold the same path twice -->
{#each shown as t, i (onmove ? i : t.path)}
  <div class="item" data-item>
    {#if onmove}
      <button class="handle" aria-label="Move {t.title} (Alt+Up/Down)" use:reorder={{ index: i, count: tracks.length, onmove }}
        ><GripVertical size={16} /></button
      >
    {/if}
    <button
      class="row"
      class:active={t.path === player.track}
      onpointerdown={(e) => (menued = e.button !== 0 || e.ctrlKey)}
      onkeydown={() => (menued = false)}
      onclick={() => !menued && play(t.path, paths, i)}
      oncontextmenu={(e) => {
        menued = true;
        if (!onmenu) return;
        e.preventDefault();
        onmenu(t, i);
      }}
    >
      <span class="num">
        {#if player.state === "playing" && t.path === player.track}<AudioLines size={14} aria-label="Playing" />{:else}{onmove
            ? i + 1
            : (t.track_no ?? i + 1)}{/if}
      </span>
      <span class="cell id">{t.title}</span>
      <span class="cell">{t.artist ?? artistOf(t)}</span>
      <span class="cell">{t.album ?? "—"}</span>
      <span class="num">{t.year ?? "—"}</span>
      <span class="num">{t.secs ? fmt(t.secs) : "—"}</span>
    </button>
  </div>
{:else}
  <p class="empty">{empty}</p>
{/each}

<style>
  .sort {
    all: unset;
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
  }
  /* the arrow hangs right of every label, so no label moves when a column is sorted */
  .sort :global(svg) {
    position: absolute;
    left: calc(100% + 2px);
  }
  .sort:hover {
    color: var(--text);
  }
  .sort:focus-visible {
    outline: 2px solid var(--accent);
  }
  .item {
    display: flex;
  }
  .handle {
    all: unset;
    flex: none;
    display: grid;
    place-items: center;
    width: 1.5rem;
    color: var(--faint);
    cursor: grab;
    touch-action: none; /* keep pointer drags from scrolling the list */
    user-select: none;
  }
  .handles {
    padding-left: calc(1.5rem + 12px);
  }
</style>
