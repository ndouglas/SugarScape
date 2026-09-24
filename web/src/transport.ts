import { transfers, type Command, type HostMessage, type HostReply, type HostRequest, type Wants, type WorldSnapshot } from './protocol';
import { serve, type SimHost } from './sim-host';

/** How the engine talks to a SimHost: each request resolves with the reply that carries its id. */
export interface Transport {
  request(cmd: Command, extra?: { wants?: Wants; frame?: ArrayBuffer }): Promise<HostReply>;
  /** Snapshots the host sends on its own (Max speed). */
  onPost: ((snapshot: WorldSnapshot) => void) | null;
  /** Called once when the host dies (a panic, or the worker failed); pending and later requests resolve as fatal. */
  onFatal: ((message: string) => void) | null;
  close(): void;
}

/** The part of a `Worker` a transport uses (the in-page stand-in and tests implement it too). */
export interface PortLike {
  onmessage: ((event: MessageEvent) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  /** Fires when a message could not be deserialized; no `data` is available, so it names no request. */
  onmessageerror: ((event: MessageEvent) => void) | null;
  postMessage(message: unknown, transfer: Transferable[]): void;
  terminate(): void;
}

/** Numbers requests and resolves each with its reply; buffers are transferred both ways. */
export class PortTransport implements Transport {
  onPost: ((snapshot: WorldSnapshot) => void) | null = null;
  onFatal: ((message: string) => void) | null = null;
  private next = 1;
  private pending = new Map<number, (reply: HostReply) => void>();
  private dead: string | null = null;

  constructor(private port: PortLike) {
    port.onmessage = (event) => this.receive(event.data as HostMessage);
    port.onerror = (event) => this.fail(event.message || 'the simulation worker failed');
    // The channel cannot say which request this was for (its data never arrived), so every
    // request pending at the time is rejected — same as any other way the other side dies
    // (Decision 7, PF9): nothing is ever left hanging.
    port.onmessageerror = () => this.fail('the simulation worker sent a message that could not be read');
  }

  request(cmd: Command, extra: { wants?: Wants; frame?: ArrayBuffer } = {}): Promise<HostReply> {
    const id = this.next++;
    if (this.dead) return Promise.resolve({ id, result: { ok: false, fatal: this.dead } });
    const req: HostRequest = { id, cmd, ...extra };
    return new Promise((resolve) => {
      this.pending.set(id, resolve);
      this.port.postMessage(req, transfers(req));
    });
  }

  /** Stops talking to the port; any request still pending resolves fatal instead of hanging forever. */
  close(): void {
    this.settle('the transport was closed');
    this.port.terminate();
  }

  private receive(message: HostMessage): void {
    if (message.id === null) {
      if ('fatal' in message) this.fail(message.fatal);
      else this.onPost?.(message.post);
      return;
    }
    const resolve = this.pending.get(message.id);
    this.pending.delete(message.id);
    resolve?.(message);
    if (!message.result.ok && 'fatal' in message.result) this.fail(message.result.fatal);
  }

  /** Marks the transport dead and settles every pending request as fatal; a no-op once already dead. */
  private settle(message: string): void {
    if (this.dead) return;
    this.dead = message;
    for (const [id, resolve] of this.pending) resolve({ id, result: { ok: false, fatal: message } });
    this.pending.clear();
  }

  /** The other side died: settles pending requests (see `settle`) and reports it once, for the crash banner. */
  private fail(message: string): void {
    const already = this.dead !== null;
    this.settle(message);
    if (!already) this.onFatal?.(message);
  }
}

/**
 * A worker stand-in on this thread: messages go both ways asynchronously, in order, and through
 * `structuredClone(message, { transfer })` exactly like `postMessage` — lent buffers are detached
 * on the sending side and arrive as fresh, usable buffers on the other, not shared by reference.
 */
function inlinePort(host: SimHost): PortLike {
  const port: PortLike = {
    onmessage: null,
    onerror: null,
    onmessageerror: null,
    postMessage: (message, transfer) => {
      const cloned = structuredClone(message, { transfer });
      queueMicrotask(() => handle(cloned as HostRequest));
    },
    terminate: () => {},
  };
  const handle = serve(
    host,
    (message, transfer) => {
      const cloned = structuredClone(message, { transfer });
      queueMicrotask(() => port.onmessage?.({ data: cloned } as MessageEvent));
    },
    (fn) => setTimeout(fn, 0),
  );
  return port;
}

/** Runs the host on the page: in tests, and when a module worker cannot start. */
export class InlineTransport extends PortTransport {
  constructor(readonly host: SimHost) {
    super(inlinePort(host));
  }
}

/** How long `startWorker` waits for `ready` before giving up (PF9): the production value. */
const READY_TIMEOUT_MS = 10_000;

/**
 * Starts the simulation worker and waits until its WASM is ready; rejects if a module worker
 * cannot start, its WASM fails to load, or it never answers within `timeoutMs` (PF9) — in every
 * case the worker is terminated and the caller falls back to `InlineTransport` (Decision 10).
 */
export async function startWorker(
  create: () => PortLike = () => new Worker(new URL('./sim-worker.ts', import.meta.url), { type: 'module' }),
  timeoutMs = READY_TIMEOUT_MS,
): Promise<Transport> {
  const transport = new PortTransport(create());
  let timer: ReturnType<typeof setTimeout>;
  const timeout = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => reject(new Error('the simulation worker did not answer ready in time')), timeoutMs);
  });
  try {
    const { result } = await Promise.race([transport.request({ type: 'ready' }), timeout]);
    if (!result.ok) throw new Error('fatal' in result ? result.fatal : 'the simulation worker did not start');
    return transport;
  } catch (e) {
    transport.close();
    throw e;
  } finally {
    clearTimeout(timer!);
  }
}
