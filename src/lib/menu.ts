import { Menu } from "@tauri-apps/api/menu";
import { playlists, add, addTracks, removeTrack } from "$lib/playlists.svelte";

type Opts = {
  from?: { id: string; index: number }; // set when the row is inside a playlist
  onnew?: (id: string) => void; // open the playlist "+ New playlist…" just made
};

// native menu: keyboard and screen-reader support come with it
export async function trackMenu(path: string, { from, onnew }: Opts = {}) {
  const menu = await Menu.new({
    items: [
      {
        text: "Add to playlist",
        items: [
          ...playlists.playlists.map((p) => ({ text: p.name, action: () => addTracks(p.id, [path]) })),
          ...(playlists.playlists.length ? [{ item: "Separator" as const }] : []),
          {
            text: "+ New playlist…",
            action: () => {
              const id = add();
              if (!id) return;
              addTracks(id, [path]);
              onnew?.(id);
            },
          },
        ],
      },
      ...(from ? [{ text: "Remove from playlist", action: () => removeTrack(from.id, from.index) }] : []),
    ],
  });
  await menu.popup();
}
