import { classifyFile } from './experiments/file';
import type { Sweep } from './experiments/types';
import type { EditCommand, LogEntry, PlaceOverrides } from './protocol';
import type { Config } from './types';

/**
 * `landscapes[i]` is good i's painted map (null: generated) — the starting maps — and `log` the
 * edits to replay (Decision 5). Decoding always sets `log` (empty for links made before it).
 */
export interface ShareState { config: Config; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }

/** Two sessions side by side: a `#c=` link or a comparison file (Decision 6). */
export interface CompareState { a: ShareState; b: ShareState }

/** What Export → Session (JSON) writes and Share → Open session… reads. */
export type SessionFile = { kind: 'session'; state: ShareState } | { kind: 'compare'; state: CompareState };

/** v1: before N goods (`l` = sugar's map). v2: `g` = one entry per good. v3: `e` = the edit log. */
interface Wire { v: 1 | 2 | 3; c: Config; s: number; l?: string; g?: (string | null)[]; e?: unknown[][] }

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function base64UrlToBytes(text: string): Uint8Array {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/') + '==='.slice((text.length + 3) % 4);
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

async function deflate(bytes: Uint8Array): Promise<Uint8Array> {
  const out = new Blob([new Uint8Array(bytes)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
  return new Uint8Array(await new Response(out).arrayBuffer());
}

/**
 * Upper bound on a decompressed link payload (guards against deflate bombs). A full 50 000-entry
 * log with live rule changes can pass 1 MiB, so the cap is 16 MiB (Decision 5).
 */
const MAX_DECODED_BYTES = 16 * 1024 * 1024;

async function inflateCapped(bytes: Uint8Array): Promise<Uint8Array> {
  const reader = new Blob([new Uint8Array(bytes)])
    .stream()
    .pipeThrough(new DecompressionStream('deflate-raw'))
    .getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    total += value.byteLength;
    if (total > MAX_DECODED_BYTES) {
      await reader.cancel();
      throw new Error('share payload too large');
    }
    chunks.push(value);
  }
  const out = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    out.set(c, offset);
    offset += c.byteLength;
  }
  return out;
}

/** base64url(deflate-raw(JSON)). */
async function packJson(value: unknown): Promise<string> {
  return bytesToBase64Url(await deflate(new TextEncoder().encode(JSON.stringify(value))));
}

async function unpackJson(token: string): Promise<unknown> {
  return JSON.parse(new TextDecoder().decode(await inflateCapped(base64UrlToBytes(token)))) as unknown;
}

function bad(): never {
  throw new Error('malformed session');
}
const isObject = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);
const int = (v: unknown, min = 0): number => (typeof v === 'number' && Number.isInteger(v) && v >= min ? v : bad());
const finite = (v: unknown, min = -Infinity): number => (typeof v === 'number' && Number.isFinite(v) && v >= min ? v : bad());

/** The log as compact arrays `[tickDelta, code, …args]` (Decision 5). */
export function encodeLog(log: LogEntry[]): unknown[][] {
  let prev = 0;
  return log.map(({ tick, cmd }) => {
    const dt = tick - prev;
    prev = tick;
    switch (cmd.type) {
      case 'paint':
        return [dt, 'p', cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good];
      case 'importLandscape':
        return [dt, 'i', cmd.good, bytesToBase64Url(cmd.capacities)];
      case 'place':
        return Object.keys(cmd.overrides).length > 0 ? [dt, 'a', cmd.x, cmd.y, cmd.overrides] : [dt, 'a', cmd.x, cmd.y];
      case 'erase':
        return [dt, 'x', cmd.x, cmd.y];
      case 'infect':
        return [dt, 'f', cmd.x, cmd.y, cmd.disease];
      case 'vaccinate':
        return [dt, 'v', cmd.x, cmd.y, cmd.radius, cmd.disease];
      case 'setConfig':
        return [dt, 'c', cmd.config];
    }
  });
}

function overrides(v: unknown): PlaceOverrides {
  if (v === undefined) return {};
  if (!isObject(v)) bad();
  const out: PlaceOverrides = {};
  if (v.sex !== undefined) out.sex = v.sex === 'female' || v.sex === 'male' ? v.sex : bad();
  if (v.tribe !== undefined) out.tribe = v.tribe === 'blue' || v.tribe === 'red' ? v.tribe : bad();
  return out;
}

function decodeCommand(code: unknown, a: unknown[]): EditCommand {
  switch (code) {
    case 'p':
      return { type: 'paint', x: int(a[0]), y: int(a[1]), radius: finite(a[2], 0), value: finite(a[3]), good: int(a[4]) };
    case 'i':
      return { type: 'importLandscape', good: int(a[0]), capacities: typeof a[1] === 'string' ? base64UrlToBytes(a[1]) : bad() };
    case 'a':
      return { type: 'place', x: int(a[0]), y: int(a[1]), overrides: overrides(a[2]) };
    case 'x':
      return { type: 'erase', x: int(a[0]), y: int(a[1]) };
    case 'f':
      return { type: 'infect', x: int(a[0]), y: int(a[1]), disease: int(a[2], -1) };
    case 'v':
      return { type: 'vaccinate', x: int(a[0]), y: int(a[1]), radius: finite(a[2], 0), disease: int(a[3]) };
    case 'c':
      return isObject(a[0]) ? { type: 'setConfig', config: a[0] as unknown as Config } : bad();
    default:
      return bad();
  }
}

/** The log from its compact form; throws on anything malformed (the link is then rejected). */
export function decodeLog(e: unknown): LogEntry[] {
  if (!Array.isArray(e)) bad();
  let tick = 0;
  return e.map((item: unknown) => {
    if (!Array.isArray(item)) bad();
    const [dt, code, ...args] = item as unknown[];
    tick += int(dt);
    return { tick, cmd: decodeCommand(code, args) };
  });
}

function toWire(state: ShareState): Wire {
  const wire: Wire = { v: 3, c: state.config, s: state.seed };
  if (state.landscapes?.some((l) => l !== null)) wire.g = state.landscapes.map((l) => (l ? bytesToBase64Url(l) : null));
  if (state.log && state.log.length > 0) wire.e = encodeLog(state.log);
  return wire;
}

function fromWire(value: unknown): ShareState {
  if (!isObject(value)) bad();
  const { v, c, s, l, g, e } = value;
  if ((v !== 1 && v !== 2 && v !== 3) || typeof s !== 'number' || !isObject(c)) bad();
  // A v1 config is in the pre-N-goods shape; the WASM side converts it.
  const state: ShareState = { config: c as unknown as Config, seed: s >>> 0, log: [] };
  if (v === 1 && typeof l === 'string') state.landscapes = [base64UrlToBytes(l)];
  if (v !== 1 && Array.isArray(g)) state.landscapes = g.map((x) => (typeof x === 'string' ? base64UrlToBytes(x) : null));
  if (v === 3 && e !== undefined) state.log = decodeLog(e);
  return state;
}

/** `#s=` links: base64url(deflate-raw(Wire v3)). */
export async function encodeShare(state: ShareState): Promise<string> {
  return packJson(toWire(state));
}

export async function decodeShare(token: string): Promise<ShareState> {
  try {
    return fromWire(await unpackJson(token));
  } catch {
    throw new Error('not a SugarScape share link');
  }
}

export function readHash(hash: string = location.hash): string | null {
  return /^#s=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}

/** `#c=` links: base64url(deflate-raw({ v: 3, a: Wire, b: Wire })) (Decision 6). */
export async function encodeCompare(state: CompareState): Promise<string> {
  return packJson({ v: 3, a: toWire(state.a), b: toWire(state.b) });
}

export async function decodeCompare(token: string): Promise<CompareState> {
  try {
    const json = await unpackJson(token);
    if (!isObject(json) || json.v !== 3) bad();
    return { a: fromWire(json.a), b: fromWire(json.b) };
  } catch {
    throw new Error('not a SugarScape compare link');
  }
}

export function readCompareHash(hash: string = location.hash): string | null {
  return /^#c=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}

/** A session file: the link's content as plain JSON (Decision 6). */
export function sessionFileText(file: SessionFile): string {
  if (file.kind === 'session') return JSON.stringify({ sugarscape: 'session', ...toWire(file.state) });
  return JSON.stringify({ sugarscape: 'compare', v: 3, a: toWire(file.state.a), b: toWire(file.state.b) });
}

export function parseSessionFile(text: string): SessionFile {
  try {
    const json = JSON.parse(text) as unknown;
    if (isObject(json) && json.sugarscape === 'session') return { kind: 'session', state: fromWire(json) };
    if (isObject(json) && json.sugarscape === 'compare') return { kind: 'compare', state: { a: fromWire(json.a), b: fromWire(json.b) } };
  } catch {
    // Falls through to the error below.
  }
  throw new Error('not a SugarScape session file');
}

/** `#x=` links (Decision 22): base64url(deflate-raw(sweep JSON)). */
export async function encodeSweep(sweep: Sweep): Promise<string> {
  return packJson(sweep);
}

export async function decodeSweep(token: string): Promise<Sweep> {
  try {
    const opened = classifyFile(await unpackJson(token));
    if (opened.kind !== 'sweep') throw new Error(opened.kind);
    return opened.sweep;
  } catch {
    throw new Error('not a SugarScape experiment link');
  }
}

export function readSweepHash(hash: string = location.hash): string | null {
  return /^#x=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
