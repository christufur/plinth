<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import { player, toggle, seek, setVolume, next, prev, fmt } from "$lib/player.svelte";
  import { library, pick, artistOf, albumKey, group, type Track } from "$lib/library.svelte";
  import { playlists, add, rename, remove, moveTrack, type Playlist } from "$lib/playlists.svelte";
  import { trackMenu } from "$lib/menu";
  import Download from "$lib/Download.svelte";
  import PlayerBar from "$lib/PlayerBar.svelte";
  import TrackTable from "$lib/TrackTable.svelte";

  let showURL = $state(false);

  type View = "artists" | "albums" | "tracks" | "playlists";
  let view = $state<View>("tracks");
  let sel = $state<{ kind: "artists" | "albums" | "playlist"; v: string } | null>(null);
  let renaming = $state(false);

  let artists = $derived(group(library.tracks, artistOf));
  let albums = $derived(group(library.tracks, albumKey));
  let visible = $derived(
    sel && sel.kind !== "playlist"
      ? library.tracks.filter((t) => (sel!.kind === "artists" ? artistOf(t) : albumKey(t)) === sel!.v)
      : library.tracks,
  );
  let nav = $derived([
    ["artists", artists.length],
    ["albums", albums.length],
    ["tracks", library.tracks.length],
    ["playlists", playlists.playlists.length],
  ] as const);
  const label = (k: string) => k.replace("\0", " — ");

  // playlists store paths; a file that left the library still shows, so row indexes stay right
  let byPath = $derived(new Map(library.tracks.map((t) => [t.path, t])));
  const missing = (path: string): Track => ({
    path,
    title: `${path.split(/[\\/]/).pop()} (missing)`,
    artist: null,
    album_artist: null,
    album: null,
    track_no: null,
    disc: null,
    year: null,
    secs: 0,
  });
  const tracksOf = (p: Playlist) => p.tracks.map((path) => byPath.get(path) ?? missing(path));
  const totalSecs = (ts: Track[]) => ts.reduce((s, t) => s + t.secs, 0);
  let pl = $derived(sel?.kind === "playlist" ? playlists.playlists.find((p) => p.id === sel!.v) : undefined);
  let plTracks = $derived(pl ? tracksOf(pl) : []);

  function go(v: View) {
    view = v;
    sel = null;
  }
  function drill(kind: "artists" | "albums", v: string) {
    sel = { kind, v };
    view = "tracks";
  }
  function openPlaylist(id: string, renameNow = false) {
    sel = { kind: "playlist", v: id };
    view = "playlists";
    renaming = renameNow;
  }
  function newPlaylist() {
    const id = add();
    if (id) openPlaylist(id, true);
  }
  const menuFor = (t: Track) => trackMenu(t.path, { onnew: (id) => openPlaylist(id, true) });

  function commitRename(input: HTMLInputElement) {
    if (!renaming || !pl) return; // Esc already cancelled; this is the blur from removing the input
    renaming = false;
    const name = input.value.trim();
    if (name && name !== pl.name) rename(pl.id, name);
  }
  async function deletePlaylist() {
    if (!pl) return;
    const { id, name } = pl;
    const ok = await ask(`Delete "${name}"? The songs stay in your library.`, { title: "Delete playlist", kind: "warning" });
    if (!ok) return;
    remove(id);
    sel = null;
  }

  const keys: Record<string, () => void> = {
    " ": toggle,
    ArrowLeft: () => seek(player.pos - 5),
    ArrowRight: () => seek(player.pos + 5),
    ArrowUp: () => setVolume(Math.min(1, player.volume + 0.05)),
    ArrowDown: () => setVolume(Math.max(0, player.volume - 0.05)),
    n: next,
    p: prev,
  };
  function onkey(e: KeyboardEvent) {
    if ((e.target as HTMLElement).tagName === "INPUT") return;
    const fn = keys[e.key];
    if (fn) {
      e.preventDefault();
      fn();
    }
  }
</script>

<svelte:window onkeydown={onkey} />

<div class="body">
  <aside>
    {#each nav as [v, n]}
      <button class:active={view === v} onclick={() => go(v)}>
        <span>{v}</span><span class="n">{n}</span>
      </button>
    {/each}
    <hr />
    <div class="head playlists-head">
      <span>playlists</span>
      <button onclick={newPlaylist} aria-label="New playlist">＋</button>
    </div>
    {#each playlists.playlists as p (p.id)}
      <button class:active={pl?.id === p.id} onclick={() => openPlaylist(p.id)}>
        <span class="cell">{p.name}</span><span class="n">{p.tracks.length}</span>
      </button>
    {:else}
      <em class="n">none yet</em>
    {/each}
    {#if playlists.error}
      <p class="error" role="alert">{playlists.error}</p>
    {/if}
  </aside>

  <main>
    <div class="bar">
      <span class="n path">{library.folder ?? "No folder"}</span>
      <button onclick={pick}>{library.folder ? "Change folder" : "Open folder"}</button>
      <button onclick={() => (showURL = !showURL)}>＋ URL</button>
    </div>
    <Download open={showURL} />

    {#if view === "tracks"}
      {#if sel}
        <p>{label(sel.v)} <button onclick={() => (sel = null)}>×</button></p>
      {/if}
      <TrackTable
        tracks={visible}
        empty={library.folder ? "No audio files found." : "Open a folder to start."}
        onmenu={menuFor}
      />
    {:else if view === "playlists" && pl}
      {@const id = pl.id}
      <div class="band">
        {#if renaming}
          <input
            aria-label="Playlist name"
            value={pl.name}
            {@attach (el: HTMLInputElement) => {
              el.focus();
              el.select();
            }}
            onkeydown={(e) => {
              if (e.key === "Enter") commitRename(e.currentTarget);
              if (e.key === "Escape") renaming = false;
            }}
            onblur={(e) => commitRename(e.currentTarget)}
          />
        {:else}
          <h2>{pl.name}</h2>
        {/if}
        <span class="n">{pl.tracks.length} tracks • {fmt(totalSecs(plTracks))}</span>
        <span class="actions">
          <button onclick={() => (renaming = true)}>rename</button>
          <button onclick={deletePlaylist}>delete</button>
        </span>
      </div>
      <TrackTable
        tracks={plTracks}
        empty="Empty playlist. Right-click a track and choose Add to playlist."
        onmenu={(t, i) => trackMenu(t.path, { from: { id, index: i }, onnew: (nid) => openPlaylist(nid, true) })}
        onmove={(from, to) => moveTrack(id, from, to)}
      />
    {:else if view === "playlists"}
      <div class="row head" style:--cols="2.5rem 1fr 4rem 4rem">
        <span>#</span><span>Playlist</span><span>Tracks</span><span>Time</span>
      </div>
      {#each playlists.playlists as p, i (p.id)}
        <button class="row" style:--cols="2.5rem 1fr 4rem 4rem" onclick={() => openPlaylist(p.id)}>
          <span class="n">{i + 1}</span>
          <span class="cell">{p.name}</span>
          <span class="n">{p.tracks.length}</span>
          <span class="n">{fmt(totalSecs(tracksOf(p)))}</span>
        </button>
      {:else}
        <p><em>No playlists yet. Press ＋ in the sidebar.</em></p>
      {/each}
    {:else}
      {@const kind = view as "artists" | "albums"}
      {@const items = kind === "artists" ? artists : albums}
      <div class="row head" style:--cols={kind === "artists" ? "2.5rem 1fr 4rem" : "2.5rem 3fr 2fr 4rem"}>
        <span>#</span>
        <span>{kind === "artists" ? "Artist" : "Album"}</span>
        {#if kind === "albums"}<span>Artist</span>{/if}
        <span>Tracks</span>
      </div>
      {#each items as [k, n], i (k)}
        <button class="row" style:--cols={kind === "artists" ? "2.5rem 1fr 4rem" : "2.5rem 3fr 2fr 4rem"} onclick={() => drill(kind, k)}>
          <span class="n">{i + 1}</span>
          {#if kind === "albums"}
            <span class="cell">{k.split("\0")[1]}</span>
            <span class="cell">{k.split("\0")[0]}</span>
          {:else}
            <span class="cell">{k}</span>
          {/if}
          <span class="n">{n.length}</span>
        </button>
      {:else}
        <p><em>None found.</em></p>
      {/each}
    {/if}
  </main>
</div>

<PlayerBar />

<style>
  .body {
    display: grid;
    grid-template-columns: 180px 1fr;
    height: calc(100vh - 4rem);
  }
  aside {
    border-right: 1px solid GrayText;
    padding: 8px;
    overflow: auto;
  }
  aside button {
    all: unset;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    width: 100%;
    box-sizing: border-box;
    padding: 2px 6px;
    border-radius: var(--r);
    text-transform: capitalize;
  }
  aside .cell {
    text-transform: none; /* playlist names show as the user typed them */
  }
  aside button.active {
    background: var(--accent);
    color: #fff;
  }
  .playlists-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  aside .playlists-head button {
    width: auto;
    padding: 0 4px;
  }
  .error {
    color: crimson;
  }
  main {
    padding: 1rem 2rem;
    overflow: auto;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }
  .path {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .band {
    display: flex;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.75rem;
  }
  .band h2 {
    margin: 0;
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 0.5rem;
  }
</style>
