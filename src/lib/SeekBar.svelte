<script lang="ts">
  import { player, seek, fmt } from "$lib/player.svelte";

  let scrubbing = $state(false);
  let moved = false; // a press with no drag fires no change event, so it must end the scrub itself
  let scrubPos = $state(0);
  let shownPos = $derived(scrubbing ? scrubPos : player.pos);
  let showLeft = $state(localStorage.getItem("timeLeft") === "1");
  function toggleTimeLeft() {
    showLeft = !showLeft;
    localStorage.setItem("timeLeft", showLeft ? "1" : "0");
  }
</script>

<div class="progress">
  <span class="time">{player.track ? fmt(shownPos) : "—:—"}</span>
  <input
    class="seek"
    type="range"
    aria-label="Position"
    aria-valuetext="{fmt(shownPos)} of {fmt(player.duration)}"
    min="0"
    max={player.duration}
    step="0.5"
    value={shownPos}
    disabled={!player.track}
    onpointerdown={() => {
      scrubPos = player.pos;
      scrubbing = true;
      moved = false;
    }}
    onpointerup={() => {
      if (!moved) scrubbing = false;
    }}
    oninput={(e) => {
      scrubPos = +e.currentTarget.value;
      moved = true;
    }}
    onchange={(e) => {
      seek(+e.currentTarget.value);
      scrubbing = false;
    }}
  />
  <button class="time total" aria-label={showLeft ? "Show total time" : "Show time left"} disabled={!player.track} onclick={toggleTimeLeft}>
    {#if !player.track}—:—{:else if showLeft}-{fmt(Math.max(0, player.duration - shownPos))}{:else}{fmt(player.duration)}{/if}
  </button>
</div>

<style>
  .progress {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .time {
    flex: none;
    min-width: 2.5rem; /* m:ss and mm:ss keep the slider still; longer times just widen it */
    font-size: var(--fs-small);
    color: var(--muted);
  }
  .time:first-child {
    text-align: right;
  }
  .total {
    all: unset;
    flex: none;
    min-width: 2.5rem;
    font-size: var(--fs-small);
    color: var(--muted);
    cursor: pointer;
  }
  .total:disabled {
    cursor: default;
  }
  .total:hover:not(:disabled) {
    color: var(--text);
    text-decoration: underline;
  }
  .total:focus-visible {
    outline: 2px solid var(--accent);
  }
  .seek:disabled {
    opacity: 0.4;
  }
  input[type="range"] {
    accent-color: var(--accent); /* the progress is data, so it carries the blue */
  }
  .seek {
    flex: 1;
    min-width: 0;
  }
</style>
