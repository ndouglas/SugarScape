/** Ticks per second, measured from (time, tick) samples over a sliding window. */
export class RateMeter {
  private samples: { now: number; tick: number }[] = [];

  constructor(private readonly windowMs = 1000) {}

  sample(now: number, tick: number): void {
    const last = this.samples.at(-1);
    if (last && tick < last.tick) this.samples = [];
    this.samples.push({ now, tick });
    // Keep one sample at or beyond the window's start, so the window is always covered.
    while (this.samples.length > 2 && this.samples[1].now <= now - this.windowMs) this.samples.shift();
  }

  rate(): number | null {
    const first = this.samples[0];
    const last = this.samples.at(-1);
    if (!first || !last || last.now <= first.now) return null;
    return ((last.tick - first.tick) * 1000) / (last.now - first.now);
  }

  reset(): void {
    this.samples = [];
  }
}
