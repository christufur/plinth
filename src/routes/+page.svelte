<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { player, play, toggle, seek, setVolume, fmt } from "$lib/player.svelte";

  let tracks = $state<string[]>([]);
  let folder = $state<string | null>(null);
  let scrubbing = $state(false);
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : player.pos);
  let idx = $derived(tracks.indexOf(player.track ?? ""));

  const keys: Record<string, () => void> = {
    " ": toggle,
    ArrowLeft: () => seek(player.pos - 5),
    ArrowRight: () => seek(player.pos + 5),
    ArrowUp: () => setVolume(Math.min(1, player.volume + 0.05)),
    ArrowDown: () => setVolume(Math.max(0, player.volume - 0.05)),
    n: next,
    p: prev,
  };
  function onkey(e: KeyboardEvent) {
    if ((e.target as HTMLElement).tagName === "INPUT") return;
    const fn = keys[e.key];
    if (fn) {
      e.preventDefault();
      fn();
    }
  }

  async function load(dir: string) {
    folder = dir;
    localStorage.setItem("folder", dir);
    tracks = await invoke<string[]>("scan_folder", { path: dir });
  }
  async function pick() {
    const dir = await open({ directory: true });
    if (dir) load(dir);
  }
  async function next() {
    if (idx >= 0 && idx < tracks.length - 1) {
      play(tracks[idx + 1]);
    }
  }
  async function prev() {
    if (idx > 0) {
      play(tracks[idx - 1]);
    }
  }
  listen("track_ended", next);
  const saved = localStorage.getItem("folder");
  if (saved) load(saved);
</script>

<svelte:window onkeydown={onkey} />

<main>
  <button onclick={pick}>{folder ? "Change folder" : "Open folder"}</button>
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
  <button onclick={prev} disabled={idx <= 0}>⏮</button>
  <button onclick={toggle} disabled={!player.track}>
    {player.state === "playing" ? "⏸" : "▶"}
  </button>
  <button onclick={next} disabled={idx < 0 || idx >= tracks.length - 1}>⏭</button>
  <span class="title">{player.track?.split("/").pop() ?? "—"}</span>
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
