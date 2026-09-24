/** A caught value's message, for a banner or notice. */
export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
