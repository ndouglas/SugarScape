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

/** A file-name stem for a sweep. */
export function slug(name: string): string {
  return (
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'sweep'
  );
}
