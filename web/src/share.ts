import type { Config } from './types';

/** `landscapes[i]` is good i's painted map, or null where it is generated. */
export interface ShareState { config: Config; seed: number; landscapes?: (Uint8Array | null)[] }

/** v1: before N goods (`l` = sugar's map). v2: `g` = one entry per good. */
interface Wire { v: 1 | 2; c: Config; s: number; l?: string; g?: (string | null)[] }

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

/** Upper bound on a decompressed share payload (guards against deflate bombs). */
const MAX_DECODED_BYTES = 1024 * 1024;

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
export async function encodeShare(state: ShareState): Promise<string> {
  const wire: Wire = { v: 2, c: state.config, s: state.seed };
  if (state.landscapes?.some((l) => l !== null)) {
    wire.g = state.landscapes.map((l) => (l ? bytesToBase64Url(l) : null));
  }
  const json = new TextEncoder().encode(JSON.stringify(wire));
  return bytesToBase64Url(await deflate(json));
}

export async function decodeShare(token: string): Promise<ShareState> {
  try {
    const json = await inflateCapped(base64UrlToBytes(token));
    const wire = JSON.parse(new TextDecoder().decode(json)) as Partial<Wire>;
    if (
      (wire.v !== 1 && wire.v !== 2) ||
      typeof wire.s !== 'number' ||
      typeof wire.c !== 'object' ||
      wire.c === null ||
      Array.isArray(wire.c)
    ) {
      throw new Error('not a SugarScape share link');
    }
    // A v1 config is in the pre-N-goods shape; the WASM side converts it.
    const state: ShareState = { config: wire.c, seed: wire.s >>> 0 };
    if (wire.v === 1 && typeof wire.l === 'string') state.landscapes = [base64UrlToBytes(wire.l)];
    if (wire.v === 2 && Array.isArray(wire.g)) {
      state.landscapes = wire.g.map((x) => (typeof x === 'string' ? base64UrlToBytes(x) : null));
    }
    return state;
  } catch {
    throw new Error('not a SugarScape share link');
  }
}

export function readHash(hash: string = location.hash): string | null {
  return /^#s=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
