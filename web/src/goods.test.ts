import { describe, expect, it } from 'vitest';
import type { Config, Good } from './types';
import {
  clampSeed,
  MAX_GOODS,
  MAX_POLLUTANTS,
  addGood,
  addPollutant,
  defaultMap,
  goodsEditorSignature,
  pollutionEditorSignature,
  removeGood,
  removePollutant,
} from './goods';

const sugar: Good = {
  name: 'sugar',
  color: '#f2c14e',
  map: { kind: 'two_peaks', transform: 'identity' },
  metabolism: { min: 1, max: 4 },
  endowment: { min: 5, max: 25 },
};

function config(): Config {
  return {
    width: 50,
    height: 50,
    goods: [structuredClone(sugar)],
    pollution: { enabled: false, pollutants: [{ name: 'pollution', production: [1], consumption: [1], devalues: [true] }] },
    combat: { enabled: true, unlimited: true, reward: 2 },
    trade: { enabled: false },
    foresight: { enabled: false, range: { min: 0, max: 10 } },
    schedule: [],
  } as unknown as Config;
}

describe('goods', () => {
  it('adds a good with the next transform, color and name, extending every pollutant', () => {
    const c = config();
    addGood(c);
    expect(c.goods[1]).toMatchObject({ name: 'good 2', color: '#e07a3f', map: { kind: 'two_peaks', transform: 'mirror_x' } });
    expect(c.pollution.pollutants[0]).toEqual({ name: 'pollution', production: [1, 0], consumption: [1, 0], devalues: [true, false] });
    expect(c.combat.enabled).toBe(false);
  });

  it('stops at eight goods with distinct names', () => {
    const c = config();
    for (let i = 0; i < 10; i++) addGood(c);
    expect(c.goods).toHaveLength(MAX_GOODS);
    expect(new Set(c.goods.map((g) => g.name)).size).toBe(MAX_GOODS);
  });

  it('removes a good and its pollutant columns, switching two-good rules off at one good', () => {
    const c = config();
    addGood(c);
    c.trade.enabled = true;
    c.foresight.enabled = true;
    removeGood(c, 0);
    expect(c.goods.map((g) => g.name)).toEqual(['good 2']);
    expect(c.pollution.pollutants[0].production).toEqual([0]);
    expect(c.trade.enabled || c.foresight.enabled).toBe(false);
    removeGood(c, 0);
    expect(c.goods).toHaveLength(1);
  });

  it('keeps one to four pollutants', () => {
    const c = config();
    addGood(c);
    for (let i = 0; i < 5; i++) addPollutant(c);
    expect(c.pollution.pollutants).toHaveLength(MAX_POLLUTANTS);
    expect(c.pollution.pollutants[1]).toEqual({ name: 'pollutant 2', production: [0, 0], consumption: [0, 0], devalues: [false, false] });
    for (let i = 0; i < 5; i++) removePollutant(c, 0);
    expect(c.pollution.pollutants).toHaveLength(1);
  });

  it('uses flat maps off the 50×50 grid and a central peak for peaks', () => {
    const c = config();
    c.width = 40;
    expect(defaultMap(c)).toEqual({ kind: 'flat', capacity: 2 });
    expect(defaultMap(c, 'peaks')).toEqual({ kind: 'peaks', peaks: [{ x: 20, y: 25, radius: 10, height: 4 }] });
  });

  it('builds a noise map with the given seed, in the core key order', () => {
    const c = config();
    expect(defaultMap(c, 'noise', 42)).toEqual({ kind: 'noise', seed: 42, scale: 8, octaves: 3, height: 4 });
    expect(JSON.stringify(defaultMap(c, 'noise'))).toBe('{"kind":"noise","seed":1,"scale":8,"octaves":3,"height":4}');
  });
});

describe('schedule retargeting', () => {
  const at = (tick: number, ...paths: string[]) => ({ tick, set: Object.fromEntries(paths.map((p) => [p, 0])) });
  const paths = (c: Config) => c.schedule.map((s) => [s.tick, Object.keys(s.set)]);

  it('drops and shifts scheduled paths when a good is removed or added', () => {
    const c = config();
    addGood(c);
    addGood(c);
    c.schedule = [
      at(5, 'goods.0.name', 'goods.1.name'),
      at(6, 'goods.1.metabolism.max'),
      at(7, 'goods.2.endowment'),
      at(8, 'pollution.pollutants.0.production.1'),
      at(9, 'pollution.pollutants.0.devalues.2'),
      at(10, 'pollution.pollutants.0.consumption'),
      at(11, 'pollution.pollutants.0.name'),
      at(12, 'pollution.pollutants.0'),
      at(13, 'trade.enabled'),
    ];
    removeGood(c, 1);
    expect(paths(c)).toEqual([
      [5, ['goods.0.name']],
      [7, ['goods.1.endowment']],
      [9, ['pollution.pollutants.0.devalues.1']],
      [11, ['pollution.pollutants.0.name']],
      [13, ['trade.enabled']],
    ]);
    c.schedule.push(at(14, 'pollution.pollutants.0.production'), at(15, 'pollution.pollutants.0.production.1'));
    addGood(c);
    expect(c.schedule.map((s) => s.tick)).toEqual([5, 7, 9, 11, 13, 15]);
  });

  it('drops and shifts scheduled paths when a pollutant is removed', () => {
    const c = config();
    addPollutant(c);
    addPollutant(c);
    c.schedule = [
      at(5, 'pollution.pollutants.0.name'),
      at(6, 'pollution.pollutants.1.production.0'),
      at(7, 'pollution.pollutants.2.devalues'),
      at(8, 'pollution.enabled'),
    ];
    removePollutant(c, 1);
    expect(paths(c)).toEqual([
      [5, ['pollution.pollutants.0.name']],
      [7, ['pollution.pollutants.1.devalues']],
      [8, ['pollution.enabled']],
    ]);
  });
});

describe('structure signatures', () => {
  it('change with structure, not with values', () => {
    const c = config();
    const all = () => [goodsEditorSignature(c), pollutionEditorSignature(c)];
    const before = all();
    c.goods[0].metabolism.max = 9;
    c.goods[0].endowment.min = 1;
    c.pollution.pollutants[0].production[0] = 3;
    c.pollution.pollutants[0].devalues[0] = false;
    expect(all()).toEqual(before);

    c.goods[0].name = 'honey';
    expect(goodsEditorSignature(c)).toBe(before[0]);
    expect(pollutionEditorSignature(c)).not.toBe(before[1]);

    const named = all();
    c.pollution.pollutants[0].name = 'smog';
    expect(all()).toEqual(named);

    const g = goodsEditorSignature(c);
    c.goods[0].map = { kind: 'peaks', peaks: [{ x: 1, y: 1, radius: 1, height: 1 }] };
    const one = goodsEditorSignature(c);
    expect(one).not.toBe(g);
    c.goods[0].map.peaks[0].x = 7;
    expect(goodsEditorSignature(c)).toBe(one);
    c.goods[0].map.peaks.push({ x: 2, y: 2, radius: 1, height: 1 });
    expect(goodsEditorSignature(c)).not.toBe(one);
    c.width = 60;
    const wide = goodsEditorSignature(c);
    addGood(c);
    expect(goodsEditorSignature(c)).not.toBe(wide);
  });
});

describe('clampSeed', () => {
  it('clamps to the u32 range instead of wrapping', () => {
    expect(clampSeed(-1)).toBe(0);
    expect(clampSeed(4294967296)).toBe(4294967295);
    expect(clampSeed(1e12)).toBe(4294967295);
    expect(clampSeed(42)).toBe(42);
    expect(clampSeed(7.8)).toBe(7);
    expect(clampSeed(Number.NaN)).toBe(0);
  });
});
