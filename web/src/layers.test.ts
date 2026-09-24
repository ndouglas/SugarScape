import { describe, expect, it } from 'vitest';
import { noOverlays, type DisplayState } from './protocol';
import type { Config } from './types';
import { clampDisplay, layerOptions, overlayAvailable, validLayer } from './layers';

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
});
