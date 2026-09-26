import type { ChartGroup, Wants } from '../protocol';
import type { Config, ModelConfig, ModelKind } from '../types';

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

/** A time chart of another model: its title, lines and y range, and (civil Model II's) when it shows. */
export interface ModelChart { title: string; lines: ChartLine[]; range?: [number, number]; shown?: (c: ModelConfig) => boolean }

/** A civil config of Model II (its groups and kills have charts). */
/** A classes config with two tags (its per-tag charts show). */
export const hasTags = (c: ModelConfig): boolean => 'tags' in c && (c as { tags: unknown }).tags === true;

export const isEthnic = (c: ModelConfig): boolean => 'variant' in c && c.variant === 'ethnic';

/**
 * The other models' charts (Decision 13), each a time chart of the model's own series:
 * Schelling's segregation, share unsatisfied, moves and Red share; Ring World's flocks, flock
 * size (mean and largest) and distance moved; the anasazi's households against the historical
 * record, carrying capacity, fit, stored corn, and births, moves and departures; civil violence's
 * actives, quiet and jailed, legitimacy, cops, tension, outbursts, and in Model II its groups and
 * kills; the spatial games' cooperators, changes, switches and payoffs; the tags model's donation,
 * tolerance, clusters, tags and takeovers; the ethnocentrism model's strategies (in the frame's
 * colors), cooperation, population and kin.
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
  civil: [
    {
      title: 'Actives, quiet and jailed',
      lines: [
        { key: 'active', label: 'Active', color: '--red' },
        { key: 'quiet', label: 'Quiet', color: '--blue' },
        { key: 'jailed', label: 'Jailed', color: '--muted' },
      ],
    },
    { title: 'Legitimacy', lines: [{ key: 'legitimacy', label: 'L', color: '--c2' }], range: [0, 1] },
    { title: 'Cops', lines: [{ key: 'cops', label: 'Cops', color: '--c3' }] },
    { title: 'Tension', lines: [{ key: 'tension', label: 'Mean G × quiet share ÷ mean R', color: '--c4' }] },
    { title: 'Outbursts', lines: [{ key: 'outbursts', label: 'Outbursts ended', color: '--c1' }] },
    { title: 'Wait between outbursts', lines: [{ key: 'mean_wait', label: 'Mean (ticks)', color: '--c2' }] },
    { title: 'Activation per outburst', lines: [{ key: 'mean_activation', label: 'Mean total actives', color: '--c3' }] },
    {
      title: 'Groups',
      lines: [
        { key: 'blue', label: 'Blue', color: '--blue' },
        { key: 'green', label: 'Green', color: '--lender' },
      ],
      shown: isEthnic,
    },
    { title: 'Killed', lines: [{ key: 'killed', label: 'Killed this tick', color: '--red' }], shown: isEthnic },
  ],
  spatial: [
    { title: 'Cooperators', lines: [{ key: 'fraction_c', label: 'Share C', color: '--blue' }], range: [0, 1] },
    { title: 'Changes', lines: [{ key: 'changed', label: 'Share that switched', color: '--c4' }], range: [0, 1] },
    {
      title: 'Switches',
      lines: [
        { key: 'c_to_d', label: 'C → D', color: '--both' },
        { key: 'd_to_c', label: 'D → C', color: '--lender' },
      ],
    },
    {
      title: 'Payoffs',
      lines: [
        { key: 'mean_payoff_c', label: 'Mean C', color: '--blue' },
        { key: 'mean_payoff_d', label: 'Mean D', color: '--red' },
      ],
    },
  ],
  tags: [
    { title: 'Donation rate', lines: [{ key: 'donation_rate', label: 'Donations per pairing', color: '--c1' }], range: [0, 1] },
    {
      title: 'Tolerance',
      lines: [
        { key: 'mean_tolerance', label: 'Mean', color: '--c2' },
        { key: 'cluster_tolerance', label: 'Dominant cluster', color: '--c3' },
      ],
    },
    {
      title: 'Clusters',
      lines: [
        { key: 'cluster_share', label: 'Cluster share', color: '--c1' },
        { key: 'relatedness', label: 'Relatedness', color: '--c4' },
        { key: 'zero_tolerance_share', label: 'Zero tolerance', color: '--red' },
      ],
      range: [0, 1],
    },
    { title: 'Distinct tags', lines: [{ key: 'distinct_tags', label: 'Distinct tags', color: '--c2' }] },
    { title: 'Takeovers', lines: [{ key: 'takeovers', label: 'Dominant clusters replaced', color: '--c3' }] },
  ],
  culture: [
    {
      title: 'Regions, zones and cultures',
      lines: [
        { key: 'regions', label: 'Regions', color: '--c1' },
        { key: 'zones', label: 'Zones', color: '--c3' },
        { key: 'cultures', label: 'Cultures', color: '--c2' },
      ],
    },
    { title: 'Largest region', lines: [{ key: 'largest_region', label: 'Share of sites', color: '--c4' }], range: [0, 1] },
    { title: 'Mean similarity', lines: [{ key: 'mean_similarity', label: 'Features shared by neighbors', color: '--c2' }], range: [0, 1] },
    { title: 'Active bonds', lines: [{ key: 'active_bonds', label: 'Pairs that can still interact', color: '--red' }] },
    { title: 'Changes', lines: [{ key: 'changes', label: 'Traits changed this tick', color: '--c3' }] },
  ],
  classes: [
    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per agent per match', color: '--c1' }], range: [0, 70] },
    {
      title: 'Outcomes',
      lines: [
        { key: 'outcome_mm', label: 'M–M', color: '--c2' },
        { key: 'outcome_hl', label: 'H–L', color: '--c3' },
        { key: 'outcome_fail', label: 'Over 100', color: '--red' },
        { key: 'outcome_waste', label: 'Under 100', color: '--muted' },
      ],
      range: [0, 1],
    },
    { title: 'M in memory', lines: [{ key: 'm_share', label: 'Share of remembered demands', color: '--c2' }], range: [0, 1] },
    { title: 'Regime', lines: [{ key: 'regime', label: '0 mixed · 1 equity · 2 fractious · 3 classes · 4 split within · 5 divided below', color: '--c4' }], range: [0, 5] },
    {
      title: 'Payoffs by tag',
      lines: [
        { key: 'payoff_dark', label: 'Dark', color: '--red' },
        { key: 'payoff_light', label: 'Light', color: '--blue' },
      ],
      range: [0, 70],
      shown: hasTags,
    },
    { title: 'Inter-type advantage', lines: [{ key: 'payoff_inter', label: 'Dark minus light, against each other', color: '--c3' }], shown: hasTags },
  ],
  ethno: [
    {
      title: 'Strategies',
      lines: [
        { key: 'ethnocentric', label: 'Ethnocentric', color: '--lender' },
        { key: 'humanitarian', label: 'Humanitarian', color: '--blue' },
        { key: 'selfish', label: 'Selfish', color: '--red' },
        { key: 'traitorous', label: 'Traitorous', color: '--both' },
        { key: 'kin', label: 'Kin', color: '--c4' },
        { key: 'nonkin', label: 'Non-kin', color: '--c2' },
        { key: 'mixed', label: 'Mixed', color: '--muted' },
      ],
      range: [0, 1],
    },
    {
      title: 'Cooperation',
      lines: [
        { key: 'cooperation', label: 'Helps per decision', color: '--c1' },
        { key: 'same_tag', label: 'Decisions toward the same tag', color: '--c3' },
      ],
      range: [0, 1],
    },
    { title: 'Population', lines: [{ key: 'population', label: 'Agents', color: '--c2' }] },
    {
      title: 'Kin',
      lines: [
        { key: 'relatives', label: 'Neighbors related', color: '--muted' },
        { key: 'kin_help', label: 'Helps to relatives', color: '--c4' },
        { key: 'tag_given_relative', label: 'Same tag if related', color: '--c1' },
        { key: 'relative_given_tag', label: 'Related if same tag', color: '--c3' },
      ],
      range: [0, 1],
    },
  ],
  opinions: [
    { title: 'Clusters', lines: [{ key: 'clusters', label: 'Surviving opinions', color: '--c1' }] },
    {
      title: 'Largest camps',
      lines: [
        { key: 'largest', label: 'Largest', color: '--c2' },
        { key: 'second', label: 'Second', color: '--c3' },
      ],
      range: [0, 1],
    },
    {
      title: 'Mean and median',
      lines: [
        { key: 'mean_opinion', label: 'Mean', color: '--c1' },
        { key: 'median_opinion', label: 'Median', color: '--c4' },
      ],
      range: [0, 1],
    },
    {
      title: 'Splits',
      lines: [
        { key: 'splits', label: 'Two-sided', color: '--red' },
        { key: 'one_sided_splits', label: 'One-sided', color: '--c3' },
      ],
    },
    { title: 'Change', lines: [{ key: 'max_change', label: 'Largest move this period', color: '--c4' }] },
  ],
};

/** A model's time charts count calendar years (the anasazi's), generations (tags), periods (ethnocentrism, HA06's word) or ticks. */
export function timeAxisLabel(model: ModelKind): string {
  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' ? 'Periods' : model === 'ethno' ? 'Period' : 'Tick';
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
