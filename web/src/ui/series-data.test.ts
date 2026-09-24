import { describe, expect, it } from 'vitest';
import { bandData, ChartFreshness, lineData } from './series-data';

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

describe('ChartFreshness', () => {
  it('is behind for a group it has never seen', () => {
    const f = new ChartFreshness();
    expect(f.behind([['population']], 9)).toBe(true);
  });

  it('catches up once it receives a group whose last tick matches, and asks again once behind', () => {
    const f = new ChartFreshness();
    f.receive({ population: group }); // last tick: 9
    expect(f.behind([['population']], 9)).toBe(false);
    expect(f.behind([['population']], 10)).toBe(true);
  });

  it('ignores a reply that does not carry the group it is asking about', () => {
    const f = new ChartFreshness();
    f.receive({ population: group });
    expect(f.behind([['gini']], 9)).toBe(true);
  });

  it('is behind if any group in the list is behind', () => {
    const f = new ChartFreshness();
    f.receive({ population: group, gini: group });
    expect(f.behind([['population'], ['gini']], 9)).toBe(false);
    expect(f.behind([['population'], ['gini']], 10)).toBe(true);
  });

  it('forgets every group on reset, so the next ask is behind again', () => {
    const f = new ChartFreshness();
    f.receive({ population: group });
    f.reset();
    expect(f.behind([['population']], 9)).toBe(true);
  });
});
