export type AxisScalar = number | boolean;
export type Parsed = { values: AxisScalar[] } | { error: string };

const NUMBER = /^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/;

function number(token: string): number | null {
  return NUMBER.test(token) ? Number(token) : null;
}

/** Tidies float steps: 0 + 3 × 0.1 is 0.3, not 0.30000000000000004. */
function tidy(v: number): number {
  return Number(v.toPrecision(12));
}

/**
 * Parses an axis's values: `a, b, c` (all numbers or all true/false) or
 * `from:to:step` (numbers, step > 0, `to` included when on a step).
 */
export function parseValues(text: string, max: number): Parsed {
  const t = text.trim();
  if (t === '') return { error: 'Enter values: a, b, c or from:to:step' };
  if (t.includes(':')) {
    const parts = t.split(':').map((s) => s.trim());
    if (parts.length !== 3) return { error: 'A range is from:to:step' };
    const [from, to, step] = parts.map(number);
    if (from === null || to === null || step === null) return { error: 'A range needs three numbers' };
    if (!(step > 0)) return { error: 'The step must be > 0' };
    if (to < from) return { error: 'The range must not end before it starts' };
    const count = Math.floor((to - from) / step + 1e-9) + 1;
    if (count > max) return { error: `At most ${max} values` };
    return { values: Array.from({ length: count }, (_, k) => tidy(from + k * step)) };
  }
  const values: AxisScalar[] = [];
  for (const token of t.split(',').map((s) => s.trim())) {
    if (token === 'true' || token === 'false') {
      values.push(token === 'true');
      continue;
    }
    const n = number(token);
    if (n === null) return { error: `"${token}" is not a number, true or false` };
    values.push(n);
  }
  if (new Set(values.map((v) => typeof v)).size > 1) return { error: 'Use numbers or true/false, not both' };
  if (values.length > max) return { error: `At most ${max} values` };
  return { values };
}

export function formatValues(values: AxisScalar[]): string {
  return values.map(String).join(', ');
}
