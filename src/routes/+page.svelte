<script lang="ts">
  import { ask } from "@tauri-apps/plugin-dialog";
  import MicVocal from "@lucide/svelte/icons/mic-vocal";
  import Disc3 from "@lucide/svelte/icons/disc-3";
  import Music from "@lucide/svelte/icons/music";
  import ListMusic from "@lucide/svelte/icons/list-music";
  import Plus from "@lucide/svelte/icons/plus";
  import Search from "@lucide/svelte/icons/search";
  import Folder from "@lucide/svelte/icons/folder";
  import LinkIcon from "@lucide/svelte/icons/link";
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Sun from "@lucide/svelte/icons/sun";
  import Moon from "@lucide/svelte/icons/moon";
  import { player, play, toggle, seek, setVolume, next, prev, toggleMute, toggleShuffle, cycleRepeat, fmt } from "$lib/player.svelte";
  import { library, pick, artistOf, albumKey, group, matches, type Track } from "$lib/library.svelte";
  import { playlists, add, rename, remove, moveTrack, type Playlist } from "$lib/playlists.svelte";
  import { syncPopout } from "$lib/popout.svelte";
  import { trackMenu } from "$lib/menu";
  import Download from "$lib/Download.svelte";
  import PlayerBar from "$lib/PlayerBar.svelte";
  import TrackTable from "$lib/TrackTable.svelte";

  syncPopout();

  let showURL = $state(false);
  let query = $state("");
  let searchEl: HTMLInputElement;

  // light/dark switch; until the user picks one, the system setting decides.
  // app.css colors use light-dark(), which follows color-scheme on the root element
  type Theme = "light" | "dark";
  let theme = $state<Theme>(
    (localStorage.getItem("theme") as Theme | null) ?? (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"),
  );
  $effect(() => {
    document.documentElement.style.colorScheme = theme;
  });
  function toggleTheme() {
    theme = theme === "dark" ? "light" : "dark";
    localStorage.setItem("theme", theme);
  }

  type View = "artists" | "albums" | "tracks" | "playlists";
  let view = $state<View>("tracks");
  let sel = $state<{ kind: "artists" | "albums" | "playlist"; v: string } | null>(null);
  let renaming = $state(false);

  let artists = $derived(group(library.tracks, artistOf));
  let albums = $derived(group(library.tracks, albumKey));
  let visible = $derived(
    query
      ? matches(library.tracks, query)
      : sel && sel.kind !== "playlist"
        ? library.tracks.filter((t) => (sel!.kind === "artists" ? artistOf(t) : albumKey(t)) === sel!.v)
        : library.tracks,
  );
  let nav = $derived([
    ["artists", "Artists", MicVocal, artists.length],
    ["albums", "Albums", Disc3, albums.length],
    ["tracks", "Tracks", Music, library.tracks.length],
    ["playlists", "Playlists", ListMusic, playlists.playlists.length],
  ] as const);

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
  const albumCount = (ts: Track[]) => new Set(ts.map(albumKey).filter(Boolean)).size;
  const plural = (n: number, word: string) => `${n} ${word}${n === 1 ? "" : "s"}`;
  let pl = $derived(sel?.kind === "playlist" ? playlists.playlists.find((p) => p.id === sel!.v) : undefined);
  let plTracks = $derived(pl ? tracksOf(pl) : []);
  let folderName = $derived(library.folder?.split(/[\\/]/).filter(Boolean).pop());

  // the open artist, album or playlist: its tracks
  let headTracks = $derived(pl ? plTracks : sel && !query ? visible : []);

  function go(v: View) {
    view = v;
    sel = null;
    query = "";
  }
  function drill(kind: "artists" | "albums", v: string) {
    sel = { kind, v };
    view = kind;
  }
  function openPlaylist(id: string, renameNow = false) {
    sel = { kind: "playlist", v: id };
    view = "playlists";
    query = "";
    renaming = renameNow;
  }
  function newPlaylist() {
    const id = add();
    if (id) openPlaylist(id, true);
  }
  const playAll = (ts: Track[]) => ts.length && play(ts[0].path, ts.map((t) => t.path), 0);
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
    n: () => next(),
    p: prev,
    m: toggleMute,
    s: toggleShuffle,
    r: cycleRepeat,
  };
  function onkey(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "f") {
      e.preventDefault();
      searchEl.focus();
      return;
    }
    if (e.metaKey || e.ctrlKey || e.altKey || (e.target as HTMLElement).tagName === "INPUT") return; // leave ⌘M, ⌘R and the like to the system
    const fn = keys[e.key];
    if (fn) {
      e.preventDefault();
      fn();
    }
  }
</script>

<svelte:window onkeydown={onkey} />

<div class="app">
  <header class="top">
    <label class="search">
      <Search size={16} aria-hidden="true" />
      <input
        type="search"
        placeholder="Search library"
        aria-label="Search the library (⌘F)"
        bind:this={searchEl}
        bind:value={query}
        oninput={() => {
          view = "tracks";
          sel = null;
        }}
        onkeydown={(e) => e.key === "Escape" && ((query = ""), searchEl.blur())}
      />
      <kbd>⌘F</kbd>
    </label>
    <div class="top-actions">
      <button class="btn" aria-expanded={showURL} onclick={() => (showURL = !showURL)}><LinkIcon size={16} /> Add from URL</button>
      <button class="icon" onclick={toggleTheme} aria-label={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}>
        {#if theme === "dark"}<Sun size={16} />{:else}<Moon size={16} />{/if}
      </button>
    </div>
  </header>

  <nav aria-label="Library">
    {#each nav as [v, label, Icon, n]}
      <button class="nav lib" class:active={view === v && !pl} aria-current={view === v && !pl ? "page" : undefined} onclick={() => go(v)}>
        <Icon size={16} />
        <span>{label}</span>
        <span class="num count">{n}</span>
      </button>
    {/each}
    <div class="group">
      <span>Your playlists</span>
      <button class="icon small" onclick={newPlaylist} aria-label="New playlist"><Plus size={16} /></button>
    </div>
    {#each playlists.playlists as p (p.id)}
      <button class="nav" class:active={pl?.id === p.id} aria-current={pl?.id === p.id ? "page" : undefined} onclick={() => openPlaylist(p.id)}>
        <span class="cell">{p.name}</span>
      </button>
    {:else}
      <p class="hint">No playlists yet</p>
    {/each}
    {#if playlists.error}
      <p class="error" role="alert">{playlists.error}</p>
    {/if}
    <button class="switcher" onclick={pick} title={library.folder ?? "Choose your music folder"}>
      <Folder size={16} />
      <span class="cell">{folderName ?? "Choose folder"}</span>
    </button>
  </nav>

  <main>
    <Download open={showURL} />

    {#if !library.folder}
      <div class="title-row">
        <div>
          <h1>Tracks</h1>
          <p class="meta">No library yet</p>
        </div>
      </div>
      <div class="row head">
        <span class="num">#</span><span>Title</span><span>Artist</span><span>Album</span><span class="num">Year</span><span
          class="num">Time</span
        >
      </div>
      <section class="first-run">
        <p>Choose the folder your music is in. Plinth reads it and never moves your files.</p>
        <button class="btn primary" onclick={pick}><Folder size={16} /> Open folder</button>
      </section>
    {:else if pl || (sel && sel.kind !== "playlist" && !query)}
      {@const kind = pl ? "playlist" : sel!.kind}
      {@const id = pl?.id}
      {@const name = pl ? pl.name : kind === "albums" ? sel!.v.split("\0")[1] : sel!.v}
      <button class="back" onclick={() => (pl ? go("playlists") : go(sel!.kind as View))}>
        <ArrowLeft size={14} />
        {pl ? "Playlists" : kind === "albums" ? "Albums" : "Artists"}
      </button>
      <section class="title-row">
        <div>
          {#if renaming && pl}
            <input
              class="rename"
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
            <h1 class="cell">{name}</h1>
          {/if}
          <p class="meta">
            {#if kind === "albums"}{sel!.v.split("\0")[0]} •{/if}
            {#if kind === "artists"}{plural(albumCount(headTracks), "album")} •{/if}
            {plural(headTracks.length, "track")} • {fmt(totalSecs(headTracks))}
          </p>
        </div>
        <div class="actions">
          <button class="btn primary" onclick={() => playAll(headTracks)} disabled={!headTracks.length}>Play</button>
          {#if pl}
            <button class="btn" onclick={() => (renaming = true)}>Rename</button>
            <button class="btn danger" onclick={deletePlaylist}>Delete</button>
          {/if}
        </div>
      </section>
      {#if pl && id}
        <TrackTable
          tracks={plTracks}
          empty="This playlist is empty. Right-click a track and choose Add to playlist."
          onmenu={(t, i) => trackMenu(t.path, { from: { id, index: i }, onnew: (nid) => openPlaylist(nid, true) })}
          onmove={(from, to) => moveTrack(id, from, to)}
        />
      {:else}
        <TrackTable tracks={visible} onmenu={menuFor} />
      {/if}
    {:else if view === "tracks"}
      <div class="title-row">
        <div>
          <h1>{query ? "Search" : "Tracks"}</h1>
          <p class="meta">
            {query
              ? `${visible.length} of ${plural(library.tracks.length, "track")} match “${query}”`
              : `${plural(albums.length, "album")} • ${plural(library.tracks.length, "track")} • ${fmt(totalSecs(library.tracks))}`}
          </p>
        </div>
        <button class="btn primary" onclick={() => playAll(visible)} disabled={!visible.length}>Play</button>
      </div>
      <TrackTable
        tracks={visible}
        empty={query ? "No tracks match. Esc clears the search." : "No audio files found in this folder."}
        onmenu={menuFor}
      />
    {:else if view === "playlists"}
      <div class="title-row">
        <div>
          <h1>Playlists</h1>
          <p class="meta">{plural(playlists.playlists.length, "playlist")}</p>
        </div>
        <button class="btn primary" onclick={newPlaylist}><Plus size={16} /> New playlist</button>
      </div>
      <div class="row head" style:--cols="1fr 5rem 5rem">
        <span>Name</span><span class="num">Tracks</span><span class="num">Time</span>
      </div>
      {#each playlists.playlists as p (p.id)}
        <button class="row" style:--cols="1fr 5rem 5rem" onclick={() => openPlaylist(p.id)}>
          <span class="cell id">{p.name}</span>
          <span class="num">{p.tracks.length}</span>
          <span class="num">{fmt(totalSecs(tracksOf(p)))}</span>
        </button>
      {:else}
        <p class="empty">No playlists yet. Make one with New playlist, or right-click a track.</p>
      {/each}
    {:else if view === "artists"}
      <div class="title-row">
        <div>
          <h1>Artists</h1>
          <p class="meta">{plural(artists.length, "artist")}</p>
        </div>
      </div>
      <div class="row head" style:--cols="1fr 5rem 5rem">
        <span>Artist</span><span class="num">Albums</span><span class="num">Tracks</span>
      </div>
      {#each artists as [name, ts] (name)}
        <button class="row" style:--cols="1fr 5rem 5rem" onclick={() => drill("artists", name)}>
          <span class="cell id">{name}</span>
          <span class="num">{albumCount(ts)}</span>
          <span class="num">{ts.length}</span>
        </button>
      {:else}
        <p class="empty">No artists found.</p>
      {/each}
    {:else}
      <div class="title-row">
        <div>
          <h1>Albums</h1>
          <p class="meta">{plural(albums.length, "album")}</p>
        </div>
      </div>
      <div class="row head" style:--cols="3fr 2fr 3.5rem 4rem 4rem">
        <span>Album</span><span>Artist</span><span class="num">Year</span><span class="num">Tracks</span><span class="num">Time</span>
      </div>
      {#each albums as [k, ts] (k)}
        <button class="row" style:--cols="3fr 2fr 3.5rem 4rem 4rem" onclick={() => drill("albums", k)}>
          <span class="cell id">{k.split("\0")[1]}</span>
          <span class="cell">{k.split("\0")[0]}</span>
          <span class="num">{ts.find((t) => t.year)?.year ?? "—"}</span>
          <span class="num">{ts.length}</span>
          <span class="num">{fmt(totalSecs(ts))}</span>
        </button>
      {:else}
        <p class="empty">No albums found. Tracks without an album tag only show under Tracks.</p>
      {/each}
    {/if}
  </main>
</div>

<PlayerBar
  onopen={(t, kind) => {
    const key = kind === "albums" ? albumKey(t) : artistOf(t);
    if (!key) return;
    query = "";
    drill(kind, key);
  }}
/>

<style>
  /* sidebar runs the full height; the toolbar and list share the column beside it */
  .app {
    display: grid;
    grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
    grid-template-rows: var(--top-h) minmax(0, 1fr);
    height: calc(100vh - var(--bar-h));
  }
  nav {
    grid-row: 1 / -1;
    grid-column: 1;
  }
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 0 var(--pad);
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    width: min(100%, 22rem);
    height: 36px;
    padding: 0 8px 0 14px;
    box-sizing: border-box;
    border-radius: 999px;
    background: var(--surface);
    color: var(--muted);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: none;
    color: var(--text);
  }
  .search input::placeholder {
    color: var(--faint);
  }
  .search:focus-within {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  kbd {
    padding: 1px 6px;
    border-radius: 4px;
    background: var(--raised);
    font: inherit;
    font-size: var(--fs-small);
    color: var(--muted);
  }
  .top-actions {
    display: flex;
    align-items: center;
    gap: 16px;
    height: 100%;
  }
  /* the library folder is the app's scope, so it stays pinned to the foot of the sidebar */
  .switcher {
    all: unset;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    padding: 12px 10px 4px;
    color: var(--muted);
    cursor: pointer;
  }
  .switcher:hover {
    color: var(--text);
  }
  .switcher:focus-visible {
    outline: 2px solid var(--accent);
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 10px;
    overflow: auto;
    border-right: 1px solid var(--rule);
  }
  .nav {
    all: unset;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    padding: 0 10px;
    border-radius: var(--radius-row);
    color: light-dark(var(--muted), #bbb);
    cursor: pointer;
  }
  .nav:hover {
    color: var(--text);
  }
  /* library sections as boxed buttons, count on the right */
  .lib {
    margin-bottom: 4px;
    min-height: 36px;
    border: 1px solid var(--rule);
    font-weight: 600;
  }
  .count {
    margin-left: auto;
    color: var(--faint);
  }
  /* active by brightness on dark, a grey pill on light */
  .nav.active {
    color: var(--text);
    background: light-dark(var(--raised), transparent);
    font-weight: 500;
  }
  .lib.active {
    border-color: var(--accent);
    background: var(--accent);
    color: #fff;
    font-weight: 600;
  }
  .lib.active .count {
    color: inherit;
  }
  .nav:focus-visible {
    outline: 2px solid var(--accent);
  }
  .group {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 24px 0 4px;
    padding: 0 0 0 10px;
    font-size: var(--fs-small);
    color: var(--faint);
  }
  .icon.small {
    width: 28px;
    height: 28px;
  }
  .hint {
    margin: 4px 10px;
    font-size: var(--fs-small);
    color: var(--faint);
  }
  .error {
    margin: 8px 10px;
    color: var(--negative);
  }

  main {
    padding: 0 var(--pad) 40px;
    overflow: auto;
  }
  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 12px;
    padding: 10px 14px;
    border-radius: var(--radius-row);
    background: var(--raised);
  }
  /* name, then a short summary on the same line */
  .title-row > div:first-child {
    display: flex;
    flex: 1;
    align-items: baseline;
    gap: 16px;
    min-width: 0;
  }
  h1 {
    font-size: var(--fs-title);
    line-height: 1.3;
  }
  .meta {
    margin: 0;
    flex: none;
    color: var(--muted);
  }
  .back {
    all: unset;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
    font-size: var(--fs-small);
    color: var(--muted);
    cursor: pointer;
  }
  .back:hover {
    color: var(--text);
  }
  .back:focus-visible {
    outline: 2px solid var(--accent);
  }
  .rename {
    width: 100%;
    max-width: 24rem;
    box-sizing: border-box;
    padding: 4px 10px;
    border: 1px solid var(--control);
    border-radius: 6px;
    background: var(--ground);
    font-size: var(--fs-title);
    font-weight: 600;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  /* first run: the table keeps its frame, and the one action sits where the tracks will appear */
  .first-run {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    margin-top: 8px;
    padding: 24px;
    border: 1px dashed var(--control);
    border-radius: var(--radius-row);
  }
  .first-run p {
    margin: 0;
    color: var(--muted);
  }
</style>
