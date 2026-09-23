import { describe, expect, it } from 'vitest';
import type { Config } from '../types';
import { controlFor, formToSweep, numericPaths, sweepToForm, TIMESERIES_X, type SweepForm } from './form';
import type { Sweep } from './types';

// every = round(ticks / 20) and from = ticks − 100: what sweepToForm fills in for unused metric fields.
const form: SweepForm = {
  name: 'Vision',
  x: { path: 'vision.max', values: '1, 2, 3' },
  series: { path: 'trade.enabled', values: 'false, true' },
  seeds: 4,
  ticks: 300,
  metric: { kind: 'window_mean', series: 'population', from: 200, to: null, every: 15 },
};

describe('form → sweep', () => {
  it('writes shorthand axes', () => {
    const { sweep, errors } = formToSweep(form, { preset: 'iv-3-trade' });
    expect(errors).toEqual([]);
    expect(sweep).toEqual({
      name: 'Vision',
      base: { preset: 'iv-3-trade' },
      x: { path: 'vision.max', values: [1, 2, 3] },
      series: { path: 'trade.enabled', values: [false, true] },
      seeds: { from: 1, count: 4 },
      ticks: 300,
      metric: { kind: 'window_mean', series: 'population', from: 200 },
    });
  });

  it('keeps an explicit window end and writes the other metric kinds', () => {
    const final = formToSweep({ ...form, metric: { ...form.metric, kind: 'final' } }, { preset: 'p' }).sweep!;
    expect(final.metric).toEqual({ kind: 'final', series: 'population' });
    const windowed = formToSweep({ ...form, metric: { ...form.metric, to: 250 } }, { preset: 'p' }).sweep!;
    expect(windowed.metric).toEqual({ kind: 'window_mean', series: 'population', from: 200, to: 250 });
  });

  it('gives a time series its single implicit x value', () => {
    const ts = formToSweep({ ...form, x: { path: '', values: '' }, metric: { ...form.metric, kind: 'timeseries' } }, { preset: 'p' });
    expect(ts.errors).toEqual([]);
    expect(ts.sweep!.x).toEqual(TIMESERIES_X);
    expect(ts.sweep!.metric).toEqual({ kind: 'timeseries', series: 'population', every: 15 });
  });

  it('reports path and value errors on their controls', () => {
    const { sweep, errors } = formToSweep(
      { ...form, x: { path: '', values: 'a' }, series: { path: 's', values: '1:2' } },
      { preset: 'p' },
    );
    expect(sweep).toBeNull();
    expect(errors.map((e) => e.field)).toEqual(['x.path', 'x.values', 'series.values']);
  });

  it('drops the second axis when unset', () => {
    expect(formToSweep({ ...form, series: null }, { preset: 'p' }).sweep).not.toHaveProperty('series');
  });
});

describe('sweep → form', () => {
  it('round-trips a form-made sweep', () => {
    expect(sweepToForm(formToSweep(form, { preset: 'iv-3-trade' }).sweep!)).toEqual(form);
  });

  it('reads the full axes the core writes', () => {
    const full: Sweep = {
      ...formToSweep(form, { preset: 'p' }).sweep!,
      x: { label: 'vision.max', values: [1, 2, 3].map((v) => ({ at: v, set: { 'vision.max': v } })) },
      series: { label: 'trade.enabled', values: [false, true].map((v, i) => ({ at: i, set: { 'trade.enabled': v } })) },
    };
    expect(sweepToForm(full)).toEqual(form);
  });

  it('round-trips a time series', () => {
    const ts: SweepForm = { ...form, x: { path: '', values: '' }, metric: { ...form.metric, kind: 'timeseries' } };
    expect(sweepToForm(formToSweep(ts, { preset: 'p' }).sweep!)).toEqual(ts);
  });

  it('declines sweeps the form cannot express', () => {
    const base = formToSweep(form, { preset: 'p' }).sweep!;
    const ranges: Sweep = { ...base, x: { label: 'Mean vision', values: [{ at: 1, set: { vision: { min: 1, max: 1 } } }] } };
    expect(sweepToForm(ranges)).toBeNull();
    const named: Sweep = { ...base, series: { label: 'trade.enabled', values: [{ at: 0, name: 'No trade', set: { 'trade.enabled': false } }] } };
    expect(sweepToForm(named)).toBeNull();
    expect(sweepToForm({ ...base, set: { population: 500 } })).toBeNull();
    expect(sweepToForm({ ...base, seeds: { from: 3, count: 2 } })).toBeNull();
  });
});

describe('error placement', () => {
  it('maps core fields to form controls', () => {
    expect(controlFor('x[2].set.vision.max')).toBe('x.path');
    expect(controlFor('series[0].set.trade.enabled')).toBe('series.path');
    expect(controlFor('x.values')).toBe('x.values');
    expect(controlFor('metric.from')).toBe('metric.from');
    expect(controlFor('seeds.count')).toBe('seeds.count');
    expect(controlFor('points')).toBe('general');
    expect(controlFor('base')).toBe('general');
  });
});

describe('path suggestions', () => {
  it('lists numeric and boolean leaves, skipping the schedule and outbreaks', () => {
    const config = {
      population: 400,
      vision: { min: 1, max: 6 },
      trade: { enabled: false },
      goods: [{ name: 'sugar', metabolism: { min: 1, max: 4 } }],
      schedule: [{ tick: 5, set: {} }],
      disease: { enabled: false, outbreaks: [{ tick: 1, agents: 2 }] },
    } as unknown as Config;
    expect(numericPaths(config)).toEqual([
      'population',
      'vision.min',
      'vision.max',
      'trade.enabled',
      'goods.0.metabolism.min',
      'goods.0.metabolism.max',
      'disease.enabled',
    ]);
  });
});
