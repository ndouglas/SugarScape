import type { ChartGroup, Wants } from '../protocol';
import type { Config, ModelKind } from '../types';

/** uPlot data: x values, then one array per line (null is a gap). */
export type LineData = [number[], ...(number | null)[][]];

/** NaN (no trades, no agents) is a gap in the line. */
const gap = (v: number): number | null => (Number.isNaN(v) ? null : v);

/** A time chart's data: the group's ticks (plus `offset`: the anasazi's start year) as x, then one line per series. */
export function lineData(g: ChartGroup, offset = 0): LineData {
  return [Array.from(g.ticks, (t) => t + offset), ...g.columns.map((c) => Array.from(c, gap))];
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

/** The wealth histogram `[binWidth, counts…]` as bars: bin centers and counts. */
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

/** The tag histogram (percent with a 0 at each position, position 0 first) as bars at positions 1…L. */
export function positionBars(pct: Float64Array | null | undefined): LineData {
  if (!pct) return [[], []];
  const values = Array.from(pct);
  return [values.map((_, i) => i + 1), values];
}

/** The tag histogram as a step outline: each position's bar from p − ½ to p + ½, closed at the right. */
export function positionSteps(pct: Float64Array | null | undefined): LineData {
  if (!pct) return [[], []];
  const values = Array.from(pct);
  return [Array.from({ length: values.length + 1 }, (_, k) => k + 0.5), [...values, 0]];
}

/** The age histogram shows while lifetimes are finite (Animation III-1). */
export const showsAgeHist = (c: Config): boolean => c.lifespan.enabled;
/** The cultural tag histogram shows while culture is on (Animation III-7). */
export const showsTagHist = (c: Config): boolean => c.culture.enabled;
/** A world has a market: trade, and the Economy section's charts, need two or more goods. */
export const twoGoods = (c: Config): boolean => c.goods.length >= 2;
/** Total wealth (its Lorenz curve and Gini) shows with two or more goods (VI-1's "total wealth"). */
export const showsTotalWealth = twoGoods;
/** Good `good`'s own wealth histogram shows with two or more goods, while the world has that good. */
export const showsGoodWealth =
  (good: number) =>
  (c: Config): boolean =>
    showsTotalWealth(c) && good < c.goods.length;

/**
 * A line of a time chart: its series, legend label and color (a CSS variable or `#rrggbb`). A
 * `reference` line (the anasazi's historical record) is the same for every world, so Compare draws
 * it once, from A.
 */
export interface ChartLine { key: string; label: string; color: string; reference?: true }

/** A time chart of another model: its title, lines and y range. */
export interface ModelChart { title: string; lines: ChartLine[]; range?: [number, number] }

/**
 * The other models' charts (Decision 13), each a time chart of the model's own series:
 * Schelling's segregation, share unsatisfied, moves and Red share; Ring World's flocks, flock
 * size (mean and largest) and distance moved; the anasazi's households against the historical
 * record, carrying capacity, fit, stored corn, and births, moves and departures.
 */
export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]> = {
  schelling: [
    { title: 'Segregation', lines: [{ key: 'segregation', label: 'Like neighbors (mean share)', color: '--c2' }], range: [0, 1] },
    { title: 'Unsatisfied', lines: [{ key: 'unsatisfied', label: 'Unsatisfied share', color: '--red' }], range: [0, 1] },
    { title: 'Moves', lines: [{ key: 'moves', label: 'Agents moved', color: '--c1' }] },
    { title: 'Red share', lines: [{ key: 'red_share', label: 'Red', color: '--red' }], range: [0, 1] },
  ],
  ring: [
    { title: 'Flocks', lines: [{ key: 'flocks', label: 'Flocks', color: '--c1' }] },
    {
      title: 'Flock size',
      lines: [
        { key: 'mean_flock', label: 'Mean', color: '--c2' },
        { key: 'largest_flock', label: 'Largest', color: '--c3' },
      ],
    },
    { title: 'Distance moved', lines: [{ key: 'mean_distance', label: 'Sites per agent', color: '--c4' }] },
  ],
  anasazi: [
    {
      title: 'Households vs historical',
      lines: [
        { key: 'households', label: 'Simulated', color: '--c1' },
        { key: 'historical', label: 'Historical estimate', color: '--muted', reference: true },
      ],
    },
    { title: 'Carrying capacity', lines: [{ key: 'capacity', label: 'Plots that feed a household', color: '--c3' }] },
    { title: 'Fit', lines: [{ key: 'fit', label: 'Sum of squared differences', color: '--c2' }] },
    { title: 'Mean stored corn', lines: [{ key: 'mean_corn', label: 'kg per household', color: '--c4' }] },
    {
      title: 'Births, moves and departures',
      lines: [
        { key: 'births', label: 'Births', color: '--c3' },
        { key: 'moves', label: 'Moves', color: '--c1' },
        { key: 'departures', label: 'Departures', color: '--red' },
      ],
    },
  ],
};

/** A model's time charts count calendar years (the anasazi's) or ticks. */
export function timeAxisLabel(model: ModelKind): string {
  return model === 'anasazi' ? 'Year' : 'Tick';
}

/** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
export function yearTickLabels(splits: number[]): string[] {
  return splits.map((v) => String(Number(v.toFixed(3))));
}

/** A world's lines of a chart: every line for A (world 0), all but the reference lines for B. */
export function worldLines(lines: ChartLine[], world: number): ChartLine[] {
  return world === 0 ? lines : lines.filter((l) => !l.reference);
}

/** A chart of `chartModel` shows while some world on screen runs that model (Compare pairs one model). */
export function showsForModel(chartModel: ModelKind, worlds: ModelKind[]): boolean {
  return worlds.includes(chartModel);
}

/** A world's distributions as last received: when, at which tick, and whether an edit, reset or config change has made them stale. */
export interface DistState { at: number; tick: number; stale: boolean }

/**
 * Whether a world's distributions are fetched again: only once the tick moved or they went stale,
 * and at most every `every` ms — so a paused, caught-up Charts tab asks for nothing (7a's rule).
 */
export function distributionsDue(d: DistState, tick: number, now: number, every: number): boolean {
  return (d.stale || tick !== d.tick) && now - d.at >= every;
}

/**
 * The distributions a world's charts draw: the (sugar) Lorenz curve and wealth histogram always;
 * with two or more goods supply & demand, the total-wealth Lorenz curve and each good's wealth
 * histogram; the age histogram while lifetimes are finite; the tag histogram while culture is on.
 */
export function distributionWants(c: Config): Wants {
  const out: Wants = { lorenz: true, wealthHist: true };
  if (twoGoods(c)) out.supplyDemand = true;
  if (showsTotalWealth(c)) Object.assign(out, { lorenzTotal: true, goodWealthHists: true });
  if (showsAgeHist(c)) out.ageHist = true;
  if (showsTagHist(c)) out.tagHist = true;
  return out;
}

/** One chart, named by its caption (not its shared title — several charts, like the per-good wealth histograms, share a title but not a caption), and whether it is on show. */
export interface NamedChart { name: string; hidden: boolean }

/**
 * The charts Export → Charts writes, in order: only the ones on show, each under its caption. Used
 * by `ChartsPanel.canvases()`, which cannot itself be unit-tested (it needs the DOM its plots and
 * captions live in); this pure slice of its logic can.
 */
export function shownCharts<T extends NamedChart>(charts: T[]): T[] {
  return charts.filter((c) => !c.hidden);
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
