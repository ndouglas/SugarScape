import type { FieldError } from '../types';
export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type StudyId = 'wink' | 'testimony' | 'testimony_game' | 'strategic_reporting' | 'strategy_inference' | 'adversarial_audit' | 'shared_surface' | 'active_surface';
export interface StudyDescriptor {
  id: StudyId; family: 'game' | 'testimony' | 'surface'; title: string;
  supplied: string; question: string; default_input: Json; controls: Json;
}
export interface Checkpoint {
  index: number; clock: Json; kind: string; public: Json;
  local: Record<string, Json>; researcher: Json;
}
export interface EpisodeRecord {
  kind: 'experiment_episode'; version: 1; study: StudyId; rules_identity: string;
  input: Json; semantics: 'trajectory' | 'conditional_case' | 'evidence_sequence';
  checkpoints: Checkpoint[]; payload: Json;
}
export type WorkerOp = 'run' | 'validate' | 'catalog' | 'recorded';
export interface WorkerRequest { id: number; op: WorkerOp; text: string }
export type WorkerReply =
  | { id: number; kind: 'episode'; record: EpisodeRecord }
  | { id: number; kind: 'catalog'; catalog: StudyDescriptor[] }
  | { id: number; kind: 'recorded'; recorded: Json }
  | { id: number; errors: FieldError[] };
export interface EpisodeClient {
  request(op: 'run' | 'validate', text: string): Promise<EpisodeRecord>;
  catalog(): Promise<StudyDescriptor[]>;
  recorded(): Promise<Json>;
  cancel(): void;
  dispose(): void;
}
export interface WorkerLike {
  onmessage: ((event: MessageEvent<WorkerReply>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  onmessageerror: ((event: MessageEvent) => void) | null;
  postMessage(message: WorkerRequest): void;
  terminate(): void;
}
