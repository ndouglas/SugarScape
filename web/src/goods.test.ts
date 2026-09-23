import { describe, expect, it } from 'vitest';
import type { Config, Good } from './types';
import { MAX_GOODS, MAX_POLLUTANTS, addGood, addPollutant, defaultMap, removeGood, removePollutant } from './goods';

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
});
