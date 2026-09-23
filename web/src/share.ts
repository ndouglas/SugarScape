import type { Config } from './types';

export interface ShareState { config: Config; seed: number; landscape?: Uint8Array }

interface Wire { v: 1; c: Config; s: number; l?: string }

export function bytesToBase64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function base64UrlToBytes(text: string): Uint8Array {
  const b64 = text.replace(/-/g, '+').replace(/_/g, '/') + '==='.slice((text.length + 3) % 4);
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

async function pipe(bytes: Uint8Array, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const out = new Blob([new Uint8Array(bytes)]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(out).arrayBuffer());
}

/** base64url(deflate-raw(JSON)). */
export async function encodeShare(state: ShareState): Promise<string> {
  const wire: Wire = { v: 1, c: state.config, s: state.seed };
  if (state.landscape) wire.l = bytesToBase64Url(state.landscape);
  const json = new TextEncoder().encode(JSON.stringify(wire));
  return bytesToBase64Url(await pipe(json, new CompressionStream('deflate-raw')));
}

export async function decodeShare(token: string): Promise<ShareState> {
  const json = await pipe(base64UrlToBytes(token), new DecompressionStream('deflate-raw'));
  const wire = JSON.parse(new TextDecoder().decode(json)) as Partial<Wire>;
  if (wire.v !== 1 || typeof wire.s !== 'number' || typeof wire.c !== 'object' || wire.c === null) {
    throw new Error('not a SugarScape share link');
  }
  const state: ShareState = { config: wire.c, seed: wire.s >>> 0 };
  if (wire.l) state.landscape = base64UrlToBytes(wire.l);
  return state;
}

export function readHash(hash: string = location.hash): string | null {
  return /^#s=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
