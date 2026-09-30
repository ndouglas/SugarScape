import { describe, expect, it } from 'vitest';
import { cacheMarks, memoryDrawSpec, memoryMarks } from './memory-overlay';

describe('memoryDrawSpec', () => {
  it('is fully opaque (0.4) for a site seen this tick (age 0)', () => {
    expect(memoryDrawSpec([3, 4, 0, 0], 100)).toEqual({ alpha: 0.4, shape: 'square', filled: false });
  });

  it('fades to 0 at the edge of the span (age = span)', () => {
    expect(memoryDrawSpec([3, 4, 100, 0], 100)).toEqual({ alpha: 0, shape: 'square', filled: false });
  });

  it('fades linearly in between', () => {
    expect(memoryDrawSpec([0, 0, 25, 0], 100).alpha).toBeCloseTo(0.3);
  });

  it('draws no known spot (spot 0) as an unfilled square', () => {
    expect(memoryDrawSpec([0, 0, 0, 0], 10)).toMatchObject({ shape: 'square', filled: false });
  });

  it('draws a known spot believed unripe (spot 1) as an unfilled circle', () => {
    expect(memoryDrawSpec([0, 0, 0, 1], 10)).toMatchObject({ shape: 'circle', filled: false });
  });

  it('draws a known spot believed ripe (spot 2) as a filled circle', () => {
    expect(memoryDrawSpec([0, 0, 0, 2], 10)).toMatchObject({ shape: 'circle', filled: true });
  });

  it('is fully transparent when memory is off (span 0)', () => {
    expect(memoryDrawSpec([0, 0, 0, 0], 0).alpha).toBe(0);
  });
});

describe('memoryMarks', () => {
  it('maps flat [x, y, age, spot, …] into one mark per remembered site, in order', () => {
    const flat = Uint32Array.of(1, 2, 0, 0, 5, 6, 10, 2);
    expect(memoryMarks(flat, 20)).toEqual([
      { x: 1, y: 2, alpha: 0.4, shape: 'square', filled: false },
      { x: 5, y: 6, alpha: 0.2, shape: 'circle', filled: true },
    ]);
  });

  it('gives no marks for an empty list', () => {
    expect(memoryMarks(new Uint32Array(0), 20)).toEqual([]);
  });
});

describe('cacheMarks', () => {
  it('draws each cache at its site, sized by the square root of its share of the largest', () => {
    expect(cacheMarks([{ x: 1, y: 2, amount: 16 }, { x: 3, y: 4, amount: 4 }])).toEqual([
      { x: 1, y: 2, size: 0.35 },
      { x: 3, y: 4, size: 0.25 },
    ]);
  });

  it('draws nothing for no caches, and the smallest mark for an empty one', () => {
    expect(cacheMarks([])).toEqual([]);
    expect(cacheMarks([{ x: 0, y: 0, amount: 0 }])).toEqual([{ x: 0, y: 0, size: 0.15 }]);
  });
});
