/**
 * Segments from the agent's cell through its planned path (oldest first). A step of more than half
 * the grid in x or y is a wrap around the torus and is left out, in the style of `trailSegments`.
 */
export function planSegments(
  from: [number, number],
  path: [number, number][],
  width: number,
  height: number,
): [number, number, number, number][] {
  const out: [number, number, number, number][] = [];
  let [x1, y1] = from;
  for (const [x2, y2] of path) {
    if (Math.abs(x2 - x1) <= width / 2 && Math.abs(y2 - y1) <= height / 2) out.push([x1, y1, x2, y2]);
    [x1, y1] = [x2, y2];
  }
  return out;
}
