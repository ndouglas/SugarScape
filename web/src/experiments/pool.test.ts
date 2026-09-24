import { describe, expect, it } from 'vitest';
import { poolSize, WorkerPool, type PointReply, type PointRequest, type WorkerLike } from './pool';
import { parseErrors } from '../types';
import type { RunResult } from './types';

/** Answers each request after a short, index-dependent delay (so replies arrive out of order). */
class FakeWorker implements WorkerLike {
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;
  terminated = false;

  constructor(private log: number[], private failAt?: number, private erroredAt?: number) {}

  postMessage(message: unknown): void {
    const { index } = message as PointRequest;
    this.log.push(index);
    setTimeout(() => {
      if (this.terminated) return;
      if (index === this.erroredAt) {
        this.onerror?.({ message: 'worker crashed' } as ErrorEvent);
        return;
      }
      const reply: PointReply =
        index === this.failAt
          ? { index, errors: [{ field: 'points', message: 'boom' }] }
          : { index, run: { point: index, series: 0, x: 0, seed: 1, value: index } };
      this.onmessage?.({ data: reply } as MessageEvent);
    }, (index * 7) % 5);
  }

  terminate(): void {
    this.terminated = true;
  }
}

function fakes(failAt?: number, erroredAt?: number) {
  const log: number[] = [];
  const workers: FakeWorker[] = [];
  const create = () => {
    const w = new FakeWorker(log, failAt, erroredAt);
    workers.push(w);
    return w;
  };
  return { log, workers, create };
}

describe('worker pool', () => {
  it('leaves a core for the page', () => {
    expect(poolSize(8)).toBe(7);
    expect(poolSize(1)).toBe(1);
    expect(poolSize(undefined)).toBe(1);
  });

  it('hands out every index once, in order, and terminates its workers', async () => {
    const { log, workers, create } = fakes();
    const runs: RunResult[] = [];
    const outcome = await new WorkerPool(3, create).run('{}', 10, (run) => runs.push(run));
    expect(outcome).toBe('done');
    expect(log).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    expect(runs.map((r) => r.point).sort((a, b) => a - b)).toEqual(log);
    expect(workers).toHaveLength(3);
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('creates no more workers than points', async () => {
    const { workers, create } = fakes();
    await new WorkerPool(8, create).run('{}', 2, () => {});
    expect(workers).toHaveLength(2);
    expect(await new WorkerPool(8, create).run('{}', 0, () => {})).toBe('done');
    expect(workers).toHaveLength(2);
  });

  it('stops on cancel and keeps what finished', async () => {
    const { workers, create } = fakes();
    const pool = new WorkerPool(3, create);
    const runs: RunResult[] = [];
    const outcome = await pool.run('{}', 50, (run) => {
      runs.push(run);
      if (runs.length === 4) pool.cancel();
    });
    expect(outcome).toBe('cancelled');
    expect(runs).toHaveLength(4);
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('rejects on a point error', async () => {
    const { workers, create } = fakes(5);
    await expect(new WorkerPool(2, create).run('{}', 10, () => {})).rejects.toThrow('points: boom');
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('rejects and stops the run on a worker error event', async () => {
    const { workers, create } = fakes(undefined, 3);
    await expect(new WorkerPool(2, create).run('{}', 10, () => {})).rejects.toThrow('worker crashed');
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('rejects and terminates its workers when onResult throws', async () => {
    const { workers, create } = fakes();
    const pool = new WorkerPool(2, create);
    await expect(
      pool.run('{}', 10, () => {
        throw new Error('bad callback');
      }),
    ).rejects.toThrow('bad callback');
    expect(workers.every((w) => w.terminated)).toBe(true);
  });
});

describe('worker errors', () => {
  it('labels errors that are not field errors with the given field', () => {
    expect(parseErrors(new Error('WASM failed to load'), 'worker')).toEqual([{ field: 'worker', message: 'WASM failed to load' }]);
    expect(parseErrors('boom')).toEqual([{ field: 'config', message: 'boom' }]);
    expect(parseErrors('[{"field":"ticks","message":"m"}]', 'worker')).toEqual([{ field: 'ticks', message: 'm' }]);
  });
});
