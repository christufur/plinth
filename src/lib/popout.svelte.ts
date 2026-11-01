import { emit, listen } from "@tauri-apps/api/event";
import { player, next, prev, upcoming, canPrev } from "$lib/player.svelte";
import { library } from "$lib/library.svelte";

function send() {
  const current = library.tracks.find((t) => t.path === player.track);
  return emit("popout_state", {
    path: player.track,
    title: current?.title ?? (player.track ? "—" : "Nothing Playing..."),
    artist: current?.artist ?? "No artist",
    canPrev: canPrev(),
    canNext: upcoming() > 0,
  });
}

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
      listen<"next" | "prev">("popout_cmd", (e) => (e.payload === "next" ? next() : prev())),
    ];
    // listen is async: the cleanup can run before it resolves
    return () => unlisten.forEach((u) => u.then((off) => off()));
  });
}
