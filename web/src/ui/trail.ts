export interface TrailSegment { x1: number; y1: number; x2: number; y2: number; alpha: number }

/**
 * Segments between consecutive trail cells (`[x0, y0, x1, y1, …]`, oldest first). A step of more than
 * half the grid in x or y is a wrap around the torus and is left out. Segment i has alpha (i + 1)/(n − 1),
 * so the line fades in from the oldest position to the newest.
 */
export function trailSegments(trail: ArrayLike<number>, width: number, height: number): TrailSegment[] {
  const n = Math.floor(trail.length / 2);
  const out: TrailSegment[] = [];
  for (let i = 0; i + 1 < n; i++) {
    const [x1, y1, x2, y2] = [trail[2 * i], trail[2 * i + 1], trail[2 * i + 2], trail[2 * i + 3]];
    if (Math.abs(x2 - x1) > width / 2 || Math.abs(y2 - y1) > height / 2) continue;
    out.push({ x1, y1, x2, y2, alpha: (i + 1) / (n - 1) });
  }
  return out;
}
