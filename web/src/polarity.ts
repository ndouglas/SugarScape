import type { PolarityConfig, PolarityInspection } from './types';
const resource = (v: number | null): string => typeof v === 'number' && Number.isFinite(v) ? v.toFixed(3) : 'unavailable';
interface FrontView { key: { a: number; b: number }; actions: boolean[]; previous: boolean[]; commitments: (number | null)[]; path: number[] | null }
const actions = (v: boolean[]) => v.map(x => x ? 'D' : 'C').join('/');
const fronts = (v: unknown): string => Array.isArray(v) && v.length ? (v as FrontView[]).map(f => `${f.key.a}–${f.key.b}: ${actions(f.actions)} · previous ${actions(f.previous)} · resources ${f.commitments.map(resource).join('/')} · path ${f.path?.join(' → ') ?? 'None'}`).join('; ') : 'None';
const trust = (v: unknown): string => Array.isArray(v) && v.length ? (v as {neighbor: number; trust: number | null}[]).map(t => `${t.neighbor}: ${resource(t.trust)}`).join('; ') : 'None';
const event = (v: unknown): string => {
  if (v == null) return 'None';
  const e = v as { id: number; period: number; kind: string; cells: number[]; front: number[] | null };
  return `#${e.id} · period ${e.period} · ${e.kind}${e.cells.length ? ` · cells ${e.cells.join(', ')}` : ''}${e.front ? ` · front ${e.front.join('–')}` : ''}`;
};
/** Sovereignty and resources stay separate from defensive coalition membership. */
export function polarityRows(view: PolarityInspection): [string, string][] {
  const rows: [string, string][] = [
    ['Government', `Cell ${view.cell.id} · capital ${view.capital} · latent ${view.cell.predator ? 'predator' : 'status quo'}`],
    ['Resources', `Province ${resource(view.province_stock)} · corporate ${resource(view.corporate_stock)}`],
    ['Territory', `${view.members.length} cells: ${view.members.join(', ')}`],
    ['Sovereign neighbors', view.neighbors.join(', ') || 'None'],
    ['Trust', trust(view.trust)], ['Prime threat', view.threat === null ? 'None' : String(view.threat)],
    ['Defensive coalition', Array.isArray(view.coalition) ? view.coalition.join(', ') || 'None' : 'None'], ['Foreign fronts', fronts(view.foreign_fronts)],
    ['Domestic fronts', fronts(view.domestic_fronts)], ['Last structural event', event(view.last_event)],
  ];
  if (view.periods !== undefined) rows.unshift(['Session', `${view.periods} of ${view.horizon ?? 'unavailable'} periods`]);
  if (view.invalidity) rows.push(['Invalid reconstruction', view.invalidity]);
  else if (view.finish_reason) rows.push(['Completion', view.finish_reason]);
  if (view.events !== undefined) {
    rows.push(['Event log', `${view.events.length} retained · cap ${view.event_log_limit ?? 'unavailable'} · ${view.events_dropped ?? 0} older events dropped`]);
    for (const entry of view.events.slice(-10)) rows.push(['Recent event', event(entry)]);
  }
  return rows;
}
/** Charts retain display ticks and state the economic period batching for each world. */
export function polarityChartCaption(title: string, configs: PolarityConfig[]): string {
  return `${title} · ${configs.map((c, i) => `${configs.length > 1 ? `${i === 0 ? 'A' : 'B'}: ` : ''}${c.periods_per_tick} periods/tick`).join(' · ')}`;
}

export function polarityLegend(config: PolarityConfig, mode: import('./types').ColorMode, periods: number): string {
  const meaning = mode === 'coalitions' ? 'Defensive coalitions do not transfer sovereignty' : mode === 'strategy' ? 'Latent predator/status-quo strategy is restored on independence' : mode === 'resources' ? 'Primitive cell stock; corporate control is shown in Inspect' : 'Colors identify sovereign ownership; black borders separate governments';
  return `${periods} of ${config.horizon} periods · ${config.periods_per_tick} periods/tick · ${config.source_profile === 'chapter4' ? 'Cederman 1994 / chapter 4' : 'Cederman 1997 / chapter 5'} reconstruction · ${meaning} · White center: capital · orange/green corner: latent predator/status quo`;
}
