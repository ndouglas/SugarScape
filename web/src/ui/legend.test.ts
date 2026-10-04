import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { COLOR_MODES } from '../models';
import type { Config, LabView } from '../types';
import { colorLegend, HOARD_COLORS, hoardLegend, overlayLegend, RENDER_COLORS, type MapMarks } from './legend';

const config = {
  culture: { groups: [{ name: 'Blue', color: '#3d7eff' }, { name: 'Red', color: '#ff4d4d' }] },
  vision: { min: 1, max: 6 },
  walls: [],
} as unknown as Config;

const none: MapMarks = { allCaches: false, cheaterCaches: false, ownCaches: false, homes: false, larders: false, memory: false, spots: false, path: false, route: false, lab: null };

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

  it('lists the strategies present and the caching rules agents follow', () => {
    const labels = (mode: Parameters<typeof colorLegend>[0], c: Config) => colorLegend(mode, c).map((i) => i.label);
    const caching = (rule: string, mixed: boolean) => ({ rule, capacity: 50, share: 0.5, lambda: 0.5, lookahead: 1, mixed });
    const theft = (cheaters: number) => ({ find: 0.25, owner_memory: true, loot: 'keep', cheaters });
    const c = (over: object) => ({ ...config, ...over }) as Config;
    expect(labels('strategy', c({ theft: theft(0.5) }))).toEqual(['hoarder', 'cheater']);
    expect(labels('strategy', c({ theft: theft(0) }))).toEqual(['hoarder']);
    expect(labels('caching_rule', c({ caching: caching('none', true) }))).toEqual(['none', 'even', 'compensate', 'plan']);
    expect(labels('caching_rule', c({ caching: caching('even', false) }))).toEqual(['even']);
    expect(labels('caching_rule', c({ caching: caching('even', false), theft: theft(0.5) }))).toEqual(['none (cheaters)', 'even']);
    expect(labels('caching_rule', c({ caching: caching('none', true), theft: theft(0.5) }))[0]).toBe('none (and cheaters)');
    expect(labels('watching', config)).toEqual(['watcher who buries', 'scrounger (watches, never buries)', 'does not watch']);
    expect(labels('memory', config)).toEqual(['remembers', "doesn't remember"]);
    expect(labels('tribe', config)).toEqual(['Blue', 'Red']);
  });

  it('lists only the overlay symbols on screen', () => {
    expect(overlayLegend(none, config)).toEqual([]);
    const labels = (m: Partial<MapMarks>, c = config) => overlayLegend({ ...none, ...m }, c).map((i) => i.label);
    expect(labels({ allCaches: true })).toEqual(['a cache (size: sugar)']);
    expect(labels({ allCaches: true, cheaterCaches: true })).toEqual(["a hoarder's cache (size: sugar)", "a cheater's cache"]);
    expect(labels({ homes: true, ownCaches: true })).toEqual(["the selected agent's caches", 'home']);
    expect(labels({ homes: true, larders: true })).toEqual(['home', 'larder (size: sugar)']);
    expect(labels({ memory: true })).toEqual(['remembered site (fades with age)']);
    expect(labels({ memory: true, spots: true, path: true, route: true })).toEqual([
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

describe('the hoard legend (Minds 7)', () => {
  it("uses the hoard frame's own colors", () => {
    const source = new TextDecoder().decode(readFileSync(new URL('../../../crates/sugarscape-core/src/hoard/view.rs', import.meta.url)));
    for (const [name, color] of Object.entries(HOARD_COLORS)) {
      const m = new RegExp(`pub const ${name}: Rgb = \\[0x(..), 0x(..), 0x(..)\\];`).exec(source);
      expect(m ? `#${m[1]}${m[2]}${m[3]}` : null, name).toBe(color);
    }
  });

  it('explains a column: the state colors, the L and D strips, larder up and scatter down', () => {
    const labels = (cheaters: boolean) => hoardLegend(cheaters).map((i) => i.label);
    expect(labels(false)).toEqual(['hungry', 'fed', 'defending its larder', 'raiding a larder', 'dead', 'L and D strips, 0 → 1', 'larder items (up)', 'scattered items (down)']);
    expect(labels(true)).toContain('cheater');
    expect(hoardLegend(false).find((i) => i.label.startsWith('L and D'))?.mark).toEqual({ kind: 'ramp', from: HOARD_COLORS.LOW, to: HOARD_COLORS.HIGH });
  });
});

it('labels a guarded spatial home with its paid-action marker', () => {
  expect(overlayLegend({ ...none, homes: true, guarding: true }, config).map(i => i.label)).toContain('guarding (paid action)');
});
