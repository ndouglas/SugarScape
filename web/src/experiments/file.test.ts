import { describe, expect, it } from 'vitest';
import { classifyFile, readOpened, slug, type OpenCore } from './file';
import type { Sweep } from './types';

const sweep: Sweep = {
  name: 'Vision',
  base: { preset: 'ii-2-unit' },
  x: { path: 'vision.max', values: [1, 2] },
  seeds: { from: 1, count: 2 },
  ticks: 100,
  metric: { kind: 'final', series: 'population' },
};

describe('opening files', () => {
  it('recognizes sweeps and results', () => {
    expect(classifyFile(sweep)).toEqual({ kind: 'sweep', sweep });
    const result = { version: 1, sweep, runs: [], summary: { kind: 'scalar', rows: [] } };
    expect(classifyFile(result)).toEqual({ kind: 'result', result });
  });

  it('explains what it cannot open', () => {
    expect(classifyFile([])).toEqual({ kind: 'error', message: 'expected a sweep or a sweep result (a JSON object)' });
    expect(classifyFile({ version: 2, sweep, runs: [] })).toEqual({ kind: 'error', message: 'unsupported result version 2' });
    expect(classifyFile({ version: 1, runs: [] })).toEqual({ kind: 'error', message: 'a result needs "sweep" and "runs"' });
    expect(classifyFile({ name: 'x' })).toEqual({
      kind: 'error',
      message: 'expected a sweep (with base, x and metric) or a sweep result',
    });
  });

  it('makes file names', () => {
    expect(slug('Figure II-5: carrying capacity')).toBe('figure-ii-5-carrying-capacity');
    expect(slug('!!!')).toBe('sweep');
  });
});

describe('reading opened payloads with the core', () => {
  const full = { ...sweep, x: { label: 'vision.max', values: [1, 2].map((v) => ({ at: v, set: { 'vision.max': v } })) } };
  const calls: string[] = [];
  const core = (errors: { parse?: string; aggregate?: string } = {}): OpenCore => ({
    parseSweep: (spec) => {
      calls.push(`parse ${spec}`);
      if (errors.parse) throw errors.parse;
      return JSON.stringify(full);
    },
    aggregate: (spec, runs) => {
      calls.push(`aggregate ${spec} ${runs}`);
      if (errors.aggregate) throw errors.aggregate;
      return '{"kind":"scalar","rows":[]}';
    },
  });

  it('returns sweeps in the full form the core writes', () => {
    calls.length = 0;
    expect(readOpened(sweep, core())).toEqual({ kind: 'sweep', sweep: full });
    expect(calls).toEqual([`parse ${JSON.stringify(sweep)}`]);
  });

  it('checks a result\'s runs against its full sweep', () => {
    calls.length = 0;
    const runs = [{ point: 0, series: 0, x: 0, seed: 1, value: 3 }];
    const result = { version: 1, sweep, runs, summary: { kind: 'scalar', rows: [] } };
    expect(readOpened(result, core())).toEqual({ kind: 'result', result: { ...result, sweep: full } });
    expect(calls).toEqual([`parse ${JSON.stringify(sweep)}`, `aggregate ${JSON.stringify(full)} ${JSON.stringify(runs)}`]);
  });

  it('turns core errors into messages', () => {
    const ticks = '[{"field":"ticks","message":"must be 1 to 100000"}]';
    expect(readOpened(sweep, core({ parse: ticks }))).toEqual({ kind: 'error', message: 'ticks: must be 1 to 100000' });
    const result = { version: 1, sweep, runs: [{}], summary: {} };
    const runs = '[{"field":"runs[0]","message":"does not match"}]';
    expect(readOpened(result, core({ aggregate: runs }))).toEqual({ kind: 'error', message: 'runs[0]: does not match' });
    expect(readOpened([], core())).toEqual({ kind: 'error', message: 'expected a sweep or a sweep result (a JSON object)' });
  });
});
