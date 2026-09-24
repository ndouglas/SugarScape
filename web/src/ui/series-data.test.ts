import { describe, expect, it } from 'vitest';
import { chartKey } from '../protocol';
import { bandData, barsData, chartsBehind, emptyTable, histTable, lineData, overlayData, supplyDemandTable, type LineData } from './series-data';

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

describe('two worlds on one axis', () => {
  it('puts tables on the union of their x values: holes are undefined (drawn through), gaps stay null', () => {
    const a: LineData = [[0, 2, 4], [1, null, 3]];
    const b: LineData = [[0, 3, 4], [5, 6, 7], [8, 9, 10]];
    expect(overlayData([a, b])).toEqual([
      [0, 2, 3, 4],
      [1, null, undefined, 3],
      [5, undefined, 6, 7],
      [8, undefined, 9, 10],
    ]);
  });

  it('gives a missing group an empty table with the right number of lines', () => {
    expect(emptyTable(2)).toEqual([[], [], []]);
    expect(overlayData([emptyTable(1), [[1, 2], [5, 6]]])).toEqual([[1, 2], [undefined, undefined], [5, 6]]);
  });

  it('draws a histogram as bars or as a step outline over its bin edges', () => {
    const hist = Float64Array.of(2, 5, 0, 1);
    expect(barsData(hist)).toEqual([[1, 3, 5], [5, 0, 1]]);
    expect(histTable(hist)).toEqual([[0, 2, 4, 6], [5, 0, 1, 0]]);
    expect(histTable(null)).toEqual([[], []]);
  });

  it('turns supply and demand into two curves and two marked points', () => {
    const sd = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, 1.1, 5, 2, 9);
    expect(supplyDemandTable(sd)).toEqual([
      [0.5, 1, 2],
      [9, 5, 1],
      [1, 5, 9],
      [null, 5, null],
      [null, null, 9],
    ]);
    const none = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, NaN, NaN, NaN, NaN);
    expect(supplyDemandTable(none).slice(3)).toEqual([
      [null, null, null],
      [null, null, null],
    ]);
    expect(supplyDemandTable(undefined)).toEqual([[], [], [], [], []]);
  });
});
