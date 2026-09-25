// The schema-driven Rules panel's pure parts (Decision 8): grouping, reading and writing fields.
import { getPath, setPath } from './paths';
import type { ModelConfig, Param } from './types';

/** What a control's input holds: a number box's text, a checkbox, a choice, or a range's two boxes. */
export type ParamInput = string | boolean | { min: string; max: string };

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
 * sent as typed; the core validates them and names the field in its errors.
 */
export function paramEdit(p: Param, input: ParamInput): (c: ModelConfig) => void {
  return (c) => {
    if (p.kind === 'range') {
      const r = input as { min: string; max: string };
      setPath(c, `${p.path}.min`, number(p, r.min));
      setPath(c, `${p.path}.max`, number(p, r.max));
    } else if (p.kind === 'bool') {
      setPath(c, p.path, input === true);
    } else if (p.kind === 'choice') {
      setPath(c, p.path, String(input));
    } else {
      setPath(c, p.path, number(p, String(input)));
    }
  };
}

/** The control's current value in `config`, as its input shows it. */
export function paramInput(p: Param, config: ModelConfig): ParamInput {
  const v = getPath(config, p.path);
  if (p.kind === 'range') {
    const r = v as { min: number; max: number };
    return { min: String(r.min), max: String(r.max) };
  }
  if (p.kind === 'bool') return v === true;
  return String(v);
}
