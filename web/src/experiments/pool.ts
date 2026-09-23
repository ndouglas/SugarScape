import type { FieldError } from '../types';
import type { RunResult } from './types';

export interface PointRequest { spec: string; index: number }
export type PointReply = { index: number; run: RunResult } | { index: number; errors: FieldError[] };

/** The part of `Worker` the pool uses (tests pass fakes). */
export interface WorkerLike {
  onmessage: ((event: MessageEvent) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage(message: unknown): void;
  terminate(): void;
}

/** `max(1, hardwareConcurrency − 1)`: leave a core for the page (Decision 19). */
export function poolSize(hardwareConcurrency: number | undefined): number {
  return Math.max(1, (hardwareConcurrency ?? 2) - 1);
}

export type PoolOutcome = 'done' | 'cancelled';

/**
 * Runs a sweep's points on workers, one point per worker at a time, handing
 * out indices in order. Workers are created per run and terminated when it
 * ends, fails or is cancelled.
 */
export class WorkerPool {
  private workers: WorkerLike[] = [];
  /** Ends the current run as cancelled; null when no run is active. */
  private stop: (() => void) | null = null;

  constructor(
    private size: number,
    private create: () => WorkerLike,
  ) {}

  run(spec: string, count: number, onResult: (run: RunResult) => void): Promise<PoolOutcome> {
    return new Promise<PoolOutcome>((resolve, reject) => {
      if (count === 0) {
        resolve('done');
        return;
      }
      let next = 0;
      let finished = 0;
      const end = (settle: () => void) => {
        for (const w of this.workers) w.terminate();
        this.workers = [];
        this.stop = null;
        settle();
      };
      this.stop = () => end(() => resolve('cancelled'));
      const feed = (w: WorkerLike) => {
        if (next < count) {
          const request: PointRequest = { spec, index: next++ };
          w.postMessage(request);
        }
      };
      for (let i = 0; i < Math.min(this.size, count); i++) {
        const w = this.create();
        w.onmessage = (event: MessageEvent) => {
          if (this.stop === null) return;
          const reply = event.data as PointReply;
          if ('errors' in reply) {
            const message = reply.errors.map((e) => `${e.field}: ${e.message}`).join('; ');
            end(() => reject(new Error(message)));
            return;
          }
          onResult(reply.run);
          finished += 1;
          if (finished === count) end(() => resolve('done'));
          else if (this.stop !== null) feed(w);
        };
        w.onerror = (event: ErrorEvent) => {
          if (this.stop !== null) end(() => reject(new Error(event.message || 'a sweep worker failed')));
        };
        this.workers.push(w);
      }
      for (const w of this.workers) feed(w);
    });
  }

  cancel(): void {
    this.stop?.();
  }
}
