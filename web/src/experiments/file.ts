import { parseErrors } from '../types';
import type { Sweep, SweepResult } from './types';

export type Opened = { kind: 'sweep'; sweep: Sweep } | { kind: 'result'; result: SweepResult } | { kind: 'error'; message: string };

const isObject = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);

/** What an opened JSON file (or link payload) is. The core validates the contents when it is used. */
export function classifyFile(json: unknown): Opened {
  if (!isObject(json)) return { kind: 'error', message: 'expected a sweep or a sweep result (a JSON object)' };
  if ('version' in json || 'runs' in json || 'summary' in json) {
    if (json.version !== 1) return { kind: 'error', message: `unsupported result version ${String(json.version)}` };
    if (!isObject(json.sweep) || !Array.isArray(json.runs)) return { kind: 'error', message: 'a result needs "sweep" and "runs"' };
    return { kind: 'result', result: json as unknown as SweepResult };
  }
  if ('base' in json && 'x' in json && 'metric' in json) return { kind: 'sweep', sweep: json as unknown as Sweep };
  return { kind: 'error', message: 'expected a sweep (with base, x and metric) or a sweep result' };
}

/** The core calls `readOpened` needs (the WASM functions of the same names; tests pass fakes). */
export interface OpenCore {
  /** The sweep in full form, or throws JSON field errors. */
  parseSweep(spec: string): string;
  /** The runs' summary, or throws JSON field errors when they are not the sweep's. */
  aggregate(spec: string, runs: string): string;
}

/**
 * What an opened file or link payload is, checked by the core before anything
 * is shown: a sweep (in full form) or a result whose runs belong to its sweep.
 */
export function readOpened(json: unknown, core: OpenCore): Opened {
  const opened = classifyFile(json);
  if (opened.kind === 'error') return opened;
  try {
    const raw = opened.kind === 'sweep' ? opened.sweep : opened.result.sweep;
    const spec = core.parseSweep(JSON.stringify(raw));
    const sweep = JSON.parse(spec) as Sweep;
    if (opened.kind === 'sweep') return { kind: 'sweep', sweep };
    core.aggregate(spec, JSON.stringify(opened.result.runs));
    return { kind: 'result', result: { ...opened.result, sweep } };
  } catch (e) {
    return { kind: 'error', message: parseErrors(e, 'file').map((f) => `${f.field}: ${f.message}`).join('; ') };
  }
}

/** A file-name stem for a sweep. */
export function slug(name: string): string {
  return (
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'sweep'
  );
}
