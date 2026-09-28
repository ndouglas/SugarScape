import { describe, expect, it } from 'vitest';
import { defaultGroups, threeTribes } from './groups';
import { setPath } from './paths';
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

describe('trade', () => {
  it('offers the two price rules, applied live', () => {
    const price = control('trade.price');
    expect(price.kind).toBe('select');
    if (price.kind !== 'select') return;
    expect(price.options.map((o) => o.value)).toEqual(['geometric_mean', 'random']);
    expect(price.reset).toBeUndefined();
    const c = { trade: { enabled: true, price: 'geometric_mean' } } as unknown as Config;
    price.options[1].apply(c);
    expect(price.current(c)).toBe('random');
    price.options[0].apply(c);
    expect(c.trade.price).toBe('geometric_mean');
  });
});

describe('decision', () => {
  it('offers the book and the utility mind, rebuilding the world, with live knobs', () => {
    const rule = control('decision.rule');
    expect(rule.kind).toBe('select');
    if (rule.kind !== 'select') return;
    expect(rule.reset).toBe(true);
    expect(rule.options.map((o) => o.value)).toEqual(['book', 'utility']);
    const c = {} as unknown as Config;
    expect(rule.current(c)).toBe('book'); // older configs have no decision
    rule.options[1].apply(c);
    expect(rule.current(c)).toBe('utility');
    expect(c.decision).toEqual({ rule: 'utility', travel: 0, crowding: 0, idle: 'stay' });
    for (const path of ['decision.travel', 'decision.crowding']) {
      const k = control(path);
      expect(k.kind).toBe('number');
      expect(k.reset).toBeUndefined();
    }
    const idle = control('decision.idle');
    if (idle.kind !== 'select') throw new Error('idle is a select');
    expect(idle.options.map((o) => o.value)).toEqual(['stay', 'wander']);
    const d = {} as unknown as Config;
    idle.options[1].apply(d);
    expect(d.decision?.idle).toBe('wander');
    expect(d.decision?.rule).toBe('book');
  });

  it('creates a complete decision object when a number control is set on a config missing it', () => {
    const travel = control('decision.travel');
    const c = {} as unknown as Config;
    const before = structuredClone(c);
    expect(() => setPath(c, 'decision.travel', 2)).toThrow();
    travel.adjust!(c, before);
    setPath(c, 'decision.travel', 2);
    expect(c.decision).toEqual({ rule: 'book', travel: 2, crowding: 0, idle: 'stay' });
  });
});

describe('movement', () => {
  it('offers jump and walk, live, defaulting older configs to jump', () => {
    const mode = control('movement.mode');
    expect(mode.kind).toBe('select');
    if (mode.kind !== 'select') return;
    expect(mode.reset).toBeUndefined();
    expect(mode.options.map((o) => o.value)).toEqual(['jump', 'walk']);
    const c = {} as unknown as Config;
    expect(mode.current(c)).toBe('jump'); // older configs have no movement
    mode.options[1].apply(c);
    expect(mode.current(c)).toBe('walk');
    expect(c.movement).toEqual({ mode: 'walk', speed: 1 });

    const speed = control('movement.speed');
    expect(speed.kind).toBe('number');
    if (speed.kind !== 'number') return;
    expect(speed.reset).toBeUndefined();
    expect(speed.min).toBe(1);
    expect(speed.max).toBe(50);
  });

  it('creates a complete movement object when speed is set on a config missing it', () => {
    const speed = control('movement.speed');
    const c = {} as unknown as Config;
    const before = structuredClone(c);
    expect(() => setPath(c, 'movement.speed', 5)).toThrow();
    speed.adjust!(c, before);
    setPath(c, 'movement.speed', 5);
    expect(c.movement).toEqual({ mode: 'jump', speed: 5 });
  });
});
