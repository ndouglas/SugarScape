import { base64UrlToBytes, bytesToBase64Url, deflate, inflateCapped } from '../share';
import type { Json } from './types';
export const MAX_INPUT_BYTES = 64 * 1024;
export function object(value: unknown): value is Record<string, Json> { return typeof value === 'object' && value !== null && !Array.isArray(value); }
export function checkedInput(input: Json): string {
  const text = JSON.stringify(input);
  if (!object(input) || typeof input.study !== 'string') throw new Error('episode input needs a study');
  if (new TextEncoder().encode(text).byteLength > MAX_INPUT_BYTES) throw new Error('episode input exceeds 64 KiB');
  return text;
}
export async function encodeEpisode(input: Json): Promise<string> {
  checkedInput(input);
  const bytes = new TextEncoder().encode(JSON.stringify({ v: 1, input }));
  if (bytes.byteLength > MAX_INPUT_BYTES) throw new Error('episode link exceeds 64 KiB');
  return bytesToBase64Url(await deflate(bytes));
}
export async function decodeEpisode(token: string): Promise<Json> {
  if (!/^[A-Za-z0-9_-]+$/.test(token) || token.length > 90000) throw new Error('invalid episode link');
  const bytes = await inflateCapped(base64UrlToBytes(token), MAX_INPUT_BYTES);
  const value: unknown = JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  if (!object(value) || value.v !== 1 || !object(value.input)) throw new Error('incompatible episode link');
  checkedInput(value.input);
  return value.input;
}
export function readEpisodeHash(hash = location.hash): string | null { return /^#e=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null; }
