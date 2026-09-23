import type { Config, Good, GoodMap, Peak, Pollutant, Transform } from './types';

export const MAX_GOODS = 8;
export const MAX_POLLUTANTS = 4;
export const MAX_PEAKS = 16;

/** Two-peak transforms, in the order new goods take them (the book's sugar, then spice). */
export const TRANSFORMS: [Transform, string][] = [
  ['identity', 'As in the book'],
  ['mirror_x', 'Mirrored left↔right'],
  ['rotate_90', 'Rotated 90°'],
  ['rotate_180', 'Rotated 180°'],
  ['rotate_270', 'Rotated 270°'],
  ['mirror_y', 'Mirrored top↔bottom'],
  ['transpose', 'Transposed'],
  ['anti_transpose', 'Anti-transposed'],
];

/** Colors new goods take, first unused first (sugar's and spice's lead). */
export const PALETTE = ['#f2c14e', '#e07a3f', '#7fb3d5', '#8fcf6b', '#c58fd6', '#e0d06a', '#6ad0c4', '#d67a8f'];

function unusedName(taken: string[], stem: string, from: number): string {
  let k = from;
  while (taken.includes(`${stem} ${k}`)) k++;
  return `${stem} ${k}`;
}

export function newPeak(config: Config): Peak {
  return {
    x: Math.floor(config.width / 2),
    y: Math.floor(config.height / 2),
    radius: Math.max(1, Math.floor(Math.min(config.width, config.height) / 4)),
    height: 4,
  };
}

/** A map of `kind`: the next unused two-peak transform (flat off the 50×50 grid), one central peak, or flat 2. */
export function defaultMap(config: Config, kind: GoodMap['kind'] = 'two_peaks'): GoodMap {
  if (kind === 'peaks') return { kind: 'peaks', peaks: [newPeak(config)] };
  if (kind === 'flat' || config.width !== 50 || config.height !== 50) return { kind: 'flat', capacity: 2 };
  const used = new Set(config.goods.flatMap((g) => (g.map.kind === 'two_peaks' ? [g.map.transform] : [])));
  const transform = TRANSFORMS.map(([t]) => t).find((t) => !used.has(t)) ?? 'identity';
  return { kind: 'two_peaks', transform };
}

export function newGood(config: Config): Good {
  const colors = new Set(config.goods.map((g) => g.color.toLowerCase()));
  return {
    name: unusedName(config.goods.map((g) => g.name), 'good', config.goods.length + 1),
    color: PALETTE.find((c) => !colors.has(c)) ?? PALETTE[config.goods.length % PALETTE.length],
    map: defaultMap(config),
    metabolism: { min: 1, max: 4 },
    endowment: { min: 5, max: 25 },
  };
}

/** Appends a good; every pollutant gets zero coefficients for it. Combat needs one good, so it switches off. */
export function addGood(config: Config): void {
  if (config.goods.length >= MAX_GOODS) return;
  config.goods.push(newGood(config));
  for (const p of config.pollution.pollutants) {
    p.production.push(0);
    p.consumption.push(0);
    p.devalues.push(false);
  }
  config.combat.enabled = false;
}

/** Removes good `i` (never the last one) and its pollutant columns; trade and foresight switch off below two goods. */
export function removeGood(config: Config, i: number): void {
  if (config.goods.length <= 1 || i < 0 || i >= config.goods.length) return;
  config.goods.splice(i, 1);
  for (const p of config.pollution.pollutants) {
    p.production.splice(i, 1);
    p.consumption.splice(i, 1);
    p.devalues.splice(i, 1);
  }
  if (config.goods.length < 2) {
    config.trade.enabled = false;
    config.foresight.enabled = false;
  }
}

export function newPollutant(config: Config): Pollutant {
  const n = config.goods.length;
  const names = config.pollution.pollutants.map((p) => p.name);
  return {
    name: unusedName(names, 'pollutant', names.length + 1),
    production: Array<number>(n).fill(0),
    consumption: Array<number>(n).fill(0),
    devalues: Array<boolean>(n).fill(false),
  };
}

export function addPollutant(config: Config): void {
  if (config.pollution.pollutants.length < MAX_POLLUTANTS) config.pollution.pollutants.push(newPollutant(config));
}

export function removePollutant(config: Config, k: number): void {
  const list = config.pollution.pollutants;
  if (list.length > 1 && k >= 0 && k < list.length) list.splice(k, 1);
}
