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

