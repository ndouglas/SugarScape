import type { ScalarRow } from './types';

/** Formats a summary value for display: 4 significant figures, or an em dash for null (NaN). */
export function fmt(v: number | null): string {
  return v === null ? '—' : Number(v.toPrecision(4)).toString();
}

/** A scalar summary row's table cells; `at` is the sweep's own value, printed unrounded. */
export function scalarCells(r: ScalarRow): string[] {
  return [r.series_name, String(r.at), String(r.n), fmt(r.mean), fmt(r.sd), fmt(r.min), fmt(r.max)];
}
