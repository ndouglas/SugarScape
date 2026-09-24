import { describe, expect, it } from 'vitest';
import type { Config } from '../types';
import { axisLabel, baseLabel, metricLabel, xLabel } from './labels';
import type { Sweep } from './types';

const sweep: Sweep = {
  name: 's',
  base: { preset: 'ii-2-unit' },
  x: { path: 'vision.max', values: [1, 2] },
  seeds: { from: 1, count: 1 },
  ticks: 500,
  metric: { kind: 'final', series: 'population' },
};

describe('labels', () => {
  it('describes the x axis, which is the tick for a time series', () => {
    expect(xLabel(sweep)).toBe('vision.max: 2 values');
    const ts: Sweep = { ...sweep, x: { label: 'All runs', values: [{ at: 0, set: {} }] }, metric: { kind: 'timeseries', series: 'population', every: 5 } };
    expect(xLabel(ts)).toBe('tick');
  });

  it('labels axes', () => {
    expect(axisLabel({ path: 'vision.max', values: [] })).toBe('vision.max');
    expect(axisLabel({ label: 'Vision', path: 'vision.max', values: [] })).toBe('Vision');
    expect(axisLabel({ label: 'Mean vision', values: [] })).toBe('Mean vision');
  });

  it('labels metrics', () => {
    expect(metricLabel(sweep)).toBe('population at t = 500');
    expect(metricLabel({ ...sweep, metric: { kind: 'window_mean', series: 'population', from: 400 } })).toBe(
      'population, mean over t = 400–500',
    );
    expect(metricLabel({ ...sweep, metric: { kind: 'window_mean', series: 'gini', from: 10, to: 20 } })).toBe(
      'gini, mean over t = 10–20',
    );
    expect(metricLabel({ ...sweep, metric: { kind: 'timeseries', series: 'sd_log_price', every: 50 } })).toBe(
      'sd_log_price, 50-tick means',
    );
  });

  it('labels bases', () => {
    expect(baseLabel(sweep)).toBe('preset ii-2-unit');
    expect(baseLabel({ ...sweep, base: { config: {} as Config } })).toBe('a custom config');
  });
});
