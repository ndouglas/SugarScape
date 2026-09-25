import { describe, expect, it } from 'vitest';
import type { ModelConfig } from './types';
import { linkSegments, PDSI_CLASSES, readoutText, settlementRadius, settlements, waterText } from './valley';

describe('the valley overlays', () => {
  it('reads settlements as triples', () => {
    const state = { water: new Uint32Array(0), links: new Uint32Array(0), settlements: Uint32Array.of(3, 4, 2, 10, 11, 9) };
    expect(settlements(state)).toEqual([
      { x: 3, y: 4, households: 2 },
      { x: 10, y: 11, households: 9 },
    ]);
  });

  it('sizes a settlement by the square root of its households, at most 1.5 cells', () => {
    expect(settlementRadius(1, 10)).toBeCloseTo(5);
    expect(settlementRadius(4, 10)).toBeCloseTo(7);
    expect(settlementRadius(1000, 10)).toBe(15);
  });
});

describe('farm–home links', () => {
  const valley = (wrap_edges: boolean) => ({ model: 'anasazi', quirks: { wrap_edges } }) as ModelConfig;

  it('wrap across the map’s edges only when the world wraps', () => {
    expect(linkSegments(valley(true), 1, 5, 78, 5, 80, 120)).toEqual([
      [1, 5, -2, 5],
      [81, 5, 78, 5],
    ]);
    expect(linkSegments(valley(false), 1, 5, 78, 5, 80, 120)).toEqual([[1, 5, 78, 5]]);
  });

  it('keep a short link as one segment either way', () => {
    for (const wrap of [true, false]) expect(linkSegments(valley(wrap), 3, 4, 5, 6, 80, 120)).toEqual([[3, 4, 5, 6]]);
  });
});

describe('the readout', () => {
  const valley = { model: 'anasazi', start_year: 800 } as ModelConfig;
  const sugar = {} as ModelConfig;

  it('shows the anasazi’s calendar year and households', () => {
    expect(readoutText({ config: valley, tick: 342, population: 213 }, null)).toBe('AD 1142 · 213 households');
    expect(readoutText({ config: valley, tick: 342, population: 213 }, { config: valley, tick: 342, population: 190 })).toBe(
      'AD 1142 · A 213 · B 190 households',
    );
  });

  it('shows other models’ ticks and agents as before', () => {
    expect(readoutText({ config: sugar, tick: 5, population: 40 }, null)).toBe('t = 5 · 40 agents');
    expect(readoutText({ config: sugar, tick: 5, population: 40 }, { config: sugar, tick: 5, population: 38 })).toBe('t = 5 · A 40 · B 38 agents');
  });
});

describe('a valley cell in Inspect', () => {
  it('names its PDSI class and where it stands with water', () => {
    expect(PDSI_CLASSES[4]).toBe('[3, ∞)');
    expect(waterText({ water: true, water_near: true })).toBe('a water source');
    expect(waterText({ water: false, water_near: true })).toBe('within a mile of water');
    expect(waterText({ water: false, water_near: false })).toBe('more than a mile from water');
  });
});
