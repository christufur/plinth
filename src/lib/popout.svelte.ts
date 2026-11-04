import { emit, listen } from "@tauri-apps/api/event";
import { player, next, prev, canPrev, canNext, toggleShuffle, cycleRepeat, setVolume } from "$lib/player.svelte";
import { library } from "$lib/library.svelte";
import { isFavorite, toggleFavorite } from "$lib/playlists.svelte";

function send() {
  const current = library.tracks.find((t) => t.path === player.track);
  return emit("popout_state", {
    path: player.track,
    title: current?.title ?? (player.track ? "—" : "Nothing Playing..."),
    artist: current?.artist ?? "No artist",
    canPrev: canPrev(),
    canNext: canNext(),
    favorite: isFavorite(player.track),
    shuffle: player.shuffle,
    repeat: player.repeat,
    volume: player.volume,
  });
}

// everything here lives in main; the popout only asks
const commands: Record<string, () => unknown> = {
  next: () => next(),
  prev,
  shuffle: toggleShuffle,
  repeat: cycleRepeat,
  favorite: () => player.track && toggleFavorite(player.track),
};

// main-window side of the popout remote control; call once from a component
export function syncPopout() {
  $effect(() => {
    send();
  });
  $effect(() => {
    const unlisten = [
      listen("popout_ready", () => {
        send();
        emit("state_changed", player.state); // a new window missed the last one, so its play button would be wrong
      }),
      listen<string>("popout_cmd", (e) => commands[e.payload]?.()),
      listen<number>("popout_volume", (e) => setVolume(e.payload)),
    ];
    // listen is async: the cleanup can run before it resolves
    return () => unlisten.forEach((u) => u.then((off) => off()));
  });
}
