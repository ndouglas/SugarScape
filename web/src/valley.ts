// The Long House Valley's page-side overlays and readout (milestone 10).
import { calendarYear, modelOf } from './models';
import type { ValleyOverlay, ValleyState } from './protocol';
import type { AnasaziConfig, ModelConfig, ValleyCellView } from './types';
import { wrappedSegments } from './ui/overlay';

/** The Display panel's checkboxes for the valley's overlays, in order. */
export const VALLEY_OVERLAY_LABELS: [ValleyOverlay, string][] = [
  ['water', 'Water sources'],
  ['settlements', 'Settlements'],
  ['links', 'Farm–home links'],
];

/** Overlay colors: water blue, settlements the Occupation mode's home yellow (core `anasazi::HOME`). */
export const WATER_COLOR = '#4fc3ff';
export const SETTLEMENT_COLOR = '#ffe04d';

/** A settlement's dot radius in pixels for `households` on cells `cell` pixels wide: grows with the square root, capped at 1.5 cells. */
export function settlementRadius(households: number, cell: number): number {
  return Math.min(1.5, 0.3 + 0.2 * Math.sqrt(households)) * cell;
}

/** Each triple of `[x, y, households, …]` as a settlement. */
export function settlements(state: ValleyState): { x: number; y: number; households: number }[] {
  const out: { x: number; y: number; households: number }[] = [];
  const s = state.settlements;
  for (let i = 0; i + 2 < s.length; i += 3) out.push({ x: s[i], y: s[i + 1], households: s[i + 2] });
  return out;
}

/**
 * The segments that draw a farm–home link from (ax, ay) to (bx, by) on a `width` × `height` map:
 * split across the edges when the world measures distance around them (`quirks.wrap_edges`, on
 * unless a config says otherwise, as in the core), else one straight segment.
 */
export function linkSegments(
  config: ModelConfig,
  ax: number,
  ay: number,
  bx: number,
  by: number,
  width: number,
  height: number,
): [number, number, number, number][] {
  const wraps = modelOf(config) === 'anasazi' && (config as AnasaziConfig).quirks?.wrap_edges !== false;
  return wraps ? wrappedSegments(ax, ay, bx, by, width, height) : [[ax, ay, bx, by]];
}

/** A world for the toolbar's readout. */
export interface ReadoutWorld { config: ModelConfig; tick: number; population: number }

/**
 * The toolbar's readout: `t = 42 · 400 agents`, or for the anasazi the calendar year and its
 * households (`AD 1142 · 213 households`); in Compare both worlds' counts.
 */
export function readoutText(a: ReadoutWorld, b: ReadoutWorld | null): string {
  const year = calendarYear(a.config, a.tick);
  const when = year === null ? `t = ${a.tick}` : `AD ${year}`;
  const noun = year === null ? 'agents' : 'households';
  return b ? `${when} · A ${a.population} · B ${b.population} ${noun}` : `${when} · ${a.population} ${noun}`;
}

/** The five PDSI classes of the yield table (JASSS ¶2.11), driest first. */
export const PDSI_CLASSES = ['(−∞, −3]', '(−3, −1]', '(−1, 1)', '[1, 3)', '[3, ∞)'];

/** Where a cell stands with water: a source itself, within the mile farms need, or beyond it. */
export function waterText(site: Pick<ValleyCellView, 'water' | 'water_near'>): string {
  if (site.water) return 'a water source';
  return site.water_near ? 'within a mile of water' : 'more than a mile from water';
}
