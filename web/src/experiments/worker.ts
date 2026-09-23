// A sweep worker: its own WASM instance, one point per message.
import init, { run_point } from '../wasm-pkg/sugarscape.js';
import { parseErrors } from '../types';
import type { PointReply, PointRequest } from './pool';
import type { RunResult } from './types';

const ready = init();

addEventListener('message', async (event: MessageEvent<PointRequest>) => {
  const { spec, index } = event.data;
  let reply: PointReply;
  try {
    // Awaited inside the try: if WASM fails to load, this rejects here and
    // is reported back as a point error instead of hanging the pool forever.
    await ready;
    reply = { index, run: JSON.parse(run_point(spec, index)) as RunResult };
  } catch (e) {
    reply = { index, errors: parseErrors(e) };
  }
  postMessage(reply);
});
