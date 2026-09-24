import type { Session } from './protocol';
import { encodeCompare, encodeShare, type ShareState } from './share';
import type { Config } from './types';

/** Links longer than this still work in browsers, but some apps cut them (Decision 6). */
export const LONG_LINK = 32_000;
export const LONG_NOTICE =
  'This link is long, and some apps cut long links. Export → Session (JSON) saves the same session as a file.';
export const LOG_FULL_NOTICE =
  'The edit log is full (50 000 edits), so this has the setup and painted maps only, not the edits.';

/** What a link or session file is made from; an `Engine` is one. */
export interface SessionSource {
  baseConfig: Config;
  seed: number;
  editedLandscapes(): (Uint8Array | null)[] | undefined;
  session(): Promise<{ session: Session; full: boolean; tick: number }>;
}

/** The whole session, or — when the log overflowed — the setup and painted maps (today's link). */
export async function shareable(source: SessionSource): Promise<{ state: ShareState; full: boolean }> {
  const { session, full } = await source.session();
  if (!full) return { state: session, full };
  return { state: { config: source.baseConfig, seed: source.seed, landscapes: source.editedLandscapes() }, full };
}

function notice(token: string, full: boolean): string | undefined {
  if (full) return LOG_FULL_NOTICE;
  return token.length > LONG_LINK ? LONG_NOTICE : undefined;
}

/** `#s=` for one world's session, and a notice when the link is long or the log was full. */
export async function sessionLink(source: SessionSource): Promise<{ hash: string; notice?: string }> {
  const { state, full } = await shareable(source);
  const token = await encodeShare(state);
  return { hash: `#s=${token}`, notice: notice(token, full) };
}

/** `#c=` for a comparison: both sessions, opening straight into Compare. */
export async function compareLink(a: SessionSource, b: SessionSource): Promise<{ hash: string; notice?: string }> {
  const sa = await shareable(a);
  const sb = await shareable(b);
  const token = await encodeCompare({ a: sa.state, b: sb.state });
  return { hash: `#c=${token}`, notice: notice(token, sa.full || sb.full) };
}
