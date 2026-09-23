import { describe, expect, it } from 'vitest';
import { wrappedSegments } from './overlay';

describe('wrappedSegments', () => {
  it('keeps nearby edges as one segment', () => {
    expect(wrappedSegments(1, 1, 2, 1, 50, 50)).toEqual([[1, 1, 2, 1]]);
  });
  it('splits an edge that wraps horizontally', () => {
    expect(wrappedSegments(0, 5, 49, 5, 50, 50)).toEqual([[0, 5, -1, 5], [50, 5, 49, 5]]);
  });
  it('splits an edge that wraps vertically', () => {
    expect(wrappedSegments(3, 49, 3, 0, 50, 50)).toEqual([[3, 49, 3, 50], [3, -1, 3, 0]]);
  });
});
