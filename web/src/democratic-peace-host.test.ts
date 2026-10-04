import { describe, expect, it } from 'vitest';
import { finishedNotice } from './engine';
import { COLOR_MODES, MODELS, modelOf, MODEL_LABELS } from './models';
import { defaultForm } from './experiments/form';
import { readoutText } from './valley';
import type { ModelConfig, ModelKind, ModelStats } from './types';

const config = { model: 'democratic_peace', width: 4, height: 4, horizon_periods: 14, periods_per_tick: 3 } as unknown as ModelConfig;
const kind = 'democratic_peace' as ModelKind;
const snapshot = (periods: number, extra: Record<string, unknown> = {}) => ({ periods, completed_periods: periods, attempted_period: periods, last_tick_periods: 3, finish_reason: null, invalidity: null, ...extra }) as unknown as ModelStats;

describe('Democratic peace shared host integration', () => {
  it('discovers the separate model with six explained canvas modes', () => {
    expect(modelOf(config)).toBe('democratic_peace');
    expect(MODELS).toContain(kind);
    expect(MODEL_LABELS[kind]).toContain('Democratic');
    expect(COLOR_MODES[kind].map(([mode]) => mode)).toEqual(['territory', 'governing_regime', 'latent_regime', 'resources', 'alliances', 'pariahs']);
  });
  it('reports completed source clocks after the partial final display tick', () => {
    expect(finishedNotice(config, 5, snapshot(14, { last_tick_periods: 2, finish_reason: 'complete' }))).toContain('14');
    expect(finishedNotice(config, 5, snapshot(14, { finish_reason: 'complete' }))).toContain('period');
  });
  it('reports atomic invalid attempts with the committed period and phase', () => {
    const notice = finishedNotice(config, 1, snapshot(2, { attempted_period: 3, invalidity: 'zero denominator', invalid_phase: 'allocation' }));
    expect(notice).toContain('attempted period 3');
    expect(notice).toContain('2 completed');
    expect(notice).toContain('allocation');
  });
  it('presents each Compare world source clock independently', () => {
    expect(readoutText({ config, tick: 1, population: 10, latest: snapshot(3) }, { config: { ...config, periods_per_tick: 2 } as ModelConfig, tick: 1, population: 12, latest: snapshot(2) })).toContain('A 3 periods');
    expect(readoutText({ config, tick: 1, population: 10, latest: snapshot(3) }, { config, tick: 1, population: 12, latest: snapshot(2) })).toContain('B 2 periods');
  });
  it('suggests an exploratory captured-density sweep covering the grouped horizon', () => {
    expect(defaultForm(kind, config)).toMatchObject({ x: { path: 'initial_democratic_share' }, ticks: 5, metric: { kind: 'final', series: 'democratic_share' } });
    expect(defaultForm(kind, config).description).toContain('Exploratory');
  });
});
