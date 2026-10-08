import type { FieldError } from '../types';
import type { EpisodeClient, EpisodeRecord, Json, StudyDescriptor, WorkerLike, WorkerOp, WorkerReply } from './types';

/** Contextual engine errors and operational worker failures share one UI error shape. */
export class EpisodeError extends Error {
  constructor(readonly errors: FieldError[]) {
    super(errors.map(error => `${error.field}: ${error.message}`).join('; '));
    this.name = 'EpisodeError';
  }
}
const failure = (message: string) => new EpisodeError([{ field: 'worker', message }]);

/** One request at a time. Every end state terminates the worker and releases WASM. */
export function createEpisodeClient(
  create: () => WorkerLike = () => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' }),
): EpisodeClient {
  let nextId = 0;
  let disposed = false;
  let stop: ((error: Error) => void) | null = null;
  function send(op: WorkerOp, text: string): Promise<EpisodeRecord | StudyDescriptor[] | Json> {
    if (disposed) return Promise.reject(failure('client disposed'));
    if (stop) return Promise.reject(failure('an episode request is already running'));
    const limit = op === 'run' ? 64 * 1024 : 16 * 1024 * 1024;
    if (new TextEncoder().encode(text).byteLength > limit) {
      return Promise.reject(new EpisodeError([{ field: op === 'run' ? 'input' : 'episode', message: `raw input exceeds ${op === 'run' ? '64 KiB' : '16 MiB'}` }]));
    }
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      let worker: WorkerLike | null = null;
      let ended = false;
      const finish = (error: Error | null, result?: EpisodeRecord | StudyDescriptor[] | Json) => {
        if (ended) return;
        ended = true;
        clearTimeout(timer);
        if (worker) {
          worker.onmessage = null;
          worker.onerror = null;
          worker.onmessageerror = null;
          worker.terminate();
          worker = null;
        }
        stop = null;
        if (error) reject(error);
        else resolve(result!);
      };
      const timer = setTimeout(() => finish(failure('episode request timed out after 60 seconds')), 60_000);
      stop = error => finish(error);
      try {
        worker = create();
        worker.onmessage = (event: MessageEvent<WorkerReply>) => {
          const reply = event.data;
          if (ended || !reply || reply.id !== id) return;
          try {
            if ('errors' in reply) finish(new EpisodeError(reply.errors));
            else if ((op === 'run' || op === 'validate') && reply.kind === 'episode') finish(null, reply.record);
            else if (op === 'catalog' && reply.kind === 'catalog') finish(null, reply.catalog);
            else if (op === 'recorded' && reply.kind === 'recorded') finish(null, reply.recorded);
            else finish(failure('unexpected episode worker reply'));
          } catch (error) {
            finish(failure(error instanceof Error ? error.message : String(error)));
          }
        };
        worker.onerror = event => finish(failure(event.message || 'episode worker failed to load or run'));
        worker.onmessageerror = () => finish(failure('could not decode episode worker reply'));
        worker.postMessage({ id, op, text });
      } catch (error) {
        finish(failure(error instanceof Error ? error.message : String(error)));
      }
    });
  }
  return {
    request: (op, text) => send(op, text) as Promise<EpisodeRecord>,
    catalog: () => send('catalog', '') as Promise<StudyDescriptor[]>,
    recorded: () => send('recorded', '') as Promise<Json>,
    cancel: () => stop?.(failure('episode request canceled')),
    dispose: () => { disposed = true; stop?.(failure('client disposed')); },
  };
}
