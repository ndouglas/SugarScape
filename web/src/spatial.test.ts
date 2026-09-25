import { describe, expect, it } from 'vitest';
import { middleSlice, sliceOptions } from './spatial';
import type { SpatialConfig } from './types';

const spatial = (c: Partial<SpatialConfig>): SpatialConfig => ({ lattice: 'square', width: 30, ...c }) as SpatialConfig;

describe('slices', () => {
  it('offers a cube’s z-slices and none otherwise', () => {
    expect(sliceOptions(spatial({ lattice: 'cube', width: 3 }))).toEqual([
      ['slice:0', 'z = 0'],
      ['slice:1', 'z = 1'],
      ['slice:2', 'z = 2'],
    ]);
    expect(sliceOptions(spatial({}))).toEqual([]);
    expect(sliceOptions(spatial({ lattice: 'random' }))).toEqual([]);
  });

  it('starts at the middle slice, as the core does', () => {
    expect(middleSlice(spatial({ lattice: 'cube', width: 30 }))).toBe('slice:15');
    expect(middleSlice(spatial({ lattice: 'cube', width: 5 }))).toBe('slice:2');
  });
});
