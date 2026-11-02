// pure queue helpers: no runes, so `bun test` can import them

// Fisher-Yates on the items after index `at`; items up to `at` keep their place
export function shuffleAfter<T>(items: T[], at: number, rand = Math.random): T[] {
  const out = [...items];
  for (let i = out.length - 1; i > at + 1; i--) {
    const j = at + 1 + Math.floor(rand() * (i - at));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

// shuffle mode order: the item at `at` first, then all the others (before and after it) in random order,
// so turning shuffle on at the last track still plays the rest of the list
export function shuffleFrom<T>(items: T[], at: number, rand = Math.random): T[] {
  if (at < 0) return shuffleAfter(items, -1, rand);
  return shuffleAfter([items[at], ...items.slice(0, at), ...items.slice(at + 1)], 0, rand);
}

// move one item; `at` (the playing index) follows so the same track stays current
export function moveItem<T>(items: T[], at: number, from: number, to: number): [T[], number] {
  const out = [...items];
  out.splice(to, 0, ...out.splice(from, 1));
  if (from === at) at = to;
  else if (from < at && to >= at) at--;
  else if (from > at && to <= at) at++;
  return [out, at];
}

// where the lists stand after going back from `current` to `path`, the last track in history:
// a queued track returns to the queue and a list track stays ahead, so next plays it again
export function stepBack<T>(
  { context, at, queue, fromQueue }: { context: T[]; at: number; queue: T[]; fromQueue: boolean },
  current: T,
  path: T,
) {
  const q = fromQueue ? [current, ...queue] : queue;
  const a = fromQueue ? at : at - 1;
  return { at: a, queue: q, fromQueue: path !== context[a] };
}

<<<<<<< HEAD
export const fmt = (s: number) =>
  `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

// what plays next; `ended` means the track ran out, so repeat one only applies there, not to the next button
export function nextStep(queued: number, left: number, repeat: "off" | "all" | "one", ended: boolean, hasList: boolean) {
  if (ended && repeat === "one") return "same";
  if (queued) return "queue";
  if (left) return "list";
  return repeat === "all" && hasList ? "wrap" : null;
=======
// where previous goes: back through what played since the list was started, then up the list itself,
// then anything older. `started` is how much history there was when the list began
export function backStep(history: number, started: number, at: number, fromQueue: boolean) {
  if (history > started) return "history";
  if (fromQueue && at >= 0) return "current";
  if (!fromQueue && at > 0) return "list";
  return history ? "history" : null;
}

// previous moves up the list: queued tracks stepped back over (`returned`, at the front of the queue)
// join the list where they played, so going forward again keeps the order they were heard in
export function stepUp<T>({ context, at, queue }: { context: T[]; at: number; queue: T[] }, returned: number) {
  return {
    context: [...context.slice(0, at + 1), ...queue.slice(0, returned), ...context.slice(at + 1)],
    at: at - 1,
    queue: queue.slice(returned),
  };
>>>>>>> feat/design
}
