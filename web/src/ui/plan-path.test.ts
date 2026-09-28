import { describe, expect, it } from 'vitest';
import { planSegments } from './plan-path';

describe('planSegments', () => {
  it('joins the agent through consecutive path cells', () => {
    expect(planSegments([0, 0], [[1, 0], [2, 0], [3, 0]], 50, 50)).toEqual([
      [0, 0, 1, 0],
      [1, 0, 2, 0],
      [2, 0, 3, 0],
    ]);
  });

  it('leaves out a segment that crosses the torus seam', () => {
    expect(planSegments([49, 0], [[0, 0]], 50, 50)).toEqual([]);
  });

  it('keeps the rest of the path when only one step wraps', () => {
    expect(planSegments([48, 0], [[49, 0], [0, 0], [1, 0]], 50, 50)).toEqual([[48, 0, 49, 0], [0, 0, 1, 0]]);
  });

  it('gives no segments for an empty path', () => {
    expect(planSegments([5, 5], [], 50, 50)).toEqual([]);
  });
});
