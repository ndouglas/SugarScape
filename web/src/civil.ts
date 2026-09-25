// Civil violence's pure page helpers (milestone 11): its schedule as the Rules panel lists it.
import type { CitizenView, CivilConfig, CivilInspection } from './types';

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

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** An agent's Inspect rows: its state, H, R, G and (while free) the risk it runs; in Model II its group and age. */
export function citizenRows(a: CitizenView): [string, string][] {
  const state = a.state === 'jailed' ? `jailed (${a.jail_life ? 'for life' : `${a.jail_left} ticks left`})` : a.state;
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · ${state}`],
    ['Hardship (H)', fmt(a.hardship)],
    ['Risk aversion (R)', fmt(a.risk_aversion)],
    ['Grievance (G)', `${fmt(a.grievance)} = H × (1 − L)`],
  ];
  if (a.state !== 'jailed') rows.push(['Arrest risk (P)', `${fmt(a.arrest_probability)} · net risk R·P ${fmt(a.net_risk)}`]);
  if (a.group !== null) rows.push(['Group', a.group === 'blue' ? 'Blue' : 'Green'], ['Age', `${a.age} of ${a.death_age}`]);
  return rows;
}

/** The agent Inspect shows at a civil site: the one it follows (free there, or jailed after arrest there), else the free occupant. */
export function shownCitizen(view: CivilInspection, followed: number | null): CitizenView | null {
  if (followed !== null) {
    if (view.agent?.id === followed) return view.agent;
    const jailed = view.jailed.find((a) => a.id === followed);
    if (jailed) return jailed;
  }
  return view.agent;
}
