<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { player, play, toggle, seek, setVolume, fmt } from "$lib/player.svelte";

  let tracks = $state<string[]>([]);
  let folder = $state<string | null>(null);

  async function pick() {
    const dir = await open({ directory: true });
    if (!dir) return;
    folder = dir;
    tracks = await invoke<string[]>("scan_folder", { path: dir });
  }
</script>

<main>
  <button onclick={pick}>Open folder</button>
  {#if folder}
    <p>{folder} — {tracks.length} tracks</p>
    <ul>
      {#each tracks as t (t)}
        <li class:active={t === player.track}>
          <button onclick={() => play(t)}>{t.split("/").pop()}</button>
        </li>
      {/each}
    </ul>
  {/if}
</main>

<footer>
  <button onclick={toggle} disabled={!player.track}>
    {player.state === "playing" ? "⏸" : "▶"}
  </button>
  <span class="title">{player.track?.split("/").pop() ?? "—"}</span>
  <input
    type="range"
    min="0"
    max={player.duration}
    step="0.5"
    value={player.pos}
    onchange={(e) => seek(+e.currentTarget.value)}
  />
  <span>{fmt(player.pos)} / {fmt(player.duration)}</span>
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
  :root {
    font-family: system-ui, sans-serif;
    color-scheme: light dark;
  }
  main {
    padding: 2rem;
    padding-bottom: 5rem;
  }
  ul {
    list-style: none;
    padding: 0;
  }
  li button {
    all: unset;
    cursor: pointer;
    display: block;
    padding: 0.25rem 0;
  }
  li.active button {
    font-weight: bold;
  }
  footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    gap: 1rem;
    align-items: center;
    padding: 0.75rem 1rem;
    background: Canvas;
    border-top: 1px solid GrayText;
  }
  .title {
    flex: 0 1 12rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  footer input[type="range"]:first-of-type {
    flex: 1;
  }
</style>
