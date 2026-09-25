import { describe, expect, it } from 'vitest';
import { siteAngle, siteAt, sugarShade } from './ring';

describe('the ring view geometry', () => {
  it('puts site 0 at the top and runs counterclockwise', () => {
    expect(siteAngle(0, 4)).toBeCloseTo(-Math.PI / 2);
    // A quarter of the way round counterclockwise from the top is the left (canvas y down).
    const a = siteAngle(1, 4);
    expect(Math.cos(a)).toBeCloseTo(-1);
    expect(Math.sin(a)).toBeCloseTo(0);
  });

  it('finds the site under a point, whatever its angle', () => {
    for (const sites of [10, 150, 1000]) {
      for (let i = 0; i < sites; i += 7) {
        const a = siteAngle(i, sites);
        expect(siteAt(Math.cos(a) * 50, Math.sin(a) * 50, sites)).toBe(i);
      }
    }
    expect(siteAt(0, -1, 150)).toBe(0);
  });

  it('shades sugar from the background to sugar yellow', () => {
    expect(sugarShade(0, 4)).toBe('rgb(22, 21, 18)');
    expect(sugarShade(4, 4)).toBe('rgb(242, 193, 78)');
    expect(sugarShade(9, 4)).toBe('rgb(242, 193, 78)');
  });
});
