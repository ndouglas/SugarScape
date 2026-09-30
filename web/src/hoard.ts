// Minds 7 (Vander Wall and Jenkins's genetic algorithm): the page's words for where a run is.
import type { HoardConfig, HoardStatus } from './types';

/**
 * The bout just run, from the next one to run (`day`, `bout`, both 1-based, as the core keeps
 * them); null before the season's first bout. A finished season's next is day `days` + 1, bout 1.
 */
export function lastBout(day: number, bout: number, bouts: number): { day: number; bout: number } | null {
  if (bout > 1) return { day, bout: bout - 1 };
  return day > 1 ? { day: day - 1, bout: bouts } : null;
}

/** "Generation 3 · day 12 · bout 5 · public food 64", the bout just run; "· season over" once it has ended. */
export function hoardStatusText(s: HoardStatus, c: Pick<HoardConfig, 'bouts'>): string {
  const last = lastBout(s.day, s.bout, c.bouts);
  const when = last ? `day ${last.day} · bout ${last.bout}` : 'before the first bout';
  return `Generation ${s.generation} · ${when} · public food ${s.public}${s.season_over ? ' · season over' : ''}`;
}

/**
 * The frame columns `[from, to)` of the agent whose column holds `x`, of `n` in a frame `wide`
 * cells across (the core's `view::agent_at` and `view::columns`, less the gutter).
 */
export function hoardColumn(x: number, n: number, wide: number): [number, number] {
  const i = Math.min(Math.floor((x * n) / wide), n - 1);
  const from = Math.floor((i * wide) / n);
  const to = Math.max(Math.floor(((i + 1) * wide) / n) - 1, from + 1);
  return [from, Math.min(to, wide)];
}
