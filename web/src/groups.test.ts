import { describe, expect, it } from 'vitest';
import type { Config, TagGroup } from './types';
import {
  MAX_GROUPS,
  addGroup,
  canAddGroup,
  defaultGroups,
  groupsEditorSignature,
  moveBoundary,
  removeGroup,
  sameGroups,
  threeTribes,
} from './groups';

const config = (groups: TagGroup[], tagLength = 11): Config =>
  ({ tag_length: tagLength, culture: { enabled: false, groups } }) as unknown as Config;
const ranges = (c: Config) => c.culture.groups.map((g) => [g.zeros.min, g.zeros.max]);
/** Every zero count 0..=L lies in exactly one group. */
const tiles = (c: Config) =>
  Array.from({ length: c.tag_length + 1 }, (_, z) => c.culture.groups.filter((g) => g.zeros.min <= z && z <= g.zeros.max).length).every(
    (n) => n === 1,
  );

describe('book groups (mirroring config::default_groups and config::three_tribes)', () => {
  it('makes Blue the tags whose zeros outnumber their ones', () => {
    expect(defaultGroups(11)).toEqual([
      { name: 'Blue', color: '#3d7eff', zeros: { min: 6, max: 11 } },
      { name: 'Red', color: '#ff4d4d', zeros: { min: 0, max: 5 } },
    ]);
    expect(defaultGroups(4).map((g) => g.zeros)).toEqual([{ min: 3, max: 4 }, { min: 0, max: 2 }]);
    expect(defaultGroups(1).map((g) => g.zeros)).toEqual([{ min: 1, max: 1 }, { min: 0, max: 0 }]);
  });

  it("splits 11-bit tags into the book's three tribes", () => {
    expect(threeTribes(11)).toEqual([
      { name: 'Blue', color: '#3d7eff', zeros: { min: 0, max: 3 } },
      { name: 'Green', color: '#3dd66b', zeros: { min: 4, max: 7 } },
      { name: 'Red', color: '#ff4d4d', zeros: { min: 8, max: 11 } },
    ]);
    expect(threeTribes(2).map((g) => g.zeros)).toEqual([{ min: 0, max: 0 }, { min: 1, max: 1 }, { min: 2, max: 2 }]);
  });

  it("writes keys in the core's order, so preset matching works", () => {
    expect(JSON.stringify(defaultGroups(11)[0])).toBe('{"name":"Blue","color":"#3d7eff","zeros":{"min":6,"max":11}}');
    expect(sameGroups(defaultGroups(11), defaultGroups(11))).toBe(true);
    expect(sameGroups(defaultGroups(11), threeTribes(11))).toBe(false);
  });
});

describe('groups table', () => {
  it('moves the neighbouring end with a range end, so the ranges still tile', () => {
    const c = config(threeTribes(11));
    moveBoundary(c.culture.groups, 0, 'max', 5);
    expect(ranges(c)).toEqual([[0, 5], [6, 7], [8, 11]]);
    moveBoundary(c.culture.groups, 2, 'min', 7);
    expect(ranges(c)).toEqual([[0, 5], [6, 6], [7, 11]]);
    expect(tiles(c)).toBe(true);
  });

  it('adds a group by splitting the widest and removes one into its neighbour', () => {
    const c = config(defaultGroups(11));
    addGroup(c);
    expect(ranges(c)).toEqual([[6, 8], [0, 5], [9, 11]]);
    expect(c.culture.groups[2]).toEqual({ name: 'group 3', color: '#3dd66b', zeros: { min: 9, max: 11 } });
    expect(tiles(c)).toBe(true);
    removeGroup(c, 0);
    expect(ranges(c)).toEqual([[0, 8], [9, 11]]);
    removeGroup(c, 0);
    expect(ranges(c)).toEqual([[0, 11]]);
    removeGroup(c, 0);
    expect(c.culture.groups).toHaveLength(1);
    expect(tiles(c)).toBe(true);
  });

  it('adds only while a group can be split and fewer than eight exist', () => {
    expect(canAddGroup(config(defaultGroups(1), 1))).toBe(false);
    const eight = config(Array.from({ length: MAX_GROUPS }, (_, k) => ({ name: `g${k}`, color: '#000000', zeros: { min: k, max: k === 7 ? 11 : k } })));
    expect(canAddGroup(eight)).toBe(false);
    addGroup(eight);
    expect(eight.culture.groups).toHaveLength(MAX_GROUPS);
    expect(canAddGroup(config(defaultGroups(11)))).toBe(true);
  });

  it('rebuilds the table only when the tag length or a range changes', () => {
    const c = config(defaultGroups(11));
    const before = groupsEditorSignature(c);
    c.culture.groups[0].name = 'Azure';
    expect(groupsEditorSignature(c)).toBe(before);
    moveBoundary(c.culture.groups, 0, 'min', 7);
    expect(groupsEditorSignature(c)).not.toBe(before);
    expect(groupsEditorSignature(config(defaultGroups(11), 12))).not.toBe(groupsEditorSignature(config(defaultGroups(11))));
  });
});
