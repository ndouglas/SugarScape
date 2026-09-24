import type { Axis, ShorthandAxis, Sweep } from './types';

/** An axis's label (the shorthand's defaults to its path). */
export function axisLabel(axis: Axis | ShorthandAxis): string {
  return 'path' in axis ? (axis.label ?? axis.path) : axis.label;
}

/** What each run is summarized by, for the chart's y axis and the read-only panel. */
export function metricLabel(sweep: Sweep): string {
  const m = sweep.metric;
  switch (m.kind) {
    case 'final':
      return `${m.series} at t = ${sweep.ticks}`;
    case 'window_mean':
      return `${m.series}, mean over t = ${m.from}–${m.to ?? sweep.ticks}`;
    case 'timeseries':
      return `${m.series}, ${m.every}-tick means`;
  }
}

export function baseLabel(sweep: Pick<Sweep, 'base'>): string {
  return 'preset' in sweep.base ? `preset ${sweep.base.preset}` : 'a custom config';
}

/** The read-only panel's x entry: the tick for a time series, else the axis and its size. */
export function xLabel(sweep: Sweep): string {
  return sweep.metric.kind === 'timeseries' ? 'tick' : `${axisLabel(sweep.x)}: ${sweep.x.values.length} values`;
}
