import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { initSync, sweep_csv, sweep_result } from '../wasm-pkg/sugarscape.js';

initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const spec = JSON.stringify({ name: 'Saved exploratory availability', base: { config: { population: 0 } }, x: { path: 'population', values: [0] }, seeds: { from: 1, count: 1 }, ticks: 1, metric: { kind: 'final', series: 'population' } });
const receipt = { status: 'complete', completed_periods: 14, attempted_period: 14, last_tick_periods: 2, invalid_reason: null, invalid_phase: null, clustering_reason: 'undefined_extinction' };
const run = { point: 0, series: 0, x: 0, seed: 1, value: null };
describe('Exploratory availability export compatibility', () => {
  it('retains optional source-clock and undefined-reason metadata in saved JSON', () => {
    const exported = JSON.parse(sweep_result(spec, JSON.stringify([{ ...run, democratic_peace: receipt }])));
    expect(exported.runs[0].democratic_peace).toEqual(receipt);
  });
  it('exports complete-history availability and reasons in runs CSV', () => {
    const csv = sweep_csv(spec, JSON.stringify([{ ...run, democratic_peace: receipt }]), 'runs');
    expect(csv.split('\n')[0]).toContain('status,completed_periods,attempted_period,last_tick_periods,invalid_reason,invalid_phase,clustering_reason');
    expect(csv).toContain('complete,14,14,2,,,undefined_extinction');
  });
  it('preserves older results and other models exact CSV shape', () => {
    expect(sweep_csv(spec, JSON.stringify([{ ...run, value: 0 }]), 'runs')).toBe('series,x,seed,value\n0,0,1,0\n');
    expect(JSON.parse(sweep_result(spec, JSON.stringify([run]))).runs[0]).not.toHaveProperty('democratic_peace');
  });
});
