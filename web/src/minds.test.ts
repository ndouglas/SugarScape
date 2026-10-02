import { describe, expect, it } from 'vitest';
import { ageText, spatialHoardingRows, allocationText, cacheSize, cacheSummary, isCheaterOnly, isLarder, labStatus, siteCachesText, winterBands, winterShown } from './minds';
import { CACHE_CHEATER, CACHE_HOARDER, CACHE_LARDER, type Config, type LabView } from './types';

const seasons = (mode: 'global' | 'hemispheres', enabled = true) =>
  ({ seasons: { enabled, winter_divisor: 32, period: 100, mode } }) as unknown as Config;

const lab = (over: Partial<LabView>): LabView => ({
  protocol: 'raby',
  days: 6,
  phase: 'morning',
  day: 1,
  place: 0,
  food: true,
  turn: null,
  turn_at: null,
  compartments: [],
  trays: [],
  ...over,
});

describe('winter', () => {
  it('shows the badge only for a global winter', () => {
    expect(winterShown(null)).toBe(false);
    expect(winterShown({ winter: null, homes: [], lab: null })).toBe(false);
    expect(winterShown({ winter: false, homes: [], lab: null })).toBe(false);
    expect(winterShown({ winter: true, homes: [], lab: null })).toBe(true);
  });

  it('bands the ticks that ended in winter, clipped to the range', () => {
    expect(winterBands(seasons('global'), 0, 450)).toEqual([
      [100, 200],
      [300, 400],
    ]);
    expect(winterBands(seasons('global'), 150, 320)).toEqual([
      [150, 200],
      [300, 320],
    ]);
    expect(winterBands(seasons('global'), 200, 300)).toEqual([]);
  });

  it('has no bands by hemisphere, with seasons off, or over an empty range', () => {
    expect(winterBands(seasons('hemispheres'), 0, 450)).toEqual([]);
    expect(winterBands(seasons('global', false), 0, 450)).toEqual([]);
    expect(winterBands(seasons('global'), 50, 50)).toEqual([]);
  });
});

describe('labStatus', () => {
  it('names the day, the compartment and whether it has food', () => {
    expect(labStatus(lab({}))).toBe('Day 1 of 6 · morning in K1 · food');
    expect(labStatus(lab({ day: 2, place: 2, food: false }))).toBe('Day 2 of 6 · morning in K3 · none');
    expect(labStatus(lab({ phase: 'evening', day: 4 }))).toBe('Day 4 of 6 · evening, on the perches');
    expect(labStatus(lab({ phase: 'start' }))).toBe('Day 1 of 6 · before the first morning');
  });

  it("says whose turn it is on the test evening, and when it's done", () => {
    expect(labStatus(lab({ phase: 'test', day: 7, turn: 3 }))).toBe("Test evening · agent #3's turn");
    expect(labStatus(lab({ phase: 'done', day: 7 }))).toBe('Test evening · done');
  });
});

describe('the flat cache sites', () => {
  it('finds the fullest site and whether any holds only cheaters’ caches, in one pass', () => {
    const flat = Float64Array.of(1, 2, 4, CACHE_HOARDER, 3, 4, 16, CACHE_HOARDER | CACHE_CHEATER, 5, 6, 0, CACHE_HOARDER | CACHE_LARDER);
    expect(cacheSummary(flat)).toEqual({ max: 16, cheaterOnly: false });
    expect(cacheSummary(Float64Array.of(1, 1, 2, CACHE_CHEATER)).cheaterOnly).toBe(true);
    expect(cacheSummary(new Float64Array(0))).toEqual({ max: 0, cheaterOnly: false });
    expect([isCheaterOnly(CACHE_CHEATER), isCheaterOnly(CACHE_CHEATER | CACHE_HOARDER)]).toEqual([true, false]);
    expect([isLarder(CACHE_HOARDER | CACHE_LARDER), isLarder(CACHE_HOARDER)]).toEqual([true, false]);
  });

  it('sizes a site by the square root of its total against the fullest', () => {
    expect(cacheSize(16, 16)).toBeCloseTo(0.3);
    expect(cacheSize(4, 16)).toBeCloseTo(0.12 + 0.18 * 0.5);
    expect(cacheSize(0, 16)).toBeCloseTo(0.12);
    expect(cacheSize(3, 0)).toBeCloseTo(0.12);
  });
});

describe('Inspect texts', () => {
  it("lists a site's caches by owner, marking cheaters only where there are any", () => {
    const caches = [
      { owner: 12, amount: 4.5, cheater_owner: false },
      { owner: 30, amount: 2, cheater_owner: true },
    ];
    expect(siteCachesText(caches, true)).toBe('#12: 4.50 · #30 (cheater): 2');
    expect(siteCachesText(caches, false)).toBe('#12: 4.50 · #30: 2');
    expect(siteCachesText([], true)).toBe('none');
    expect(siteCachesText(caches, false, 1)).toBe('#12: 4.50 · and 1 more');
  });

  it('writes the frozen allocation by compartment', () => {
    expect(allocationText([[0, 15], [2, 15]])).toBe('K1 15 · K3 15');
    expect(allocationText([[0, 0], [1, 30], [2, 0]])).toBe('K2 30');
    expect(allocationText([])).toBe('nothing');
  });

  it('shows the maximum age only where agents die of old age', () => {
    expect(ageText(150, 63, false)).toBe('150');
    expect(ageText(50, 63, true)).toBe('50 / 63');
  });
});

describe('spatial inspection', () => {
  it('discloses carried pending delivery and guard state independently of scatter caches', () => {
    expect(spatialHoardingRows({ home: { x: 2, y: 3 }, larder_trait: 1, defense_trait: 0.5, larder: 8, delivery: 4, guarding: true, observed_larders: 2 })).toEqual([
      ['Home', '(2, 3)'], ['Larder probability L', '1'], ['Defense propensity D', '0.50'], ['Larder', '8 at home'], ['Delivery', '4 pending (still carried)'], ['Guarding', 'yes'], ['Observed larders', '2'],
    ]);
  });
});
