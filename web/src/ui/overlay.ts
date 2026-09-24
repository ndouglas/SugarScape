/** Line segments (in cell units) for an edge on a torus: one if the endpoints
 *  are within half the grid, otherwise two segments running off each side.
 *  The last segment always ends at (x2, y2), where a directed edge's marker goes. */
export function wrappedSegments(
  x1: number, y1: number, x2: number, y2: number, width: number, height: number,
): [number, number, number, number][] {
  let dx = x2 - x1;
  let dy = y2 - y1;
  if (dx > width / 2) dx -= width;
  if (dx < -width / 2) dx += width;
  if (dy > height / 2) dy -= height;
  if (dy < -height / 2) dy += height;
  if (x1 + dx === x2 && y1 + dy === y2) return [[x1, y1, x2, y2]];
  return [[x1, y1, x1 + dx, y1 + dy], [x2 - dx, y2 - dy, x2, y2]];
}

/**
 * A direction marker for the segment (ax, ay) → (bx, by), in pixels: a triangle whose tip is `back`
 * short of b (outside the target agent's cell) and whose base is `size` behind the tip and `size`
 * wide. Returns [tipX, tipY, leftX, leftY, rightX, rightY], or null for a zero-length segment.
 */
export function arrowHead(
  ax: number, ay: number, bx: number, by: number, size: number, back: number,
): [number, number, number, number, number, number] | null {
  const len = Math.hypot(bx - ax, by - ay);
  if (len === 0) return null;
  const ux = (bx - ax) / len;
  const uy = (by - ay) / len;
  const tx = bx - ux * back;
  const ty = by - uy * back;
  const cx = tx - ux * size;
  const cy = ty - uy * size;
  const half = size / 2;
  return [tx, ty, cx - uy * half, cy + ux * half, cx + uy * half, cy - ux * half];
}
