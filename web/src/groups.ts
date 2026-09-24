import type { Config, TagGroup } from './types';

/** Mirrors `config::MAX_GROUPS` and `config::{BLUE, GREEN, RED}_COLOR`. */
export const MAX_GROUPS = 8;
export const BLUE = '#3d7eff';
export const GREEN = '#3dd66b';
export const RED = '#ff4d4d';
/** Colors new groups take, first unused first. */
export const GROUP_PALETTE = [BLUE, RED, GREEN, '#ffe04d', '#b894f0', '#ff8a5c', '#6ad0c4', '#d67a8f'];

/** A group with keys in the core's serde order. */
function group(name: string, color: string, min: number, max: number): TagGroup {
  return { name, color, zeros: { min, max } };
}

/** The book's two tribes on `tagLength`-bit tags (mirrors `config::default_groups`): Blue when zeros outnumber ones. */
export function defaultGroups(tagLength: number): TagGroup[] {
  const blueFrom = Math.floor(tagLength / 2) + 1;
  return [group('Blue', BLUE, blueFrom, tagLength), group('Red', RED, 0, blueFrom - 1)];
}

/** Chapter III note 20's three groups, thirds of 0..=L (mirrors `config::three_tribes`); needs tags of at least 2 bits. */
export function threeTribes(tagLength: number): TagGroup[] {
  const cut = (k: number) => Math.floor((k * (tagLength + 1)) / 3);
  return [group('Blue', BLUE, 0, cut(1) - 1), group('Green', GREEN, cut(1), cut(2) - 1), group('Red', RED, cut(2), tagLength)];
}

export function sameGroups(a: TagGroup[], b: TagGroup[]): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** Fewer than MAX_GROUPS, and some group holds two or more zero counts to split. */
export function canAddGroup(config: Config): boolean {
  const groups = config.culture.groups;
  return groups.length < MAX_GROUPS && groups.some((g) => g.zeros.max > g.zeros.min);
}

/** Splits the widest group (the first of equals): the new group takes the upper half of its zero counts. */
export function addGroup(config: Config): void {
  if (!canAddGroup(config)) return;
  const groups = config.culture.groups;
  const width = (g: TagGroup) => g.zeros.max - g.zeros.min;
  const widest = groups.reduce((best, g) => (width(g) > width(best) ? g : best), groups[0]);
  const mid = Math.floor((widest.zeros.min + widest.zeros.max) / 2);
  const names = groups.map((g) => g.name);
  let k = groups.length + 1;
  while (names.includes(`group ${k}`)) k++;
  const colors = new Set(groups.map((g) => g.color.toLowerCase()));
  const color = GROUP_PALETTE.find((c) => !colors.has(c)) ?? GROUP_PALETTE[groups.length % GROUP_PALETTE.length];
  groups.push(group(`group ${k}`, color, mid + 1, widest.zeros.max));
  widest.zeros.max = mid;
}

/** Removes group `k` (never the last); its zero counts join the group just below them, else the one just above. */
export function removeGroup(config: Config, k: number): void {
  const groups = config.culture.groups;
  if (groups.length <= 1 || k < 0 || k >= groups.length) return;
  const g = groups[k];
  const below = groups.find((o) => o !== g && o.zeros.max === g.zeros.min - 1);
  const above = groups.find((o) => o !== g && o.zeros.min === g.zeros.max + 1);
  if (below) below.zeros.max = g.zeros.max;
  else if (above) above.zeros.min = g.zeros.min;
  groups.splice(k, 1);
}

/** Sets one end of group `k`'s range, moving the adjoining group's end with it so the ranges still tile. */
export function moveBoundary(groups: TagGroup[], k: number, end: 'min' | 'max', value: number): void {
  const g = groups[k];
  if (!g) return;
  if (end === 'max') {
    const next = groups.find((o) => o !== g && o.zeros.min === g.zeros.max + 1);
    g.zeros.max = value;
    if (next) next.zeros.min = value + 1;
  } else {
    const prev = groups.find((o) => o !== g && o.zeros.max === g.zeros.min - 1);
    g.zeros.min = value;
    if (prev) prev.zeros.max = value - 1;
  }
}

/** The groups table's structure: the tag length and each range (names and colors refresh in place). */
export function groupsEditorSignature(config: Config): string {
  return JSON.stringify([config.tag_length, config.culture.groups.map((g) => [g.zeros.min, g.zeros.max])]);
}

/** The Group shares chart's lines: each group's name and color. */
export function groupSharesSignature(config: Config): string {
  return JSON.stringify(config.culture.groups.map((g) => [g.name, g.color]));
}
