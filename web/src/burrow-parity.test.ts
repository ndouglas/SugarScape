import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { burrow_replay_json, initSync } from './wasm-pkg/sugarscape.js';

initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const examples = ['direct-blind', 'direct-responsive', 'relay-blind', 'relay-responsive'];
const seeds = ['7', '18446744073709551615'];
const configPath = (example: string) => fileURLToPath(new URL(`../../docs/examples/burrow/${example}.json`, import.meta.url));
const configText = readFileSync(configPath('relay-responsive'), 'utf8');
function errors(config: string, seed: string, sampleEvery: number) {
  try {
    burrow_replay_json(config, seed, 16, sampleEvery);
  } catch (error) {
    return JSON.parse(String(error)) as { field: string; message: string }[];
  }
  throw new Error('expected checked rejection');
}

describe('Burrow native/WASM full-record parity', () => {
  for (const example of examples) {
    it.each(seeds)(`${example} seed %s matches every shared episode field`, seed => {
      const text = readFileSync(configPath(example), 'utf8');
      const scratch = mkdtempSync(`${tmpdir()}/burrow-parity-`);
      try {
        const out = `${scratch}/out`;
        execFileSync(fileURLToPath(new URL('../../target/release/sugarscape', import.meta.url)), [
          'burrow', '--config', configPath(example), '--seed', seed,
          '--ticks', '16', '--sample-every', '4', '--out', out,
        ], { cwd: fileURLToPath(new URL('../../', import.meta.url)), encoding: 'utf8' });
        const native = JSON.parse(readFileSync(`${out}/episode.json`, 'utf8'));
        expect(JSON.parse(burrow_replay_json(text, seed, 16, 4))).toEqual(native);
      } finally {
        rmSync(scratch, { recursive: true, force: true });
      }
    });
  }
  it.each(['-1', '7.0', '18446744073709551616'])('rejects strict decimal seed %s', seed => {
    expect(errors(configText, seed, 4)).toEqual([{ field: 'seed', message: expect.any(String) }]);
  });
  it('rejects zero sampling through the shared checked core', () => {
    expect(errors(configText, '7', 0)).toContainEqual({ field: 'sample_every', message: expect.any(String) });
  });
  it('reports contextual JSON parse errors', () => {
    expect(errors('{invalid', '7', 4)).toEqual([{ field: 'burrow_config', message: expect.any(String) }]);
  });
  it('reports invalid configuration fields', () => {
    expect(errors(JSON.stringify({ freshness_window: 0 }), '7', 4))
      .toContainEqual({ field: 'freshness_window', message: expect.any(String) });
  });
});
