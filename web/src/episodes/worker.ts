// This worker owns its WASM instance. The page terminates it on every end state.
import init, { experiment_catalog_json, experiment_recorded_json, experiment_run_json, experiment_validate_json } from '../wasm-pkg/sugarscape.js';
import { parseErrors } from '../types';
import type { EpisodeRecord, Json, StudyDescriptor, WorkerReply, WorkerRequest } from './types';
let started = false;
addEventListener('message', async ({ data }: MessageEvent<WorkerRequest>) => {
  if (started) {
    postMessage({ id: data.id, errors: [{ field: 'worker', message: 'worker already owns a request' }] } satisfies WorkerReply);
    return;
  }
  started = true;
  let reply: WorkerReply;
  try {
    // Initialization happens inside try, after a request: rejected loads are never unhandled.
    await init();
    switch (data.op) {
      case 'run':
        reply = { id: data.id, kind: 'episode', record: JSON.parse(experiment_run_json(data.text)) as EpisodeRecord };
        break;
      case 'validate':
        reply = { id: data.id, kind: 'episode', record: JSON.parse(experiment_validate_json(data.text)) as EpisodeRecord };
        break;
      case 'catalog':
        reply = { id: data.id, kind: 'catalog', catalog: JSON.parse(experiment_catalog_json()) as StudyDescriptor[] };
        break;
      case 'recorded':
        reply = { id: data.id, kind: 'recorded', recorded: JSON.parse(experiment_recorded_json()) as Json };
        break;
      default:
        throw new Error('unknown episode worker operation');
    }
  } catch (error) {
    reply = { id: data.id, errors: parseErrors(error, 'worker') };
  }
  postMessage(reply);
});
