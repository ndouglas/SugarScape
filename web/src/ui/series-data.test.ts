import { describe, expect, it } from 'vitest';
import { chartKey } from '../protocol';
import { bandData, chartsBehind, lineData } from './series-data';

const group = { ticks: Float64Array.of(0, 5, 9), columns: [Float64Array.of(1, NaN, 3), Float64Array.of(0.5, 0.5, NaN)] };

describe('series data', () => {
  it('uses the ticks as x and turns NaN into gaps', () => {
    expect(lineData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [0.5, 0.5, null],
    ]);
  });

  it('draws the price band around the mean', () => {
    expect(bandData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [1.5, null, null],
      [0.5, null, null],
    ]);
  });
});

describe('chartsBehind', () => {
  const cache = (groups: Record<string, typeof group>) => (names: string[]) => groups[chartKey(names)];

  it('is behind for a group that is not cached', () => {
    expect(chartsBehind([['population']], 9, cache({}))).toBe(true);
  });

  it('is caught up while the cached group reaches the tick, and behind once the tick moves on', () => {
    const cached = cache({ population: group }); // last tick: 9
    expect(chartsBehind([['population']], 9, cached)).toBe(false);
    expect(chartsBehind([['population']], 10, cached)).toBe(true);
  });

  it('is behind if any group in the list is behind or missing', () => {
    expect(chartsBehind([['population'], ['gini']], 9, cache({ population: group, gini: group }))).toBe(false);
    expect(chartsBehind([['population'], ['gini']], 9, cache({ population: group }))).toBe(true);
  });

  it('looks multi-line groups up by their chart key', () => {
    expect(chartsBehind([['population', 'gini']], 9, cache({ [chartKey(['population', 'gini'])]: group }))).toBe(false);
  });
});
