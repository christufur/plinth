<script lang="ts">
  import SkipBack from "@lucide/svelte/icons/skip-back";
  import SkipForward from "@lucide/svelte/icons/skip-forward";
  import Play from "@lucide/svelte/icons/play";
  import Pause from "@lucide/svelte/icons/pause";
  import Shuffle from "@lucide/svelte/icons/shuffle";
  import Repeat from "@lucide/svelte/icons/repeat";
  import Repeat1 from "@lucide/svelte/icons/repeat-1";
  import { player, toggle, next, prev, canPrev, canNext, toggleShuffle, cycleRepeat } from "$lib/player.svelte";

  let { compact = false }: { compact?: boolean } = $props(); // compact: sits in a row of its own, not under a seek bar
</script>

<div class="controls" class:compact>
  <button class="icon end" class:on={player.shuffle} aria-pressed={player.shuffle} aria-label="Shuffle" onclick={toggleShuffle}>
    <Shuffle size={16} />
  </button>
  <div class="transport">
    <button class="icon" aria-label="Previous" onclick={prev} disabled={!canPrev()}><SkipBack size={20} fill="currentColor" /></button>
    <button class="play" aria-label={player.state === "playing" ? "Pause" : "Play"} onclick={toggle} disabled={!player.track}>
      {#if player.state === "playing"}<Pause size={18} fill="currentColor" />{:else}<Play size={18} fill="currentColor" />{/if}
    </button>
    <button class="icon" aria-label="Next" onclick={() => next()} disabled={!canNext()}><SkipForward size={20} fill="currentColor" /></button>
  </div>
  <button class="icon end" class:on={player.repeat !== "off"} aria-label="Repeat: {player.repeat}" onclick={cycleRepeat}>
    {#if player.repeat === "one"}<Repeat1 size={16} />{:else}<Repeat size={16} />{/if}
  </button>
</div>

<style>
  /* shuffle and repeat sit under the ends of the seek bar, so previous, play and next stay on the centre line */
  .controls {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding-inline: calc(2.5rem + 8px); /* the width of a time plus its gap */
  }
  .controls.compact {
    display: flex;
    gap: 4px;
    padding-inline: 0;
  }
  .controls .end {
    width: 32px;
    height: 32px;
    position: relative;
  }
  .controls .end:last-child {
    justify-self: end;
  }
  .controls .end.on::after {
    content: "";
    position: absolute;
    bottom: 2px;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: currentColor; /* on means more than blue, so it reads without colour too */
  }
  .transport {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .transport .icon {
    width: 36px;
    height: 36px;
  }
  .play {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin: 0 6px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: var(--primary);
    color: var(--on-primary);
    cursor: pointer;
  }
  .play:hover:not(:disabled) {
    transform: scale(1.05);
  }
  .play:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
