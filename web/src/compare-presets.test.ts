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

  it('builds both worlds from copies of the two presets at one seed', () => {
    const presets = [preset('vi-2-no-trade', false), preset('vi-3-trade', true)];
    const states = comparePresetStates(presets, entry, 42)!;
    expect(states.a.seed).toBe(42);
    expect(states.b.seed).toBe(42);
    expect(states.a.config.trade.enabled).toBe(false);
    expect(states.b.config.trade.enabled).toBe(true);
    states.a.config.trade.enabled = true;
    expect(presets[0].config.trade.enabled).toBe(false);
  });

  it('is null when a preset is missing', () => {
    expect(comparePresetStates([preset('vi-2-no-trade', false)], entry, 1)).toBeNull();
  });
});
