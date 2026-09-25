import { describe, expect, it } from 'vitest';
import { groupParams, paramEdit, paramInput } from './schema-form';
import type { ModelConfig, Param, RingConfig, SchellingConfig } from './types';

const ring = (): RingConfig => ({
  model: 'ring',
  sites: 150,
  agents: 40,
  vision: { min: 15, max: 30 },
  capacity: 4,
  growback: 1,
  start: 'random',
});
const param = (p: Partial<Param> & Pick<Param, 'path' | 'kind'>): Param => ({ label: p.path, apply: 'reset', group: 'Setup', ...p });

describe('the schema form', () => {
  it('groups params in the order their groups first appear', () => {
    const params = [param({ path: 'a', kind: 'integer' }), param({ path: 'b', kind: 'bool', group: 'Other' }), param({ path: 'c', kind: 'integer' })];
    expect(groupParams(params).map((s) => [s.group, s.params.map((p) => p.path)])).toEqual([
      ['Setup', ['a', 'c']],
      ['Other', ['b']],
    ]);
  });

  it('writes each kind of field: whole numbers, numbers, ranges, switches and choices', () => {
    const c: ModelConfig = ring();
    paramEdit(param({ path: 'sites', kind: 'integer' }), '99.6')(c);
    paramEdit(param({ path: 'growback', kind: 'number', step: 0.1 }), '2.5')(c);
    paramEdit(param({ path: 'vision', kind: 'range', step: 1 }), { min: '3', max: '7.2' })(c);
    paramEdit(param({ path: 'start', kind: 'choice' }), 'megagroup')(c);
    expect(c).toMatchObject({ sites: 100, growback: 2.5, vision: { min: 3, max: 7 }, start: 'megagroup' });
    const s = { model: 'schelling', preference: { min: 0.25, max: 0.25 }, residence: { enabled: false, min: 80, max: 100 } } as SchellingConfig;
    paramEdit(param({ path: 'preference', kind: 'range', step: 0.05 }), { min: '0.25', max: '0.5' })(s);
    paramEdit(param({ path: 'residence.enabled', kind: 'bool' }), true)(s);
    paramEdit(param({ path: 'residence', kind: 'range', step: 1 }), { min: '10', max: '20' })(s);
    expect(s.preference).toEqual({ min: 0.25, max: 0.5 });
    expect(s.residence).toEqual({ enabled: true, min: 10, max: 20 });
  });

  it('keeps a range in order: min raised past max raises max, max lowered past min lowers min', () => {
    const pref = param({ path: 'preference', kind: 'range', step: 0.05 });
    const s = { model: 'schelling', preference: { min: 0.25, max: 0.25 } } as SchellingConfig;
    paramEdit(pref, { min: '0.5', max: '0.25', edited: 'min' })(s);
    expect(s.preference).toEqual({ min: 0.5, max: 0.5 });
    paramEdit(pref, { min: '0.5', max: '0.1', edited: 'max' })(s);
    expect(s.preference).toEqual({ min: 0.1, max: 0.1 });
    const c = ring();
    const vision = param({ path: 'vision', kind: 'range', step: 1 });
    paramEdit(vision, { min: '40', max: '30', edited: 'min' })(c);
    expect(c.vision).toEqual({ min: 40, max: 40 });
    paramEdit(vision, { min: '40', max: '5', edited: 'max' })(c);
    expect(c.vision).toEqual({ min: 5, max: 5 });
    // An ordered edit, or one with no edited end, is written as typed.
    paramEdit(vision, { min: '2', max: '9', edited: 'min' })(c);
    expect(c.vision).toEqual({ min: 2, max: 9 });
    paramEdit(vision, { min: '9', max: '2' })(c);
    expect(c.vision).toEqual({ min: 9, max: 2 });
  });

  it('refuses a path the config does not have', () => {
    expect(() => paramEdit(param({ path: 'vision.maxx', kind: 'integer' }), '3')(ring())).toThrow('unknown field vision.maxx');
  });

  it('reads each field back as its control shows it', () => {
    const c = ring();
    expect(paramInput(param({ path: 'vision', kind: 'range' }), c)).toEqual({ min: '15', max: '30' });
    expect(paramInput(param({ path: 'start', kind: 'choice' }), c)).toBe('random');
    expect(paramInput(param({ path: 'growback', kind: 'number' }), c)).toBe('1');
  });
});
