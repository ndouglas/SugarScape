import { describe, expect, it } from 'vitest';
import { middleSlice, playerRows, sliceOptions } from './spatial';
import type { PlayerView, SpatialConfig } from './types';

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

const player = (c: Partial<PlayerView>): PlayerView => ({
  id: 7,
  strategy: 'C',
  previous: 'D',
  score: 6,
  candidates: [
    { x: 1, y: 1, z: 0, strategy: 'C', score: 6 },
    { x: 0, y: 1, z: 0, strategy: 'D', score: 7.6 },
    { x: 2, y: 1, z: 0, strategy: 'C', score: 5 },
  ],
  next: 'D',
  p_c: null,
  ...c,
});

describe('playerRows', () => {
  it('shows the strategy, the score, the neighborhood’s best of each kind and what comes next', () => {
    expect(playerRows(player({}))).toEqual([
      ['Player', '#7 · cooperates (defected last generation)'],
      ['Score', '6'],
      ['Neighborhood', '2 C (best 6), 1 D (best 7.60), itself included'],
      ['Next generation', 'defects'],
    ]);
  });

  it('gives P(C) under probabilistic winning, and says so when no score counts', () => {
    expect(playerRows(player({ next: null, p_c: 0.25 }))[3]).toEqual(['Next generation', 'cooperates with probability 25%']);
    expect(playerRows(player({ next: null, p_c: null }))[3]).toEqual(['Next generation', 'keeps its strategy (every score is 0)']);
  });
});
