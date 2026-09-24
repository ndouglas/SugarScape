// The playground's simulation worker: its own WASM instance and one SimHost, answering in order.
import type { HostMessage, HostRequest } from './protocol';
import { channelDefer, serve, SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import init from './wasm-pkg/sugarscape.js';

const post = (message: HostMessage, transfer: Transferable[]) => postMessage(message, { transfer });
/** Requests that arrive while the WASM loads wait here, in order. */
const queue: HostRequest[] = [];
let handle: ((req: HostRequest) => void) | null = null;

addEventListener('message', (event: MessageEvent<HostRequest>) => {
  if (handle) handle(event.data);
  else queue.push(event.data);
});

init().then(
  (wasm) => {
    handle = serve(new SimHost(wasmSimModule(wasm.memory)), post, channelDefer());
    for (const req of queue.splice(0)) handle(req);
  },
  (e: unknown) => {
    const fatal = `WASM failed to load: ${e instanceof Error ? e.message : String(e)}`;
    handle = (req) => post({ id: req.id, result: { ok: false, fatal } }, []);
    for (const req of queue.splice(0)) handle(req);
  },
);
