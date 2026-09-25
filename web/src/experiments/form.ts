import type { FieldError, ModelConfig, ModelKind } from '../types';
import type { Axis, Metric, ShorthandAxis, Sweep, SweepBase } from './types';
import { formatValues, parseValues, type AxisScalar } from './values';

export interface AxisForm { path: string; values: string }
export interface MetricForm { kind: Metric['kind']; series: string; from: number; to: number | null; every: number }
export interface SweepForm {
  name: string;
  /** Kept as opened; the form does not edit it. */
  description?: string;
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
export function defaultForm(model: ModelKind = 'sugarscape'): SweepForm {
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

/** Dotted paths of every number or boolean in a config (the path input's suggestions). */
export function numericPaths(config: ModelConfig): string[] {
  const out: string[] = [];
  const walk = (value: unknown, path: string) => {
    if (path === 'schedule' || path === 'disease.outbreaks') return;
    if (typeof value === 'number' || typeof value === 'boolean') {
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
