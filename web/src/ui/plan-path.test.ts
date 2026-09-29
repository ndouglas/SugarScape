import { describe, expect, it } from 'vitest';
import { planSegments, routeSegments } from './plan-path';

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

describe('routeSegments', () => {
  it('joins the agent through its targets, legs that stay on the grid whole', () => {
    expect(routeSegments([0, 0], [[5, 0], [5, 7]], 60, 60)).toEqual([
      [0, 0, 5, 0],
      [5, 0, 5, 7],
    ]);
  });

  it('draws a leg across the seam as two pieces, not zero', () => {
    expect(routeSegments([58, 10], [[2, 10]], 60, 60)).toEqual([
      [58, 10, 62, 10],
      [-2, 10, 2, 10],
    ]);
  });

  it('wraps the other way and in y', () => {
    expect(routeSegments([1, 3], [[57, 3]], 60, 60)).toEqual([
      [1, 3, -3, 3],
      [61, 3, 57, 3],
    ]);
    expect(routeSegments([5, 58], [[5, 1]], 60, 60)).toEqual([
      [5, 58, 5, 61],
      [5, -2, 5, 1],
    ]);
  });

  it('draws a leg across a corner as four pieces', () => {
    expect(routeSegments([59, 59], [[1, 1]], 60, 60)).toHaveLength(4);
  });

  it('continues the next leg from the true target, not the unwrapped end', () => {
    expect(routeSegments([58, 10], [[2, 10], [4, 10]], 60, 60).at(-1)).toEqual([2, 10, 4, 10]);
  });
});
