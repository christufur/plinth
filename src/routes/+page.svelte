<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

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
        <li>{t.split("/").pop()}</li>
      {/each}
    </ul>
  {/if}
</main>

<style>
  :root {
    font-family: system-ui, sans-serif;
    color-scheme: light dark;
  }
  main {
    padding: 2rem;
  }
</style>
