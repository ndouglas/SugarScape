import { describe, expect, it } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import { noOverlays, type Command, type DisplayState, type HostRequest, type WorldSnapshot } from './protocol';
import { SimHost } from './sim-host';
import { InlineTransport, PortTransport, startWorker, type PortLike } from './transport';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() };
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
    expect(rb.result).toEqual({ ok: true, value: '0x0|1@1,1|10' });
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

  it('emulates postMessage transfer: a lent frame is detached on send and both sides see a real, usable buffer', async () => {
    const t = new InlineTransport(new SimHost(fakeModule()));
    await t.request(init);
    const buf = new ArrayBuffer(48); // 4 x 3 x 4 bytes, matching the world rendered above
    const pending = t.request({ type: 'refresh' }, { frame: buf });
    // postMessage detaches the transferred buffer synchronously, before the reply is even queued.
    expect(buf.byteLength).toBe(0);
    const reply = await pending;
    const snapshot = reply.result.ok ? reply.result.snapshot : undefined;
    expect(snapshot?.frame).toBeInstanceOf(ArrayBuffer);
    expect(snapshot?.frame).not.toBe(buf);
    expect(snapshot?.frame?.byteLength).toBe(48);
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

  it('routes an unsolicited post to onPost, leaving pending requests untouched', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const posts: WorldSnapshot[] = [];
    t.onPost = (s) => posts.push(s);
    const pending = t.request({ type: 'ready' });
    const snapshot = {
      width: 1,
      height: 1,
      tick: 5,
      population: 0,
      latest: { tick: 5, population: 0 },
      followed: null,
      followedAlive: false,
    } as WorldSnapshot;
    port.onmessage?.({ data: { id: null, post: snapshot } } as MessageEvent);
    expect(posts).toEqual([snapshot]);
    port.onmessage?.({ data: { id: 1, result: { ok: true } } } as MessageEvent);
    expect((await pending).result).toEqual({ ok: true });
  });

  it('passes a lent frame in the transfer list given to postMessage', () => {
    const port = fakePort();
    let transferred: Transferable[] | undefined;
    port.postMessage = (_message, transfer) => {
      transferred = transfer;
    };
    const t = new PortTransport(port);
    const buf = new ArrayBuffer(8);
    void t.request({ type: 'refresh' }, { frame: buf });
    expect(transferred).toEqual([buf]);
  });

  it('ignores a reply for an id that was never requested', async () => {
    const port = fakePort();
    const t = new PortTransport(port);
    const pending = t.request({ type: 'ready' });
    expect(() => port.onmessage?.({ data: { id: 999, result: { ok: true } } } as MessageEvent)).not.toThrow();
    port.onmessage?.({ data: { id: 1, result: { ok: true, value: 'real' } } } as MessageEvent);
    expect((await pending).result).toEqual({ ok: true, value: 'real' });
  });

  it('close settles any still-pending request instead of leaving it hanging, and is not treated as a crash', async () => {
    const port = fakePort();
    let terminated = false;
    port.terminate = () => {
      terminated = true;
    };
    const t = new PortTransport(port);
    const fatal: string[] = [];
    t.onFatal = (message) => fatal.push(message);
    const pending = t.request({ type: 'ready' });
    t.close();
    expect(terminated).toBe(true);
    const reply = await pending;
    expect(reply.result.ok).toBe(false);
    expect(reply.result).toMatchObject({ fatal: expect.any(String) });
    expect(fatal).toEqual([]); // closing on purpose is not the host dying
    const after = await t.request({ type: 'fingerprint' });
    expect(after.result).toEqual(reply.result);
  });
});

describe('startWorker', () => {
  it('resolves once the worker is ready and rejects when it cannot start', async () => {
    const ready: PortLike = {
      onmessage: null,
      onerror: null,
      onmessageerror: null,
      postMessage: (m) => queueMicrotask(() => ready.onmessage?.({ data: { id: (m as HostRequest).id, result: { ok: true } } } as MessageEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => ready)).resolves.toBeInstanceOf(PortTransport);
    const broken: PortLike = {
      onmessage: null,
      onerror: null,
      onmessageerror: null,
      postMessage: () => queueMicrotask(() => broken.onerror?.({ message: 'import failed' } as ErrorEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => broken)).rejects.toThrow('import failed');
    const noWasm: PortLike = {
      onmessage: null,
      onerror: null,
      onmessageerror: null,
      postMessage: (m) =>
        queueMicrotask(() => noWasm.onmessage?.({ data: { id: (m as HostRequest).id, result: { ok: false, fatal: 'WASM failed to load' } } } as MessageEvent)),
      terminate: () => {},
    };
    await expect(startWorker(() => noWasm)).rejects.toThrow('WASM failed to load');
  });

  it('rejects and terminates the worker if it never answers ready within the timeout', async () => {
    let terminated = false;
    const silent: PortLike = {
      onmessage: null,
      onerror: null,
      onmessageerror: null,
      postMessage: () => {},
      terminate: () => {
        terminated = true;
      },
    };
    await expect(startWorker(() => silent, 10)).rejects.toThrow();
    expect(terminated).toBe(true);
  });
});
