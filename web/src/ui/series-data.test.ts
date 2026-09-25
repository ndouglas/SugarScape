import { describe, expect, it } from 'vitest';
import { chartKey } from '../protocol';
import type { Config } from '../types';
import {
  bandData,
  barsData,
  chartsBehind,
  distributionsDue,
  distributionWants,
  emptyTable,
  histTable,
  lineData,
  MODEL_CHARTS,
  overlayData,
  positionBars,
  positionSteps,
  shownCharts,
  showsAgeHist,
  showsGoodWealth,
  showsTagHist,
  showsTotalWealth,
  supplyDemandTable,
  showsForModel,
  timeAxisLabel,
  twoGoods,
  worldLines,
  yearTickLabels,
  type LineData,
} from './series-data';

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

describe('Chapter VI histograms', () => {
  const config = (goods: number, lifespan: boolean, culture: boolean) =>
    ({ goods: Array.from({ length: goods }, () => ({})), lifespan: { enabled: lifespan }, culture: { enabled: culture } }) as unknown as Config;

  it('shows the age histogram while lifetimes are finite and the tag histogram while culture is on', () => {
    expect([showsAgeHist(config(1, true, false)), showsTagHist(config(1, true, false))]).toEqual([true, false]);
    expect([showsAgeHist(config(1, false, true)), showsTagHist(config(1, false, true))]).toEqual([false, true]);
  });

  it('asks for the distributions each world draws', () => {
    expect(distributionWants(config(1, false, false))).toEqual({ lorenz: true, wealthHist: true });
    expect(distributionWants(config(2, true, true))).toEqual({
      lorenz: true,
      wealthHist: true,
      supplyDemand: true,
      lorenzTotal: true,
      goodWealthHists: true,
      ageHist: true,
      tagHist: true,
    });
  });

  it('fetches distributions again only once the tick moved or they went stale, at most every 250 ms', () => {
    const d = { at: 1000, tick: 7, stale: false };
    expect(distributionsDue(d, 7, 5000, 250)).toBe(false); // paused and caught up
    expect(distributionsDue(d, 8, 1100, 250)).toBe(false); // too soon
    expect(distributionsDue(d, 8, 1250, 250)).toBe(true);
    expect(distributionsDue({ ...d, stale: true }, 7, 1250, 250)).toBe(true);
  });

  it('draws the tag histogram as bars at positions 1…L or as a step outline around them', () => {
    const pct = Float64Array.of(49, 100, 0);
    expect(positionBars(pct)).toEqual([[1, 2, 3], [49, 100, 0]]);
    expect(positionSteps(pct)).toEqual([[0.5, 1.5, 2.5, 3.5], [49, 100, 0, 0]]);
    expect(positionBars(null)).toEqual([[], []]);
    expect(positionSteps(undefined)).toEqual([[], []]);
  });

  it('draws the age histogram like the wealth histogram ([bin, counts…])', () => {
    const ages = Float64Array.of(5, 3, 0, 2);
    expect(barsData(ages)).toEqual([[2.5, 7.5, 12.5], [3, 0, 2]]);
    expect(histTable(ages)).toEqual([[0, 5, 10, 15], [3, 0, 2, 0]]);
  });
});

describe('wealth views', () => {
  const goods = (n: number) => ({ goods: Array.from({ length: n }, () => ({})) }) as unknown as Config;

  it("show total wealth and each good's histogram only with two or more goods", () => {
    expect(showsTotalWealth(goods(1))).toBe(false);
    expect(showsTotalWealth(goods(2))).toBe(true);
    expect(showsGoodWealth(0)(goods(1))).toBe(false);
    expect([0, 1, 2].map((g) => showsGoodWealth(g)(goods(2)))).toEqual([true, true, false]);
  });

  it('twoGoods and showsTotalWealth read the same world the same way, under their own names', () => {
    expect(twoGoods(goods(1))).toBe(false);
    expect(twoGoods(goods(2))).toBe(true);
    expect(twoGoods).toBe(showsTotalWealth);
  });
});

describe('shownCharts', () => {
  it('keeps only the charts on show, in order, each under its own caption', () => {
    const charts = [
      { name: 'Wealth distribution · sugar', hidden: false },
      { name: 'Wealth distribution · spice', hidden: false },
      { name: 'Wealth distribution · salt', hidden: true },
      { name: 'Age histogram', hidden: true },
      { name: 'Cultural tags (% zeros by position)', hidden: false },
    ];
    const shown = shownCharts(charts);
    expect(shown.map((c) => c.name)).toEqual(['Wealth distribution · sugar', 'Wealth distribution · spice', 'Cultural tags (% zeros by position)']);
    // Distinct captions (not the shared 'Wealth distribution' title) keep every exported file name unique.
    expect(new Set(shown.map((c) => c.name)).size).toBe(shown.length);
  });
});

describe('model charts', () => {
  it('shows a model’s charts while some world on screen runs it', () => {
    expect(showsForModel('schelling', ['schelling'])).toBe(true);
    expect(showsForModel('sugarscape', ['schelling'])).toBe(false);
    expect(showsForModel('ring', ['sugarscape', 'ring'])).toBe(true);
  });
});

describe('the anasazi’s charts', () => {
  it('count years: a group’s ticks move by the start year', () => {
    expect(lineData(group, 800)[0]).toEqual([800, 805, 809]);
    expect(timeAxisLabel('anasazi')).toBe('Year');
    expect(timeAxisLabel('schelling')).toBe('Tick');
  });

  it('label the year axis without digit grouping', () => {
    expect(yearTickLabels([800, 1000, 1350])).toEqual(['800', '1000', '1350']);
    expect(yearTickLabels([1000.5, 1000.1 + 0.2])).toEqual(['1000.5', '1000.3']);
  });

  it('draw households against the historical record, which Compare draws once', () => {
    expect(MODEL_CHARTS.anasazi.map((c) => c.title)).toEqual([
      'Households vs historical',
      'Carrying capacity',
      'Fit',
      'Mean stored corn',
      'Births, moves and departures',
    ]);
    const lines = MODEL_CHARTS.anasazi[0].lines;
    expect(worldLines(lines, 0).map((l) => l.key)).toEqual(['households', 'historical']);
    expect(worldLines(lines, 1).map((l) => l.key)).toEqual(['households']);
  });
});
