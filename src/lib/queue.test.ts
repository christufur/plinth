// @ts-expect-error bun's built-in test module; run with `bun test src/lib` (no @types/bun installed)
import { describe, expect, test } from "bun:test";
<<<<<<< HEAD
import { moveItem, nextStep, shuffleAfter, shuffleFrom, stepBack } from "./queue";
=======
import { backStep, moveItem, shuffleAfter, shuffleFrom, stepBack, stepUp } from "./queue";
>>>>>>> feat/design

test("moveItem keeps the playing track current", () => {
  const q = ["a", "b", "c", "d", "e"]; // "c" playing, at = 2
  const at = (r: [string[], number]) => r[0][r[1]];
  expect(moveItem(q, 2, 4, 3)).toEqual([["a", "b", "c", "e", "d"], 2]); // both after current
  expect(at(moveItem(q, 2, 0, 4))).toBe("c"); // from before to after
  expect(at(moveItem(q, 2, 4, 0))).toBe("c"); // from after to before
  expect(moveItem(q, 2, 2, 4)).toEqual([["a", "b", "d", "e", "c"], 4]); // the current one itself
  expect(at(moveItem(q, 2, 3, 2))).toBe("c"); // onto the current slot
  expect(q).toEqual(["a", "b", "c", "d", "e"]); // input not changed
});

test("shuffleAfter keeps the head in place and the same items", () => {
  const q = ["a", "b", "c", "d", "e", "f"];
  for (let n = 0; n < 50; n++) {
    const s = shuffleAfter(q, 1);
    expect(s.slice(0, 2)).toEqual(["a", "b"]);
    expect([...s].sort()).toEqual(q);
  }
  expect(q).toEqual(["a", "b", "c", "d", "e", "f"]); // input not changed
});

test("shuffleAfter moves items", () => {
  // rand() = 0 always swaps with the first shufflable slot, so the order must change
  expect(shuffleAfter(["a", "b", "c", "d"], -1, () => 0)).not.toEqual(["a", "b", "c", "d"]);
});

test("shuffleAfter handles empty and short lists", () => {
  expect(shuffleAfter([], -1)).toEqual([]);
  expect(shuffleAfter(["a"], 0)).toEqual(["a"]);
});

test("shuffleFrom puts the current item first and keeps all the others", () => {
  const q = ["a", "b", "c", "d"];
  const s = shuffleFrom(q, 3); // last track playing: the rest must still come after it
  expect(s[0]).toBe("d");
  expect([...s].sort()).toEqual(q);
  expect([...shuffleFrom(q, -1)].sort()).toEqual(q); // nothing playing: shuffle all
  expect(q).toEqual(["a", "b", "c", "d"]); // input not changed
});

describe("stepBack", () => {
  const context = ["a", "b", "c"];
  test("list track back to the one before it", () => {
    expect(stepBack({ context, at: 1, queue: [], fromQueue: false }, "b", "a")).toEqual({ at: 0, queue: [], fromQueue: false });
  });
  test("queued track back to the previous queued track returns to the queue", () => {
    expect(stepBack({ context, at: 1, queue: ["z"], fromQueue: true }, "y", "x")).toEqual({ at: 1, queue: ["y", "z"], fromQueue: true });
  });
  test("queued track back to the list track before it", () => {
    expect(stepBack({ context, at: 1, queue: [], fromQueue: true }, "x", "b")).toEqual({ at: 1, queue: ["x"], fromQueue: false });
  });
  test("list track back to a queued track leaves the list track next", () => {
    expect(stepBack({ context, at: 1, queue: [], fromQueue: false }, "b", "x")).toEqual({ at: 0, queue: [], fromQueue: true });
  });
  test("first list track back to a track from another list", () => {
    expect(stepBack({ context, at: 0, queue: [], fromQueue: false }, "a", "old")).toEqual({ at: -1, queue: [], fromQueue: true });
  });
});

<<<<<<< HEAD
describe("nextStep", () => {
  test("repeat one replays only when the track ran out", () => {
    expect(nextStep(1, 3, "one", true, true)).toBe("same");
    expect(nextStep(0, 3, "one", false, true)).toBe("list");
  });
  test("the queue plays before the list", () => {
    expect(nextStep(2, 3, "off", true, true)).toBe("queue");
  });
  test("the end of the list stops, or wraps with repeat all", () => {
    expect(nextStep(0, 0, "off", true, true)).toBe(null);
    expect(nextStep(0, 0, "all", true, true)).toBe("wrap");
    expect(nextStep(0, 0, "all", true, false)).toBe(null);
    expect(nextStep(0, 0, "one", false, true)).toBe(null);
  });
});
=======
describe("backStep", () => {
  test("a song clicked in a list goes up the list", () => {
    expect(backStep(0, 0, 1, false)).toBe("list"); // x y z, played y
  });
  test("a click in a new list goes up that list, not to the old one", () => {
    expect(backStep(3, 3, 1, false)).toBe("list");
  });
  test("what played since the list began comes first", () => {
    expect(backStep(4, 3, 2, false)).toBe("history");
  });
  test("a queued song with nothing behind it goes to the list song it follows", () => {
    expect(backStep(0, 0, 1, true)).toBe("current");
  });
  test("the top of a list goes on to older history, then stops", () => {
    expect(backStep(3, 3, 0, false)).toBe("history");
    expect(backStep(0, 0, 0, false)).toBe(null);
  });
});

test("stepUp puts queued tracks back where they played", () => {
  const list = { context: ["x", "y", "z"], at: 1, queue: ["a", "b", "new"] };
  expect(stepUp(list, 2)).toEqual({ context: ["x", "y", "a", "b", "z"], at: 0, queue: ["new"] });
  expect(stepUp(list, 0)).toEqual({ context: ["x", "y", "z"], at: 0, queue: ["a", "b", "new"] });
});
>>>>>>> feat/design
