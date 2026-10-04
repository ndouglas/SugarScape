import { describe, expect, it } from 'vitest';
import * as format from './format';
import type { RunResult } from './types';

const describeRuns = (runs: unknown[]) => {
  const fn = (format as unknown as { runAvailability?: (runs: RunResult[]) => string }).runAvailability;
  expect(fn).toBeTypeOf('function');
  return fn!(runs as RunResult[]);
};
const run = (status: string, reason: string | null = null) => ({ point: 0, series: 0, x: 0, seed: 1, value: null, democratic_peace: { status, completed_periods: 14, attempted_period: 14, last_tick_periods: 2, invalid_reason: null, invalid_phase: null, clustering_reason: reason } });
describe('Democratic peace experiment availability', () => {
  it('counts legitimate undefined clustering as a complete history with its reason', () => {
    expect(describeRuns([run('complete', 'undefined_initial_density'), run('complete', 'undefined_extinction'), { ...run('complete'), value: 0 }])).toBe('3 complete · 0 invalid · clustering unavailable: undefined_initial_density (1), undefined_extinction (1)');
  });
  it('keeps invalid histories distinct from valid zero and undefined complete metrics', () => {
    expect(describeRuns([{ ...run('invalid'), democratic_peace: { ...run('invalid').democratic_peace, completed_periods: 2, attempted_period: 3, invalid_reason: 'zero denominator', invalid_phase: 'allocation' } }, { ...run('complete'), value: 0 }])).toContain('1 complete · 1 invalid');
  });
  it('leaves legacy results without model availability metadata unchanged', () => {
    expect(describeRuns([{ point: 0, series: 0, x: 0, seed: 1, value: null }])).toBe('');
  });
});
