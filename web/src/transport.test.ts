import { describe, expect, it } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import type { Command, DisplayState } from './protocol';
import { SimHost } from './sim-host';
import { InlineTransport, PortTransport, type PortLike } from './transport';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };
const init: Command = { type: 'init', config, seed: 1, landscapes: [], display };

function fakePort(): PortLike {
  return { onmessage: null, onerror: null, onmessageerror: null, postMessage: () => {}, terminate: () => {} };
}

describe('InlineTransport', () => {
  it('resolves each request with its own reply, asynchronously and in order', async () => {
    const t = new InlineTransport(new SimHost(fakeModule()));
    const order: number[] = [];
    const a = t.request(init).then((r) => (order.push(r.id), r));
    const b = t.request({ type: 'fingerprint' }).then((r) => (order.push(r.id), r));
    expect(order).toEqual([]);
    const [ra, rb] = await Promise.all([a, b]);
    expect(ra.id).toBeLessThan(rb.id);
    expect(order).toEqual([ra.id, rb.id]);
    expect(ra.result.ok).toBe(true);
    expect(rb.result).toEqual({ ok: true, value: '0x0' });
  });

  it('reports a fatal reply once and answers later requests as fatal', async () => {
    const t = new InlineTransport(new SimHost(fakeModule()));
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    await t.request(init);
    const r = await t.request({ type: 'paint', x: 0, y: 0, radius: 0, value: -1, good: 0 });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
    expect((await t.request({ type: 'fingerprint' })).result).toEqual(r.result);
    expect(fatal).toEqual(['The simulation stopped: unreachable executed']);
  });
});

describe('PortTransport', () => {
  it('fails pending requests when the port errors', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    const pending = t.request({ type: 'ready' });
    port.onerror?.({ message: 'boom' } as ErrorEvent);
    expect((await pending).result).toEqual({ ok: false, fatal: 'boom' });
    expect(fatal).toEqual(['boom']);
  });

  it('never leaves a request hanging: a later port error still rejects every request pending at the time', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const a = t.request({ type: 'ready' });
    const b = t.request({ type: 'fingerprint' });
    port.onerror?.({ message: 'worker crashed' } as ErrorEvent);
    expect((await a).result).toEqual({ ok: false, fatal: 'worker crashed' });
    expect((await b).result).toEqual({ ok: false, fatal: 'worker crashed' });
  });

  it('calls onFatal only once and answers a request made after death as fatal too', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    port.onerror?.({ message: 'worker crashed' } as ErrorEvent);
    port.onerror?.({ message: 'again' } as ErrorEvent);
    expect(fatal).toEqual(['worker crashed']);
    expect((await t.request({ type: 'fingerprint' })).result).toEqual({ ok: false, fatal: 'worker crashed' });
    expect(fatal).toEqual(['worker crashed']);
  });

  it('fails pending requests, with a clear message, when the port cannot deserialize a message', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    const pending = t.request({ type: 'ready' });
    port.onmessageerror?.({} as MessageEvent);
    const reply = await pending;
    expect(reply.result.ok).toBe(false);
    expect(reply.result).toMatchObject({ fatal: expect.any(String) });
    expect(fatal).toHaveLength(1);
    expect(fatal[0]).toMatch(/\S/);
  });

  it('reports a fatal host message once and answers later requests as fatal', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    const pending = t.request({ type: 'ready' });
    port.onmessage?.({ data: { id: null, fatal: 'the host panicked' } } as MessageEvent);
    expect((await pending).result).toEqual({ ok: false, fatal: 'the host panicked' });
    expect(fatal).toEqual(['the host panicked']);
    expect((await t.request({ type: 'fingerprint' })).result).toEqual({ ok: false, fatal: 'the host panicked' });
    expect(fatal).toEqual(['the host panicked']);
  });
});
