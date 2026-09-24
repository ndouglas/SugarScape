import { describe, expect, it } from 'vitest';
import { capacitiesFromPixels } from './image';

const px = (...rgba: number[][]) => new Uint8ClampedArray(rgba.flat());
const caps = (pixels: Uint8ClampedArray, max: number, invert = false) => Array.from(capacitiesFromPixels(pixels, max, invert));

describe('capacitiesFromPixels', () => {
  it('maps luminance onto 0..max', () => {
    expect(caps(px([255, 255, 255, 255], [0, 0, 0, 255], [128, 128, 128, 255]), 4)).toEqual([4, 0, 2]);
  });

  it('weights the channels by Rec. 709 luma', () => {
    // Y = 182.4, 54.2 and 18.4: 7.15, 2.13 and 0.72 of 10.
    expect(caps(px([0, 255, 0, 255], [255, 0, 0, 255], [0, 0, 255, 255]), 10)).toEqual([7, 2, 1]);
  });

  it('inverts', () => {
    expect(caps(px([255, 255, 255, 255], [0, 0, 0, 255]), 4, true)).toEqual([0, 4]);
  });

  it('counts mostly transparent pixels as 0, even inverted', () => {
    expect(caps(px([255, 255, 255, 127], [255, 255, 255, 128]), 4)).toEqual([0, 4]);
    expect(caps(px([255, 255, 255, 127]), 4, true)).toEqual([0]);
  });

  it('rounds to the nearest unit', () => {
    // 96/255 · 4 = 1.506 and 95/255 · 4 = 1.490.
    expect(caps(px([96, 96, 96, 255], [95, 95, 95, 255]), 4)).toEqual([2, 1]);
    expect(caps(px([255, 255, 255, 255]), 0)).toEqual([0]);
  });
});
