// The spatial games' pure page helpers (milestone 12): a cube's slices; Inspect's rows (Task 5).
import type { Layer, PlayerView, SpatialConfig } from './types';

/** A cube's z-slices for the Slice menu; none for a square or random lattice. */
export function sliceOptions(c: SpatialConfig): [Layer, string][] {
  if (c.lattice !== 'cube') return [];
  return Array.from({ length: c.width }, (_, z): [Layer, string] => [`slice:${z}`, `z = ${z}`]);
}

/** The slice the core draws when none is chosen: the middle, `width / 2` rounded down. */
export function middleSlice(c: SpatialConfig): Layer {
  return `slice:${Math.floor(c.width / 2)}`;
}

const num = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));
const verb = (s: 'C' | 'D', past = false) => (s === 'C' ? (past ? 'cooperated' : 'cooperates') : past ? 'defected' : 'defects');

/**
 * A player's Inspect rows: its strategy (and last generation's), score, its candidates by kind, and its next strategy.
 * The next strategy comes from this generation's scores: under asynchronous updating it holds only if the player is
 * chosen now, before any neighbor changes, so the row says so.
 */
export function playerRows(a: PlayerView, asynchronous = false): [string, string][] {
  const kind = (s: 'C' | 'D') => a.candidates.filter((c) => c.strategy === s);
  const best = (s: 'C' | 'D') => Math.max(...kind(s).map((c) => c.score));
  const part = (s: 'C' | 'D') => (kind(s).length === 0 ? `no ${s}` : `${kind(s).length} ${s} (best ${num(best(s))})`);
  const next =
    a.next !== null
      ? verb(a.next)
      : a.p_c !== null
        ? `cooperates with probability ${Math.round(a.p_c * 100)}%`
        : 'keeps its strategy (every score is 0)';
  return [
    ['Player', `#${a.id} · ${verb(a.strategy)} (${verb(a.previous, true)} last generation)`],
    ['Score', num(a.score)],
    ['Neighborhood', `${part('C')}, ${part('D')}, itself included`],
    ['Next generation', asynchronous ? `${next} (if chosen now)` : next],
  ];
}
