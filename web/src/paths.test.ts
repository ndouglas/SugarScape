import { describe, expect, it } from 'vitest';
import { errorsFor, getPath, setPath } from './paths';

describe('paths', () => {
  const obj = () => ({ a: 1, sex: { enabled: false, fertility_onset: { min: 12, max: 15 } } });

  it('reads nested values', () => {
    expect(getPath(obj(), 'sex.fertility_onset.max')).toBe(15);
    expect(getPath(obj(), 'nope.x')).toBeUndefined();
  });

  it('writes nested values in place', () => {
    const o = obj();
    setPath(o, 'sex.enabled', true);
    setPath(o, 'sex.fertility_onset', { min: 1, max: 2 });
    expect(o.sex.enabled).toBe(true);
    expect(o.sex.fertility_onset).toEqual({ min: 1, max: 2 });
  });

  it('rejects unknown paths', () => {
    expect(() => setPath(obj(), 'sex.bogus', 1)).toThrow(/unknown field/);
    expect(() => setPath(obj(), 'a.b', 1)).toThrow();
  });

  it('matches errors to a control and its sub-fields', () => {
    const errors = [
      { field: 'vision.max', message: 'too big' },
      { field: 'vision', message: 'min > max' },
      { field: 'visionary', message: 'unrelated' },
    ];
    expect(errorsFor(errors, 'vision').map((e) => e.message)).toEqual(['too big', 'min > max']);
  });
});
