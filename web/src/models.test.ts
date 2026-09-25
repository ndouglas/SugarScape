import { describe, expect, it } from 'vitest';
import {
  calendarYear,
  COLOR_MODES,
  finishesUnpredictably,
  isCivilView,
  isRingView,
  isSpatialView,
  isSugar,
  isSugarView,
  isValleyView,
  MODEL_OVERLAYS,
  modelOf,
  presetGroups,
  ticksLeft,
} from './models';
import type { AnyInspection, Config, ModelConfig, Preset } from './types';

describe('modelOf', () => {
  it('reads a config without a model key (every config before milestone 9) as a sugarscape', () => {
    const old = { width: 50, goods: [] } as unknown as Config;
    expect(modelOf(old)).toBe('sugarscape');
    expect(isSugar(old)).toBe(true);
    expect(modelOf({ ...old, model: 'sugarscape' } as unknown as ModelConfig)).toBe('sugarscape');
  });

  it('reads the other models by their tag', () => {
    expect(modelOf({ model: 'schelling' } as ModelConfig)).toBe('schelling');
    expect(modelOf({ model: 'ring' } as ModelConfig)).toBe('ring');
    expect(isSugar({ model: 'ring' } as ModelConfig)).toBe(false);
  });

  it('tells a sugarscape inspection by its resources and Ring World’s by its sugar', () => {
    const sugar = { site: { x: 0, y: 0, resources: [1], capacities: [4], pollution: [] }, agent: null } as AnyInspection;
    const ring = { site: { x: 3, sugar: 2, capacity: 4 }, agent: null } as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect(isSugarView(sugar)).toBe(true);
    expect(isSugarView(ring)).toBe(false);
    expect([sugar, ring, schelling].map(isRingView)).toEqual([false, true, false]);
  });

  it('offers each model its color modes, Schelling first Color, Ring World none', () => {
    expect(COLOR_MODES.sugarscape[0][0]).toBe('tribe');
    expect(COLOR_MODES.schelling.map(([m]) => m)).toEqual(['color', 'satisfaction', 'preference']);
    expect(COLOR_MODES.ring).toEqual([]);
  });
});

describe('presetGroups', () => {
  it('groups the presets menu by model in a fixed order, leaving out models without presets', () => {
    const p = (id: string, config: unknown): Preset => ({ id, name: id, source: '', description: '', config: config as ModelConfig });
    const presets = [p('ring-1', { model: 'ring' }), p('ii-2', {}), p('ii-3', {}), p('ring-2', { model: 'ring' })];
    expect(presetGroups(presets).map((g) => [g.label, g.presets.map((x) => x.id)])).toEqual([
      ['Sugarscape', ['ii-2', 'ii-3']],
      ['Ring World', ['ring-1', 'ring-2']],
    ]);
  });
});

describe('the anasazi model', () => {
  const valley = { model: 'anasazi', start_year: 800, end_year: 1350 } as ModelConfig;

  it('is read by its tag, and its inspections by their zone', () => {
    expect(modelOf(valley)).toBe('anasazi');
    expect(isSugar(valley)).toBe(false);
    const cell = { site: { x: 1, y: 2, zone: 'north', zone_name: 'North Valley Floor' }, agent: null } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([cell, schelling].map(isValleyView)).toEqual([true, false]);
    expect(isRingView(cell) || isSugarView(cell)).toBe(false);
  });

  it('offers Occupation first, then Zones and Yield, and the valley’s three overlays', () => {
    expect(COLOR_MODES.anasazi).toEqual([
      ['occupation', 'Occupation'],
      ['zones', 'Zones'],
      ['yield', 'Yield'],
    ]);
    expect(MODEL_OVERLAYS.anasazi).toEqual(['water', 'settlements', 'links']);
    expect(MODEL_OVERLAYS.sugarscape).not.toContain('water');
    expect(MODEL_OVERLAYS.ring).toEqual([]);
  });

  it('counts years from its start year and ticks until its end year', () => {
    expect(calendarYear(valley, 342)).toBe(1142);
    expect(ticksLeft(valley, 540)).toBe(10);
    expect(ticksLeft(valley, 550)).toBe(0);
    const ring = { model: 'ring' } as ModelConfig;
    expect(calendarYear(ring, 5)).toBeNull();
    expect(ticksLeft(ring, 5)).toBe(Infinity);
  });

  it('groups its presets under Artificial Anasazi, last', () => {
    const p = (id: string, config: unknown): Preset => ({ id, name: id, source: '', description: '', config: config as ModelConfig });
    const groups = presetGroups([p('lhv', valley), p('ii-2', {}), p('vi-8', { model: 'ring' })]);
    expect(groups.map((g) => g.label)).toEqual(['Sugarscape', 'Ring World', 'Artificial Anasazi']);
  });
});

describe('the civil model', () => {
  it('is read by its tag, and its inspections by their jailed list', () => {
    const civil = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect(modelOf(civil)).toBe('civil');
    expect(isSugar(civil)).toBe(false);
    const site = { site: { x: 1, y: 2 }, agent: null, cop: null, jailed: [] } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([site, schelling].map(isCivilView)).toEqual([true, false]);
    expect(isRingView(site) || isSugarView(site) || isValleyView(site)).toBe(false);
  });

  it('offers the paper’s two screens and its groups, and no overlays', () => {
    expect(COLOR_MODES.civil).toEqual([
      ['action', 'Action'],
      ['grievance', 'Grievance'],
      ['group', 'Group'],
    ]);
    expect(MODEL_OVERLAYS.civil).toEqual([]);
    expect(ticksLeft({ model: 'civil' } as unknown as ModelConfig, 5)).toBe(Infinity);
    expect(calendarYear({ model: 'civil' } as unknown as ModelConfig, 5)).toBeNull();
  });

  it('can finish unpredictably only as Model II stopping at extinction', () => {
    const civil = (variant: string, stop_at_extinction: boolean) => ({ model: 'civil', variant, stop_at_extinction }) as unknown as ModelConfig;
    expect(finishesUnpredictably(civil('ethnic', true))).toBe(true);
    expect(finishesUnpredictably(civil('ethnic', false))).toBe(false);
    expect(finishesUnpredictably(civil('rebellion', true))).toBe(false);
    const valley = { model: 'anasazi', start_year: 800, end_year: 1350 } as unknown as ModelConfig;
    expect(finishesUnpredictably(valley)).toBe(false);
  });
});

describe('the spatial model', () => {
  it('is read by its tag, and its inspections by their z', () => {
    const c = { model: 'spatial', lattice: 'square' } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('spatial');
    const cell = { site: { x: 1, y: 2, z: 0 }, agent: null } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([cell, schelling].map(isSpatialView)).toEqual([true, false]);
  });

  it('offers the papers’ four-color change view first', () => {
    expect(COLOR_MODES.spatial).toEqual([
      ['change', 'Change'],
      ['strategy', 'Strategy'],
      ['payoff', 'Payoff'],
    ]);
    expect(MODEL_OVERLAYS.spatial).toEqual([]);
  });
});
