import type { InitialState } from './engine';
import type { Preset } from './types';

/** A presets-menu entry that opens Compare with two presets side by side at one seed. */
export interface ComparePreset {
  id: string;
  label: string;
  /** World A's preset id. */
  a: string;
  /** World B's preset id. */
  b: string;
}

export const COMPARE_PRESETS: ComparePreset[] = [
  { id: 'vi-2-vs-vi-3', label: 'Indecomposability — VI-2 vs VI-3 (Compare)', a: 'vi-2-no-trade', b: 'vi-3-trade' },
];

/**
 * A's seed and B's setup: the entry's two presets at `seed`, or null if either preset is missing.
 * A is loaded by id (`Engine.loadPreset`), which needs no copied config, so only B's (for
 * `buildCompare`) is returned.
 */
export function comparePresetStates(
  presets: Preset[],
  entry: ComparePreset,
  seed: number,
): { aSeed: number; b: InitialState } | null {
  const a = presets.find((p) => p.id === entry.a);
  const b = presets.find((p) => p.id === entry.b);
  if (!a || !b) return null;
  return { aSeed: seed, b: { config: structuredClone(b.config), seed } };
}
