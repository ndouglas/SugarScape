// Mirrors of the sweep JSON shapes in sugarscape-core's `sweep` module.
import type { Config } from '../types';

export interface AxisValue { at: number; name?: string; set: Record<string, unknown> }
/** The full form (what the core writes). */
export interface Axis { label: string; values: AxisValue[] }
/** The shorthand: value i sets `path` and sits at the value itself (numbers) or at i. */
export interface ShorthandAxis { label?: string; path: string; values: unknown[] }
export type SweepBase = { preset: string } | { config: Config };
export type Metric =
  | { kind: 'final'; series: string }
  | { kind: 'window_mean'; series: string; from: number; to?: number }
  | { kind: 'timeseries'; series: string; every: number };
export interface Sweep {
  name: string;
  description?: string;
  base: SweepBase;
  set?: Record<string, unknown>;
  x: Axis | ShorthandAxis;
  series?: Axis | ShorthandAxis;
  seeds: { from: number; count: number };
  ticks: number;
  metric: Metric;
}
export interface Point { index: number; series: number; x: number; seed: number }
interface RunBase { point: number; series: number; x: number; seed: number }
/** `null` stands for NaN. */
export type RunResult = (RunBase & { value: number | null }) | (RunBase & { values: (number | null)[] });
export interface ScalarRow {
  series: number;
  series_name: string;
  x: number;
  at: number;
  n: number;
  nan: number;
  mean: number | null;
  sd: number | null;
  min: number | null;
  max: number | null;
}
export interface BlockRow { series: number; series_name: string; t: number; n: number; mean: number | null; sd: number | null }
export type Summary = { kind: 'scalar'; rows: ScalarRow[] } | { kind: 'timeseries'; rows: BlockRow[] };
export interface SweepResult { version: 1; sweep: Sweep; runs: RunResult[]; summary: Summary; incomplete?: boolean }
export interface BuiltinSweep { id: string; sweep: Sweep }
