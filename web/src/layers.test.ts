import { describe, expect, it } from 'vitest';
import type { DisplayState } from './protocol';
import type { Config } from './types';
import { clampDisplay, layerOptions, validLayer } from './layers';

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
  const display: DisplayState = { colorMode: 'disease', layer: 'pollution:0', overlays: { trade: true, credit: false, disease: true } };
  const withDisease = (enabled: boolean) => ({ ...config, disease: { enabled } }) as unknown as Config;

  it('keeps a valid display as the same object', () => {
    expect(clampDisplay(display, withDisease(true))).toBe(display);
  });

  it('drops the disease mode and overlay while disease is off, and a layer the world lacks', () => {
    expect(clampDisplay(display, withDisease(false))).toEqual({
      colorMode: 'tribe',
      layer: 'pollution:0',
      overlays: { trade: true, credit: false, disease: false },
    });
    expect(clampDisplay({ ...display, layer: 'capacity:2' }, withDisease(true)).layer).toBe('resource:0');
  });
});
