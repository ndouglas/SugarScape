// The spatial games' pure page helpers (milestone 12): a cube's slices; Inspect's rows (Task 5).
import type { Layer, SpatialConfig } from './types';

/** A cube's z-slices for the Slice menu; none for a square or random lattice. */
export function sliceOptions(c: SpatialConfig): [Layer, string][] {
  if (c.lattice !== 'cube') return [];
  return Array.from({ length: c.width }, (_, z): [Layer, string] => [`slice:${z}`, `z = ${z}`]);
}

/** The slice the core draws when none is chosen: the middle, `width / 2` rounded down. */
export function middleSlice(c: SpatialConfig): Layer {
  return `slice:${Math.floor(c.width / 2)}`;
}
