import { describe, expect, it } from 'vitest';
import { chartData, describePoint } from './chart-data';
import type { ScalarRow, Summary, Sweep } from './types';

const sweep: Sweep = {
  name: 's',
  base: { preset: 'p' },
  x: { label: 'Mean vision', values: [] },
  seeds: { from: 1, count: 2 },
  ticks: 500,
  metric: { kind: 'window_mean', series: 'population', from: 400 },
};

const row = (series: number, name: string, x: number, at: number, mean: number | null, sd: number | null): ScalarRow => ({
  series,
  series_name: name,
  x,
  at,
  n: mean === null ? 0 : 2,
  nan: 0,
  mean,
  sd,
  min: mean,
  max: mean,
});

describe('chart data', () => {
  it('maps scalar rows to one line per series, x sorted by at', () => {
    const summary: Summary = {
      kind: 'scalar',
      rows: [row(0, 'a', 0, 3, 10, 1), row(0, 'a', 1, 1, 20, 2), row(1, 'b', 0, 3, 30, 0), row(1, 'b', 1, 1, null, null)],
    };
    const data = chartData(sweep, summary);
    expect(data.x).toEqual([1, 3]);
    expect(data.xLabel).toBe('Mean vision');
    expect(data.yLabel).toBe('population, mean over t = 400–500');
    expect(data.lines.map((l) => l.name)).toEqual(['a', 'b']);
    expect(data.lines[0].mean).toEqual([20, 10]);
    expect(data.lines[0].lo).toEqual([18, 9]);
    expect(data.lines[0].hi).toEqual([22, 11]);
    expect(data.lines[1].mean).toEqual([null, 30]);
    expect(data.lines[1].lo).toEqual([null, 30]);
    expect(data.lines[1].n).toEqual([0, 2]);
  });

  it('maps time-series blocks to ticks', () => {
    const ts: Sweep = { ...sweep, metric: { kind: 'timeseries', series: 'sd_log_price', every: 50 } };
    const summary: Summary = {
      kind: 'timeseries',
      rows: [
        { series: 0, series_name: 'short', t: 50, n: 2, mean: 0.5, sd: 0.1 },
        { series: 0, series_name: 'short', t: 100, n: 2, mean: 0.4, sd: 0.1 },
        { series: 1, series_name: 'long', t: 50, n: 2, mean: 0.3, sd: null },
        { series: 1, series_name: 'long', t: 100, n: 2, mean: 0.2, sd: 0 },
      ],
    };
    const data = chartData(ts, summary);
    expect(data.x).toEqual([50, 100]);
    expect(data.xLabel).toBe('Tick');
    expect(data.yLabel).toBe('sd_log_price, 50-tick means');
    expect(data.lines.map((l) => l.name)).toEqual(['short', 'long']);
    expect(data.lines[1].mean).toEqual([0.3, 0.2]);
    expect(data.lines[1].lo).toEqual([null, 0.2]);
    expect(data.lines[0].min).toEqual([null, null]);
  });

  it('describes a hovered point', () => {
    const data = chartData(sweep, { kind: 'scalar', rows: [row(0, 'a', 0, 1, 12.3456, 1.5)] });
    expect(describePoint(data.lines[0], 0)).toBe('12.35 ± 1.5, 12.35–12.35, n = 2');
    const empty = chartData(sweep, { kind: 'scalar', rows: [row(0, 'a', 0, 1, null, null)] });
    expect(describePoint(empty.lines[0], 0)).toBe('— (n = 0)');
  });
});
