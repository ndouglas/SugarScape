import type { EpisodeRecord, Checkpoint, Json } from './types';
export type Perspective = { kind: 'agent'; agent: string } | { kind: 'researcher' };
export function projectCheckpoint(record: EpisodeRecord, index: number, perspective: Perspective): Checkpoint {
  const at = record.checkpoints[index];
  if (!at) throw new Error('checkpoint out of range');
  return perspective.kind === 'researcher' ? at : { ...at, researcher: null, local: { [perspective.agent]: at.local[perspective.agent] ?? null } };
}
/** Only the separately labeled result pane may receive full terminal material. */
export function resultPayload(record: EpisodeRecord, index: number, perspective: Perspective): Json {
  return perspective.kind === 'researcher' || index === record.checkpoints.length - 1 ? record.payload : null;
}
