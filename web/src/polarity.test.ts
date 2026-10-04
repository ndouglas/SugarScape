import { describe, expect, it } from 'vitest';
import { readoutText } from './valley';
import { finishesUnpredictably, modelOf, ticksLeft } from './models';
import { defaultForm } from './experiments/form';
import { finishedNotice } from './engine';
import { COMPARE_PRESETS, comparePresetStates } from './compare-presets';
import type { ModelConfig, ModelStats } from './types';
const config = { model: 'polarity', horizon: 15, periods_per_tick: 7, stop_at_hegemony: true } as unknown as ModelConfig;
describe('emergent polarity hosts', () => {
  it('names governments and uses actual completed periods in the toolbar', () => {
    expect(readoutText({ config, tick: 3, population: 4, latest: { periods: 15 } } as never, null)).toBe('t = 3 · 15 periods · 4 sovereign governments');
  });
  it('budgets the partial final tick while allowing early hegemony', () => {
    expect([modelOf(config), ticksLeft(config, 0), ticksLeft(config, 2), finishesUnpredictably(config)]).toEqual(['polarity', 3, 1, true]);
  });
  it('suggests terminal polarity at the complete economic horizon', () => {
    expect(defaultForm('polarity' as never, config)).toMatchObject({ x: { path: 'predator_share', values: '0,0.05,0.1,0.2,0.4,0.6,0.8,1' }, ticks: 3, metric: { kind: 'final', series: 'sovereign_count' } });
  });
  it('compares offense and defense without changing source damage rules', () => {
    const preset = (id: string, superiority: number) => ({ id, title: id, name: id, source: 'Cederman 1994', description: '', config: { ...config, superiority, victory: superiority, source_profile: 'chapter4', asymmetric_damage: 'source' } as ModelConfig });
    const pair = COMPARE_PRESETS.find((p) => p.id === 'polarity-offense-defense');
    expect(pair).toBeDefined();
    expect(comparePresetStates([preset('polarity-original', 2), preset('polarity-defense', 3)], pair!, 17)?.b.config).toMatchObject({ superiority: 3, victory: 3, source_profile: 'chapter4', asymmetric_damage: 'source' });
  });
  it('reports invalidity instead of claiming a valid horizon endpoint', () => {
    const latest = { periods: 4, finish_reason: 'invalid', invalidity: 'cell 2 has nonpositive stock' } as unknown as ModelStats;
    expect(finishedNotice(config, 1, latest)).toContain('Invalid reconstruction');
    expect(finishedNotice(config, 1, latest)).toContain('cell 2 has nonpositive stock');
  });
});
