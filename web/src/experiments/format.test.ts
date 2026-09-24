import { describe, expect, it } from 'vitest';
import { fmt, scalarCells } from './format';
import type { ScalarRow } from './types';

describe('formatting', () => {
  it('rounds summary values to 4 significant figures', () => {
    expect(fmt(3.14159265)).toBe('3.142');
    expect(fmt(null)).toBe('—');
  });

  it('prints a row\'s x value unrounded', () => {
    const row: ScalarRow = { series: 0, series_name: 'a', x: 0, at: 0.123456789, n: 2, nan: 0, mean: 1.23456, sd: null, min: 1, max: 2 };
    expect(scalarCells(row)).toEqual(['a', '0.123456789', '2', '1.235', '—', '1', '2']);
  });
});
