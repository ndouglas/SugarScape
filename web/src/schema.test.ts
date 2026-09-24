import { describe, expect, it } from 'vitest';
import { defaultGroups, threeTribes } from './groups';
import { GROUPS } from './schema';
import type { Config, TagGroup } from './types';

const config = (groups: TagGroup[], tagLength: number): Config =>
  ({ tag_length: tagLength, culture: { enabled: false, groups } }) as unknown as Config;
const control = (path: string) => GROUPS.flatMap((g) => g.controls).find((c) => c.path === path)!;

describe('tag length', () => {
  it('rebuilds the default groups for the new length', () => {
    const next = config(defaultGroups(11), 5);
    control('tag_length').adjust!(next, config(defaultGroups(11), 11));
    expect(next.culture.groups).toEqual(defaultGroups(5));
  });

  it('keeps custom groups (validation reports a mismatch)', () => {
    const next = config(threeTribes(11), 5);
    control('tag_length').adjust!(next, config(threeTribes(11), 11));
    expect(next.culture.groups).toEqual(threeTribes(11));
  });
});

describe('culture', () => {
  it('shows the groups table', () => {
    expect(GROUPS.find((g) => g.enable === 'culture.enabled')?.custom).toBe('groups');
  });
});
