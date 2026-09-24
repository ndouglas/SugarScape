import { describe, expect, it } from 'vitest';
import type { Sweep } from './experiments/types';
import { LEGACY_SHARE_TOKEN } from './legacy-share.fixture';
import {
  base64UrlToBytes,
  bytesToBase64Url,
  decodeShare,
  decodeSweep,
  encodeShare,
  encodeSweep,
  readHash,
  readSweepHash,
} from './share';
import type { Config } from './types';

const config = { width: 50, height: 50, population: 400, sex: { enabled: true } } as unknown as Config;

describe('share links', () => {
  it('round-trips config and seed', async () => {
    const token = await encodeShare({ config, seed: 123456789 });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeShare(token);
    expect(back).toEqual({ config, seed: 123456789 });
  });

  it('round-trips per-good painted landscapes', async () => {
    const painted = Uint8Array.from({ length: 2500 }, (_, i) => i % 5);
    const back = await decodeShare(await encodeShare({ config, seed: 1, landscapes: [null, painted] }));
    expect(back.landscapes?.length).toBe(2);
    expect(back.landscapes?.[0]).toBeNull();
    expect(Array.from(back.landscapes![1]!)).toEqual(Array.from(painted));
  });

  it('compresses a mostly-uniform landscape well', async () => {
    const token = await encodeShare({ config, seed: 1, landscapes: [new Uint8Array(2500).fill(2)] });
    expect(token.length).toBeLessThan(400);
  });

  it('rejects an unknown version', async () => {
    const json = new TextEncoder().encode(JSON.stringify({ v: 3, s: 1, c: {} }));
    const compressed = new Blob([json]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const token = bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
    await expect(decodeShare(token)).rejects.toThrow('not a SugarScape share link');
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

  it('rejects a payload that decompresses past the size cap', async () => {
    const wire = JSON.stringify({ v: 1, s: 1, c: {}, l: 'A'.repeat(2 * 1024 * 1024) });
    const compressed = new Blob([wire]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const token = bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
    expect(token.length).toBeLessThan(10_000);
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

describe('legacy share links', () => {
  it('decodes a link made before N goods, its map as good 0', async () => {
    const state = await decodeShare(LEGACY_SHARE_TOKEN);
    expect(state.seed).toBe(7);
    const config = state.config as unknown as Record<string, { enabled?: boolean }>;
    expect(config.spice?.enabled).toBe(true);
    expect(state.landscapes?.length).toBe(1);
    expect(state.landscapes?.[0]?.length).toBe(2500);
  });
});

describe('experiment links', () => {
  const sweep: Sweep = {
    name: 'Vision',
    base: { preset: 'ii-2-unit' },
    x: { path: 'vision.max', values: [1, 2, 3] },
    seeds: { from: 1, count: 2 },
    ticks: 100,
    metric: { kind: 'final', series: 'population' },
  };

  it('round-trips a sweep', async () => {
    const token = await encodeSweep(sweep);
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    expect(await decodeSweep(token)).toEqual(sweep);
  });

  it('reads the #x= hash, and #s= stays the playground', () => {
    expect(readSweepHash('#x=ab_-9')).toBe('ab_-9');
    expect(readSweepHash('#s=abc')).toBeNull();
    expect(readHash('#x=abc')).toBeNull();
  });

  it('rejects playground links and garbage', async () => {
    const playground = await encodeShare({ config, seed: 1 });
    await expect(decodeSweep(playground)).rejects.toThrow('not a SugarScape experiment link');
    await expect(decodeSweep('garbage')).rejects.toThrow('not a SugarScape experiment link');
  });
});
