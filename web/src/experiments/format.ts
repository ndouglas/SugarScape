import type { RunResult, ScalarRow } from './types';

/** Formats a summary value for display: 4 significant figures, or an em dash for null (NaN). */
export function fmt(v: number | null): string {
  return v === null ? '—' : Number(v.toPrecision(4)).toString();
}

/** A scalar summary row's table cells; `at` is the sweep's own value, printed unrounded. */
export function scalarCells(r: ScalarRow): string[] {
  return [r.series_name, String(r.at), String(r.n), fmt(r.mean), fmt(r.sd), fmt(r.min), fmt(r.max)];
}

/** Completion is distinct from metric availability: valid extinction is still a complete history. */
export function runAvailability(runs: RunResult[]): string {
  const statuses = runs.flatMap((run) => run.democratic_peace ? [run.democratic_peace] : []);
  if (statuses.length === 0) return '';
  const complete = statuses.filter((status) => status.status === 'complete');
  const invalid = statuses.filter((status) => status.status === 'invalid').length;
  const incomplete = statuses.filter((status) => status.status === 'incomplete').length;
  const reasons = new Map<string, number>();
  for (const status of complete) {
    if (status.clustering_reason) reasons.set(status.clustering_reason, (reasons.get(status.clustering_reason) ?? 0) + 1);
  }
  return `${complete.length} complete · ${invalid} invalid${incomplete ? ` · ${incomplete} short histories` : ''}${reasons.size ? ` · clustering unavailable: ${[...reasons].map(([reason, count]) => `${reason} (${count})`).join(', ')}` : ''}`;
}
