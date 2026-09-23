import type { DiseaseEntry } from '../types';

/** `[value, label]` options for a disease picker; value `-1` is "New disease". */
export function diseaseOptions(list: DiseaseEntry[], allowNew: boolean): [string, string][] {
  const options: [string, string][] = list.map((d) => [String(d.id), `#${d.id} · ${d.bits}`]);
  return allowNew ? [['-1', 'New disease'], ...options] : options;
}
