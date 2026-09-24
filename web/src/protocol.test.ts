import { describe, expect, it } from 'vitest';
import { chartKey, mergeWants, transfers, type WorldSnapshot } from './protocol';
import type { Snapshot } from './types';

describe('mergeWants', () => {
  it('ORs flags, unions networks (in overlay order) and chart groups (by key), and keeps the first selection', () => {
    const merged = mergeWants([
      { select: { x: 1, y: 2, agentId: null }, networks: ['disease'], charts: { groups: [['population']], max: 100 } },
      {
        select: { x: 9, y: 9, agentId: 3 },
        lorenz: true,
        networks: ['trade', 'disease'],
        charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
      },
      {},
    ]);
    expect(merged).toEqual({
      select: { x: 1, y: 2, agentId: null },
      lorenz: true,
      networks: ['trade', 'disease'],
      charts: { groups: [['population'], ['gini', 'births']], max: 2000 },
    });
  });

  it('is empty when nothing is wanted', () => {
    expect(mergeWants([{}, {}])).toEqual({});
  });
});

describe('chartKey', () => {
  it('joins the names', () => {
    expect(chartKey(['mean_log_price', 'sd_log_price'])).toBe('mean_log_price|sd_log_price');
  });
});

describe('transfers', () => {
  it('lists the buffers a message carries', () => {
    const a = new ArrayBuffer(4);
    const b = new ArrayBuffer(4);
    const snapshot: WorldSnapshot = {
      width: 1,
      height: 1,
      tick: 0,
      population: 0,
      latest: {} as Snapshot,
      followed: null,
      followedAlive: false,
      frame: a,
    };
    const request = transfers({ id: 1, cmd: { type: 'refresh' }, frame: a });
    expect(request).toHaveLength(1);
    expect(request[0]).toBe(a);
    const reply = transfers({ id: 1, result: { ok: true, snapshot }, spare: [b] });
    expect(reply).toHaveLength(2);
    expect(reply[0]).toBe(b);
    expect(reply[1]).toBe(a);
    expect(transfers({ id: null, post: snapshot })[0]).toBe(a);
    expect(transfers({ id: 2, result: { ok: false, errors: [] } })).toEqual([]);
    expect(transfers({ id: 3, cmd: { type: 'fingerprint' } })).toEqual([]);
  });
});
