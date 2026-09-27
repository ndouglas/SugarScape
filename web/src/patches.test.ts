import { describe, expect, it } from 'vitest';
import { hasPatches } from './patches';
import type { Config } from './types';

const withMap = (map: unknown) => ({ goods: [{ map }] }) as unknown as Config;
const peak = { x: 1, y: 1, radius: 2, height: 4 };

describe('hasPatches', () => {
  it('is true only for a peaks map of two or more peaks', () => {
    expect(hasPatches(withMap({ kind: 'peaks', peaks: [peak, peak] }))).toBe(true);
    expect(hasPatches(withMap({ kind: 'peaks', peaks: [peak] }))).toBe(false);
    expect(hasPatches(withMap({ kind: 'two_peaks', transform: 'identity' }))).toBe(false);
  });
});
