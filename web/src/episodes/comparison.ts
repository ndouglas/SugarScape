import type { Json, StudyDescriptor, Checkpoint } from './types';
import { obj, list, valueText } from './presentation';
export type EpisodeInput = Json;
export type SurfaceInput = Record<string, Json>;
export function canonical(value: Json): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value !== null && typeof value === 'object') return `{${Object.keys(value).sort().map(k => `${JSON.stringify(k)}:${canonical(value[k])}`).join(',')}}`;
  return JSON.stringify(value);
}
/** Only the declared controller selector is excluded; every scientific setting remains. */
export function comparisonKey(input: EpisodeInput): string {
  const copy = structuredClone(obj(input));
  if (copy.study === 'active_surface' || copy.study === 'shared_surface') {
    const protocol = obj(copy.protocol);
    delete protocol[copy.study === 'active_surface' ? 'policy' : 'pair'];
  }
  return canonical(copy);
}
export function withActivePolicy(input: SurfaceInput, policy: string): SurfaceInput {
  if (input.study !== 'active_surface' || !['Adaptive', 'FixedThree', 'NoProbe', 'InspectOnly', 'Known'].includes(policy)) throw new Error('Unknown active surface policy');
  return { ...structuredClone(input), protocol: { ...structuredClone(obj(input.protocol)), policy } };
}
export function matchedInputs(input: Json, descriptor: StudyDescriptor): Json[] {
  const original = obj(input), seen = new Set<string>();
  return list(obj(descriptor.controls).settings).flatMap(setting => {
    const row = obj(setting);
    const candidate = { ...structuredClone(original), environment: structuredClone(row.environment), protocol: { ...structuredClone(obj(row.protocol)), ids: structuredClone(obj(original.protocol).ids) } };
    const key = canonical(candidate);
    if (comparisonKey(candidate) !== comparisonKey(input) || key === canonical(input) || seen.has(key)) return [];
    seen.add(key); return [original.study === 'active_surface' ? withActivePolicy(original, String(obj(candidate.protocol).policy)) : candidate];
  });
}
export function surfaceSettingLabel(input: Json): string {
  const p = obj(obj(input).protocol);
  return `${valueText(obj(input).environment)} · ${p.policy ? `Agent ${valueText(p.experimenter)} / ${valueText(p.policy)}` : `${valueText(p.pair)} / calibration ${valueText(p.calibration_rounds)} / ${valueText(p.prior_mode)}`}`;
}
/** Exact public clock and decision stage; absent means unavailable, never an inferred paid wait. */
export function matchingCheckpoint(points: Checkpoint[], at: Checkpoint): number {
  const clock = canonical(at.clock);
  return points.findIndex(p => p.kind === at.kind && canonical(p.clock) === clock);
}
