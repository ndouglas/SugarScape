import type { FieldError } from './types';

type Obj = Record<string, unknown>;

export function getPath(obj: unknown, path: string): unknown {
  return path.split('.').reduce<unknown>((o, k) => (o && typeof o === 'object' ? (o as Obj)[k] : undefined), obj);
}

/** Sets an existing field (so key order, and preset comparison, is preserved). */
export function setPath<T extends object>(obj: T, path: string, value: unknown): T {
  const keys = path.split('.');
  const last = keys.pop()!;
  let o = obj as Obj;
  for (const k of keys) {
    const next = o[k];
    if (!next || typeof next !== 'object') throw new Error(`no object at "${k}" in ${path}`);
    o = next as Obj;
  }
  if (!(last in o)) throw new Error(`unknown field ${path}`);
  o[last] = value;
  return obj;
}

/** Errors for a control at `path`, including errors on its sub-fields. */
export function errorsFor(errors: FieldError[], path: string): FieldError[] {
  return errors.filter((e) => e.field === path || e.field.startsWith(`${path}.`));
}
