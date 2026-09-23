/** Line segments (in cell units) for an edge on a torus: one if the endpoints
 *  are within half the grid, otherwise two segments running off each side. */
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
