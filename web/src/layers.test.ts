import { describe, expect, it } from 'vitest';
import type { Config } from './types';
import { layerOptions, validLayer } from './layers';

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
