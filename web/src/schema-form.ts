// The schema-driven Rules panel's pure parts (Decision 8): grouping, reading and writing fields.
import { getPath, setPath } from './paths';
import type { ModelConfig, Param } from './types';

/**
 * What a control's input holds: a number box's text, a checkbox, a choice, or a range's two boxes
 * (with the end just edited, if any).
 */
export type ParamInput = string | boolean | { min: string; max: string; edited?: 'min' | 'max' };

/** The params in panel sections, in the order each group first appears. */
export function groupParams(params: Param[]): { group: string; params: Param[] }[] {
  const out: { group: string; params: Param[] }[] = [];
  for (const p of params) {
    let section = out.find((s) => s.group === p.group);
    if (!section) out.push((section = { group: p.group, params: [] }));
    section.params.push(p);
  }
  return out;
}

/** A number from a box: whole for `integer` params and ranges stepping by whole numbers. */
function number(p: Param, raw: string): number {
  const v = Number(raw);
  return p.kind === 'integer' || (p.kind === 'range' && Number.isInteger(p.step ?? 1)) ? Math.round(v) : v;
}

/**
 * The edit a control makes: sets `path` (a range sets `path.min` and `path.max`). The values are
 * sent as typed; the core validates them and names the field in its errors. A range keeps its ends
 * in order: editing min above max raises max to match, and editing max below min lowers min. An
 * empty box of a `nullable` field sets null.
 */
export function paramEdit(p: Param, input: ParamInput): (c: ModelConfig) => void {
  return (c) => {
    if (p.kind === 'range') {
      const r = input as { min: string; max: string; edited?: 'min' | 'max' };
      let [lo, hi] = [number(p, r.min), number(p, r.max)];
      if (lo > hi) {
        if (r.edited === 'min') hi = lo;
        else if (r.edited === 'max') lo = hi;
      }
      setPath(c, `${p.path}.min`, lo);
      setPath(c, `${p.path}.max`, hi);
    } else if (p.kind === 'bool') {
      setPath(c, p.path, input === true);
    } else if (p.kind === 'choice') {
      setPath(c, p.path, String(input));
    } else if (p.nullable && String(input).trim() === '') {
      setPath(c, p.path, null);
    } else {
      setPath(c, p.path, number(p, String(input)));
    }
  };
}

/**
 * Whether a field shows in `config`: always, or while its `show_if` field equals its value (a bool
 * field compared as `'true'` or `'false'`).
 */
export function paramShown(p: Param, config: ModelConfig): boolean {
  return !p.show_if || String(getPath(config, p.show_if.path)) === p.show_if.equals;
}

/** The control's current value in `config`, as its input shows it. */
export function paramInput(p: Param, config: ModelConfig): ParamInput {
  const v = getPath(config, p.path);
  if (p.kind === 'range') {
    const r = v as { min: number; max: number };
    return { min: String(r.min), max: String(r.max) };
  }
  if (p.kind === 'bool') return v === true;
  // A nullable field's null (the ethnocentrism model's tag mutation: "the mutation rate") is an empty box.
  if (v === null && p.nullable) return '';
  return String(v);
}

/**
 * The ids a control's `aria-describedby` names: its help (when it has one) and its error slot while
 * that shows an error, so a screen reader reads both with the control.
 */
export function describedBy(ids: { help: string | null; error: string }, hasError: boolean): string | null {
  const list = [ids.help, hasError ? ids.error : null].filter((id): id is string => id !== null);
  return list.length > 0 ? list.join(' ') : null;
}
