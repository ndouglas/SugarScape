import { axisLabel, metricLabel } from './labels';
import { fmt } from './format';
import type { Summary, Sweep } from './types';

/** One chart line: per x position, the mean over seeds and its ±1 sd band. `null` is a gap. */
export interface ChartLine {
  name: string;
  mean: (number | null)[];
  sd: (number | null)[];
  lo: (number | null)[];
  hi: (number | null)[];
  min: (number | null)[];
  max: (number | null)[];
  n: number[];
}

export interface ChartData { xLabel: string; yLabel: string; x: number[]; lines: ChartLine[] }

interface Cell { mean: number | null; sd: number | null; n: number; min?: number | null; max?: number | null }

const finite = (v: number | null | undefined): number | null => (typeof v === 'number' && Number.isFinite(v) ? v : null);

const EMPTY: Cell = { mean: null, sd: null, n: 0 };

function lineOf(name: string, cells: Cell[]): ChartLine {
  const line: ChartLine = { name, mean: [], sd: [], lo: [], hi: [], min: [], max: [], n: [] };
  for (const cell of cells) {
    const mean = finite(cell.mean);
    const sd = finite(cell.sd);
    line.mean.push(mean);
    line.sd.push(sd);
    line.lo.push(mean !== null && sd !== null ? mean - sd : null);
    line.hi.push(mean !== null && sd !== null ? mean + sd : null);
    line.min.push(finite(cell.min));
    line.max.push(finite(cell.max));
    line.n.push(cell.n);
  }
  return line;
}

/** The core's summary as chart arrays: x = `at` (sorted) for scalar metrics, the tick for time series. */
export function chartData(sweep: Sweep, summary: Summary): ChartData {
  const yLabel = metricLabel(sweep);
  if (summary.kind === 'timeseries') {
    const rows = summary.rows;
    const ticks = [...new Set(rows.map((r) => r.t))].sort((a, b) => a - b);
    const lines = [...new Set(rows.map((r) => r.series))].map((s) => {
      const own = rows.filter((r) => r.series === s);
      return lineOf(
        own[0].series_name,
        ticks.map((t) => own.find((r) => r.t === t) ?? EMPTY),
      );
    });
    return { xLabel: 'Tick', yLabel, x: ticks, lines };
  }
  const rows = summary.rows;
  const positions = [...new Map(rows.map((r) => [r.x, r.at] as const)).entries()].sort((a, b) => a[1] - b[1]);
  const lines = [...new Set(rows.map((r) => r.series))].map((s) => {
    const own = rows.filter((r) => r.series === s);
    return lineOf(
      own[0].series_name,
      positions.map(([x]) => own.find((r) => r.x === x) ?? EMPTY),
    );
  });
  return { xLabel: axisLabel(sweep.x), yLabel, x: positions.map(([, at]) => at), lines };
}

/** The legend text for a hovered point: mean ± sd, the range over seeds where known, and n. */
export function describePoint(line: ChartLine, i: number): string {
  const mean = line.mean[i];
  const n = line.n[i] ?? 0;
  if (mean === null || mean === undefined) return `— (n = ${n})`;
  const min = line.min[i];
  const max = line.max[i];
  const range = min !== null && min !== undefined && max !== null && max !== undefined ? `, ${fmt(min)}–${fmt(max)}` : '';
  return `${fmt(mean)} ± ${fmt(line.sd[i])}${range}, n = ${n}`;
}
