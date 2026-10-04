import type { DemocraticPeaceConfig, GeosimConfig, AuctionsConfig, PolarityConfig, FieldError, ModelConfig, ModelKind, Param } from '../types';
import type { Axis, Metric, ShorthandAxis, Sweep, SweepBase } from './types';
import { formatValues, parseValues, type AxisScalar } from './values';

export interface AxisForm { path: string; values: string }
export interface MetricForm { kind: Metric['kind']; series: string; from: number; to: number | null; every: number }
export interface SweepForm {
  name: string;
  /** Kept as opened; the form does not edit it. */
  description?: string;
  /** Fixed treatment fields retained by a model's suggested experiment. */
  set?: Record<string, unknown>;
  x: AxisForm;
  /** The second axis: one line per value. */
  series: AxisForm | null;
  seeds: number;
  ticks: number;
  metric: MetricForm;
}

export const MAX_X_VALUES = 64;
export const MAX_SERIES_VALUES = 16;

/** A time series' single x value: the chart's x axis is the tick (Decisions 4 and 18). */
export const TIMESERIES_X: Axis = { label: 'All runs', values: [{ at: 0, set: {} }] };

/** A new sweep's form for a base of `model`: an axis and a statistic that model has (Decision 14). */
export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfig): SweepForm {
  const form: SweepForm = {
    name: 'Untitled sweep',
    x: { path: 'vision.max', values: '1:6:1' },
    series: null,
    seeds: 3,
    ticks: 500,
    metric: { kind: 'window_mean', series: 'population', from: 400, to: null, every: 25 },
  };
  if (model === 'schelling') {
    return { ...form, x: { path: 'population', values: '1000:2400:200' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'segregation' } };
  }
  if (model === 'ring') {
    return { ...form, x: { path: 'agents', values: '10:70:10' }, metric: { ...form.metric, series: 'flocks' } };
  }
  if (model === 'anasazi') {
    // The calibration's axis (JASSS ¶4.11), over a whole run to AD 1350.
    return {
      ...form,
      x: { path: 'harvest_adjustment', values: '0.5:0.62:0.02' },
      ticks: 550,
      metric: { ...form.metric, kind: 'final', series: 'fit' },
    };
  }
  if (model === 'structure') {
    // The paper's dial (the built-in cra-dial): Table 2's mean payoff against random substitution.
    return { ...form, x: { path: 'substitution', values: '0:1:0.1' }, ticks: 2500, metric: { ...form.metric, kind: 'window_mean', series: 'mean_payoff', from: 1501, to: null } };
  }
  if (model === 'opinions') {
    // Fig. 3's axis (the built-in hk-diagonal): surviving opinions against confidence.
    return { ...form, x: { path: 'epsilon', values: '0.05:0.3:0.05' }, ticks: 1000, metric: { ...form.metric, kind: 'final', series: 'clusters' } };
  }
  if (model === 'classes') {
    // Fig. 4's axis (the built-in aey-memory): the way out of a fractious start against memory.
    return { ...form, x: { path: 'memory', values: '6:14:2' }, ticks: 100000, metric: { ...form.metric, kind: 'final', series: 'equity_at' } };
  }
  if (model === 'culture') {
    // Table 2's axis (the built-in ac-table-2): stable regions against traits per feature.
    return { ...form, x: { path: 'traits', values: '5:15:5' }, ticks: 20000, metric: { ...form.metric, kind: 'final', series: 'regions' } };
  }
  if (model === 'tags') {
    // Table 1's axis (the built-in rca-pairings): the donation rate over the whole run.
    return { ...form, x: { path: 'pairings', values: '1:4:1' }, ticks: 3000, metric: { ...form.metric, kind: 'window_mean', series: 'donation_rate', from: 0 } };
  }
  if (model === 'civil') {
    // The Finding's axis (the built-in cv-ratio-rules): outbursts against legitimacy.
    return { ...form, x: { path: 'legitimacy', values: '0.6:0.95:0.05' }, ticks: 1000, metric: { ...form.metric, kind: 'final', series: 'outbursts' } };
  }
  if (model === 'spatial') {
    // NBM94's axis: cooperators against the temptation b.
    return { ...form, x: { path: 'b', values: '1.05:2.05:0.05' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'fraction_c' } };
  }
  if (model === 'ethno') {
    // HA06's summary (the mean over the last 100 of 2,000 periods) against the cost of helping (the built-in ha-cost).
    return {
      ...form,
      x: { path: 'cost', values: '0.005:0.03:0.0025' },
      ticks: 2000,
      metric: { ...form.metric, kind: 'window_mean', series: 'ethnocentric', from: 1901, to: null },
    };
  }
  if (model === 'agreement') {
    // Fig. 9's axis (the built-in ra-map): y against the moderates' uncertainty.
    return { ...form, x: { path: 'uncertainty', values: '0.2:2:0.2' }, ticks: 20000, metric: { ...form.metric, kind: 'final', series: 'y' } };
  }
  if (model === 'norms') {
    // Galán & Izquierdo's horizon (the built-in norms-horizon): collapse against how long runs last.
    return { ...form, x: { path: 'stop_at', values: '100,1000,10000' }, ticks: 10000, metric: { ...form.metric, kind: 'final', series: 'collapsed' } };
  }
  if (model === 'dpd') {
    // Table 9.3's axis at T = 6: cooperators after 500 cycles against the reward R (R = 1 dies out).
    return { ...form, x: { path: 'r', values: '1:5:1' }, ticks: 500, metric: { ...form.metric, kind: 'final', series: 'cooperators' } };
  }
  if (model === 'farol' && config && 'game' in config && config.game === 'minority') {
    // The built-in mg-memory's axis: how far the crowd swings against memory (strategies stop at 16).
    return { ...form, x: { path: 'memory', values: '1:12:1' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
  }
  if (model === 'farol') {
    // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
    return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
  }
  if (model === 'firms') {
    // The built-in firms-beta's axis: the size exponent against increasing returns (A99 Table 3).
    return { ...form, x: { path: 'beta', values: '1.7:2.1:0.1' }, ticks: 5000, metric: { ...form.metric, kind: 'final', series: 'mu' } };
  }
  if (model === 'democratic_peace') {
    const d = config as DemocraticPeaceConfig | undefined;
    return { ...form, description: 'Exploratory initial democratic density; terminal territory share after the captured source horizon. Undefined clustering and invalid runs retain their reasons. This is not the registered scientific study.', x: { path: 'initial_democratic_share', values: '0,0.1,0.3,0.5,1' }, ticks: d ? Math.ceil(d.horizon_periods / d.periods_per_tick) : 1000, metric: { ...form.metric, kind: 'final', series: 'democratic_share' } };
  }
  if (model === 'geosim') {
    const g = config as GeosimConfig | undefined;
    return { ...form, description: 'Exploratory GeoSim technology settings; completed abstract conflict clusters after the captured source horizon. This is not the registered scientific study.', x: { path: 'shock_shift', values: '0,10,20' }, ticks: g ? Math.ceil((g.initialization_periods + g.observation_periods) / g.periods_per_tick) : 10500, metric: { ...form.metric, kind: 'final', series: 'completed_wars' } };
  }
  if (model === 'polarity') {
    const p = config as PolarityConfig | undefined;
    return { ...form, description: 'Cederman predator-share settings; terminal sovereign count after the economic horizon or hegemony. Parameters and source profile are retained; ticks batch complete periods.', x: { path: 'predator_share', values: '0,0.05,0.1,0.2,0.4,0.6,0.8,1' }, ticks: p ? Math.ceil(p.horizon / p.periods_per_tick) : 1000, metric: { ...form.metric, kind: 'final', series: 'sovereign_count' } };
  }
  if (model === 'auctions') {
    return { ...form, set: { auction: 'mixture' }, description: 'Formats from first price (1) to second price (2), using mixture payment. Terminal policy revenue after the fixed horizon; one tick executes periods_per_tick auctions.', x: { path: 'auction_alpha', values: '1:2:0.1' }, ticks: config ? Math.ceil((config as AuctionsConfig).horizon / (config as AuctionsConfig).periods_per_tick) : 1000, metric: { ...form.metric, kind: 'final', series: 'terminal_revenue' } };
  }
  if (model === 'collusion') {
    // The built-in collusion-delta's axis: the profit gain against the discount factor (CCDP Fig. 6).
    return { ...form, x: { path: 'delta', values: '0:0.9:0.15' }, ticks: 100_000, metric: { ...form.metric, kind: 'final', series: 'cycle_gain' } };
  }
  if (model === 'bali') {
    // The built-in bali-imitation-growth's axis: the scored harvest against pest growth.
    return { ...form, x: { path: 'growth', values: '2:2.4:0.1' }, ticks: 360, metric: { ...form.metric, kind: 'final', series: 'scored' } };
  }
  if (model === 'hoard') {
    // The built-in hoard-ratio's axis: the hoarders' mean L in generation 50 (the sweep format's
    // 100 000 ticks) against how findable scattered caches are, at app_lard 2.
    return {
      ...form,
      x: { path: 'app_scat', values: '0.05,0.1,0.2,0.3,0.44,0.6,0.8,0.9' },
      ticks: 100000,
      metric: { ...form.metric, kind: 'window_mean', series: 'hoarder_larder_prob', from: 98001, to: 100000 },
    };
  }
  if (model === 'zi') {
    // The built-in gs-shouts' axis: efficiency against the period's length (Gode and Sunder's "30 seconds").
    return { ...form, x: { path: 'shouts', values: '25,50,100,200,500,1000,2000' }, ticks: 12000, metric: { ...form.metric, kind: 'final', series: 'avg_efficiency' } };
  }
  if (model === 'punishment') {
    // The built-in bg-fig1b's axis: the long-run cooperation (the last 1 000 of 2 000 periods) against group size.
    return { ...form, x: { path: 'size', values: '4,8,16,32,64,128,256' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'long_run' } };
  }
  if (model === 'retirement') {
    // The built-in ae-rational's axis: the period the age 65 norm sets in (kept once reached) against the rational share.
    return {
      ...form,
      x: { path: 'rational', values: '0.02,0.05,0.1,0.15,0.2,0.25' },
      ticks: 400,
      metric: { ...form.metric, kind: 'final', series: 'transition' },
    };
  }
  if (model === 'thresholds') {
    // The built-in gr-sd's axis: the share rioting at equilibrium against the spread of thresholds.
    return { ...form, x: { path: 'sd', values: '0.1:0.2:0.01' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'acting' } };
  }
  if (model === 'ants') {
    // The built-in ants-flips' axis: flips between sources against self-conversion ε.
    return { ...form, x: { path: 'epsilon', values: '0.001,0.002,0.003,0.005,0.01' }, ticks: 20000, metric: { ...form.metric, kind: 'final', series: 'flips' } };
  }
  if (model === 'image') {
    // NS98's rounds axis (the built-in ns-rounds, shortened): cooperative strategies against m.
    return {
      ...form,
      x: { path: 'rounds', values: '50:500:50' },
      ticks: 2000,
      metric: { ...form.metric, kind: 'window_mean', series: 'cooperative', from: 1001, to: null },
    };
  }
  return form;
}

const isScalar = (v: unknown): v is AxisScalar => typeof v === 'number' || typeof v === 'boolean';

function axisFromForm(axis: AxisForm, field: 'x' | 'series', max: number, errors: FieldError[]): ShorthandAxis | null {
  const path = axis.path.trim();
  if (path === '') errors.push({ field: `${field}.path`, message: 'Enter a config path' });
  const parsed = parseValues(axis.values, max);
  if ('error' in parsed) {
    errors.push({ field: `${field}.values`, message: parsed.error });
    return null;
  }
  return path === '' ? null : { path, values: parsed.values };
}

/** The sweep the form describes (shorthand axes, seeds from 1), or the form's own errors. */
export function formToSweep(form: SweepForm, base: SweepBase): { sweep: Sweep | null; errors: FieldError[] } {
  const errors: FieldError[] = [];
  const m = form.metric;
  const x = m.kind === 'timeseries' ? structuredClone(TIMESERIES_X) : axisFromForm(form.x, 'x', MAX_X_VALUES, errors);
  const series = form.series ? axisFromForm(form.series, 'series', MAX_SERIES_VALUES, errors) : undefined;
  if (errors.length > 0 || !x || series === null) return { sweep: null, errors };
  const metric: Metric =
    m.kind === 'final'
      ? { kind: 'final', series: m.series }
      : m.kind === 'timeseries'
        ? { kind: 'timeseries', series: m.series, every: m.every }
        : { kind: 'window_mean', series: m.series, from: m.from, ...(m.to === null ? {} : { to: m.to }) };
  const sweep: Sweep = {
    name: form.name.trim() || 'Untitled sweep',
    base,
    x,
    seeds: { from: 1, count: form.seeds },
    ticks: form.ticks,
    metric,
  };
  if (form.set) sweep.set = structuredClone(form.set);
  if (series) sweep.series = series;
  if (form.description !== undefined) sweep.description = form.description;
  return { sweep, errors: [] };
}

/** An axis the form can edit (one path, scalar values at their natural `at`, no line names), or null. */
export function axisToForm(axis: Axis | ShorthandAxis): AxisForm | null {
  if ('path' in axis) {
    if (axis.label !== undefined && axis.label !== axis.path) return null;
    return axis.values.every(isScalar) ? { path: axis.path, values: formatValues(axis.values as AxisScalar[]) } : null;
  }
  const keys = axis.values.map((v) => Object.keys(v.set));
  const path = keys[0]?.[0];
  if (path === undefined || axis.label !== path) return null;
  if (keys.some((k) => k.length !== 1 || k[0] !== path) || axis.values.some((v) => v.name !== undefined)) return null;
  const values = axis.values.map((v) => v.set[path]);
  if (!values.every(isScalar)) return null;
  if (!axis.values.every((v, i) => v.at === (typeof values[i] === 'number' ? values[i] : i))) return null;
  return { path, values: formatValues(values) };
}

function isTimeseriesX(axis: Axis | ShorthandAxis): boolean {
  return !('path' in axis) && axis.values.length === 1 && Object.keys(axis.values[0].set).length === 0;
}

/** The form for a sweep it can express (Decision 18), or null. */
export function sweepToForm(sweep: Sweep): SweepForm | null {
  if (Object.keys(sweep.set ?? {}).length > 0 || sweep.seeds.from !== 1) return null;
  const m = sweep.metric;
  const x: AxisForm | null =
    m.kind === 'timeseries' ? (isTimeseriesX(sweep.x) ? { path: '', values: '' } : null) : axisToForm(sweep.x);
  const series = sweep.series ? axisToForm(sweep.series) : null;
  if (!x || (sweep.series && !series)) return null;
  return {
    name: sweep.name,
    ...(sweep.description === undefined ? {} : { description: sweep.description }),
    x,
    series,
    seeds: sweep.seeds.count,
    ticks: sweep.ticks,
    metric: {
      kind: m.kind,
      series: m.series,
      from: m.kind === 'window_mean' ? m.from : Math.max(0, sweep.ticks - 100),
      to: m.kind === 'window_mean' ? (m.to ?? null) : null,
      every: m.kind === 'timeseries' ? m.every : Math.max(1, Math.round(sweep.ticks / 20)),
    },
  };
}

const CONTROLS = new Set([
  'name',
  'x.path',
  'x.values',
  'series.path',
  'series.values',
  'seeds.count',
  'ticks',
  'metric.kind',
  'metric.series',
  'metric.from',
  'metric.to',
  'metric.every',
]);

/** The form control that shows a core error: `x[i].set.<path>` → `x.path`; unknown fields → `general`. */
export function controlFor(field: string): string {
  const axis = /^(x|series)\[\d+\]/.exec(field);
  if (axis) return `${axis[1]}.path`;
  return CONTROLS.has(field) ? field : 'general';
}

/**
 * Dotted paths of every number or boolean in a config (the path input's suggestions), plus any
 * nullable numeric field's path while it is null (the ethnocentrism model's `tag_mutation`: a plain
 * null tells `typeof` nothing, so `schema` names which null leaves are numbers, not e.g. a range).
 */
export function numericPaths(config: ModelConfig, schema: Param[] = []): string[] {
  const nullable = new Set(schema.filter((p) => p.nullable).map((p) => p.path));
  const out: string[] = [];
  const walk = (value: unknown, path: string) => {
    if (path === 'schedule' || path === 'disease.outbreaks') return;
    if (typeof value === 'number' || typeof value === 'boolean' || (value === null && nullable.has(path))) {
      out.push(path);
      return;
    }
    if (value && typeof value === 'object') {
      for (const [key, child] of Object.entries(value)) walk(child, path ? `${path}.${key}` : key);
    }
  };
  walk(config, '');
  return out;
}

/** Capture the chosen experiment base before later world edits. */
export function captureExperimentBase(presetId: string | null, modified: boolean, config: ModelConfig): SweepBase {
  return presetId !== null && !modified ? {preset:presetId} : {config:structuredClone(config)};
}
