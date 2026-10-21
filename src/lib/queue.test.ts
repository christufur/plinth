// @ts-expect-error bun's built-in test module; run with `bun test src/lib` (no @types/bun installed)
import { expect, test } from "bun:test";
import { shuffleAfter } from "./queue";

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
