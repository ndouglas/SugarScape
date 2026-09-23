import { describe, expect, it } from 'vitest';
import { compactNumber } from './format';

describe('compactNumber', () => {
  it('leaves small magnitudes unchanged', () => {
    expect(compactNumber(0)).toBe('0');
    expect(compactNumber(12)).toBe('12');
    expect(compactNumber(999)).toBe('999');
  });

  it('adds a compact suffix for larger magnitudes', () => {
    expect(compactNumber(1234)).toBe('1.2k');
    expect(compactNumber(25000)).toBe('25k');
    expect(compactNumber(1500000)).toBe('1.5M');
  });

  it('preserves the sign', () => {
    expect(compactNumber(-1234)).toBe('-1.2k');
  });
});
