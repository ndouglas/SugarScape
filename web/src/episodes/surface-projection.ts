import type { Checkpoint, Json } from './types';
import { obj, list } from './presentation';
/** Formatting projection only. Knowledge, events and all values come from the native checkpoint. */
export function surfaceDisplay(at: Checkpoint): Json {
  return {
    clock: at.clock, status: obj(at.public).status ?? null, reset: obj(at.public).reset_phase ?? null,
    probeStop: obj(at.public).probe_stop ?? null, researcher: at.researcher,
    agents: Object.entries(at.local).map(([id, value]) => {
      const local = obj(value), prefix = obj(local.prefix);
      return { id, available: value !== null, ids: prefix.ids ?? null, role: prefix.role ?? null,
        delivered: prefix.checkpoint ?? null, credits: local.credits ?? null,
        privateBit: local.private_bit ?? null, observation: obj(local.surface).last_observation ?? null,
        events: prefix.entries ?? [], belief: local.belief ?? null,
        inferenceStatus: local.inference_status ?? null, lastSupportedBelief: local.last_supported_belief ?? null,
        discovery: local.catalog_discovery ?? null, decision: local.decision ?? null,
        decisionLabel: local.decision_label ?? null, prediction: local.prediction ?? null };
    }),
  };
}
/** Receives payload only through the shell's existing final/Researcher result gate. */
export function surfaceSummary(payload: Json): {status:string;spent:Json[];net:Json[];reward:Json[];correct:Json[]} {
  const p = obj(payload), metrics = list(p.metrics).map(obj);
  return { status: p.failure ? 'unsupported history' : 'complete', spent: metrics.map(m => m.spent ?? null),
    net: metrics.map(m => m.net ?? obj(m.terminal).net_utility ?? null),
    reward: metrics.map(m => m.reward ?? obj(m.terminal).gross_reward ?? null),
    correct: metrics.map(m => m.correct ?? obj(m.terminal).correct ?? null) };
}
/** Labels follow shared_surface::Mechanism::ALL, not an inferred or sorted model order. */
export function surfaceBelief(belief: Json): Json {
  if (belief === null) return null;
  const b = obj(belief), models = list(b.models);
  return { SharedPersistent: models[0] ?? null, SharedResetting: models[1] ?? null,
    PrivatePersistent: models[2] ?? null, Inert: models[3] ?? null, target: b.target ?? null };
}
