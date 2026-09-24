import type { ChartGroup } from '../protocol';

/** uPlot data: x values, then one array per line (null is a gap). */
export type LineData = [number[], ...(number | null)[][]];

/** NaN (no trades, no agents) is a gap in the line. */
const gap = (v: number): number | null => (Number.isNaN(v) ? null : v);

/** A time chart's data: the group's ticks as x, then one line per series. */
export function lineData(g: ChartGroup): LineData {
  return [Array.from(g.ticks), ...g.columns.map((c) => Array.from(c, gap))];
}

/** The Trade price chart (`[mean_log_price, sd_log_price]`): the mean, mean + SD and mean − SD. */
export function bandData(g: ChartGroup): LineData {
  const [mean, sd] = g.columns;
  const line = (f: (m: number, s: number) => number) => Array.from(mean, (m, i) => gap(f(m, sd[i])));
  return [Array.from(g.ticks), line((m) => m), line((m, s) => m + s), line((m, s) => m - s)];
}

/**
 * True when some group in `groups` is missing from `cached` (the engine's latest group per key) or
 * its last tick is behind `tick`. A panel asks for its groups only then (Decision 4): a paused
 * panel whose groups are caught up asks for nothing, even after it is hidden and shown again,
 * since the host sends a group only once its history has grown.
 */
export function chartsBehind(groups: string[][], tick: number, cached: (names: string[]) => ChartGroup | undefined): boolean {
  return groups.some((g) => {
    const last = cached(g)?.ticks.at(-1);
    return last === undefined || last < tick;
  });
}
