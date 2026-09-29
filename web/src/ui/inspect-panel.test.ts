import { describe, expect, it } from 'vitest';
import { cachesText, cachingRows, carryingText, centralRows, headingText, memoryText, planText, rateText } from './inspect-panel';

const plan = (path: [number, number][]) => ({ target_x: 3, target_y: 4, path, walked: true });
const memory = (sites: number, spots: number) => ({ remembers: true, sites, spots });

describe('headingText', () => {
  it('says "Staying" once the path is empty at the target (arrived, or nothing to plan)', () => {
    expect(headingText(plan([]), [3, 4])).toBe('Staying');
  });

  it("says it can't reach a target elsewhere when no path was found", () => {
    expect(headingText(plan([]), [1, 1])).toBe("Can't reach (3, 4)");
  });

  it('singularizes exactly one step left', () => {
    expect(headingText(plan([[3, 4]]), [2, 4])).toBe('(3, 4), 1 step left');
  });

  it('pluralizes more than one step left', () => {
    expect(headingText(plan([[1, 4], [2, 4], [3, 4]]), [0, 4])).toBe('(3, 4), 3 steps left');
  });
});

describe('memoryText', () => {
  it("says it doesn't remember for a non-rememberer", () => {
    expect(memoryText({ remembers: false, sites: 0, spots: 0 })).toBe("Doesn't remember");
  });

  it("says it doesn't remember when memory is off (null)", () => {
    expect(memoryText(null)).toBe("Doesn't remember");
  });

  it('pluralizes zero sites and zero truffle spots', () => {
    expect(memoryText(memory(0, 0))).toBe('0 sites (0 truffle spots)');
  });

  it('singularizes exactly one site and one truffle spot', () => {
    expect(memoryText(memory(1, 1))).toBe('1 site (1 truffle spot)');
  });

  it('pluralizes two sites and two truffle spots', () => {
    expect(memoryText(memory(2, 2))).toBe('2 sites (2 truffle spots)');
  });

  it('singularizes and pluralizes each noun independently', () => {
    expect(memoryText(memory(1, 2))).toBe('1 site (2 truffle spots)');
    expect(memoryText(memory(2, 1))).toBe('2 sites (1 truffle spot)');
  });
});

describe('planText', () => {
  const goap = (steps: [number, number][], gathers: number, goal: number) => ({ steps, gathers, goal });

  it('singularizes exactly one step', () => {
    expect(planText(goap([[3, 4]], 10.04, 10))).toBe('1 step, gathers ~10 of 10');
  });

  it('pluralizes more than one step and rounds the gathering to one decimal', () => {
    expect(planText(goap([[3, 4], [5, 6], [7, 8]], 12.345, 10))).toBe('3 steps, gathers ~12.3 of 10');
  });

  it('says done once no targets are left', () => {
    expect(planText(goap([], 11.5, 10))).toBe('Done (gathers ~11.5 of 10)');
  });

  it('says done for an empty plan (G = 0)', () => {
    expect(planText(goap([], 0, 0))).toBe('Done (gathers ~0 of 0)');
  });
});

describe('rateText', () => {
  it('gives ρ in sugar a tick to two decimals', () => {
    expect(rateText(1)).toBe('1.00 sugar a tick');
    expect(rateText(0.4567)).toBe('0.46 sugar a tick');
  });
});

describe('carryingText', () => {
  it('gives the holding against the carrying limit', () => {
    expect(carryingText(12, 20)).toBe('12 of 20');
    expect(carryingText(7.456, 320)).toBe('7.46 of 320');
  });

  it('gives the holding alone with no limit (0)', () => {
    expect(carryingText(12, 0)).toBe('12');
  });
});

describe('cachesText', () => {
  const cache = (amount: number) => ({ x: 1, y: 2, amount });

  it('singularizes exactly one cache', () => {
    expect(cachesText({ caches: [cache(4)], total: 4 })).toBe('1 cache, holding 4');
  });

  it('pluralizes none and several', () => {
    expect(cachesText({ caches: [], total: 0 })).toBe('0 caches, holding 0');
    expect(cachesText({ caches: [cache(2.5), cache(3)], total: 5.5 })).toBe('2 caches, holding 5.50');
  });
});

describe('cachingRows', () => {
  const view = (forecast: number | null) => ({ rule: 'plan' as const, holdings_cap: 20, caches: [{ x: 1, y: 2, amount: 3 }], total: 3, forecast });

  it('gives the agent’s own rule, what it carries, its caches and the forecast under plan', () => {
    expect(cachingRows(view(6.5), 9)).toEqual([
      ['Caching rule', 'plan'],
      ['Carrying', '9 of 20'],
      ['Caches', '1 cache, holding 3'],
      ['Forecast shortfall', '6.50'],
    ]);
  });

  it('leaves the forecast out when there is none (other rules, or winter)', () => {
    expect(cachingRows({ ...view(null), rule: 'even', holdings_cap: 0 }, 9)).toEqual([
      ['Caching rule', 'even'],
      ['Carrying', '9'],
      ['Caches', '1 cache, holding 3'],
    ]);
  });
});

describe('centralRows', () => {
  it('gives the home and the last load', () => {
    expect(centralRows({ home: [4, 7], last_load: 12.25 })).toEqual([
      ['Home', '(4, 7)'],
      ['Last load', '12.25'],
    ]);
    expect(centralRows({ home: [0, 0], last_load: 0 })).toEqual([['Home', '(0, 0)'], ['Last load', '0']]);
  });
});
