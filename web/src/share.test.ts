import { describe, expect, it } from 'vitest';
import type { Sweep } from './experiments/types';
import { LEGACY_SHARE_TOKEN } from './legacy-share.fixture';
import type { LogEntry } from './protocol';
import {
  base64UrlToBytes,
  bytesToBase64Url,
  decodeCompare,
  decodeLog,
  decodeShare,
  decodeSweep,
  encodeCompare,
  encodeLog,
  encodeShare,
  encodeSweep,
  parseSessionFile,
  readCompareHash,
  readHash,
  readSweepHash,
  sessionFileText,
} from './share';
import { modelOf } from './models';
import type { Config, SchellingConfig } from './types';

const config = { width: 50, height: 50, population: 400, sex: { enabled: true } } as unknown as Config;

/** A share token for any wire object (to test what the encoder would never write). */
async function tokenOf(wire: unknown): Promise<string> {
  const compressed = new Blob([JSON.stringify(wire)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
  return bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
}

const log: LogEntry[] = [
  { tick: 0, cmd: { type: 'paint', x: 3, y: 4, radius: 1.5, value: 2, good: 1 } },
  { tick: 0, cmd: { type: 'importLandscape', good: 0, capacities: Uint8Array.from({ length: 2500 }, (_, i) => i % 11) } },
  { tick: 7, cmd: { type: 'place', x: 1, y: 2, overrides: {} } },
  { tick: 7, cmd: { type: 'place', x: 2, y: 2, overrides: { sex: 'female', tribe: 'red' } } },
  { tick: 12, cmd: { type: 'erase', x: 1, y: 2 } },
  { tick: 40, cmd: { type: 'infect', x: 9, y: 9, disease: -1 } },
  { tick: 41, cmd: { type: 'vaccinate', x: 9, y: 9, radius: 2, disease: 3 } },
  { tick: 1000, cmd: { type: 'setConfig', config } },
];

describe('share links', () => {
  it('round-trips config and seed', async () => {
    const token = await encodeShare({ config, seed: 123456789 });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeShare(token);
    expect(back).toEqual({ config, seed: 123456789, log: [] });
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
    const json = new TextEncoder().encode(JSON.stringify({ v: 4, s: 1, c: {} }));
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
    const wire = JSON.stringify({ v: 1, s: 1, c: {}, l: 'A'.repeat(17 * 1024 * 1024) });
    const compressed = new Blob([wire]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const token = bytesToBase64Url(new Uint8Array(await new Response(compressed).arrayBuffer()));
    expect(token.length).toBeLessThan(40_000);
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

describe('session links', () => {
  it('round-trips an edit log of every kind', async () => {
    const back = await decodeShare(await encodeShare({ config, seed: 5, log }));
    expect(back).toEqual({ config, seed: 5, log });
  });

  it('writes entries compactly, with ticks as deltas', () => {
    expect(encodeLog(log.slice(2, 5))).toEqual([
      [7, 'a', 1, 2],
      [0, 'a', 2, 2, { sex: 'female', tribe: 'red' }],
      [5, 'x', 1, 2],
    ]);
    expect(decodeLog(encodeLog(log))).toEqual(log);
  });

  it('decodes links made before the edit log with an empty log', async () => {
    expect(await decodeShare(await tokenOf({ v: 2, c: config, s: 9, g: [null] }))).toEqual({
      config,
      seed: 9,
      landscapes: [null],
      log: [],
    });
    expect((await decodeShare(LEGACY_SHARE_TOKEN)).log).toEqual([]);
  });

  it('rejects a malformed edit log', async () => {
    const bad: unknown[] = [
      [[0, 'q', 1]],
      [[-1, 'x', 1, 1]],
      [[0, 'x', 1.5, 1]],
      [[0, 'p', 1, 1, -1, 2, 0]],
      [[0, 'a', 1, 1, { sex: 'other' }]],
      [[0, 'f', 1, 1, -2]],
      [[0, 'i', 0, 7]],
      [[0, 'c', []]],
      'x',
    ];
    for (const e of bad) {
      await expect(decodeShare(await tokenOf({ v: 3, c: config, s: 1, e }))).rejects.toThrow('not a SugarScape share link');
    }
  });
});

describe('compare links and session files', () => {
  const b = { config: { ...config, population: 10 } as Config, seed: 6, landscapes: [null, new Uint8Array(2500).fill(3)], log: log.slice(0, 3) };

  it('round-trips two sessions in a #c= link', async () => {
    const token = await encodeCompare({ a: { config, seed: 5, log }, b });
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    const back = await decodeCompare(token);
    expect(back.a).toEqual({ config, seed: 5, log });
    expect(back.b.seed).toBe(6);
    expect(back.b.log).toEqual(b.log);
    expect(back.b.landscapes?.[0]).toBeNull();
    expect(Array.from(back.b.landscapes![1]!)).toEqual(Array.from(b.landscapes[1]!));
    await expect(decodeCompare(await encodeShare({ config, seed: 1 }))).rejects.toThrow('not a SugarScape compare link');
    await expect(decodeCompare('garbage')).rejects.toThrow('not a SugarScape compare link');
  });

  it('reads #c= apart from #s= and #x=', () => {
    expect(readCompareHash('#c=ab_-9')).toBe('ab_-9');
    expect(readCompareHash('#s=abc')).toBeNull();
    expect(readHash('#c=abc')).toBeNull();
    expect(readSweepHash('#c=abc')).toBeNull();
  });

  it('writes and reads session files of one world or two', () => {
    expect(parseSessionFile(sessionFileText({ kind: 'session', state: { config, seed: 5, log } }))).toEqual({
      kind: 'session',
      state: { config, seed: 5, log },
    });
    const two = parseSessionFile(sessionFileText({ kind: 'compare', state: { a: { config, seed: 5, log }, b } }));
    if (two.kind !== 'compare') throw new Error(two.kind);
    expect(two.state.a.log).toEqual(log);
    expect(two.state.b.seed).toBe(6);
    expect(() => parseSessionFile('{"v":3}')).toThrow('not a SugarScape session file');
    expect(() => parseSessionFile('nope')).toThrow('not a SugarScape session file');
  });
});

describe('models in links', () => {
  it('opens a link made before milestone 9 as a sugarscape', async () => {
    const opened = await decodeShare(LEGACY_SHARE_TOKEN);
    expect(modelOf(opened.config)).toBe('sugarscape');
  });

  it('carries another model’s tagged config through a link, a compare link and a session file', async () => {
    const schelling = { model: 'schelling', width: 50, height: 50, population: 2000 } as SchellingConfig;
    const back = await decodeShare(await encodeShare({ config: schelling, seed: 3 }));
    expect(back.config).toEqual(schelling);
    expect(modelOf(back.config)).toBe('schelling');
    const pair = await decodeCompare(await encodeCompare({ a: { config: schelling, seed: 1 }, b: { config: schelling, seed: 2 } }));
    expect(modelOf(pair.b.config)).toBe('schelling');
    const file = parseSessionFile(sessionFileText({ kind: 'session', state: { config: schelling, seed: 4 } }));
    expect(file.kind === 'session' && modelOf(file.state.config)).toBe('schelling');
  });

  it('refuses a comparison of two models (a link or a file)', async () => {
    const schelling = { model: 'schelling' } as SchellingConfig;
    const mixed = { a: { config, seed: 1 }, b: { config: schelling, seed: 1 } };
    await expect(decodeCompare(await encodeCompare(mixed))).rejects.toThrow('not a SugarScape compare link');
    expect(() => parseSessionFile(sessionFileText({ kind: 'compare', state: mixed }))).toThrow('not a SugarScape session file');
  });
});
