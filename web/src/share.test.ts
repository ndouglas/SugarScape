import { describe, expect, it } from 'vitest';
import type { Config } from './types';
import { base64UrlToBytes, bytesToBase64Url, decodeShare, encodeShare, readHash } from './share';

const config = { width: 50, height: 50, population: 400, sex: { enabled: true } } as unknown as Config;

describe('share links', () => {
  it('round-trips config and seed', async () => {
    const token = await encodeShare({ config, seed: 123456789 });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeShare(token);
    expect(back).toEqual({ config, seed: 123456789 });
  });

  it('round-trips a painted landscape', async () => {
    const landscape = Uint8Array.from({ length: 2500 }, (_, i) => i % 5);
    const back = await decodeShare(await encodeShare({ config, seed: 1, landscape }));
    expect(Array.from(back.landscape!)).toEqual(Array.from(landscape));
  });

  it('compresses a mostly-uniform landscape well', async () => {
    const token = await encodeShare({ config, seed: 1, landscape: new Uint8Array(2500).fill(2) });
    expect(token.length).toBeLessThan(400);
  });

  it('rejects garbage', async () => {
    await expect(decodeShare('not-a-real-token')).rejects.toThrow('not a SugarScape share link');
    const wrong = bytesToBase64Url(new TextEncoder().encode('{}'));
    await expect(decodeShare(wrong)).rejects.toThrow('not a SugarScape share link');
  });

  it('rejects a valid payload whose config is an array', async () => {
    const json = new TextEncoder().encode(JSON.stringify({ v: 1, s: 1, c: [] }));
    const compressed = new Blob([json]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const token = bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
    await expect(decodeShare(token)).rejects.toThrow('not a SugarScape share link');
  });

  it('base64url round-trips every padding length', () => {
    for (const n of [0, 1, 2, 3, 4, 5]) {
      const bytes = Uint8Array.from({ length: n }, (_, i) => 250 - i);
      expect(Array.from(base64UrlToBytes(bytesToBase64Url(bytes)))).toEqual(Array.from(bytes));
    }
  });

  it('reads the share token from the hash', () => {
    expect(readHash('#s=abc_-1')).toBe('abc_-1');
    expect(readHash('#other')).toBeNull();
    expect(readHash('')).toBeNull();
  });
});
