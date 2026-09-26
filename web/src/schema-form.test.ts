import { describe, expect, it } from 'vitest';
import { describedBy, groupParams, paramEdit, paramInput, paramShown } from './schema-form';
import type { AnasaziConfig, EthnoConfig, ModelConfig, Param, RingConfig, SchellingConfig } from './types';

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

  it('reads and writes the anasazi’s quirks by their dotted paths', () => {
    const c = { model: 'anasazi', quirks: { occupancy_leak: true, wrap_edges: true } } as unknown as AnasaziConfig;
    const leak = param({ path: 'quirks.occupancy_leak', kind: 'bool', help: 'A household that moves… (A-19).' });
    expect(paramInput(leak, c)).toBe(true);
    paramEdit(leak, false)(c);
    expect(c.quirks).toEqual({ occupancy_leak: false, wrap_edges: true });
  });

  it('describes a control by its help, and by its error while one shows', () => {
    const ids = { help: 'f-help', error: 'f-error' };
    expect(describedBy(ids, false)).toBe('f-help');
    expect(describedBy(ids, true)).toBe('f-help f-error');
    expect(describedBy({ help: null, error: 'g-error' }, true)).toBe('g-error');
    expect(describedBy({ help: null, error: 'g-error' }, false)).toBeNull();
  });

  it('shows a field with show_if only while its condition holds', () => {
    const p = { path: 'max_age', label: 'Longest life', kind: 'integer', apply: 'live', group: 'Population', show_if: { path: 'variant', equals: 'ethnic' } } as Param;
    const always = { ...p, show_if: undefined } as Param;
    const ethnic = { model: 'civil', variant: 'ethnic' } as unknown as ModelConfig;
    const rebellion = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect([paramShown(p, ethnic), paramShown(p, rebellion)]).toEqual([true, false]);
    expect(paramShown(always, rebellion)).toBe(true);
  });

  it('shows a field while a bool field is on (the ethnocentrism model’s kin fields)', () => {
    const p = { path: 'kin_basis', label: 'Tag or kin basis', kind: 'choice', apply: 'reset', group: 'Traits', show_if: { path: 'kin_strategies', equals: 'true' } } as Param;
    const kin = (on: boolean) => ({ model: 'ethno', kin_strategies: on }) as unknown as ModelConfig;
    expect([paramShown(p, kin(true)), paramShown(p, kin(false))]).toEqual([true, false]);
  });

  it('shows a nullable field’s null as an empty box and writes an empty box as null', () => {
    const p = param({ path: 'tag_mutation', kind: 'number', step: 0.005, nullable: true });
    const c = { model: 'ethno', tag_mutation: null, mutation: 0.005 } as unknown as EthnoConfig;
    expect(paramInput(p, c)).toBe('');
    paramEdit(p, '0.3')(c);
    expect(c.tag_mutation).toBe(0.3);
    expect(paramInput(p, c)).toBe('0.3');
    paramEdit(p, ' ')(c);
    expect(c.tag_mutation).toBeNull();
    // A field that is not nullable reads an empty box as a number, as before.
    const r = ring();
    paramEdit(param({ path: 'growback', kind: 'number' }), '')(r);
    expect(r.growback).toBe(0);
  });
});
