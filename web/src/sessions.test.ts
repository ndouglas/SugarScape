import { describe, expect, it } from 'vitest';
import type { LogEntry, Session } from './protocol';
import { compareLink, LOG_FULL_NOTICE, LONG_LINK, LONG_NOTICE, sessionLink, shareable, type SessionSource } from './sessions';
import { decodeCompare, decodeShare } from './share';
import type { Config } from './types';

const config = { width: 50, height: 50 } as unknown as Config;
const painted = [new Uint8Array(2500).fill(2)];

function source(log: LogEntry[], full = false, seed = 3): SessionSource {
  const session: Session = { config, seed, landscapes: [], log };
  return {
    baseConfig: { ...config, population: 99 } as Config,
    seed,
    editedLandscapes: () => painted,
    session: async () => ({ session, full, tick: 10 }),
  };
}

/** Edits that deflate poorly: a pseudo-random paint trail (MINSTD). */
function noisy(n: number): LogEntry[] {
  let x = 1;
  const next = () => (x = (x * 48271) % 2147483647) % 50;
  return Array.from({ length: n }, (_, i): LogEntry => ({
    tick: i,
    cmd: { type: 'paint', x: next(), y: next(), radius: next() % 5, value: next() % 11, good: 0 },
  }));
}

describe('sessionLink', () => {
  it('links the whole session', async () => {
    const log: LogEntry[] = [{ tick: 2, cmd: { type: 'erase', x: 1, y: 1 } }];
    const { hash, notice } = await sessionLink(source(log));
    expect(notice).toBeUndefined();
    expect(hash).toMatch(/^#s=[A-Za-z0-9_-]+$/);
    expect(await decodeShare(hash.slice(3))).toEqual({ config, seed: 3, log });
  });

  it('still links a long session, and says the link is long', async () => {
    const { hash, notice } = await sessionLink(source(noisy(20_000)));
    expect(hash.length - 3).toBeGreaterThan(LONG_LINK);
    expect(notice).toBe(LONG_NOTICE);
    expect((await decodeShare(hash.slice(3))).log).toHaveLength(20_000);
  });

  it('falls back to the setup and painted maps when the log is full', async () => {
    const { state, full } = await shareable(source(noisy(10), true));
    expect(full).toBe(true);
    expect(state.log).toBeUndefined();
    const { hash, notice } = await sessionLink(source(noisy(10), true));
    expect(notice).toBe(LOG_FULL_NOTICE);
    const back = await decodeShare(hash.slice(3));
    expect(back.log).toEqual([]);
    expect(back.config.population).toBe(99);
    expect(back.landscapes).toEqual(painted);
  });
});

describe('compareLink', () => {
  it('links both sessions', async () => {
    const { hash, notice } = await compareLink(source([], false, 1), source([{ tick: 4, cmd: { type: 'erase', x: 0, y: 0 } }], false, 2));
    expect(notice).toBeUndefined();
    expect(hash).toMatch(/^#c=[A-Za-z0-9_-]+$/);
    const back = await decodeCompare(hash.slice(3));
    expect([back.a.seed, back.b.seed]).toEqual([1, 2]);
    expect(back.b.log).toHaveLength(1);
  });

  it('says when either log is full', async () => {
    expect((await compareLink(source([]), source([], true))).notice).toBe(LOG_FULL_NOTICE);
  });
});
