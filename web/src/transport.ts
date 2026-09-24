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

  close(): void {
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

  private fail(message: string): void {
    if (this.dead) return;
    this.dead = message;
    for (const [id, resolve] of this.pending) resolve({ id, result: { ok: false, fatal: message } });
    this.pending.clear();
    this.onFatal?.(message);
  }
}

/** A worker stand-in on this thread: messages go both ways asynchronously and in order. */
function inlinePort(host: SimHost): PortLike {
  const port: PortLike = {
    onmessage: null,
    onerror: null,
    onmessageerror: null,
    postMessage: (message) => queueMicrotask(() => handle(message as HostRequest)),
    terminate: () => {},
  };
  const handle = serve(host, (message) => queueMicrotask(() => port.onmessage?.({ data: message } as MessageEvent)));
  return port;
}

/** Runs the host on the page: in tests, and when a module worker cannot start. */
export class InlineTransport extends PortTransport {
  constructor(readonly host: SimHost) {
    super(inlinePort(host));
  }
}
