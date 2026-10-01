import { describe, expect, it } from 'vitest';
import { noOverlays, type DisplayState } from './protocol';
import type { AnasaziConfig, Config, RingConfig, SchellingConfig } from './types';
import { clampDisplay, defaultColorMode, layerOptions, loadedDisplay, overlayAvailable, overlayAvailableAny, validLayer } from './layers';

const config = {
  goods: [{ name: 'sugar' }, { name: 'salt' }],
  pollution: { enabled: true, pollutants: [{ name: 'smoke' }] },
} as unknown as Config;

describe('layers', () => {
  it('lists each good and its capacity by name, then each pollutant', () => {
    expect(layerOptions(config)).toEqual([
      ['resource:0', 'sugar'],
      ['capacity:0', 'sugar capacity'],
      ['resource:1', 'salt'],
      ['capacity:1', 'salt capacity'],
      ['pollution:0', 'smoke'],
    ]);
  });

  it('falls back to good 0 for a layer the world lacks', () => {
    expect(validLayer('capacity:1', config)).toBe('capacity:1');
    expect(validLayer('resource:2', config)).toBe('resource:0');
    expect(validLayer('pollution:1', config)).toBe('resource:0');
  });
});

describe('clampDisplay', () => {
  const rules = (on: boolean) =>
    ({ ...config, disease: { enabled: on }, culture: { enabled: on }, sex: { enabled: on } }) as unknown as Config;
  const display: DisplayState = {
    colorMode: 'disease',
    layer: 'pollution:0',
    overlays: { ...noOverlays(), trade: true, disease: true, neighbors: true, friends: true, family: true },
  };

  it('keeps a valid display as the same object', () => {
    expect(clampDisplay(display, rules(true))).toBe(display);
  });

  it('drops the disease mode and every overlay the world cannot show, and a layer the world lacks', () => {
    expect(clampDisplay(display, rules(false))).toEqual({
      colorMode: 'tribe',
      layer: 'pollution:0',
      overlays: { ...noOverlays(), trade: true, neighbors: true },
    });
    expect(clampDisplay({ ...display, layer: 'capacity:2' }, rules(true)).layer).toBe('resource:0');
  });

  it('clamps to the model: Schelling offers its own modes and no overlays; a sugarscape falls back to Tribe', () => {
    const schelling = { model: 'schelling' } as SchellingConfig;
    const clamped = clampDisplay(display, schelling);
    expect(clamped).toEqual({ colorMode: 'color', layer: 'pollution:0', overlays: noOverlays() });
    const satisfaction: DisplayState = { ...clamped, colorMode: 'satisfaction' };
    expect(clampDisplay(satisfaction, schelling)).toBe(satisfaction);
    expect(clampDisplay(satisfaction, rules(true)).colorMode).toBe('tribe');
    // Ring World draws no color modes: the mode is kept for the next sugarscape (and clamped there).
    const ring = { model: 'ring' } as RingConfig;
    expect(clampDisplay(display, ring)).toEqual({ ...display, overlays: noOverlays() });
  });

  it('keeps the Lineage color mode in any world', () => {
    expect(clampDisplay({ ...display, colorMode: 'lineage' }, rules(false)).colorMode).toBe('lineage');
  });

  it('offers the neighbor network always, friends with culture and family with sex', () => {
    const only = (culture: boolean, sex: boolean) =>
      ({ ...config, disease: { enabled: false }, culture: { enabled: culture }, sex: { enabled: sex } }) as unknown as Config;
    expect(overlayAvailable('neighbors', only(false, false))).toBe(true);
    expect(overlayAvailable('friends', only(false, true))).toBe(false);
    expect(overlayAvailable('friends', only(true, false))).toBe(true);
    expect(overlayAvailable('family', only(true, false))).toBe(false);
    expect(overlayAvailable('family', only(false, true))).toBe(true);
    expect(overlayAvailable('disease', only(true, true))).toBe(false);
  });

  it('offers a Compare checkbox when either world would show it, and hides it only when neither would', () => {
    const only = (culture: boolean, sex: boolean) =>
      ({ ...config, disease: { enabled: false }, culture: { enabled: culture }, sex: { enabled: sex } }) as unknown as Config;
    const a = only(true, false);
    const b = only(false, true);
    expect(overlayAvailableAny('friends', [a])).toBe(true);
    expect(overlayAvailableAny('friends', [b])).toBe(false);
    expect(overlayAvailableAny('friends', [a, b])).toBe(true);
    expect(overlayAvailableAny('family', [a, b])).toBe(true);
    expect(overlayAvailableAny('family', [a])).toBe(false);
    expect(overlayAvailableAny('disease', [only(true, true), only(true, true)])).toBe(false);
  });

  it('keeps the valley’s overlays in the anasazi and turns off everything else; a sugarscape turns them off', () => {
    const valley = { model: 'anasazi' } as AnasaziConfig;
    const on: DisplayState = { ...display, overlays: { ...display.overlays, water: true, links: true } };
    const clamped = clampDisplay(on, valley);
    expect(clamped).toEqual({ colorMode: 'occupation', layer: 'pollution:0', overlays: { ...noOverlays(), water: true, links: true } });
    expect(clampDisplay(clamped, valley)).toBe(clamped);
    expect(overlayAvailable('water', rules(true))).toBe(false);
    const back = clampDisplay({ ...clamped, colorMode: 'tribe' }, rules(true));
    expect([back.overlays.water, back.overlays.links]).toEqual([false, false]);
  });
});

describe('the Minds color modes and the caches overlay', () => {
  const minds = (over: Partial<Config>) =>
    ({ ...config, disease: { enabled: false }, culture: { enabled: false }, sex: { enabled: false }, ...over }) as unknown as Config;
  const theft = minds({ caching: { rule: 'even', capacity: 50, share: 0.5, lambda: 0.5, lookahead: 1, mixed: false }, theft: { find: 0.25, owner_memory: true, loot: 'keep', cheaters: 0.5 } });
  const mixed = minds({ caching: { rule: 'none', capacity: 50, share: 0.5, lambda: 0.5, lookahead: 1, mixed: true } });
  const even = minds({ caching: { rule: 'even', capacity: 50, share: 0.5, lambda: 0.5, lookahead: 1, mixed: false } });
  const central = minds({ central: { enabled: true } });
  const book = minds({});
  const memory = (share: number) => ({ span: 100, share, belief: 'last_seen', prior: 'none' });
  const halfRemember = minds({ ...even, memory: memory(0.5) } as Partial<Config>);
  const theftNoCheaters = minds({ ...theft, theft: { find: 0.25, owner_memory: true, loot: 'keep', cheaters: 0 }, memory: memory(0.5) } as Partial<Config>);
  const allRemember = minds({ memory: memory(1) } as Partial<Config>);
  const base: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() };

  it('defaults to Watching where watching is on and some but not all watch, or some are cheaters', () => {
    const watch = { on: true, span: 7, watchers: 1, raid_when: 'always' };
    const some = { ...watch, watchers: 0.5 };
    expect(defaultColorMode({ ...even, watching: some } as unknown as Config)).toBe('watching');
    expect(defaultColorMode({ ...theft, watching: watch } as unknown as Config)).toBe('watching');
    expect(defaultColorMode({ ...theft, watching: some } as unknown as Config)).toBe('watching');
    // Everyone watching and no cheaters: nothing to tell apart, so the earlier defaults.
    expect(defaultColorMode({ ...even, watching: watch } as unknown as Config)).toBe('caching_rule');
    expect(defaultColorMode({ ...even, watching: { ...watch, watchers: 0 } } as unknown as Config)).toBe('caching_rule');
    expect(defaultColorMode({ ...halfRemember, watching: watch } as unknown as Config)).toBe('memory');
    expect(defaultColorMode({ ...even, watching: { ...some, on: false } } as unknown as Config)).toBe('caching_rule');
    const d = { ...base, colorMode: 'watching' as const };
    expect(clampDisplay(d, { ...even, watching: watch } as unknown as Config).colorMode).toBe('watching');
    expect(clampDisplay(d, even).colorMode).toBe('caching_rule');
    expect(loadedDisplay(d, { ...even, watching: watch } as unknown as Config).colorMode).toBe('caching_rule');
  });

  it('defaults to Strategy with cheaters, Caching rule under mixed rules, Memory where some remember, Caching rule with caching on, else Tribe', () => {
    expect(defaultColorMode(theft)).toBe('strategy');
    expect(defaultColorMode(mixed)).toBe('caching_rule');
    expect(defaultColorMode(halfRemember)).toBe('memory');
    expect(defaultColorMode(theftNoCheaters)).toBe('memory');
    // A carrying limit alone (central-place worlds, rule none) has no rule to show.
    const limitOnly = { ...theftNoCheaters, theft: undefined, memory: undefined, caching: { ...(theftNoCheaters.caching ?? {}), rule: 'none', mixed: false, capacity: 320 } } as unknown as Config;
    expect(defaultColorMode(limitOnly)).toBe('tribe');
    expect(defaultColorMode(even)).toBe('caching_rule');
    expect(defaultColorMode(allRemember)).toBe('tribe');
    expect(defaultColorMode(book)).toBe('tribe');
  });

  it('keeps Strategy only with theft on and Caching rule only with caching on', () => {
    expect(clampDisplay({ ...base, colorMode: 'strategy' }, theft).colorMode).toBe('strategy');
    expect(clampDisplay({ ...base, colorMode: 'strategy' }, mixed).colorMode).toBe('caching_rule');
    expect(clampDisplay({ ...base, colorMode: 'strategy' }, book).colorMode).toBe('tribe');
    expect(clampDisplay({ ...base, colorMode: 'caching_rule' }, even).colorMode).toBe('caching_rule');
    expect(clampDisplay({ ...base, colorMode: 'caching_rule' }, book).colorMode).toBe('tribe');
    expect(clampDisplay({ ...base, colorMode: 'strategy' }, theftNoCheaters).colorMode).toBe('memory');
    expect(clampDisplay({ ...base, colorMode: 'memory' }, allRemember).colorMode).toBe('memory');
    expect(clampDisplay({ ...base, colorMode: 'memory' }, book).colorMode).toBe('tribe');
  });

  it('offers the caches overlay where there can be caches (a central world’s larders too)', () => {
    expect(overlayAvailable('caches', even)).toBe(true);
    expect(overlayAvailable('caches', central)).toBe(true);
    expect(overlayAvailable('caches', book)).toBe(false);
  });

  it('starts a loaded world on its default mode with the caches overlay, keeping a mode picked by hand', () => {
    expect(loadedDisplay(base, theft)).toEqual({ ...base, colorMode: 'strategy', overlays: { ...noOverlays(), caches: true } });
    expect(loadedDisplay({ ...base, colorMode: 'strategy' }, mixed).colorMode).toBe('caching_rule');
    expect(loadedDisplay({ ...base, colorMode: 'wealth' }, theft).colorMode).toBe('wealth');
    const back = loadedDisplay({ ...base, colorMode: 'caching_rule', overlays: { ...noOverlays(), caches: true } }, book);
    expect(back).toEqual(base);
    expect(loadedDisplay(base, book)).toBe(base);
    expect(loadedDisplay({ ...base, colorMode: 'memory' }, theft).colorMode).toBe('strategy');
    // A caches overlay turned off by hand stays off.
    expect(loadedDisplay(base, theft, true)).toEqual({ ...base, colorMode: 'strategy' });
    const schelling = { model: 'schelling' } as SchellingConfig;
    expect(loadedDisplay(base, schelling)).toBe(base);
  });
});
