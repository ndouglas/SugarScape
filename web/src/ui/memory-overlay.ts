/** How a remembered site or spot draws (Minds 3's overlay). */
export type MemoryShape = 'square' | 'circle';

export interface MemoryDrawSpec {
  /** 0.4 · (1 − age / span) at age 0, fading to 0 at age = span. */
  alpha: number;
  /** A square for a plain remembered site; a circle for one where the agent knows a truffle spot. */
  shape: MemoryShape;
  /** A known spot believed ripe (spot 2) draws filled; unripe (spot 1) or no spot (0) draws unfilled. */
  filled: boolean;
}

export interface MemoryMark extends MemoryDrawSpec {
  x: number;
  y: number;
}

/**
 * One remembered site `[x, y, age, spot]` (as `inspect_memory` flattens them), plus the memory
 * config's `span`, mapped to how it draws: a square, fading with age, or — for a site where the
 * agent knows a truffle spot (`spot` 1 or 2) — a circle, filled when it believes the spot ripe
 * (`spot` 2).
 */
export function memoryDrawSpec([, , age, spot]: [number, number, number, number], span: number): MemoryDrawSpec {
  const alpha = span > 0 ? Math.max(0, 0.4 * (1 - age / span)) : 0;
  return spot === 0 ? { alpha, shape: 'square', filled: false } : { alpha, shape: 'circle', filled: spot === 2 };
}

/** Every remembered site's mark, from `inspect_memory`'s flat `[x, y, age, spot, …]`, in memory order. */
export function memoryMarks(flat: ArrayLike<number>, span: number): MemoryMark[] {
  const out: MemoryMark[] = [];
  for (let i = 0; i + 3 < flat.length; i += 4) {
    const x = flat[i];
    const y = flat[i + 1];
    out.push({ x, y, ...memoryDrawSpec([x, y, flat[i + 2], flat[i + 3]], span) });
  }
  return out;
}
