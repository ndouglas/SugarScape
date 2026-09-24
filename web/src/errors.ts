import type { FieldError } from './types';

/** A caught value's message, for a banner or notice. */
export function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

/** Joins field errors into one message, e.g. "seed: must be an integer; size: too small". */
export function fieldErrorsMessage(errors: FieldError[]): string {
  return errors.map((e) => `${e.field}: ${e.message}`).join('; ');
}
