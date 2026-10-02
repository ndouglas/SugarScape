import { describe, expect, it } from 'vitest';
import type { Config, EthnoConfig, Param, SchellingConfig } from '../types';
import { controlFor, defaultForm, formToSweep, numericPaths, sweepToForm, TIMESERIES_X, type SweepForm } from './form';
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

  it('keeps the description', () => {
    const described: SweepForm = { ...form, description: 'Why vision matters' };
    const sweep = formToSweep(described, { preset: 'p' }).sweep!;
    expect(sweep.description).toBe('Why vision matters');
    expect(sweepToForm(sweep)).toEqual(described);
    expect(formToSweep(form, { preset: 'p' }).sweep).not.toHaveProperty('description');
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

  it('suggests a null nullable numeric field too (the ethnocentrism model’s tag mutation), given its schema', () => {
    const config = { model: 'ethno', mutation: 0.005, tag_mutation: null } as unknown as EthnoConfig;
    const nullable = { path: 'tag_mutation', label: 'Tag mutation rate', kind: 'number', apply: 'live', group: 'Mutation', nullable: true } as Param;
    const schema = [{ path: 'mutation', label: 'Mutation rate', kind: 'number', apply: 'live', group: 'Mutation' } as Param, nullable];
    expect(numericPaths(config, schema)).toEqual(['mutation', 'tag_mutation']);
    // Without the schema, a null leaf is skipped, as before — never guessed at.
    expect(numericPaths(config)).toEqual(['mutation']);
  });
});

describe('sweeps over other models', () => {
  it('start from an axis and a statistic the base’s model has', () => {
    expect(defaultForm().x.path).toBe('vision.max');
    expect(defaultForm('schelling')).toMatchObject({ x: { path: 'population' }, metric: { kind: 'final', series: 'segregation' } });
    expect(defaultForm('ring')).toMatchObject({ x: { path: 'agents' }, metric: { series: 'flocks' } });
    expect(defaultForm('anasazi')).toMatchObject({
      x: { path: 'harvest_adjustment', values: '0.5:0.62:0.02' },
      ticks: 550,
      metric: { kind: 'final', series: 'fit' },
    });
    expect(defaultForm('firms')).toMatchObject({
      x: { path: 'beta', values: '1.7:2.1:0.1' },
      ticks: 5000,
      metric: { kind: 'final', series: 'mu' },
    });
    expect(defaultForm('collusion')).toMatchObject({
      x: { path: 'delta', values: '0:0.9:0.15' },
      ticks: 100_000,
      metric: { kind: 'final', series: 'cycle_gain' },
    });
    expect(defaultForm('bali')).toMatchObject({
      x: { path: 'growth', values: '2:2.4:0.1' },
      ticks: 360,
      metric: { kind: 'final', series: 'scored' },
    });
    expect(defaultForm('hoard')).toMatchObject({
      x: { path: 'app_scat', values: '0.05,0.1,0.2,0.3,0.44,0.6,0.8,0.9' },
      ticks: 100000,
      metric: { kind: 'window_mean', series: 'hoarder_larder_prob', from: 98001, to: 100000 },
    });
    expect(defaultForm('zi')).toMatchObject({
      x: { path: 'shouts', values: '25,50,100,200,500,1000,2000' },
      ticks: 12000,
      metric: { kind: 'final', series: 'avg_efficiency' },
    });
    expect(defaultForm('punishment')).toMatchObject({
      x: { path: 'size', values: '4,8,16,32,64,128,256' },
      ticks: 2000,
      metric: { kind: 'final', series: 'long_run' },
    });
    expect(defaultForm('retirement')).toMatchObject({
      x: { path: 'rational', values: '0.02,0.05,0.1,0.15,0.2,0.25' },
      ticks: 400,
      metric: { kind: 'final', series: 'transition' },
    });
    expect(defaultForm('thresholds')).toMatchObject({
      x: { path: 'sd', values: '0.1:0.2:0.01' },
      ticks: 200,
      metric: { kind: 'final', series: 'acting' },
    });
    expect(defaultForm('ants')).toMatchObject({
      x: { path: 'epsilon', values: '0.001,0.002,0.003,0.005,0.01' },
      ticks: 20000,
      metric: { kind: 'final', series: 'flips' },
    });
    expect(defaultForm('farol')).toMatchObject({
      x: { path: 'strategies', values: '2:24:2' },
      ticks: 2000,
      metric: { kind: 'final', series: 'fluctuation' },
    });
    expect(defaultForm('agreement')).toMatchObject({
      x: { path: 'uncertainty', values: '0.2:2:0.2' },
      ticks: 20000,
      metric: { kind: 'final', series: 'y' },
    });
    expect(defaultForm('norms')).toMatchObject({
      x: { path: 'stop_at', values: '100,1000,10000' },
      ticks: 10000,
      metric: { kind: 'final', series: 'collapsed' },
    });
    expect(defaultForm('structure')).toMatchObject({
      x: { path: 'substitution', values: '0:1:0.1' },
      ticks: 2500,
      metric: { kind: 'window_mean', series: 'mean_payoff', from: 1501 },
    });
    expect(defaultForm('opinions')).toMatchObject({
      x: { path: 'epsilon', values: '0.05:0.3:0.05' },
      ticks: 1000,
      metric: { kind: 'final', series: 'clusters' },
    });
    expect(defaultForm('classes')).toMatchObject({
      x: { path: 'memory', values: '6:14:2' },
      ticks: 100000,
      metric: { kind: 'final', series: 'equity_at' },
    });
    expect(defaultForm('culture')).toMatchObject({
      x: { path: 'traits', values: '5:15:5' },
      ticks: 20000,
      metric: { kind: 'final', series: 'regions' },
    });
    expect(defaultForm('tags')).toMatchObject({
      x: { path: 'pairings', values: '1:4:1' },
      ticks: 3000,
      metric: { kind: 'window_mean', series: 'donation_rate', from: 0 },
    });
    expect(defaultForm('civil')).toMatchObject({
      x: { path: 'legitimacy', values: '0.6:0.95:0.05' },
      ticks: 1000,
      seeds: 3,
      metric: { kind: 'final', series: 'outbursts' },
    });
    expect(defaultForm('spatial')).toMatchObject({
      x: { path: 'b', values: '1.05:2.05:0.05' },
      ticks: 200,
      metric: { kind: 'final', series: 'fraction_c' },
    });
    expect(defaultForm('ethno')).toMatchObject({
      x: { path: 'cost', values: '0.005:0.03:0.0025' },
      ticks: 2000,
      metric: { kind: 'window_mean', series: 'ethnocentric', from: 1901, to: null },
    });
    expect(defaultForm('dpd')).toMatchObject({
      x: { path: 'r', values: '1:5:1' },
      series: null,
      seeds: 3,
      ticks: 500,
      metric: { kind: 'final', series: 'cooperators' },
    });
    expect(defaultForm('image')).toMatchObject({
      x: { path: 'rounds', values: '50:500:50' },
      series: null,
      seeds: 3,
      ticks: 2000,
      metric: { kind: 'window_mean', series: 'cooperative', from: 1001, to: null },
    });
  });

  it('suggest a Schelling config’s own paths, never its model tag', () => {
    const schelling: SchellingConfig = {
      model: 'schelling',
      width: 50,
      height: 50,
      population: 2000,
      preference: { min: 0.25, max: 0.25 },
      residence: { enabled: false, min: 80, max: 100 },
      neighborhood: 'von_neumann',
      radius: 1,
      edges: 'torus',
      movement: 'random',
      order: 'random',
      sweep: 'reading',
      red_share: 0.5,
      exact: false,
      red_demand: { min: [], max: [] },
      blue_demand: { min: [], max: [] },
      movers: 'discontent',
      utility: 'flat',
      beta: 10,
      start: 'random',
    };
    expect(numericPaths(schelling)).toEqual([
      'width',
      'height',
      'population',
      'preference.min',
      'preference.max',
      'residence.enabled',
      'residence.min',
      'residence.max',
      'radius',
      'red_share',
      'exact',
      'beta',
    ]);
    const { sweep } = formToSweep(defaultForm('schelling'), { config: schelling });
    expect(sweep!.base).toEqual({ config: schelling });
  });
});
