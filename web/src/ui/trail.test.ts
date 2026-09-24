import { describe, expect, it } from 'vitest';
import { trailSegments } from './trail';

describe('trailSegments', () => {
  it('joins consecutive cells, fading in from the oldest', () => {
    expect(trailSegments(new Uint32Array([1, 1, 2, 1, 2, 2]), 10, 10)).toEqual([
      { x1: 1, y1: 1, x2: 2, y2: 1, alpha: 0.5 },
      { x1: 2, y1: 1, x2: 2, y2: 2, alpha: 1 },
    ]);
  });

  it('breaks the line where the agent wraps around the torus', () => {
    const across = trailSegments(new Uint32Array([8, 5, 9, 5, 0, 5, 1, 5]), 10, 10);
    expect(across.map((s) => [s.x1, s.x2])).toEqual([[8, 9], [0, 1]]);
    expect(across.map((s) => s.alpha)).toEqual([1 / 3, 1]);
    expect(trailSegments(new Uint32Array([3, 0, 3, 9]), 10, 10)).toEqual([]);
  });

  it('keeps a step of exactly half the grid', () => {
    expect(trailSegments(new Uint32Array([0, 0, 5, 0]), 10, 10)).toHaveLength(1);
  });

  it('needs two positions', () => {
    expect(trailSegments(new Uint32Array([4, 4]), 10, 10)).toEqual([]);
    expect(trailSegments(new Uint32Array([]), 10, 10)).toEqual([]);
  });
});
