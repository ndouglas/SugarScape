import { describe, expect, it } from 'vitest';
import { formatValues, parseValues } from './values';

describe('value lists', () => {
  it('reads comma lists of numbers', () => {
    expect(parseValues('1, 2.5, -3, 1e3', 64)).toEqual({ values: [1, 2.5, -3, 1000] });
  });

  it('reads booleans', () => {
    expect(parseValues('false,true', 64)).toEqual({ values: [false, true] });
  });

  it('reads inclusive ranges with tidy steps', () => {
    expect(parseValues('1:6:1', 64)).toEqual({ values: [1, 2, 3, 4, 5, 6] });
    expect(parseValues('0:0.3:0.1', 64)).toEqual({ values: [0, 0.1, 0.2, 0.3] });
    expect(parseValues(' 2 : 3 : 5 ', 64)).toEqual({ values: [2] });
    expect(parseValues('-1:1:1', 64)).toEqual({ values: [-1, 0, 1] });
  });

  it('rejects bad input', () => {
    for (const text of ['', '1,,2', 'a', '1, true', '1:2', '1:2:0', '3:1:1', '1:x:1', 'true:false:1']) {
      expect('error' in parseValues(text, 64), text).toBe(true);
    }
  });

  it('enforces the axis limit', () => {
    expect(parseValues('1:64:1', 64)).toHaveProperty('values');
    expect(parseValues('1:65:1', 64)).toEqual({ error: 'At most 64 values' });
    expect(parseValues(Array.from({ length: 17 }, (_, i) => i).join(','), 16)).toEqual({ error: 'At most 16 values' });
  });

  it('formats values back', () => {
    expect(formatValues([1, 2.5, true])).toBe('1, 2.5, true');
  });
});
