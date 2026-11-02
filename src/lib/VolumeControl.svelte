<script lang="ts">
  import Volume2 from "@lucide/svelte/icons/volume-2";
  import Volume1 from "@lucide/svelte/icons/volume-1";
  import VolumeX from "@lucide/svelte/icons/volume-x";
  import { player, setVolume, toggleMute } from "$lib/player.svelte";
</script>

<div class="vol">
  <button class="icon" aria-label={player.volume ? "Mute" : "Unmute"} onclick={toggleMute}>
    {#if !player.volume}<VolumeX size={20} />{:else if player.volume < 0.5}<Volume1 size={20} />{:else}<Volume2 size={20} />{/if}
  </button>
  <input type="range" aria-label="Volume" min="0" max="1" step="0.01" value={player.volume} oninput={(e) => setVolume(+e.currentTarget.value)} />
</div>

<style>
  .vol {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
  }
  .vol input {
    width: clamp(4rem, 9vw, 9rem);
    accent-color: var(--muted); /* volume is a setting, not data: no blue */
  }
</style>
