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
    expect(rule.options.map((o) => o.value)).toEqual(['book', 'utility', 'goap', 'mvt']);
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

describe('goap and mvt', () => {
  it('offers GOAP and the marginal-value rule as decision rules', () => {
    const rule = control('decision.rule');
    if (rule.kind !== 'select') throw new Error('rule is a select');
    const c = {} as unknown as Config;
    rule.options.find((o) => o.value === 'goap')!.apply(c);
    expect(c.decision).toEqual({ rule: 'goap', travel: 0, crowding: 0, idle: 'stay' });
    rule.options.find((o) => o.value === 'mvt')!.apply(c);
    expect(rule.current(c)).toBe('mvt');
    expect(rule.options.map((o) => o.label)).toContain('GOAP (plan)');
    expect(rule.options.map((o) => o.label)).toContain('Marginal value (leave below your average)');
  });

  it('offers k, horizon, shortlist and α live, in their ranges', () => {
    const ranges: [string, number, number, number][] = [
      ['goap.k', 1, 12, 1],
      ['goap.horizon', 1, 100, 1],
      ['mvt.alpha', 0.01, 1, 0.01],
    ];
    for (const [path, min, max, step] of ranges) {
      const k = control(path);
      if (k.kind !== 'number') throw new Error(`${path} is a number`);
      expect(k.reset).toBeUndefined();
      expect([k.min, k.max, k.step]).toEqual([min, max, step]);
    }
    const shortlist = control('goap.shortlist');
    if (shortlist.kind !== 'select') throw new Error('shortlist is a select');
    expect(shortlist.reset).toBeUndefined();
    expect(shortlist.options.map((o) => [o.value, o.label])).toEqual([
      ['rate', 'Most sugar per step (rate)'],
      ['value', 'Most sugar (value)'],
    ]);
    const c = {} as unknown as Config;
    expect(shortlist.current(c)).toBe('rate'); // older configs have no goap
    shortlist.options[1].apply(c);
    expect(c.goap).toEqual({ k: 8, horizon: 10, shortlist: 'value' });
  });

  it('creates complete goap and mvt objects when a number control is set on a config missing them', () => {
    const c = {} as unknown as Config;
    for (const [path, value] of [['goap.k', 3], ['goap.horizon', 20], ['mvt.alpha', 0.2]] as const) {
      const before = structuredClone(c);
      control(path).adjust!(c, before);
      setPath(c, path, value);
    }
    expect(c.goap).toEqual({ k: 3, horizon: 20, shortlist: 'rate' });
    expect(c.mvt).toEqual({ alpha: 0.2 });
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

describe('memory', () => {
  it('offers span, share (reset-only) and belief (live), defaulting older configs off (span 0, project)', () => {
    const span = control('memory.span');
    expect(span.kind).toBe('number');
    if (span.kind !== 'number') return;
    expect(span.reset).toBe(true);
    expect(span.min).toBe(0);
    expect(span.max).toBe(10_000);

    const share = control('memory.share');
    expect(share.kind).toBe('number');
    if (share.kind !== 'number') return;
    expect(share.reset).toBe(true);
    expect(share.min).toBe(0);
    expect(share.max).toBe(1);

    const belief = control('memory.belief');
    expect(belief.kind).toBe('select');
    if (belief.kind !== 'select') return;
    expect(belief.reset).toBeUndefined();
    expect(belief.options.map((o) => o.value)).toEqual(['recall', 'project']);
    const c = {} as unknown as Config;
    expect(belief.current(c)).toBe('project'); // older configs have no memory
    belief.options[0].apply(c);
    expect(belief.current(c)).toBe('recall');
    expect(c.memory).toEqual({ span: 0, share: 1, belief: 'recall', prior: 'none' });
  });

  it('offers the prior, reset-only, defaulting older configs (and older memory objects) to none', () => {
    const prior = control('memory.prior');
    if (prior.kind !== 'select') throw new Error('prior is a select');
    expect(prior.reset).toBe(true);
    expect(prior.options.map((o) => o.value)).toEqual(['none', 'map']);
    expect(prior.current({} as unknown as Config)).toBe('none');
    const older = { memory: { span: 100, share: 1, belief: 'project' } } as unknown as Config;
    expect(prior.current(older)).toBe('none');
    prior.options[1].apply(older);
    expect(older.memory).toEqual({ span: 100, share: 1, belief: 'project', prior: 'map' });
  });

  it('creates a complete memory object when a number control is set on a config missing it', () => {
    const span = control('memory.span');
    const c = {} as unknown as Config;
    const before = structuredClone(c);
    expect(() => setPath(c, 'memory.span', 50)).toThrow();
    span.adjust!(c, before);
    setPath(c, 'memory.span', 50);
    expect(c.memory).toEqual({ span: 50, share: 1, belief: 'project', prior: 'none' });
  });
});

describe('truffles', () => {
  it('offers share and seed (reset-only), value and regrow (live), defaulting older configs off (share 0)', () => {
    const share = control('truffles.share');
    expect(share.kind).toBe('number');
    if (share.kind !== 'number') return;
    expect(share.reset).toBe(true);
    expect(share.min).toBe(0);
    expect(share.max).toBe(1);

    const value = control('truffles.value');
    expect(value.kind).toBe('number');
    if (value.kind !== 'number') return;
    expect(value.reset).toBeUndefined();

    const regrow = control('truffles.regrow');
    expect(regrow.kind).toBe('number');
    if (regrow.kind !== 'number') return;
    expect(regrow.reset).toBeUndefined();

    const seed = control('truffles.seed');
    expect(seed.kind).toBe('number');
    if (seed.kind !== 'number') return;
    expect(seed.reset).toBe(true);
  });

  it('creates a complete truffles object when a number control is set on a config missing it', () => {
    const value = control('truffles.value');
    const c = {} as unknown as Config;
    const before = structuredClone(c);
    expect(() => setPath(c, 'truffles.value', 8)).toThrow();
    value.adjust!(c, before);
    setPath(c, 'truffles.value', 8);
    expect(c.truffles).toEqual({ share: 0, value: 8, regrow: 30, seed: 1 });
  });
});

describe('caching (Minds 5)', () => {
  const group = GROUPS.find((g) => g.title === 'Caching (Minds 5)')!;

  it('is a Minds group holding the caching controls, the seasons mode and central-place foraging', () => {
    expect(group.minds).toBe(true);
    expect(group.controls.map((c) => c.path)).toEqual([
      'caching.rule', 'caching.mixed', 'caching.capacity', 'caching.share', 'caching.lambda', 'caching.lookahead', 'seasons.mode', 'central.enabled',
    ]);
    // The book's Seasons group keeps its own controls; the mode lives here.
    const seasons = GROUPS.find((g) => g.title === 'Seasons')!;
    expect(seasons.minds).toBeUndefined();
    expect(seasons.controls.map((c) => c.path)).toEqual(['seasons.period', 'seasons.winter_divisor']);
  });

  it('raises the winter slowdown β to 32, which the winter presets use', () => {
    const beta = control('seasons.winter_divisor');
    if (beta.kind !== 'number') throw new Error('β is a number');
    expect([beta.min, beta.max]).toEqual([1, 32]);
  });

  it('makes rule, mixed, capacity, seasons mode and central reset-only, and share, λ and lookahead live, in their ranges', () => {
    for (const path of ['caching.rule', 'caching.mixed', 'caching.capacity', 'seasons.mode', 'central.enabled']) expect(control(path).reset).toBe(true);
    const ranges: [string, number, number, number][] = [
      ['caching.capacity', 0, 500, 1],
      ['caching.share', 0.05, 1, 0.05],
      ['caching.lambda', 0.05, 1, 0.05],
      ['caching.lookahead', 1, 10, 1],
    ];
    for (const [path, min, max, step] of ranges) {
      const k = control(path);
      if (k.kind !== 'number') throw new Error(`${path} is a number`);
      expect([k.min, k.max, k.step]).toEqual([min, max, step]);
      if (path !== 'caching.capacity') expect(k.reset).toBeUndefined();
    }
    expect(control('caching.mixed').label).toBe('Mix the rules (a quarter each, by id)');
  });

  it('reads older configs as rule none, hemispheres, and seeds complete objects when set', () => {
    const rule = control('caching.rule');
    const mode = control('seasons.mode');
    if (rule.kind !== 'select' || mode.kind !== 'select') throw new Error('selects');
    const c = { seasons: { enabled: true, winter_divisor: 8, period: 50 } } as unknown as Config;
    expect(rule.current(c)).toBe('none');
    expect(mode.current(c)).toBe('hemispheres');
    expect(rule.options.map((o) => o.value)).toEqual(['none', 'even', 'compensate', 'plan']);
    rule.options.find((o) => o.value === 'plan')!.apply(c);
    expect(c.caching).toEqual({ rule: 'plan', capacity: 0, share: 0.5, lambda: 0.5, lookahead: 1, mixed: false });
    mode.options.find((o) => o.value === 'global')!.apply(c);
    expect(c.seasons).toEqual({ enabled: true, winter_divisor: 8, period: 50, mode: 'global' });
  });

  it('creates complete caching and central objects when a control is set on a config missing them', () => {
    const c = {} as unknown as Config;
    expect(() => setPath(structuredClone(c), 'caching.capacity', 20)).toThrow();
    expect(() => setPath(structuredClone(c), 'central.enabled', true)).toThrow();
    for (const [path, value] of [['caching.capacity', 20], ['caching.lookahead', 3], ['caching.mixed', true], ['central.enabled', true]] as const) {
      control(path).adjust!(c, structuredClone(c));
      setPath(c, path, value);
    }
    expect(c.caching).toEqual({ rule: 'none', capacity: 20, share: 0.5, lambda: 0.5, lookahead: 3, mixed: true });
    expect(c.central).toEqual({ enabled: true });
  });
});
