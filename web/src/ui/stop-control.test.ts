import { describe, expect, it } from 'vitest';
import { parseStopRules } from './stop-control';

describe('parseStopRules', () => {
  const blank = { tick: '', series: 'population', op: '<', value: '' };
  it('makes no rules from blank fields', () => {
    expect(parseStopRules(blank)).toEqual({});
  });
  it('reads a tick and a condition', () => {
    expect(parseStopRules({ tick: '500', series: 'gini', op: '>', value: '0.5' })).toEqual({
      tick: 500,
      when: { series: 'gini', op: '>', value: 0.5 },
    });
  });
  it('refuses a negative or fractional tick and a non-number value', () => {
    expect(typeof parseStopRules({ ...blank, tick: '-3' })).toBe('string');
    expect(typeof parseStopRules({ ...blank, tick: '2.5' })).toBe('string');
    expect(typeof parseStopRules({ ...blank, value: 'lots' })).toBe('string');
  });
});
