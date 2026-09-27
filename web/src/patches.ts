import type { Config } from './types';

/** Whether good 0's map has patches (Minds 1): a peaks map of two or more peaks. */
export function hasPatches(c: Config): boolean {
  const map = c.goods[0]?.map as { kind: string; peaks?: unknown[] } | undefined;
  return map?.kind === 'peaks' && (map.peaks?.length ?? 0) >= 2;
}
