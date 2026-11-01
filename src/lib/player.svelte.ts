import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { moveItem, nextStep, shuffleFrom, stepBack } from "$lib/queue";

type PlayState = "playing" | "paused" | "stopped";

// Two lists, like Spotify:
// - context: the list a track was played from (album, artist, playlist, Tracks view), in its natural order
// - queue: tracks the user added by hand; they play first, then the context goes on where it stopped
export const player = $state({
  track: null as string | null,
  state: "stopped" as PlayState,
  pos: 0,
  duration: 0,
  volume: 1,
  context: [] as string[],
  at: -1, // index in context of the last context track played; an index, because a path can be listed twice
  queue: [] as string[],
  fromQueue: false, // the playing track came from the queue, not from context[at]
  history: [] as string[], // tracks played before the current one, oldest first
  shuffle: false,
  repeat: (localStorage.getItem("repeat") ?? "off") as "off" | "all" | "one",
  canUndo: false,
});

const HISTORY = 100; // shortcut: kept in memory only, cleared on restart; persist it if people ask
let now: string | null = null; // set at once, before the track_changed event arrives
function start(path: string) {
  if (now) player.history = [...player.history, now].slice(-HISTORY);
  now = path;
  return invoke("play", { path });
}

// context before shuffling, restored when shuffle turns off; null once the user reorders it by hand
let ordered: string[] | null = null;
type Snapshot = { context: string[]; queue: string[]; ordered: string[] | null; shuffle: boolean };
let undo: Snapshot | null = null;

// every change makes new arrays, so a snapshot never changes under us
function snapshot() {
  undo = { context: player.context, queue: player.queue, ordered, shuffle: player.shuffle };
  player.canUndo = true;
}
function dropUndo() {
  undo = null;
  player.canUndo = false;
}

listen<string>("track_changed", (e) => {
  player.track = e.payload;
  player.pos = 0;
});
listen<PlayState>("state_changed", (e) => (player.state = e.payload));
listen<[number, number]>("position", (e) => ([player.pos, player.duration] = e.payload));
listen("track_ended", () => {
  player.state = "stopped";
  player.pos = 0;
  next(true);
});

export const playAt = (i: number) => {
  player.at = i;
  player.fromQueue = false;
  return start(player.context[i]);
};
// play one track (from history) without changing the context or the queue
export function replay(path: string) {
  player.fromQueue = true; // ⏭ goes on with queue/list
  return start(path);
}
// play queue item k; the items before it are skipped, like clicking ahead in Spotify's queue
export function playQueued(k: number) {
  const path = player.queue[k];
  player.queue = player.queue.slice(k + 1);
  player.fromQueue = true;
  dropUndo(); // the queue moved on; an old snapshot would bring played tracks back
  return start(path);
}

// start a new context; `i` is the clicked row, so a song listed twice plays the right copy.
// The hand-made queue stays and still plays before the rest of the new context.
export function play(path: string, context: string[], i = context.indexOf(path)) {
  dropUndo();
  if (player.shuffle) {
    // the clicked track plays first, the rest in random order
    ordered = context;
    player.context = shuffleFrom(context, i);
    return playAt(0);
  }
  ordered = null;
  player.context = context;
  return playAt(i);
}

export const contextLeft = () => Math.max(player.context.length - player.at - 1, 0);
export const upcoming = () => player.queue.length + contextLeft();
export const canPrev = () => player.history.length > 0;

export function next(ended = false) {
  const step = nextStep(player.queue.length, contextLeft(), player.repeat, ended, player.context.length > 0);
  if (step === "same" && now) invoke("play", { path: now }); // no history: it is the same play going round
  else if (step === "queue") playQueued(0);
  else if (step === "list") playAt(player.at + 1);
  else if (step === "wrap") playAt(0);
}
export const canNext = () => upcoming() > 0 || (player.repeat === "all" && player.context.length > 0);
export function cycleRepeat() {
  player.repeat = player.repeat === "off" ? "all" : player.repeat === "all" ? "one" : "off";
  localStorage.setItem("repeat", player.repeat);
}
// shortcut: a track replayed from history also goes back into the queue; mark replays if that annoys anyone
export function prev() {
  const path = player.history.at(-1);
  if (!path) return;
  player.history = player.history.slice(0, -1);
  if (now) Object.assign(player, stepBack(player, now, path));
  dropUndo(); // the queue moved; an old snapshot would bring it back wrong
  now = path;
  return invoke("play", { path });
}

// shuffle mixes every other context track, also the ones already played; the hand-made queue keeps its order
export function toggleShuffle() {
  snapshot();
  player.shuffle = !player.shuffle;
  const current = player.context[player.at];
  if (player.shuffle) {
    ordered = player.context;
    player.context = shuffleFrom(player.context, player.at);
    if (player.at >= 0) player.at = 0;
  } else if (ordered) {
    player.context = ordered;
    player.at = ordered.indexOf(current);
    ordered = null;
  } // no `ordered`: the context was reordered while shuffled, so keep it as it is
}

// nothing playing: start the tracks instead of queueing them
export function playNext(paths: string[]) {
  if (!player.track) return void play(paths[0], paths, 0);
  snapshot();
  player.queue = [...paths, ...player.queue];
}
export function addToQueue(paths: string[]) {
  if (!player.track) return void play(paths[0], paths, 0);
  snapshot();
  player.queue = [...player.queue, ...paths];
}
export function moveInQueue(from: number, to: number) {
  snapshot();
  [player.queue] = moveItem(player.queue, -1, from, to);
}
// indexes are absolute positions in context; `at` follows the current track
export function moveInContext(from: number, to: number) {
  snapshot();
  ordered = null;
  [player.context, player.at] = moveItem(player.context, player.at, from, to);
}
export function clearQueue() {
  snapshot();
  player.queue = [];
}

export function undoQueue() {
  if (!undo) return;
  const current = player.context[player.at];
  ({ context: player.context, queue: player.queue, ordered, shuffle: player.shuffle } = undo);
  player.at = player.context.indexOf(current);
  dropUndo();
}

export const toggle = () => invoke(player.state === "playing" ? "pause" : "resume");
export const seek = (secs: number) => {
  player.pos = Math.max(0, secs); // optimistic; Rust confirms on next tick
  return invoke("seek", { secs });
};
export const setVolume = (vol: number) => {
  player.volume = vol;
  localStorage.setItem("volume", String(vol));
  invoke("set_volume", { vol });
};

export { fmt } from "$lib/queue";

// mute keeps the old level, so a second click restores it
let unmuted = 1;
export function toggleMute() {
  if (player.volume > 0) {
    unmuted = player.volume;
    setVolume(0);
  } else setVolume(unmuted);
}
const savedVolume = localStorage.getItem("volume");
if (savedVolume !== null) setVolume(+savedVolume);
