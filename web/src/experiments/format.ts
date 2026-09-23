/** Formats a summary value for display: 4 significant figures, or an em dash for null (NaN). */
export function fmt(v: number | null): string {
  return v === null ? '—' : Number(v.toPrecision(4)).toString();
}
