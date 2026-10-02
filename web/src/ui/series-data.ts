import type { ChartGroup, HoardCharts, HoardGenerationSeries, Wants } from '../protocol';
import type { Config, HoardConfig, ModelConfig, ModelKind } from '../types';

/** uPlot data: x values, then one array per line (null is a gap). */
export type LineData = [number[], ...(number | null)[][]];

/** NaN (no trades, no agents) is a gap in the line. */
const gap = (v: number): number | null => (Number.isNaN(v) ? null : v);

/** A time chart's data: the group's ticks (plus `offset`: the anasazi's start year) as x, then one line per series. */
export function lineData(g: ChartGroup, offset = 0): LineData {
  return [Array.from(g.ticks, (t) => t + offset), ...g.columns.map((c) => Array.from(c, gap))];
}

/**
 * The "Pilfers by source" table (`[raided, pilfered]`): the second column becomes what was stumbled on,
 * `pilfered − raided` (raids are pilfers), never below 0 against rounding; a gap stays a gap.
 */
export function stumbledData(data: LineData): LineData {
  const [ticks, raided, pilfered] = data as [number[], (number | null)[], (number | null)[]];
  return [ticks as number[], raided, pilfered.map((p, i) => (p == null || raided[i] == null ? null : Math.max(0, p - raided[i]!)))];
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

/** Ordinary episode charts; unavailable denominators arrive as NaN and draw as gaps. */
export const SPATIAL_HOARDING_CHARTS: { title: string; lines: ChartLine[] }[] = [
  ...[
    ['cached', 'Stored food'], ['buried', 'Burials per tick'], ['dug', 'Recovery per tick'],
    ['pilfered', 'Pilferage per tick'], ['lost', 'Food lost on death per tick'],
    ['cache_ticks', 'Positive cache-ticks'], ['stock_ticks', 'Food-unit-ticks'],
    ['recovery', 'Cumulative recovery share'], ['loss_rate', 'Loss per food-unit-tick'],
  ].map(([field,title]) => ({ title: `Spatial ${title.toLowerCase()}`, lines: [
    { key: `scatter_${field}`, label: 'Scatter', color: '--c1' },
    { key: `larder_${field}`, label: 'Larder', color: '--c3' },
  ] })),
  { title: 'Spatial deliveries', lines: [
    { key: 'delivery_starts', label: 'Starts', color: '--c1' },
    { key: 'delivery_completions', label: 'Completions', color: '--c3' },
    { key: 'delivery_cancellations', label: 'Cancellations', color: '--red' },
    { key: 'delivery_return_turns', label: 'Return turns', color: '--c4' },
  ] },
  { title: 'Spatial guards', lines: [
    { key: 'guard_intended', label: 'Intentions', color: '--c2' },
    { key: 'guard_executed', label: 'Paid guard actions', color: '--c3' },
    { key: 'guard_blocked_raids', label: 'Blocked raids', color: '--red' },
    { key: 'guard_blocked_discoveries', label: 'Blocked discoveries', color: '--c4' },
  ] },
  { title: 'Spatial food consumed', lines: [
    { key: 'metabolic_demand', label: 'Demand', color: '--c2' },
    { key: 'metabolic_consumed', label: 'Consumed', color: '--c1' },
  ] },
];

/** A civil config of Model II (its groups and kills have charts). */
/** A classes config with two tags (its per-tag charts show). */
export const hasTags = (c: ModelConfig): boolean => 'tags' in c && (c as { tags: unknown }).tags === true;

/** A norms config with Axelrod's two groups (the by-group chart shows). */
export const hasGroups = (c: ModelConfig): boolean => 'groups' in c && (c as { groups: { enabled?: unknown } }).groups?.enabled === true;

export const isEthnic = (c: ModelConfig): boolean => 'variant' in c && c.variant === 'ethnic';

/** Whether an image-scoring config allows LH01's binary scorers (who have their own shares chart). */
const hasBinary = (c: ModelConfig): boolean => 'strategies' in c && (c.strategies as string[]).includes('binary');

/** Whether an image-scoring config allows a class with a k (k, AND, OR, binary). */
const hasK = (c: ModelConfig): boolean => 'strategies' in c && (c.strategies as string[]).some((s) => s === 'k' || s === 'and' || s === 'or' || s === 'binary');

/**
 * The other models' charts (Decision 13), each a time chart of the model's own series:
 * Schelling's segregation, share unsatisfied, moves and Red share; Ring World's flocks, flock
 * size (mean and largest) and distance moved; the anasazi's households against the historical
 * record, carrying capacity, fit, stored corn, and births, moves and departures; civil violence's
 * actives, quiet and jailed, legitimacy, cops, tension, outbursts, and in Model II its groups and
 * kills; the spatial games' cooperators, changes, switches and payoffs; the tags model's donation,
 * tolerance, clusters, tags and takeovers; the ethnocentrism model's strategies (in the frame's
 * colors), cooperation, population and kin; the demographic PD's cooperators and defectors (in the
 * frame's colors), cooperator share, surrounded cooperators, mean wealths, and births and deaths;
 * image scoring's help rate and cooperative strategies, mean k, strategy shares (the binary scorers
 * apart) and mean payoff.
 */
/** A retirement config with two sub-populations (its by-group lines show). */
export const hasRetirementGroups = (c: ModelConfig): boolean => (c as { groups?: { enabled?: unknown } }).groups?.enabled === true && (c as { model?: unknown }).model === 'retirement';

/** An El Farol config playing Arthur's game (its forecasts chart shows). */
export const isElFarol = (c: ModelConfig): boolean => 'game' in c && (c as { game: unknown }).game === 'el_farol';

/** A minority game whose memories differ or evolve (its memory chart shows). */
export const memoriesVary = (c: ModelConfig): boolean => {
  const f = c as { game?: unknown; mixed_memory?: { enabled?: unknown }; evolution?: { enabled?: unknown } };
  return f.game === 'minority' && (f.mixed_memory?.enabled === true || f.evolution?.enabled === true);
};

export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]> = {
  schelling: [
    { title: 'Segregation', lines: [{ key: 'segregation', label: 'Like neighbors (mean share)', color: '--c2' }], range: [0, 1] },
    {
      title: 'Like neighbors by color',
      lines: [
        { key: 'like_red', label: 'Red', color: '--red' },
        { key: 'like_blue', label: 'Blue', color: '--blue' },
      ],
      range: [0, 1],
    },
    { title: 'No opposite neighbor', lines: [{ key: 'no_unlike', label: 'Share of agents', color: '--c3' }], range: [0, 1] },
    { title: 'Unsatisfied', lines: [{ key: 'unsatisfied', label: 'Unsatisfied share', color: '--red' }], range: [0, 1] },
    { title: 'Moves', lines: [{ key: 'moves', label: 'Agents moved', color: '--c1' }] },
    { title: 'Red share', lines: [{ key: 'red_share', label: 'Red', color: '--red' }], range: [0, 1] },
    // Milestone 32: each variation's own measure.
    {
      title: 'Clusters',
      lines: [
        { key: 'clusters', label: 'Side by side', color: '--c1' },
        { key: 'clusters8', label: 'Sides or corners (Singh et al.)', color: '--c2' },
        { key: 'pv_clusters', label: 'Through blanks (Pancs & Vriend)', color: '--c4' },
      ],
    },
    { title: 'Segregation coefficient', lines: [{ key: 'seg_s', label: 's (Gauvin et al.)', color: '--c3' }], range: [0, 1] },
    { title: 'Mixed pairs', lines: [{ key: 'mixed_pairs', label: 'Unlike neighbors (Zhang)', color: '--c1' }] },
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
  structure: [
    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per move', color: '--c1' }], range: [0, 5] },
    { title: 'Cooperation', lines: [{ key: 'cooperation', label: 'Share of moves', color: '--c2' }], range: [0, 1] },
    {
      title: 'Strategy',
      lines: [
        { key: 'mean_p', label: 'Friendliness (p)', color: '--c2' },
        { key: 'mean_q', label: 'Forgiveness (q)', color: '--c3' },
        { key: 'mean_y', label: 'First move (y)', color: '--muted' },
      ],
      range: [0, 1],
    },
    {
      title: 'High cooperation',
      lines: [
        { key: 'high', label: 'At the threshold', color: '--c1' },
        { key: 'share_high_since', label: 'Share since first reached', color: '--c4' },
      ],
      range: [0, 1],
    },
    {
      title: 'Copying',
      lines: [
        { key: 'copied', label: 'Agents copying', color: '--c3' },
        { key: 'partner_p_slope', label: 'Partners’ p on own p (slope)', color: '--c4' },
      ],
    },
  ],
  dpd: [
    {
      title: 'Population',
      lines: [
        { key: 'cooperators', label: 'Cooperators', color: '--blue' },
        { key: 'defectors', label: 'Defectors', color: '--red' },
      ],
    },
    { title: 'Cooperator share', lines: [{ key: 'cooperator_share', label: 'Cooperators ÷ agents', color: '--blue' }], range: [0, 1] },
    { title: 'Surrounded cooperators', lines: [{ key: 'surrounded', label: 'All eight neighbors cooperate', color: '--c1' }] },
    {
      title: 'Wealth',
      lines: [
        { key: 'wealth_c', label: 'Mean, cooperators', color: '--blue' },
        { key: 'wealth_d', label: 'Mean, defectors', color: '--red' },
      ],
    },
    {
      title: 'Births and deaths',
      lines: [
        { key: 'births', label: 'Births', color: '--c2' },
        { key: 'deaths', label: 'Deaths', color: '--muted' },
      ],
    },
  ],
  norms: [
    {
      title: 'Boldness and vengefulness',
      lines: [
        { key: 'mean_boldness', label: 'Boldness', color: '--red' },
        { key: 'mean_vengefulness', label: 'Vengefulness', color: '--blue' },
      ],
      range: [0, 1],
    },
    {
      title: 'Events',
      lines: [
        { key: 'defections', label: 'Defections', color: '--red' },
        { key: 'punishments', label: 'Punishments', color: '--c3' },
        { key: 'metapunishments', label: 'Metapunishments', color: '--c4' },
      ],
    },
    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per agent per generation', color: '--c1' }] },
    {
      title: 'Norm state',
      lines: [
        { key: 'established', label: 'Established', color: '--lender' },
        { key: 'collapsed', label: 'Collapsed', color: '--red' },
      ],
      range: [0, 1],
    },
    {
      title: 'By group',
      lines: [
        { key: 'strong_boldness', label: 'Strong: boldness', color: '--red' },
        { key: 'weak_boldness', label: 'Weak: boldness', color: '--c3' },
        { key: 'strong_vengefulness', label: 'Strong: vengefulness', color: '--blue' },
        { key: 'weak_vengefulness', label: 'Weak: vengefulness', color: '--c4' },
      ],
      range: [0, 1],
      shown: hasGroups,
    },
  ],
  agreement: [
    {
      title: 'Convergence',
      lines: [
        { key: 'y', label: 'y', color: '--c1' },
        { key: 'p_plus', label: 'Moderates turned extremist (+)', color: '--red' },
        { key: 'p_minus', label: 'Moderates turned extremist (−)', color: '--blue' },
      ],
      range: [0, 1],
    },
    {
      title: 'Clusters',
      lines: [
        { key: 'clusters', label: 'Clusters', color: '--c1' },
        { key: 'major', label: 'Holding 1 % or more', color: '--c3' },
        { key: 'isolated', label: 'Isolated agents', color: '--c4' },
      ],
    },
    {
      title: 'Dispersion',
      lines: [
        { key: 'dispersion', label: 'Dispersion (Σ shares²)', color: '--c1' },
        { key: 'unmoved', label: 'Never moved', color: '--c4' },
      ],
      range: [0, 1],
    },
    {
      title: 'Opinion and uncertainty',
      lines: [
        { key: 'mean_opinion', label: 'Mean opinion', color: '--c1' },
        { key: 'mean_uncertainty', label: 'Mean uncertainty', color: '--c3' },
      ],
    },
    { title: 'Change', lines: [{ key: 'max_change', label: 'Largest move this period', color: '--c2' }] },
  ],
  image: [
    {
      title: 'Help rate',
      lines: [
        { key: 'help_rate', label: 'Helps ÷ rounds', color: '--c1' },
        { key: 'cooperative', label: 'Cooperative strategies', color: '--blue' },
      ],
      range: [0, 1],
    },
    { title: 'Mean k', lines: [{ key: 'mean_k', label: 'Over strategies with a k', color: '--c4' }], range: [-5, 6], shown: hasK },
    {
      title: 'Strategy shares',
      lines: [
        { key: 'k_cooperative', label: 'k ≤ 0', color: '--blue' },
        { key: 'k_defective', label: 'k > 0', color: '--red' },
        { key: 'h', label: 'h (own score)', color: '--c3' },
        { key: 'own_only', label: 'Own score only', color: '--muted' },
        { key: 'and', label: 'AND', color: '--c1' },
        { key: 'or', label: 'OR', color: '--c4' },
        { key: 'standing', label: 'Standing', color: '--c2' },
        { key: 'q', label: 'q strategies', color: '--lender' },
      ],
      range: [0, 1],
      shown: (c) => !hasBinary(c),
    },
    {
      title: 'Binary scorers and standing',
      lines: [
        { key: 'binary_c', label: 'Cooperators', color: '--blue' },
        { key: 'binary_x', label: 'Discriminators', color: '--c3' },
        { key: 'binary_d', label: 'Defectors', color: '--red' },
        { key: 'standing', label: 'Standing', color: '--c2' },
      ],
      range: [0, 1],
      shown: hasBinary,
    },
    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per agent, this generation', color: '--c2' }] },
  ],
  farol: [
    { title: 'Attendance', lines: [{ key: 'attendance', label: 'Attendance', color: '--c1' }] },
    {
      title: 'Fluctuations',
      lines: [
        { key: 'fluctuation', label: 'σ²/N, last 100 rounds', color: '--red' },
        { key: 'random_fluctuation', label: 'Coin-flippers', color: '--c4' },
      ],
    },
    {
      title: 'Success',
      lines: [
        { key: 'success', label: 'Right this round', color: '--c1' },
        { key: 'mean_gain', label: 'Gain per round so far', color: '--c3' },
      ],
      range: [0, 1],
    },
    {
      title: 'Forecasts',
      lines: [{ key: 'forecast_above', label: 'Forecasting above capacity', color: '--c2' }],
      range: [0, 1],
      shown: isElFarol,
    },
    { title: 'Switching', lines: [{ key: 'switching', label: 'Switched strategy', color: '--c4' }], range: [0, 1] },
    { title: 'Memory', lines: [{ key: 'mean_memory', label: 'Mean memory', color: '--c3' }], shown: memoriesVary },
  ],
  ants: [
    {
      title: 'Share',
      lines: [
        { key: 'share', label: 'At the first source', color: '--c1' },
        { key: 'top_share', label: 'At the largest source', color: '--c3' },
      ],
      range: [0, 1],
    },
    {
      title: 'Variance',
      lines: [
        { key: 'variance', label: 'Var of the share, so far', color: '--red' },
        { key: 'theory_variance', label: 'Theory', color: '--c4' },
      ],
    },
    {
      title: 'Flips',
      lines: [
        { key: 'flips', label: 'Flips so far', color: '--c2' },
        { key: 'residence', label: 'Steps between flips', color: '--c4' },
      ],
    },
    { title: 'Extremes', lines: [{ key: 'extreme', label: 'Steps with a source at 80 % or more', color: '--c1' }], range: [0, 1] },
  ],
  thresholds: [
    {
      title: 'Participation',
      lines: [
        { key: 'acting', label: 'Acting now', color: '--c1' },
        { key: 'theory', label: "Granovetter's continuous equilibrium", color: '--c4' },
      ],
      range: [0, 1],
    },
    {
      title: 'Episodes',
      lines: [
        { key: 'mean_size', label: 'Mean final share', color: '--c2' },
        { key: 'global_share', label: 'Share that went global', color: '--red' },
      ],
      range: [0, 1],
    },
    { title: 'Last cascade', lines: [{ key: 'last_size', label: 'Final share of the last episode', color: '--c3' }], range: [0, 1] },
    { title: 'Swing', lines: [{ key: 'swing', label: 'Range over the last 100 steps', color: '--c4' }], range: [0, 1] },
  ],
  retirement: [
    {
      title: 'Retired share',
      lines: [
        { key: 'retired', label: 'Of those eligible', color: '--red' },
        { key: 'retired_a', label: 'First group', color: '--c2' },
        { key: 'retired_b', label: 'Second group', color: '--c3' },
      ],
      range: [0, 1],
      shown: hasRetirementGroups,
    },
    { title: 'Retired share', lines: [{ key: 'retired', label: 'Of those eligible', color: '--red' }], range: [0, 1], shown: (c) => !hasRetirementGroups(c) },
    {
      title: 'Retirement age',
      lines: [
        { key: 'modal_age', label: 'Most common, last 10 periods', color: '--c1' },
        { key: 'mean_age', label: 'Mean, last 10 periods', color: '--c4' },
        { key: 'eligibility', label: 'Eligibility', color: '--c2' },
      ],
    },
    {
      title: 'Transition',
      lines: [
        { key: 'transition', label: 'The period the norm set in', color: '--c1' },
        { key: 'transition_new', label: 'Periods from the switch to the new norm', color: '--red' },
      ],
    },
    {
      title: 'Group transitions',
      lines: [
        { key: 'transition_a', label: 'First group (no rationals)', color: '--c2' },
        { key: 'transition_b', label: 'Second group', color: '--c3' },
      ],
      shown: hasRetirementGroups,
    },
  ],
  bali: [
    {
      title: 'Harvest',
      lines: [
        { key: 'harvest', label: 'Last year (t/ha)', color: '--c1' },
        { key: 'scored', label: 'Mean of the scored years', color: '--c4' },
        { key: 'spread', label: 'Spread across subaks', color: '--c2' },
      ],
    },
    { title: 'Changing plans', lines: [{ key: 'changing', label: 'Subaks that changed', color: '--c1' }] },
    {
      title: 'Water and pests',
      lines: [
        { key: 'water_stress', label: 'Water short (share)', color: '--blue' },
        { key: 'pest_loss', label: 'Harvest lost to pests (share)', color: '--red' },
      ],
      range: [0, 1],
    },
    {
      title: 'Patches',
      lines: [
        { key: 'patches', label: 'Patches of one plan', color: '--c1' },
        { key: 'strategies', label: 'Plans in use', color: '--c2' },
      ],
    },
    {
      title: 'Temple match',
      lines: [
        { key: 'temple_match', label: 'Patches vs mascetis (ARI)', color: '--c1' },
        { key: 'network_match', label: 'Pest network vs mascetis', color: '--c4' },
      ],
    },
  ],
  zi: [
    {
      title: 'Prices',
      lines: [
        { key: 'price', label: 'Trade', color: '--c3' },
        { key: 'mean_price', label: 'Mean this period', color: '--c1' },
        { key: 'p0', label: 'Equilibrium (P₀)', color: '--c4' },
      ],
    },
    {
      title: 'Efficiency',
      lines: [
        { key: 'efficiency', label: 'This period so far (%)', color: '--c1' },
        { key: 'last_efficiency', label: 'Last period (%)', color: '--c2' },
      ],
    },
    {
      title: 'Convergence',
      lines: [
        { key: 'alpha', label: "Smith's α this period", color: '--c1' },
        { key: 'last_alpha', label: 'α, last period', color: '--c2' },
      ],
    },
    {
      title: 'Profit dispersion',
      lines: [
        { key: 'dispersion', label: 'This period so far', color: '--c1' },
        { key: 'last_dispersion', label: 'Last period', color: '--c2' },
      ],
    },
    { title: 'Volume', lines: [{ key: 'volume', label: 'Units traded this period', color: '--c1' }] },
  ],
  punishment: [
    {
      title: 'Types',
      lines: [
        { key: 'contributors', label: 'Contributors', color: '--blue' },
        { key: 'punishers', label: 'Punishers', color: '--c2' },
        { key: 'defectors', label: 'Defectors', color: '--red' },
      ],
      range: [0, 1],
    },
    {
      title: 'Cooperation',
      lines: [
        { key: 'cooperation', label: 'Contributors and punishers', color: '--c1' },
        { key: 'long_run', label: 'Long-run average', color: '--c4' },
        { key: 'acts', label: 'Cooperated this period', color: '--c3' },
      ],
      range: [0, 1],
    },
    { title: 'Payoff', lines: [{ key: 'payoff', label: 'Mean payoff', color: '--c1' }] },
    {
      title: 'Conflict',
      lines: [
        { key: 'conflicts', label: 'Conflicts', color: '--red' },
        { key: 'spread', label: 'Spread of groups’ cooperation', color: '--c2' },
      ],
    },
  ],
  line: [
    { title: 'Groups', lines: [{ key: 'groups', label: 'Runs of one color', color: '--c1' }] },
    { title: 'Group size', lines: [{ key: 'mean_group', label: 'Mean people per group', color: '--c2' }] },
    { title: 'Like neighbors', lines: [{ key: 'like_share', label: 'Mean share alike', color: '--c3' }], range: [0, 1] },
    { title: 'Unsatisfied', lines: [{ key: 'unsatisfied', label: 'Unsatisfied share', color: '--red' }], range: [0, 1] },
  ],
  // Minds 7 charts by generation and this season's bouts (HOARD_CHARTS), not over the whole run.
  hoard: [],
  auctions: [
    { title: 'Bids', lines: [
      { key: 'bid_1', label: 'Bidder 1 played (tick mean)', color: '--c1' },
      { key: 'bid_2', label: 'Bidder 2 played (tick mean)', color: '--c2' },
      { key: 'bid_3', label: 'Bidder 3 played (tick mean)', color: '--c3' },
      { key: 'greedy_1', label: 'Bidder 1 greedy (tick end)', color: '--c4' },
      { key: 'greedy_2', label: 'Bidder 2 greedy (tick end)', color: '--red' },
      { key: 'greedy_3', label: 'Bidder 3 greedy (tick end)', color: '--muted' },
    ] },
    { title: 'Seller revenue', lines: [{ key: 'revenue', label: 'Realized payment (tick mean)', color: '--c1' }, { key: 'terminal_revenue', label: 'Final policy expected revenue', color: '--c4' }] },
    { title: 'Bidder rewards', lines: [
      { key: 'profit_1', label: 'Bidder 1 (tick mean)', color: '--c1' },
      { key: 'profit_2', label: 'Bidder 2 (tick mean)', color: '--c2' },
      { key: 'profit_3', label: 'Bidder 3 (tick mean)', color: '--c3' },
    ] },
    { title: 'Learning', lines: [
      { key: 'epsilon', label: 'Exploration probability', color: '--c1' },
      { key: 'explored', label: 'Exploring share (tick mean)', color: '--c2' },
      { key: 'downward', label: 'Downward share (tick mean)', color: '--c3' },
      { key: 'greedy_changes', label: 'Policy changes (tick count)', color: '--red' },
      { key: 'stable', label: 'Stable periods (tick end)', color: '--c4' },
    ] },
  ],
  collusion: [
    {
      title: 'Prices',
      lines: [
        { key: 'price_1', label: 'Firm 1', color: '--c1' },
        { key: 'price_2', label: 'Firm 2', color: '--c2' },
        { key: 'greedy_price', label: 'Greedy price (mean)', color: '--c4' },
      ],
    },
    { title: 'Profit gain', lines: [{ key: 'profit_gain', label: 'Δ this period', color: '--c1' }] },
    {
      title: 'Learning',
      lines: [
        { key: 'epsilon', label: 'Exploration rate', color: '--c1' },
        { key: 'explored', label: 'Firms exploring', color: '--c3' },
        { key: 'greedy_changes', label: 'Strategy changes', color: '--red' },
      ],
    },
    { title: 'Settling', lines: [{ key: 'stable', label: 'Periods unchanged', color: '--c2' }] },
  ],
  firms: [
    {
      title: 'Firms',
      lines: [
        { key: 'firms', label: 'Firms', color: '--c1' },
        { key: 'births', label: 'Founded this period', color: '--c3' },
        { key: 'deaths', label: 'Dissolved this period', color: '--red' },
      ],
    },
    {
      title: 'Sizes',
      lines: [
        { key: 'mean_size', label: 'Mean size', color: '--c1' },
        { key: 'largest', label: 'Largest firm', color: '--c2' },
      ],
    },
    {
      title: 'Effort and pay',
      lines: [
        { key: 'effort', label: 'Mean effort', color: '--c1' },
        { key: 'income', label: 'Mean income', color: '--c2' },
        { key: 'utility', label: 'Mean utility', color: '--c4' },
      ],
    },
    { title: 'Output', lines: [{ key: 'output', label: 'Total output', color: '--c1' }] },
    {
      title: 'Scaling',
      lines: [
        { key: 'mu', label: 'µ (Axtell\'s OLS)', color: '--c1' },
        { key: 'mu_mle', label: 'µ (maximum likelihood)', color: '--c4' },
      ],
    },
  ],
  tipping: [
    {
      title: 'Inside',
      lines: [
        { key: 'red_in', label: 'Red', color: '--red' },
        { key: 'blue_in', label: 'Blue', color: '--blue' },
      ],
    },
    {
      title: 'Unhappy inside',
      lines: [
        { key: 'red_unhappy', label: 'Red', color: '--red' },
        { key: 'blue_unhappy', label: 'Blue', color: '--blue' },
      ],
    },
  ],
};

/**
 * A model's time charts count calendar years (the anasazi's), generations (tags, image scoring),
 * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
 */
export function timeAxisLabel(model: ModelKind): string {
  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'bali' ? 'Months' : model === 'line' ? 'Rounds' : model === 'tipping' ? 'Steps' : model === 'hoard' ? 'Bouts' : model === 'firms' ? 'Periods' : model === 'collusion' || model === 'auctions' ? 'Ticks' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
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

/**
 * A Minds 7 chart: by generation (one point per finished season, from the world's season records),
 * or this season's bouts (`season`: the larder share). One unit to a chart.
 */
export interface HoardChart { title: string; lines: (ChartLine & { key: HoardGenerationSeries })[]; range?: [number, number]; season?: true }

export const HOARD_CHARTS: HoardChart[] = [
  {
    title: 'Larder probability (L) by generation',
    lines: [
      { key: 'mean_larder_prob', label: 'Mean L, all agents', color: '--c1' },
      { key: 'hoarder_larder_prob', label: 'Mean L, hoarders only', color: '--c3' },
    ],
    range: [0, 1],
  },
  { title: 'Defense propensity (D) by generation', lines: [{ key: 'mean_defense', label: 'Mean D', color: '--c2' }], range: [0, 1] },
  { title: 'Survivors by generation', lines: [{ key: 'survivors', label: 'Agents alive at the season’s end', color: '--c1' }] },
  {
    title: 'Larder share by generation',
    lines: [{ key: 'larder_share', label: 'Larder items ÷ all items the survivors hold', color: '--c4' }],
    range: [0, 1],
  },
  {
    title: 'Loss rates by generation (per item-day held)',
    lines: [
      { key: 'larder_loss_rate', label: 'Larder', color: '--red' },
      { key: 'scatter_loss_rate', label: 'Scattered', color: '--c1' },
    ],
  },
  {
    title: 'Larder share this season (per bout)',
    lines: [{ key: 'larder_share', label: 'Larder items ÷ all items the living hold', color: '--c4' }],
    range: [0, 1],
    season: true,
  },
];

/** A by-generation chart's data: generations as x, then each line's value per finished season (NaN a gap). */
export function hoardGenerationTable(charts: HoardCharts | null, keys: HoardGenerationSeries[]): LineData {
  if (!charts) return emptyTable(keys.length);
  return [Array.from(charts.generations.generation), ...keys.map((k) => Array.from(charts.generations[k], gap))];
}

/**
 * This season's chart: `[tick, value, …]` with x the day of the season (bout b of day d at
 * d − 1 + b / bouts), counted from the season's first tick.
 */
export function hoardSeasonTable(charts: HoardCharts | null, c: HoardConfig): LineData {
  const flat = charts?.season;
  if (!flat || flat.length === 0) return emptyTable(1);
  const season = c.days * c.bouts;
  const first = flat[0];
  // Generation g's bouts are ticks (g − 1)·season + 1 to g·season (tick 0 starts generation 1).
  const start = first === 0 ? 0 : Math.floor((first - 1) / season) * season;
  const xs: number[] = [];
  const ys: (number | null)[] = [];
  for (let i = 0; i + 1 < flat.length; i += 2) {
    xs.push((flat[i] - start) / c.bouts);
    ys.push(gap(flat[i + 1]));
  }
  return [xs, ys];
}
