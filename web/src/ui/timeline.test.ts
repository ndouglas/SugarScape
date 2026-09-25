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
