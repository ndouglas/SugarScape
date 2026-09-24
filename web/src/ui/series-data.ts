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

/** uPlot data for several worlds on one x axis: `undefined` is a hole uPlot draws through, `null` a gap. */
export type OverlayData = [number[], ...(number | null | undefined)[][]];

/**
 * Several tables' lines on the sorted union of their x values (Decision 11): each line is
 * `undefined` where its own table has no point (so two worlds' different downsampled ticks, or
 * price grids, share an axis and each line is drawn through the other's points) and keeps `null`
 * where its data has a gap.
 */
export function overlayData(tables: LineData[]): OverlayData {
  const xs = [...new Set(tables.flatMap((t) => t[0]))].sort((p, q) => p - q);
  const at = new Map(xs.map((x, i) => [x, i]));
  const out: OverlayData = [xs];
  for (const [x, ...lines] of tables) {
    for (const line of lines) {
      const column: (number | null | undefined)[] = new Array(xs.length).fill(undefined);
      line.forEach((v, i) => (column[at.get(x[i])!] = v));
      out.push(column);
    }
  }
  return out;
}

/** A table with no points yet and `lines` lines (a world whose group has not arrived). */
export function emptyTable(lines: number): LineData {
  return [[], ...Array.from({ length: lines }, () => [])] as LineData;
}

/** The wealth histogram `[binWidth, counts…]` as bars: bin centres and counts. */
export function barsData(hist: Float64Array | null | undefined): LineData {
  if (!hist) return [[], []];
  const width = hist[0];
  const counts = Array.from(hist.subarray(1));
  return [counts.map((_, i) => (i + 0.5) * width), counts];
}

/** The wealth histogram as a step outline: each bin's left edge and count, closed at the right edge. */
export function histTable(hist: Float64Array | null | undefined): LineData {
  if (!hist) return [[], []];
  const width = hist[0];
  const counts = Array.from(hist.subarray(1));
  return [Array.from({ length: counts.length + 1 }, (_, k) => k * width), [...counts, 0]];
}

/**
 * Supply & demand `[n, prices(n), demand(n), supply(n), eqP, eqQ, actP, actQ]` as price, demand,
 * supply, and the equilibrium and actual points marked at their nearest price.
 */
export function supplyDemandTable(sd: Float64Array | null | undefined): LineData {
  if (!sd) return [[], [], [], [], []];
  const n = sd[0];
  const prices = Array.from(sd.subarray(1, 1 + n));
  const demand = Array.from(sd.subarray(1 + n, 1 + 2 * n));
  const supply = Array.from(sd.subarray(1 + 2 * n, 1 + 3 * n));
  const [eqP, eqQ, actP, actQ] = Array.from(sd.subarray(1 + 3 * n));
  const nearest = (p: number) =>
    prices.reduce((best, q, i) => (Math.abs(Math.log(q / p)) < Math.abs(Math.log(prices[best] / p)) ? i : best), 0);
  const point = (p: number, q: number) => {
    const column: (number | null)[] = prices.map(() => null);
    if (Number.isFinite(p) && Number.isFinite(q)) column[nearest(p)] = q;
    return column;
  };
  return [prices, demand, supply, point(eqP, eqQ), point(actP, actQ)];
}
