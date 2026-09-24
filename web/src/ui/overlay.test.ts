import { describe, expect, it } from 'vitest';
import { arrowHead, wrappedSegments } from './overlay';

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
  it('ends a split directed edge at its target, so the marker is drawn at the right end', () => {
    // A neighbor edge across the east edge: from (49, 7) to its neighbor (0, 7).
    const segments = wrappedSegments(49, 7, 0, 7, 50, 50);
    expect(segments).toHaveLength(2);
    expect(segments.at(-1)!.slice(2)).toEqual([0, 7]);
    expect(segments.at(-1)!.slice(0, 2)).toEqual([-1, 7]);
  });
});

describe('arrowHead', () => {
  it('points along the edge, its tip short of the target', () => {
    const [tx, ty, lx, ly, rx, ry] = arrowHead(0, 0, 10, 0, 4, 2)!;
    expect([tx, ty]).toEqual([8, 0]);
    expect([lx, ly]).toEqual([4, 2]);
    expect([rx, ry]).toEqual([4, -2]);
  });
  it('works in any direction', () => {
    const [tx, ty, lx, ly, rx, ry] = arrowHead(5, 5, 5, 25, 4, 0)!;
    expect([tx, ty]).toEqual([5, 25]);
    expect([lx, ly]).toEqual([3, 21]);
    expect([rx, ry]).toEqual([7, 21]);
  });
  it('has none for a zero-length edge', () => {
    expect(arrowHead(3, 3, 3, 3, 4, 2)).toBeNull();
  });
});
