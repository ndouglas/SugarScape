import type { Engine, InitialState, Speed } from '../engine';
import type { Session } from '../protocol';

/** At Max in Compare, the ticks per pair double while a pair takes at most this long… */
export const FAST_MS = 25;
/** …and halve when it takes longer than this… */
export const SLOW_MS = 40;
/** …within 1 … MAX_BATCH. */
export const MAX_BATCH = 10_000;

/** Ticks per request, adapted to how long requests take (Decision 9). */
export class AdaptiveBatch {
  n = 1;

  update(ms: number): void {
    if (ms <= FAST_MS) this.n = Math.min(MAX_BATCH, this.n * 2);
    else if (ms > SLOW_MS) this.n = Math.max(1, Math.floor(this.n / 2));
  }
}

export type LockstepEvent = 'run' | 'tick';

/**
 * Steps two worlds together (Decision 9): each step sends `advance n` to both and waits for both
 * replies before the next, so their ticks are always equal. The worlds never run on their own
 * meanwhile. When one world is rebuilt (a 'reset' the coordinator did not cause), every world not at
 * t = 0 rewinds by replaying its session.
 */
export class Lockstep {
  running = false;
  private inFlight: Promise<void> | null = null;
  /** Steps, rewinds and realigns run one after another, each after the loop's pair settles. */
  private chain: Promise<unknown> = Promise.resolve();
  private holds = 0;
  /** Replays the coordinator started itself: their 'reset' events are not rebuilds. */
  private rewinding = 0;
  private readonly batch = new AdaptiveBatch();
  private readonly listeners = new Map<LockstepEvent, Set<() => void>>();
  private offs: (() => void)[] = [];

  constructor(
    readonly worlds: [Engine, Engine],
    public speed: Speed,
    private readonly now: () => number = () => performance.now(),
  ) {
    for (const w of worlds) {
      w.setRunning(false);
      this.offs.push(
        w.on('reset', () => {
          if (this.rewinding === 0) void this.realign();
        }),
      );
    }
    if (worlds[0].tick !== worlds[1].tick) void this.realign();
  }

  on(event: LockstepEvent, fn: () => void): () => void {
    let set = this.listeners.get(event);
    if (!set) this.listeners.set(event, (set = new Set()));
    set.add(fn);
    return () => {
      set.delete(fn);
    };
  }

  private emit(event: LockstepEvent): void {
    this.listeners.get(event)?.forEach((fn) => fn());
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  setSpeed(speed: Speed): void {
    this.speed = speed;
  }

  /** Called every animation frame instead of the engines' own pump. */
  pump(now: number = this.now()): void {
    if (this.holds > 0 || this.inFlight) return;
    if (!this.running) {
      // Paused: the engines still refresh what their panels want.
      for (const w of this.worlds) w.pump(now);
      return;
    }
    if (this.worlds.some((w) => w.crashed)) {
      this.setRunning(false);
      return;
    }
    const max = this.speed === 'max';
    const n = max ? this.batch.n : (this.speed as number);
    this.inFlight = this.stepBoth(n, max).finally(() => (this.inFlight = null));
  }

  /** Step: both worlds advance `n` ticks. */
  advance(n = 1): Promise<void> {
    return this.exclusive(() => this.stepBoth(n, false));
  }

  /** Reset: both worlds rewind to t = 0, each replaying its log. */
  reset(): Promise<void> {
    return this.exclusive(() => this.rewind(this.worlds));
  }

  /** Resolves once every step, rewind and realign queued so far has finished. */
  settled(): Promise<void> {
    return this.exclusive(async () => {});
  }

  dispose(): void {
    for (const off of this.offs) off();
    this.offs = [];
    this.listeners.clear();
  }

  private async stepBoth(n: number, adapt: boolean): Promise<void> {
    const start = this.now();
    await Promise.all(this.worlds.map((w) => w.advance(n)));
    if (adapt) this.batch.update(this.now() - start);
    this.emit('tick');
  }

  private exclusive<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    const run = this.chain.then(async () => {
      await this.inFlight;
      return fn();
    });
    this.chain = run.catch(() => undefined);
    return run.finally(() => this.holds--);
  }

  private async rewind(worlds: Engine[]): Promise<void> {
    this.rewinding++;
    try {
      await Promise.all(worlds.map((w) => w.replay()));
    } finally {
      this.rewinding--;
    }
    this.emit('tick');
  }

  /** After a rebuild: every world not at t = 0 replays its session (a few rounds, in case of races). */
  private realign(): Promise<void> {
    return this.exclusive(async () => {
      for (let round = 0; round < 3; round++) {
        const behind = this.worlds.filter((w) => w.tick !== 0);
        if (behind.length === 0) return;
        await this.rewind(behind);
      }
    });
  }
}

/**
 * Builds Compare's B (Decision 9): A's session with its log truncated to ticks ≤ `tick`, advanced to
 * `tick` in adaptive batches (its host replays the log on the way). Closes B if it cannot get there.
 */
export async function copyWorld(
  session: Session,
  tick: number,
  create: (initial: InitialState) => Promise<Engine>,
  progress: (at: number, of: number) => void = () => {},
  now: () => number = () => performance.now(),
): Promise<Engine> {
  const b = await create({ ...session, log: session.log.filter((e) => e.tick <= tick) });
  try {
    const batch = new AdaptiveBatch();
    progress(b.tick, tick);
    while (b.tick < tick) {
      const before = b.tick;
      const start = now();
      await b.advance(Math.min(batch.n, tick - b.tick));
      batch.update(now() - start);
      if (b.crashed) throw new Error(b.crashed);
      if (b.tick === before) throw new Error('the copy stopped advancing');
      progress(b.tick, tick);
    }
    return b;
  } catch (e) {
    b.close();
    throw e;
  }
}
