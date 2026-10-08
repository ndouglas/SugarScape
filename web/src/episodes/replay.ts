import type { EpisodeRecord } from './types';
/** Playback is presentation over an immutable successful record; it never calls the engine. */
export class ReplayController {
  record: EpisodeRecord | null = null;
  index = 0;
  playing = false;
  private timer: ReturnType<typeof setInterval> | null = null;
  constructor(private changed: () => void = () => {}, private reduced: () => boolean = () => globalThis.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false) {}
  load(record: EpisodeRecord): void { this.pause(); this.record = record; this.index = 0; this.changed(); }
  seek(index: number): void { this.pause(); this.index = Math.max(0, Math.min(Math.trunc(index), (this.record?.checkpoints.length ?? 1) - 1)); this.changed(); }
  step(delta = 1): void { this.seek(this.index + delta); }
  reset(): void { this.seek(0); }
  pause(): void { if (this.timer !== null) clearInterval(this.timer); this.timer = null; this.playing = false; }
  play(): void {
    if (this.reduced() || !this.record || this.playing || this.index >= this.record.checkpoints.length - 1) return;
    this.playing = true;
    this.timer = setInterval(() => {
      if (this.reduced()) { this.pause(); this.changed(); return; }
      this.index++;
      if (this.index >= this.record!.checkpoints.length - 1) this.pause();
      this.changed();
    }, 700);
    this.changed();
  }
}
