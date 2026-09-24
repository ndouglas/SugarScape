import type { DiseaseEntry } from '../types';

/** `[value, label]` options for a disease picker; value `-1` is "New disease". */
export function diseaseOptions(list: DiseaseEntry[], allowNew: boolean): [string, string][] {
  const options: [string, string][] = list.map((d) => [String(d.id), `#${d.id} · ${d.bits}`]);
  return allowNew ? [['-1', 'New disease'], ...options] : options;
}

/**
 * When a disease tool asks for the disease list: at most once per `minMs`, and only when the list
 * may have changed since the last one (the tick moved, or an edit, reset or config change) or the
 * tool was just opened. So a paused world with a disease tool open asks for nothing.
 */
export class DiseaseListPoll {
  private at = -Infinity;
  private tick = -1;
  private stale = true;

  constructor(private minMs: number) {}

  /** The world may have changed without a tick (an edit or a config change). */
  invalidate(): void {
    this.stale = true;
  }

  /** Asks for the list at once: the tool was just opened, or the world was replaced. */
  expedite(): void {
    this.stale = true;
    this.at = -Infinity;
  }

  due(now: number, tick: number): boolean {
    return (this.stale || tick !== this.tick) && now - this.at >= this.minMs;
  }

  /** A list for the world at `tick` arrived at `now`. */
  received(now: number, tick: number): void {
    this.at = now;
    this.tick = tick;
    this.stale = false;
  }
}
