import { describe, expect, it } from 'vitest';
import { classifyFile, slug } from './file';
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
