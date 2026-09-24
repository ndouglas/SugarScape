import { chartKey, type ChartGroup } from '../protocol';

/** uPlot data: x values, then one array per line (null is a gap). */
export type LineData = [number[], ...(number | null)[][]];

/** NaN (no trades, no agents) is a gap in the line. */
const gap = (v: number): number | null => (Number.isNaN(v) ? null : v);

/** A time chart's data: the group's ticks as x, then one line per series. */
export function lineData(g: ChartGroup): LineData {
  return [Array.from(g.ticks), ...g.columns.map((c) => Array.from(c, gap))];
}

/** The Trade price chart (`[mean_log_price, sd_log_price]`): the mean, mean + SD and mean − SD. */
export function bandData(g: ChartGroup): LineData {
  const [mean, sd] = g.columns;
  const line = (f: (m: number, s: number) => number) => Array.from(mean, (m, i) => gap(f(m, sd[i])));
  return [Array.from(g.ticks), line((m) => m), line((m, s) => m + s), line((m, s) => m - s)];
}

/**
 * Tracks each chart group's last known tick, so a panel asks for a group again only when it is
 * unfilled or behind the current tick (Decision 4, PF6) — not on every poll while paused.
 */
export class ChartFreshness {
  private ticks = new Map<string, number>();

  /** Marks every group unfilled: call on a rebuild, a reset, a config change, or the tab opening. */
  reset(): void {
    this.ticks.clear();
  }

  /** Records the last tick each group in a snapshot's `charts` reached. */
  receive(charts: Record<string, ChartGroup> | undefined): void {
    for (const [key, g] of Object.entries(charts ?? {})) {
      const last = g.ticks.at(-1);
      if (last !== undefined) this.ticks.set(key, last);
    }
  }

  /** True when some group in `groups` has never arrived, or its last tick is behind `tick`. */
  behind(groups: string[][], tick: number): boolean {
    return groups.some((g) => {
      const last = this.ticks.get(chartKey(g));
      return last === undefined || last < tick;
    });
  }
}
