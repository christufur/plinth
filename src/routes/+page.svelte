<script lang="ts">
  import { player, play, toggle, seek, setVolume, next, prev, fmt } from "$lib/player.svelte";
  import { library, pick, artistOf, albumKey, group } from "$lib/library.svelte";
  import Download from "$lib/Download.svelte";
  import PlayerBar from "$lib/PlayerBar.svelte";

  let showURL = $state(false);

  type View = "artists" | "albums" | "tracks";
  let view = $state<View>("tracks");
  let sel = $state<{ kind: "artists" | "albums"; v: string } | null>(null);
  let playlists = $state<string[]>([]); // stub, persist when playlists are built (Phase 4 SQLite per design)

  let artists = $derived(group(library.tracks, artistOf));
  let albums = $derived(group(library.tracks, albumKey));
  let visible = $derived(
    sel
      ? library.tracks.filter((t) => (sel!.kind === "artists" ? artistOf(t) : albumKey(t)) === sel!.v)
      : library.tracks,
  );
  let nav = $derived([
    ["artists", artists.length],
    ["albums", albums.length],
    ["tracks", library.tracks.length],
  ] as const);
  const label = (k: string) => k.replace("\0", " — ");

  function go(v: View) {
    view = v;
    sel = null;
  }
  function drill(kind: "artists" | "albums", v: string) {
    sel = { kind, v };
    view = "tracks";
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
    <div class="head"><span>playlists</span></div>
    {#each playlists as pl (pl)}
      <button>{pl}</button>
    {:else}
      <em class="n">none yet</em>
    {/each}
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
      <div class="row head"><span>#</span><span>Title</span><span>Artist</span><span>Album</span><span>Year</span><span>Time</span></div>
      {#each visible as t, i (t.path)}
        <button class="row" class:active={t.path === player.track} onclick={() => play(t.path, visible.map((x) => x.path))}>
          <span class="n">{player.state === "playing" && t.path === player.track ? "▶" : (t.track_no ?? i + 1)}</span>
          <span class="cell">{t.title}</span>
          <span class="cell">{t.artist ?? artistOf(t)}</span>
          <span class="cell">{t.album ?? "—"}</span>
          <span class="n">{t.year ?? ""}</span>
          <span class="n">{t.secs ? fmt(t.secs) : ""}</span>
        </button>
      {:else}
        <p><em>{library.folder ? "No audio files found." : "Open a folder to start."}</em></p>
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
    width: 100%;
    box-sizing: border-box;
    padding: 2px 6px;
    border-radius: var(--r);
    text-transform: capitalize;
  }
  aside button.active {
    background: var(--accent);
    color: #fff;
  }
  .n {
    font-family: ui-monospace, monospace;
    font-size: 11px;
    opacity: 0.6;
  }
  .head {
    padding: 2px 6px;
    text-transform: capitalize;
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
  .row {
    all: unset;
    box-sizing: border-box;
    display: grid;
    grid-template-columns: var(--cols, 2.5rem 3fr 2fr 2fr 3rem 3rem);
    gap: 0.5rem;
    width: 100%;
    padding: 3px 6px;
    border-radius: var(--r);
  }
  button.row {
    cursor: pointer;
  }
  button.row:hover {
    background: var(--line);
  }
  button.row.active {
    color: var(--accent);
    font-weight: 600;
  }
  .head {
    opacity: 0.6;
    border-bottom: 1px solid var(--line);
    border-radius: 0;
  }
  .cell {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
