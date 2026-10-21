import { invoke } from "@tauri-apps/api/core";

export type Playlist = {
  id: string;
  name: string;
  tracks: string[];
};

// error lives inside the object: a module can't export a $state it reassigns
export const playlists = $state({
  playlists: [] as Playlist[],
  error: null as string | null,
});

let loaded = false; // never save before a successful load, or it'd overwrite the file

async function load() {
  try {
    playlists.playlists = await invoke<Playlist[]>("load_playlists");
    loaded = true;
  } catch (e) {
    playlists.error = String(e); // damaged file: show it, keep saving disabled
  }
}
load();

function save() {
  invoke("save_playlists", { playlists: playlists.playlists }).catch((e) => (playlists.error = String(e)));
}

// replace one playlist with a changed copy, then save
function edit(id: string, change: (p: Playlist) => Playlist) {
  if (!loaded) return;
  playlists.playlists = playlists.playlists.map((p) => (p.id === id ? change(p) : p));
  save();
}

// returns the new id so "New playlist…" can add tracks and open it
export function add(name = `New playlist ${playlists.playlists.length + 1}`): string | null {
  if (!loaded) return null;
  const id = crypto.randomUUID();
  playlists.playlists.push({ id, name, tracks: [] });
  save();
  return id;
}

const FAVORITES = "favorites"; // fixed id, so the heart always finds the same playlist

export const isFavorite = (path: string | null) =>
  !!path && !!playlists.playlists.find((p) => p.id === FAVORITES)?.tracks.includes(path);

// the heart in the player bar; makes the Favorites playlist the first time (or again if deleted)
export function toggleFavorite(path: string) {
  if (!loaded) return;
  if (!playlists.playlists.some((p) => p.id === FAVORITES)) {
    playlists.playlists.push({ id: FAVORITES, name: "Favorites", tracks: [path] });
    save();
    return;
  }
  edit(FAVORITES, (p) => ({
    ...p,
    tracks: p.tracks.includes(path) ? p.tracks.filter((t) => t !== path) : [...p.tracks, path],
  }));
}

export function remove(id: string) {
  if (!loaded) return;
  playlists.playlists = playlists.playlists.filter((p) => p.id !== id);
  save();
}

export function rename(id: string, name: string) {
  edit(id, (p) => ({ ...p, name }));
}

// moves a whole playlist in the sidebar list
export function move(id: string, to: number) {
  if (!loaded) return;
  const playlist = playlists.playlists.find((p) => p.id === id);
  if (!playlist) return;
  playlists.playlists = playlists.playlists.filter((p) => p.id !== id);
  playlists.playlists.splice(to, 0, playlist);
  save();
}

export function addTracks(id: string, paths: string[]) {
  edit(id, (p) => ({ ...p, tracks: [...p.tracks, ...paths] }));
}

// by row index, not path: the same song can be in a playlist twice
export function removeTrack(id: string, index: number) {
  edit(id, (p) => ({ ...p, tracks: p.tracks.filter((_, i) => i !== index) }));
}

export function moveTrack(id: string, from: number, to: number) {
  edit(id, (p) => {
    const tracks = [...p.tracks];
    tracks.splice(to, 0, ...tracks.splice(from, 1));
    return { ...p, tracks };
  });
}
