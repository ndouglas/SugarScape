import { describe, expect, it } from 'vitest';
import { polarityRows, polarityChartCaption, polarityLegend } from './polarity';
import type { PolarityConfig, PolarityInspection } from './types';
describe('polarity inspection', () => {
  it('keeps province stock distinct from the corporate resources it cannot control', () => {
    const view = { model: 'polarity', cell: { id: 3, capital: 1, predator: true, stock: 7 }, capital: 1, province_stock: 7, corporate_stock: 21, members: [1, 3], neighbors: [2], trust: [], threat: 2, coalition: null, foreign_fronts: [], domestic_fronts: [], last_event: null, agent: null } as PolarityInspection;
    expect(polarityRows(view)).toContainEqual(['Government', 'Cell 3 · capital 1 · latent predator']);
    expect(polarityRows(view)).toContainEqual(['Resources', 'Province 7.000 · corporate 21.000']);
    expect(polarityRows(view)).toContainEqual(['Defensive coalition', 'None']);
  });
  it('keeps invalid nonfinite resources inspectable as unavailable', () => {
    const view = { model: 'polarity', cell: { id: 0, capital: 0, predator: false, stock: null }, capital: 0, province_stock: 0, corporate_stock: null, members: [0], neighbors: [], trust: [], threat: null, coalition: null, foreign_fronts: [], domestic_fronts: [], last_event: null, invalidity: 'nonfinite stock', agent: null } as unknown as PolarityInspection;
    expect(polarityRows(view)).toContainEqual(['Resources', 'Province 0.000 · corporate unavailable']);
    expect(polarityRows(view)).toContainEqual(['Invalid reconstruction', 'nonfinite stock']);
  });
  it('discloses dropped log events independently of structural event counters', () => {
    const view = { model: 'polarity', cell: { id: 0, capital: 0, predator: false, stock: 50 }, capital: 0, province_stock: 0, corporate_stock: 50, members: [0], neighbors: [], trust: [], threat: null, coalition: null, foreign_fronts: [], domestic_fronts: [], last_event: null, agent: null, event_log_limit: 2, events_dropped: 7, events: [] } as PolarityInspection;
    expect(polarityRows(view)).toContainEqual(['Event log', '0 retained · cap 2 · 7 older events dropped']);
  });
  it('explains front actions and trust without serialized implementation fields', () => {
    const view = { model: 'polarity', cell: { id: 0, capital: 0, predator: false, stock: 50 }, capital: 0, province_stock: 0, corporate_stock: 50, members: [0], neighbors: [1], trust: [{ neighbor: 1, trust: -0.5 }], threat: 1, coalition: { threat: 1, members: [0, 2] }, foreign_fronts: [{ key: { a: 0, b: 1, domestic: false }, actions: [true, false], commitments: [20, 10], previous: [false, true], path: [0, 1] }], domestic_fronts: [], last_event: null, agent: null } as PolarityInspection;
    expect(polarityRows(view)).toContainEqual(['Foreign fronts', '0–1: D/C · previous C/D · resources 20.000/10.000 · path 0 → 1']);
    expect(polarityRows(view)).toContainEqual(['Trust', '1: -0.500']);
    expect(polarityRows(view)).toContainEqual(['Defensive coalition', 'Members 0, 2 · threat 1']);
  });
  it('labels ownership separately from coalition colors and shows actual periods', () => {
    const legend = polarityLegend({ source_profile: 'chapter4', horizon: 15, periods_per_tick: 7 } as PolarityConfig, 'coalitions', 14);
    expect(legend).toContain('14 of 15 periods');
    expect(legend).toContain('Defensive coalitions do not transfer sovereignty');
  });
  it('labels comparison chart batching separately for each world', () => {
    expect(polarityChartCaption('Polarity', [{ periods_per_tick: 7 }, { periods_per_tick: 1 }] as PolarityConfig[])).toBe('Polarity · A: 7 periods/tick · B: 1 periods/tick');
  });
});
