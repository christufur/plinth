<script lang="ts">
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Undo2 from "@lucide/svelte/icons/undo-2";
  import Play from "@lucide/svelte/icons/play";
  import GripVertical from "@lucide/svelte/icons/grip-vertical";
  import {
    player,
    playAt,
    playQueued,
    replay,
    moveInQueue,
    moveInContext,
    clearQueue,
    clearHistory,
    toggleShuffle,
    undoQueue,
  } from "$lib/player.svelte";
  import { library } from "$lib/library.svelte";
  import { add, addTracks } from "$lib/playlists.svelte";
  import { reorder } from "$lib/reorder";

  let { onclose }: { onclose: () => void } = $props();

  let byPath = $derived(new Map(library.tracks.map((t) => [t.path, t])));
  const title = (path: string) => byPath.get(path)?.title ?? path.split(/[\\/]/).pop();
  let rest = $derived(player.context.slice(player.at + 1)); // what plays after the queue is empty
  // absolute context index of row k in the "next from" list
  const abs = (k: number) => player.at + 1 + k;

  function saveAsPlaylist() {
    const id = add();
    if (id) addTracks(id, [...(player.track ? [player.track] : []), ...player.queue, ...rest]);
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<section class="queue" aria-label="Queue">
  <header>
    <h2>Up next</h2>
    <button
      class="icon"
      class:on={player.shuffle}
      aria-pressed={player.shuffle}
      aria-label="Shuffle"
      onclick={toggleShuffle}
      {@attach (el: HTMLButtonElement) => el.focus()}><Shuffle size={18} /></button
    >
    <button class="icon" aria-label="Undo last queue change" disabled={!player.canUndo} onclick={undoQueue}
      ><Undo2 size={18} /></button
    >
  </header>

  <!-- opens scrolled to the current track: scroll up for history, down for what's next -->
  <div class="lists">
    {#if player.history.length}
      <h3 class="with-action">Recently played <button class="link" onclick={clearHistory}>Clear</button></h3>
      <ol>
        {#each player.history as path, k (k)}
          <li><button class="track past" onclick={() => replay(path)}><span class="cell">{title(path)}</span></button></li>
        {/each}
      </ol>
    {/if}

    {#if player.track}
      <h3>Now playing</h3>
      <p
        class="current"
        aria-current="true"
        {@attach (el: HTMLElement) => {
          // put the "Now playing" heading at the top of the scroll box
          const box = el.parentElement!;
          box.scrollTop = el.previousElementSibling!.getBoundingClientRect().top - box.getBoundingClientRect().top;
        }}
      >
        <span class="mark"><Play size={14} fill="currentColor" /></span>
        <span class="cell">{title(player.track)}</span>
      </p>
    {/if}

    {#if player.queue.length}
      <h3>Next in queue</h3>
      <!-- keyed by index: the same song can be queued twice -->
      <ol>
        {#each player.queue as path, k (k)}
          <li data-item>
            <button
              class="handle"
              aria-label="Move {title(path)} (Alt+Up/Down)"
              use:reorder={{ index: k, count: player.queue.length, onmove: moveInQueue }}
              ><GripVertical size={16} /></button
            >
            <button class="track" onclick={() => playQueued(k)}><span class="cell">{title(path)}</span></button>
          </li>
        {/each}
      </ol>
    {/if}

    {#if rest.length}
      <h3>Next from list</h3>
      <ol>
        {#each rest as path, k (k)}
          <li data-item>
            <button
              class="handle"
              aria-label="Move {title(path)} (Alt+Up/Down)"
              use:reorder={{ index: k, count: rest.length, onmove: (f, t) => moveInContext(abs(f), abs(t)) }}
              ><GripVertical size={16} /></button
            >
            <button class="track" onclick={() => playAt(abs(k))}><span class="cell">{title(path)}</span></button>
          </li>
        {/each}
      </ol>
    {/if}

    {#if !player.queue.length && !rest.length}
      <p class="empty">Nothing up next. Right-click a track and choose Add to queue.</p>
    {/if}
  </div>

  <footer>
    <button class="link" onclick={clearQueue} disabled={!player.queue.length}>Clear queue</button>
    <button class="link" onclick={saveAsPlaylist} disabled={!player.track && !player.queue.length && !rest.length}
      >Save as playlist</button
    >
  </footer>
</section>

<style>
  .queue {
    position: absolute;
    right: 0;
    bottom: calc(100% + 0.75rem);
    width: 24rem;
    height: min(28rem, 60vh); /* fixed size; the lists scroll inside */
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: light-dark(#fff, #242424); /* a step lighter than the page: it sits above it */
    border-radius: 16px;
    box-shadow: 0 12px 32px light-dark(#0002, #0005); /* it floats, so it is the one thing with a shadow */
  }
  header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 12px 12px 8px 20px;
  }
  h2 {
    flex: 1;
    font-size: 16px;
  }
  .lists {
    flex: 1;
    min-height: 0;
    padding: 0 8px 8px;
    overflow: auto;
  }
  .past {
    padding-left: 2rem;
    color: var(--faint);
  }
  .with-action {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }
  h3 {
    margin: 16px 12px 4px;
    font-size: var(--fs-small);
    font-weight: 400;
    color: var(--faint);
  }
  ol {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li,
  .current {
    margin: 0;
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 36px;
  }
  .current {
    background: var(--raised);
    border-radius: var(--radius-row);
    font-weight: 500;
  }
  .mark {
    display: grid;
    place-items: center;
    width: 1.75rem;
    color: var(--accent);
  }
  .handle {
    all: unset;
    display: grid;
    place-items: center;
    width: 1.75rem;
    height: 1.75rem;
    cursor: grab;
    color: var(--faint);
    touch-action: none; /* keep pointer drags from scrolling the list */
    user-select: none;
  }
  .track {
    all: unset;
    flex: 1;
    min-width: 0;
    padding: 8px;
    border-radius: var(--radius-row);
    cursor: pointer;
  }
  .track:hover {
    background: var(--raised);
  }
  .empty {
    padding: 8px 12px;
    color: var(--muted);
  }
  footer {
    display: flex;
    justify-content: space-between;
    padding: 12px 20px;
    border-top: 1px solid var(--rule);
  }
  .link {
    all: unset;
    color: var(--muted);
    cursor: pointer;
  }
  .link:hover:not(:disabled) {
    color: var(--text);
  }
  .link:focus-visible {
    outline: 2px solid var(--accent);
  }
  .link:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .icon {
    width: 32px;
    height: 32px;
  }
</style>
