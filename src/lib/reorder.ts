import type { Action } from "svelte/action";

type Params = { index: number; count: number; onmove: (from: number, to: number) => void };

// ≡ handle on a [data-item] row. Pointer events, not HTML5 drag-and-drop: Tauri's
// file-drop handling can swallow HTML5 drag events in the webview. Alt+↑/↓ from the keyboard.
export const reorder: Action<HTMLElement, Params> = (handle, params) => {
  let p = params;
  let item: HTMLElement | null = null;
  let startY = 0;
  const clamp = (i: number) => Math.max(0, Math.min(p.count - 1, i));

  function move(to: number) {
    if (to === p.index) return;
    const list = handle.closest("[data-item]")?.parentElement;
    p.onmove(p.index, to);
    // rows are keyed by index, so focus would stay on the old slot; follow the moved item
    requestAnimationFrame(() =>
      list?.querySelectorAll<HTMLElement>("[data-item]")[to]?.querySelector<HTMLElement>(".handle")?.focus(),
    );
  }
  function down(e: PointerEvent) {
    item = handle.closest<HTMLElement>("[data-item]");
    startY = e.clientY;
    handle.setPointerCapture(e.pointerId);
  }
  function drag(e: PointerEvent) {
    if (item) item.style.transform = `translateY(${e.clientY - startY}px)`;
  }
  function up(e: PointerEvent) {
    if (!item) return;
    const rows = Math.round((e.clientY - startY) / item.offsetHeight);
    cancel();
    move(clamp(p.index + rows));
  }
  function cancel() {
    if (item) item.style.transform = "";
    item = null;
  }
  function key(e: KeyboardEvent) {
    if (!e.altKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
    e.preventDefault();
    e.stopPropagation(); // the arrows are seek/volume keys at window level
    move(clamp(p.index + (e.key === "ArrowUp" ? -1 : 1)));
  }

  const on = [
    ["pointerdown", down],
    ["pointermove", drag],
    ["pointerup", up],
    ["pointercancel", cancel],
    ["keydown", key],
  ] as const;
  for (const [t, f] of on) handle.addEventListener(t, f as EventListener);
  return {
    update: (n) => (p = n),
    destroy: () => on.forEach(([t, f]) => handle.removeEventListener(t, f as EventListener)),
  };
};
