import type { Engine, InitialState, Speed } from '../engine';
import { fieldErrorsMessage } from '../errors';
import type { Session } from '../protocol';
import type { FieldError } from '../types';

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
        // A dead world cannot keep step: stop, and Step/Reset do nothing from now on.
        w.on('crash', () => {
          if (this.running) this.setRunning(false);
        }),
        // Both reach the cap on the same step (they are in step, and each host stops there): pause.
        w.on('full', () => {
          if (this.running) this.setRunning(false);
        }),
      );
    }
    // Compared only once both are quiet: a step (or Max's run) still in flight would move a tick
    // after the comparison and leave the worlds unequal for good. `session()` waits for that.
    void this.guard(async () => {
      await Promise.all(worlds.map((w) => (w.crashed ? null : w.session())));
      if (worlds[0].tick !== worlds[1].tick) await this.align();
    });
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
    if (this.crashed()) {
      this.setRunning(false);
      return;
    }
    const max = this.speed === 'max';
    const n = max ? this.batch.n : (this.speed as number);
    this.inFlight = this.stepBoth(n, max).finally(() => (this.inFlight = null));
  }

  /** Step: both worlds advance `n` ticks. */
  advance(n = 1): Promise<void> {
    return this.exclusive(async () => {
      if (!this.crashed()) await this.stepBoth(n, false);
    });
  }

  /** Reset: both worlds rewind to t = 0, each replaying its log. Rejects if either replay fails. */
  reset(): Promise<void> {
    return this.exclusive(async () => {
      if (!this.crashed()) await this.rewind(this.worlds);
    });
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

  private crashed(): boolean {
    return this.worlds.some((w) => w.crashed);
  }

  /** Replays each of `worlds`' sessions; throws if a replay fails. */
  private async rewind(worlds: Engine[]): Promise<void> {
    this.rewinding++;
    let errors: (FieldError[] | null)[];
    try {
      errors = await Promise.all(worlds.map((w) => w.replay()));
    } finally {
      this.rewinding--;
    }
    this.emit('tick');
    const failed = errors.flatMap((e) => e ?? []);
    if (failed.length > 0) throw new Error(`a world could not rewind: ${fieldErrorsMessage(failed)}`);
  }

  /** Every world not at t = 0 replays its session (a few rounds, in case of races); throws if they still differ. */
  private async align(): Promise<void> {
    for (let round = 0; round < 3; round++) {
      if (this.crashed()) return;
      const behind = this.worlds.filter((w) => w.tick !== 0);
      if (behind.length === 0) return;
      await this.rewind(behind);
    }
    if (this.worlds.some((w) => w.tick !== 0)) {
      throw new Error(`the worlds did not rewind together (ticks ${this.worlds.map((w) => w.tick).join(' and ')})`);
    }
  }

  /** After a rebuild: every world not at t = 0 rewinds. */
  private realign(): Promise<void> {
    return this.guard(() => this.align());
  }

  /**
   * Runs `fn` in turn with the other steps; if it fails, the worlds may no longer be in step, so
   * the coordinator stops and says so rather than running them apart.
   */
  private guard(fn: () => Promise<void>): Promise<void> {
    return this.exclusive(fn).catch((e: unknown) => {
      console.warn('Compare stopped: the worlds could not be kept in step.', e);
      if (this.running) this.setRunning(false);
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
