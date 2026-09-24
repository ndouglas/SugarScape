import { afterEach, describe, expect, it, vi } from 'vitest';
import { Engine, type InitialState, type Speed } from '../engine';
import { fakeModule } from '../fake-sim.fixture';
import type { LogEntry } from '../protocol';
import { SimHost } from '../sim-host';
import { InlineTransport } from '../transport';
import type { Config, Preset } from '../types';
import { AdaptiveBatch, copyWorld, Lockstep, MAX_BATCH } from './lockstep';

const config = { width: 4, height: 3 } as unknown as Config;
const presets: Preset[] = [{ id: 'ii-2-unit', name: 'Unit', source: 'II-2', description: '', config }];
/** Lets queued microtasks and zero-delay timers run. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const create = (initial: InitialState) =>
  Engine.create(initial, { presets, transport: new InlineTransport(new SimHost(fakeModule())) });

async function pair(speed: Speed = 1, now?: () => number) {
  const a = await create({ config, seed: 1 });
  const b = await create({ config, seed: 2 });
  const lock = new Lockstep([a, b], speed, now);
  // A new coordinator first waits for both worlds to go quiet and compares their ticks.
  await lock.settled();
  return { a, b, lock };
}

/** Runs `k` animation frames of the lockstep loop. */
async function frames(lock: Lockstep, k: number): Promise<void> {
  for (let i = 0; i < k; i++) {
    lock.pump(i);
    await settle();
  }
}

describe('AdaptiveBatch', () => {
  it('doubles while pairs take ≤ 25 ms, halves above 40 ms, and stays within 1…10 000', () => {
    const batch = new AdaptiveBatch();
    expect(batch.n).toBe(1);
    batch.update(25);
    expect(batch.n).toBe(2);
    for (let i = 0; i < 20; i++) batch.update(1);
    expect(batch.n).toBe(MAX_BATCH);
    batch.update(30);
    expect(batch.n).toBe(MAX_BATCH);
    batch.update(41);
    expect(batch.n).toBe(MAX_BATCH / 2);
    for (let i = 0; i < 20; i++) batch.update(100);
    expect(batch.n).toBe(1);
  });
});

describe('Lockstep', () => {
  it('steps both worlds together, one pair of requests at a time', async () => {
    const { a, b, lock } = await pair(5);
    const ticks: number[] = [];
    lock.on('tick', () => ticks.push(a.tick));
    lock.setRunning(true);
    lock.pump(0);
    lock.pump(1); // a pair is still in flight: skipped
    await settle();
    expect([a.tick, b.tick]).toEqual([5, 5]);
    await frames(lock, 3);
    expect([a.tick, b.tick]).toEqual([20, 20]);
    expect(ticks).toEqual([5, 10, 15, 20]);
    lock.setRunning(false);
    await frames(lock, 2);
    expect([a.tick, b.tick]).toEqual([20, 20]);
    expect(a.running || b.running).toBe(false);
  });

  it('at Max, doubles the ticks per pair while replies are quick', async () => {
    const { a, b, lock } = await pair('max', () => 0); // every pair "takes" 0 ms
    lock.setRunning(true);
    await frames(lock, 4);
    expect([a.tick, b.tick]).toEqual([15, 15]); // 1 + 2 + 4 + 8
  });

  it('Step advances both; Reset rewinds both to t = 0, each replaying its log', async () => {
    const { a, b, lock } = await pair();
    await lock.advance(3);
    expect(await b.place(0, 2, {})).toBeNull();
    await lock.advance(2);
    await lock.reset();
    expect([a.tick, b.tick]).toEqual([0, 0]);
    expect([a.replayLeft, b.replayLeft]).toEqual([0, 1]);
    await lock.advance(3);
    expect([a.population, b.population]).toEqual([1, 2]);
  });

  it('rewinds the other world when one is rebuilt, even with a pair in flight', async () => {
    const { a, b, lock } = await pair(2);
    lock.setRunning(true);
    await frames(lock, 3);
    lock.pump(10); // a pair in flight
    expect(await a.resetWith((c) => void (c.height = 5))).toBeNull();
    await lock.settled();
    expect([a.tick, b.tick]).toEqual([0, 0]);
    expect(a.size()).toEqual({ width: 4, height: 5 });
    await frames(lock, 2);
    expect(a.tick).toBe(b.tick);
    lock.setRunning(false);
    await lock.settled();
    expect(await b.reset(undefined, 9)).toBeNull();
    await lock.settled();
    expect([a.tick, b.tick, b.seed]).toEqual([0, 0, 9]);
  });

  it('realigns worlds that start at different ticks', async () => {
    const a = await create({ config, seed: 1 });
    const b = await create({ config, seed: 2 });
    await a.advance(3);
    const lock = new Lockstep([a, b], 1);
    await lock.settled();
    expect([a.tick, b.tick]).toEqual([0, 0]);
  });
});

describe('Lockstep keeping step when things go wrong', () => {
  afterEach(() => vi.restoreAllMocks());

  it('compares ticks only once a world’s own step in flight has landed', async () => {
    const a = await create({ config, seed: 1 });
    const b = await create({ config, seed: 2 });
    a.setSpeed(5);
    a.setRunning(true);
    a.pump(0); // A's own step of 5 is in flight while the coordinator is built
    const lock = new Lockstep([a, b], 1);
    await lock.settled();
    await lock.advance(1);
    expect(a.tick).toBe(b.tick);
  });

  it('compares ticks only once a world’s Max run has stopped', async () => {
    const a = await create({ config, seed: 1 });
    const b = await create({ config, seed: 2 });
    a.setSpeed('max');
    a.setRunning(true);
    while (a.tick === 0) await settle();
    const lock = new Lockstep([a, b], 1);
    await lock.settled();
    await lock.advance(1);
    expect([a.tick, b.tick]).toEqual([1, 1]);
  });

  it('says so and stops when a world cannot rewind', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
    const { a, b, lock } = await pair();
    await lock.advance(3);
    vi.spyOn(b, 'replay').mockResolvedValue([{ field: 'simulation', message: 'boom' }]);
    await expect(lock.reset()).rejects.toThrow('boom');
    lock.setRunning(true);
    expect(await a.reset()).toBeNull(); // A is rebuilt; B cannot follow it back to t = 0
    await lock.settled();
    expect(lock.running).toBe(false);
    expect(warn).toHaveBeenCalledTimes(1);
    expect(String(warn.mock.calls[0][1])).toContain('boom');
  });

  it('stops when a world crashes, and Step and Reset then do nothing', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => {});
    const { a, b, lock } = await pair();
    lock.setRunning(true);
    await frames(lock, 2);
    await b.paint(0, 0, 1, -1); // "panics": B crashes
    expect(b.crashed).not.toBeNull();
    expect(lock.running).toBe(false);
    await lock.advance(3);
    await lock.reset();
    expect(a.tick).toBe(2);
  });
});

describe('copyWorld', () => {
  it('builds B from A’s session up to A’s tick and brings it to that tick', async () => {
    // A replays a session: the place at 3 has happened by tick 9, the erase at 20 is still pending.
    const log: LogEntry[] = [
      { tick: 3, cmd: { type: 'place', x: 0, y: 2, overrides: {} } },
      { tick: 20, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    const a = await create({ config, seed: 1, log });
    await a.advance(9);
    const { session, tick } = await a.session();
    expect(session.log).toEqual(log);
    const seen: [number, number][] = [];
    const b = await copyWorld(session, tick, create, (at, of) => seen.push([at, of]));
    expect(b.tick).toBe(9);
    expect([b.population, a.population]).toEqual([2, 2]);
    // The fake's fingerprint only counts ticks: check the replayed place itself — the agent
    // placed at (0, 2) at tick 3 stands there in both worlds, with the same id.
    await Promise.all([a.select(0, 2), b.select(0, 2)]);
    expect(b.selection).toEqual({ x: 0, y: 2, agentId: 2 });
    expect(b.selection).toEqual(a.selection);
    expect(b.seed).toBe(a.seed);
    // The place at 3 is copied; A's pending erase at 20 is not (Decision 9).
    expect((await b.session()).session.log).toEqual([log[0]]);
    expect(b.replayLeft).toBe(0);
    expect(seen[0]).toEqual([0, 9]);
    expect(seen.at(-1)).toEqual([9, 9]);
  });
});
