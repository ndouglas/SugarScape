// Civil violence's pure page helpers (milestone 11): its schedule as the Rules panel lists it.
import type { CivilConfig } from './types';

const show = (v: unknown): string => (typeof v === 'number' ? String(Number(v.toFixed(4))) : String(v));

/** The schedule's steps and ramps, one line each, by their first tick (a tick's steps before its ramps). */
export function scheduleLines(c: CivilConfig): string[] {
  const lines: [number, number, string][] = [];
  for (const s of c.schedule) {
    const sets = Object.entries(s.set)
      .map(([path, v]) => `${path} → ${show(v)}`)
      .join(', ');
    lines.push([s.tick, 0, `t = ${s.tick}: ${sets}`]);
  }
  for (const r of c.ramps) lines.push([r.start, 1, `t = ${r.start}–${r.end}: ${r.path} moves steadily to ${show(r.to)}`]);
  return lines.sort((a, b) => a[0] - b[0] || a[1] - b[1]).map(([, , line]) => line);
}
