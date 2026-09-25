import type { InitialState } from './engine';
import { presetModel } from './models';
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
  {
    id: 'lhv-published-vs-documented',
    label: 'Replication vs documented — Anasazi (Compare)',
    a: 'lhv-published',
    b: 'lhv-documented',
  },
  {
    id: 'cv-salami-vs-jump',
    label: 'Salami tactics vs one jump — Civil Violence (Compare)',
    a: 'cv-run-3-salami',
    b: 'cv-run-4-one-jump',
  },
  {
    id: 'cv-cleansing-vs-peacekeepers',
    label: 'Ethnic cleansing vs peacekeepers — Civil Violence (Compare)',
    a: 'cv-run-7-cleansing',
    b: 'cv-safe-havens',
  },
  {
    id: 'nm-sync-vs-async',
    label: 'Synchronous vs asynchronous — Spatial Games (Compare)',
    a: 'nm-3-kaleidoscope',
    b: 'hg-async-kaleidoscope',
  },
  {
    id: 'nbm-discrete-vs-continuous',
    label: 'Discrete vs continuous time — Spatial Games (Compare)',
    a: 'nbm-discrete',
    b: 'nbm-continuous',
  },
  {
    id: 'rca-published-vs-literal-p2',
    label: 'Published vs literal ties at P = 2 — Tag Cooperation (Compare)',
    a: 'rca-published-p2',
    b: 'rca-literal-p2',
  },
  {
    id: 'ac-random-vs-sweep-20',
    label: 'Literal vs Sugarscape activation, 20 × 20 — Axelrod Culture (Compare)',
    a: 'ac-random-activation-20',
    b: 'ac-sweep-activation',
  },
];

/**
 * A's seed and B's setup: the entry's two presets at `seed`, or null if either preset is missing or
 * they are of different models (Compare pairs one model).
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
  if (!a || !b || presetModel(a) !== presetModel(b)) return null;
  return { aSeed: seed, b: { config: structuredClone(b.config), seed } };
}
