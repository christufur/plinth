import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export type Track = {
  path: string;
  title: string;
  artist: string | null;
  album_artist: string | null;
  album: string | null;
  track_no: number | null;
  disc: number | null;
  year: number | null;
  secs: number;
};

export const library = $state({
  tracks: [] as Track[],
  folder: null as string | null,
});

export async function load(dir: string) {
  library.folder = dir;
  localStorage.setItem("folder", dir);
  library.tracks = await invoke<Track[]>("scan_folder", { path: dir });
}
export async function pick() {
  const dir = await open({ directory: true });
  if (dir) load(dir);
}
const saved = localStorage.getItem("folder");
if (saved) load(saved);

// album artist groups compilations; untagged fields get an explicit "Unknown"

export const artistOf = (t: Track) => t.album_artist ?? t.artist ?? "Unknown Artist";
// same album name by different artists must stay separate; no album tag = not in any album (yt downloads)
export const albumKey = (t: Track) => (t.album ? `${artistOf(t)}\0${t.album}` : null);

// [key, track count], sorted by key, returns [key, Track[]]
export const group = (tracks: Track[], key: (t: Track) => string | null) => {
  const m = new Map<string, Track[]>();
  for (const t of tracks) {
    const k = key(t);
    if (k !== null) (m.get(k) ?? m.set(k, []).get(k)!).push(t);
  }
  return [...m].sort((a, b) => a[0].localeCompare(b[0]));
};

export const matches = (tracks: Track[], query: string) => {
  return tracks.filter(t => t.title.toLowerCase().includes(query.toLowerCase()) || t.artist?.toLowerCase().includes(query.toLowerCase()) || t.album?.toLowerCase().includes(query.toLowerCase()));
};

//localecompare for text, numbers for year and time
export const sortBy = (tracks: Track[], key: (t: Track) => string | number | null, dir: "asc" | "desc" = "asc") => {
  const s = dir === "asc" ? 1 : -1;
  return [...tracks].sort((a, b) => {
    const x = key(a), y = key(b);
    if (x === y) return 0;
    if (x === null) return 1; // empty values last in both directions
    if (y === null) return -1;
    return s * (typeof x === "string" ? x.localeCompare(y as string) : x - (y as number));
  });
};
