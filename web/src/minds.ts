// Minds 5–6 on the page: the season, the lab's schedule and the per-site cache marks, as text and
// shapes the grid view, toolbar, charts and Inspect draw (the core's `MindsView`).
import { CACHE_CHEATER, CACHE_HOARDER, CACHE_LARDER, type Config, type LabView, type MindsView, type SpatialHoardingInspect, type SiteCacheView } from './types';

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** Whether the tick shown ended in winter (`seasons.mode: global` only); false with no view. */
export function winterShown(minds: MindsView | null): boolean {
  return minds?.winter === true;
}

/**
 * The winter spans of ticks `from`…`to` for the charts' shading, `[start, end]` in tick units:
 * tick `t` (the state after `t` ticks) ended in winter when `(t − 1) mod 2γ ≥ γ`, as the core's
 * `growback::is_winter` of the tick just computed, so each band runs from γ(2k + 1) to 2γ(k + 1).
 * Empty unless seasons are on with `mode: 'global'`.
 */
export function winterBands(config: Config, from: number, to: number): [number, number][] {
  const s = config.seasons;
  if (!s?.enabled || s.mode !== 'global' || !(s.period > 0) || !(to > from)) return [];
  const p = s.period;
  const out: [number, number][] = [];
  for (let k = Math.max(0, Math.floor(from / (2 * p)) - 1); (2 * k + 1) * p < to; k++) {
    const start = Math.max(from, (2 * k + 1) * p);
    const end = Math.min(to, 2 * (k + 1) * p);
    if (end > start) out.push([start, end]);
  }
  return out;
}

/** "K1", "K2", "K3" for compartment 0–2. */
export const compartmentName = (k: number): string => `K${k + 1}`;

/**
 * The lab's status line: "Day d · morning in K# · food" (or "· none"), "Day d · evening, on the
 * perches", "Test evening · #id's turn", "Test evening · done", or before the first morning.
 */
export function labStatus(lab: LabView): string {
  switch (lab.phase) {
    case 'start':
      return `Day 1 of ${lab.days} · before the first morning`;
    case 'morning':
      return `Day ${lab.day} of ${lab.days} · morning in ${compartmentName(lab.place ?? 0)} · ${lab.food ? 'food' : 'none'}`;
    case 'evening':
      return `Day ${lab.day} of ${lab.days} · evening, on the perches`;
    case 'test':
      return lab.turn != null ? `Test evening · agent #${lab.turn}'s turn` : 'Test evening';
    case 'done':
      return 'Test evening · done';
  }
}

/** One pass over `cache_sites`' flat `[x, y, total, flags, …]`: the fullest site, and what kinds are there. */
export interface CacheSummary {
  /** The largest total at any site (0 with none). */
  max: number;
  /** Some site holds caches owned only by cheaters. */
  cheaterOnly: boolean;
}

export function cacheSummary(flat: ArrayLike<number>): CacheSummary {
  let max = 0;
  let cheaterOnly = false;
  for (let i = 0; i + 3 < flat.length; i += 4) {
    if (flat[i + 2] > max) max = flat[i + 2];
    if (isCheaterOnly(flat[i + 3])) cheaterOnly = true;
  }
  return { max, cheaterOnly };
}

/** Whether a site's flags say only cheaters own caches there (tinted apart). */
export const isCheaterOnly = (flags: number): boolean => (flags & CACHE_CHEATER) !== 0 && (flags & CACHE_HOARDER) === 0;

/** Whether a site's flags include a larder (drawn with the homes). */
export const isLarder = (flags: number): boolean => (flags & CACHE_LARDER) !== 0;

/**
 * A site's diamond half-width as a share of a cell: 0.12 for a near-empty site, up to 0.3 for the
 * fullest, by the square root of its total against `max` (area goes with sugar).
 */
export function cacheSize(total: number, max: number): number {
  return 0.12 + 0.18 * (max > 0 ? Math.sqrt(Math.max(0, total) / max) : 0);
}

/** A site's "Caches here" text: each owner and amount ("#12: 4.5 · #30 (cheater): 2"), or "none". */
export function siteCachesText(caches: SiteCacheView[], cheaters: boolean, max = 6): string {
  if (caches.length === 0) return 'none';
  const shown = caches.slice(0, max).map((c) => `#${c.owner}${cheaters && c.cheater_owner ? ' (cheater)' : ''}: ${fmt(c.amount)}${c.kind ? ` (${c.kind})` : ''}`);
  const more = caches.length - shown.length;
  return shown.join(' · ') + (more > 0 ? ` · and ${more} more` : '');
}

/** A lab agent's frozen test-evening allocation of F: "K1 15 · K3 15", or "nothing" when it caches none. */
export function allocationText(alloc: [number, number][]): string {
  const parts = alloc.filter(([, q]) => q > 0).map(([k, q]) => `${compartmentName(k)} ${fmt(q)}`);
  return parts.length ? parts.join(' · ') : 'nothing';
}

/** The Age row: "a / max" where agents die of old age (lifespan on), else just "a". */
export function ageText(age: number, maxAge: number, lifespan: boolean): string {
  return lifespan ? `${age} / ${maxAge}` : String(age);
}

/** Spatial fields remain separate from scatter caching; delivery is still part of holdings. */
export function spatialHoardingRows(s: SpatialHoardingInspect): [string, string][] {
  return [
    ['Home', `(${s.home.x}, ${s.home.y})`],
    ['Larder probability L', fmt(s.larder_trait)], ['Defense propensity D', fmt(s.defense_trait)],
    ['Larder', `${fmt(s.larder)} at home`],
    ['Delivery', s.delivery == null ? 'none' : `${fmt(s.delivery)} pending (still carried)`],
    ['Guarding', s.guarding ? 'yes' : 'no'], ['Observed larders', String(s.observed_larders)],
  ];
}
