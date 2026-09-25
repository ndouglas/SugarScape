import { describe, expect, it } from 'vitest';
import { SeekQueue } from './timeline';

describe('SeekQueue', () => {
  it('keeps one seek in flight and sends only the newest position after it', async () => {
    const sent: number[] = [];
    const gates: (() => void)[] = [];
    const q = new SeekQueue((t) => {
      sent.push(t);
      return new Promise<void>((resolve) => gates.push(resolve));
    });
    q.request(10);
    q.request(20);
    q.request(30);
    q.request(40);
    expect(sent).toEqual([10]);
    gates.shift()!();
    await Promise.resolve();
    await Promise.resolve();
    expect(sent).toEqual([10, 40]);
    gates.shift()!();
    await q.settled();
    expect(sent).toEqual([10, 40]);
  });

  it('exposes the tick a seek is in flight for, or the newest one queued to follow it, clearing once idle', async () => {
    const gates: (() => void)[] = [];
    const q = new SeekQueue(() => new Promise<void>((resolve) => gates.push(resolve)));
    expect(q.pending).toBeNull();
    q.request(10);
    expect(q.pending).toBe(10); // in flight
    q.request(20);
    q.request(30);
    expect(q.pending).toBe(30); // queued to follow the in-flight seek, ahead of the stale 20
    gates.shift()!();
    await Promise.resolve();
    await Promise.resolve();
    expect(q.pending).toBe(30); // now itself in flight
    gates.shift()!();
    await q.settled();
    expect(q.pending).toBeNull();
  });

  it('keeps going after a failed seek', async () => {
    const sent: number[] = [];
    const q = new SeekQueue(async (t) => {
      sent.push(t);
      if (t === 1) throw new Error('no');
    });
    q.request(1);
    q.request(2);
    await q.settled();
    expect(sent).toEqual([1, 2]);
  });
});
