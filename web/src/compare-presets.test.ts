import { describe, expect, it } from 'vitest';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { Config, Preset } from './types';

const preset = (id: string, trade: boolean): Preset => ({
  id,
  name: id,
  source: '',
  description: '',
  config: { trade: { enabled: trade } } as unknown as Config,
});

describe('compare presets', () => {
  const entry = COMPARE_PRESETS[0];

  it('opens VI-2 as A and VI-3 as B', () => {
    expect(entry).toMatchObject({ a: 'vi-2-no-trade', b: 'vi-3-trade', label: 'Indecomposability — VI-2 vs VI-3 (Compare)' });
  });

  it('gives A its seed and builds B from a copy of its preset at the same seed', () => {
    const presets = [preset('vi-2-no-trade', false), preset('vi-3-trade', true)];
    const states = comparePresetStates(presets, entry, 42)!;
    expect(states.aSeed).toBe(42);
    expect(states.b.seed).toBe(42);
    expect(states.b.config.trade.enabled).toBe(true);
    states.b.config.trade.enabled = false;
    expect(presets[1].config.trade.enabled).toBe(true);
  });

  it('is null when a preset is missing', () => {
    expect(comparePresetStates([preset('vi-2-no-trade', false)], entry, 1)).toBeNull();
  });
});
