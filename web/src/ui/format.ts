/** Largest-first magnitude/suffix pairs for `compactNumber`. */
const UNITS: [number, string][] = [
  [1e9, 'B'],
  [1e6, 'M'],
  [1e3, 'k'],
];

/**
 * Compact axis-tick formatting: 1234 -> "1.2k", 25000 -> "25k", 1500000 -> "1.5M".
 * Values under 1000 (and whole-number scaled values) render without a decimal.
 */
export function compactNumber(n: number): string {
  const sign = n < 0 ? '-' : '';
  const abs = Math.abs(n);
  const [threshold, suffix] = UNITS.find(([t]) => abs >= t) ?? [1, ''];
  const scaled = Math.round((abs / threshold) * 10) / 10;
  const text = Number.isInteger(scaled) ? String(scaled) : scaled.toFixed(1);
  return sign + text + suffix;
}
