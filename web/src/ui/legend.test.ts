import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { COLOR_MODES } from '../models';
import type { Config, LabView } from '../types';
import { colorLegend, overlayLegend, RENDER_COLORS, type MapMarks } from './legend';

const config = {
  culture: { groups: [{ name: 'Blue', color: '#3d7eff' }, { name: 'Red', color: '#ff4d4d' }] },
  vision: { min: 1, max: 6 },
  walls: [],
} as unknown as Config;

const none: MapMarks = { allCaches: false, cheaters: false, ownCaches: false, homes: false, memory: false, path: false, route: false, lab: null };

describe('the legend', () => {
  it("uses the core renderer's own colors", () => {
    const source = new TextDecoder().decode(readFileSync(new URL('../../../crates/sugarscape-core/src/render.rs', import.meta.url)));
    const hex = (name: string) => {
      const m = new RegExp(`pub const ${name}: Rgb = \\[0x(..), 0x(..), 0x(..)\\];`).exec(source);
      return m ? `#${m[1]}${m[2]}${m[3]}` : null;
    };
    for (const [name, color] of Object.entries(RENDER_COLORS)) expect(hex(name), name).toBe(color);
  });

  it('has entries for every sugarscape color mode', () => {
    for (const [mode] of COLOR_MODES.sugarscape) expect(colorLegend(mode, config).length, mode).toBeGreaterThan(0);
  });

  it('lists the strategies and the four caching rules', () => {
    expect(colorLegend('strategy', config).map((i) => i.label)).toEqual(['hoarder', 'cheater']);
    expect(colorLegend('caching_rule', config).map((i) => i.label)).toEqual(['none', 'even', 'compensate', 'plan']);
    const theft = { ...config, theft: { find: 0.25, owner_memory: true, loot: 'keep', cheaters: 0.5 } } as Config;
    expect(colorLegend('caching_rule', theft)[0].label).toBe('none (and cheaters)');
    expect(colorLegend('tribe', config).map((i) => i.label)).toEqual(['Blue', 'Red']);
  });

  it('lists only the overlay symbols on screen', () => {
    expect(overlayLegend(none, config)).toEqual([]);
    const labels = (m: Partial<MapMarks>, c = config) => overlayLegend({ ...none, ...m }, c).map((i) => i.label);
    expect(labels({ allCaches: true })).toEqual(['a cache (size: sugar)']);
    expect(labels({ allCaches: true, cheaters: true })).toEqual(["a hoarder's cache (size: sugar)", "a cheater's cache"]);
    expect(labels({ homes: true, ownCaches: true })).toEqual(["the selected agent's caches", 'home', 'larder (size: sugar)']);
    expect(labels({ memory: true, path: true, route: true })).toEqual([
      'remembered site (fades with age)',
      'known truffle spot (filled: ripe)',
      'planned walk',
      'GOAP route and targets',
    ]);
    const lab = { phase: 'test' } as LabView;
    expect(labels({ lab })).toEqual(['caching tray', 'whose turn it is']);
    expect(labels({ lab: { ...lab, phase: 'morning' } })).toEqual(['caching tray']);
    const walled = { ...config, walls: [{ x: 0, y: 0, width: 1, height: 1, opaque: true }] } as unknown as Config;
    expect(labels({}, walled)).toEqual(['wall']);
  });
});
