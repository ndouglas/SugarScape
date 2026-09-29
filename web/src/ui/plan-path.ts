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

/** `d` wrapped to the shortest signed step on a ring of `size` cells. */
const shortest = (d: number, size: number) => d - size * Math.round(d / size);

/**
 * Segments from the agent's cell through a GOAP plan's targets, each leg taken the short way around
 * the torus. Unlike `planSegments` (A*'s unit steps, where a wrap is dropped), a leg that crosses a
 * seam is drawn as pieces: the leg from its start toward its unwrapped end, and copies shifted back
 * by the grid's width and/or height (two pieces across one seam, up to four across a corner). Their
 * ends lie off the grid; the canvas clips them at its edge.
 */
export function routeSegments(
  from: [number, number],
  steps: [number, number][],
  width: number,
  height: number,
): [number, number, number, number][] {
  const out: [number, number, number, number][] = [];
  let [x1, y1] = from;
  for (const [x2, y2] of steps) {
    const ex = x1 + shortest(x2 - x1, width);
    const ey = y1 + shortest(y2 - y1, height);
    const sxs = ex < 0 ? [0, width] : ex >= width ? [0, -width] : [0];
    const sys = ey < 0 ? [0, height] : ey >= height ? [0, -height] : [0];
    for (const sy of sys) for (const sx of sxs) out.push([x1 + sx, y1 + sy, ex + sx, ey + sy]);
    [x1, y1] = [x2, y2];
  }
  return out;
}
