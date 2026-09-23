<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let pong = $state("…");
  invoke<string>("ping").then((r) => (pong = r)).catch((e) => (pong = String(e)));
</script>

<main>
  <h1>Plinth</h1>
  <p>ping → {pong}</p>
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
