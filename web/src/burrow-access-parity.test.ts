import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { burrow_access_replay_json, initSync } from './wasm-pkg/sugarscape.js';

initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const examples = ['access-explore', 'access-known-goal'];
const seeds = ['7', '18446744073709551615'];
const configPath = (example: string) => fileURLToPath(new URL(`../../docs/examples/burrow/${example}.json`, import.meta.url));
const configText = readFileSync(configPath('access-known-goal'), 'utf8');
function errors(config: string, seed: string, sampleEvery: number, ticks = 16) {
  try {
    burrow_access_replay_json(config, seed, ticks, sampleEvery);
  } catch (error) {
    return JSON.parse(String(error)) as { field: string; message: string }[];
  }
  throw new Error('expected checked rejection');
}

describe('Burrow resource-access native/WASM full-record parity', () => {
  for (const example of examples) {
    it.each(seeds)(`${example} seed %s matches every shared episode field`, seed => {
      const text = readFileSync(configPath(example), 'utf8');
      const scratch = mkdtempSync(`${tmpdir()}/burrow-parity-`);
      try {
        const out = `${scratch}/out`;
        execFileSync(fileURLToPath(new URL('../../target/release/sugarscape', import.meta.url)), [
          'burrow-access', '--config', configPath(example), '--seed', seed,
          '--ticks', '16', '--sample-every', '4', '--out', out,
        ], { cwd: fileURLToPath(new URL('../../', import.meta.url)), encoding: 'utf8' });
        const native = JSON.parse(readFileSync(`${out}/episode.json`, 'utf8'));
        expect(JSON.parse(burrow_access_replay_json(text, seed, 16, 4))).toEqual(native);
      } finally {
        rmSync(scratch, { recursive: true, force: true });
      }
    });
  }
  it.each(['-1', '7.0', '18446744073709551616'])('rejects strict decimal seed %s', seed => {
    expect(errors(configText, seed, 4)).toEqual([{ field: 'seed', message: expect.any(String) }]);
  });
  const invalidNumbers = [
    { label: 'fractional', value: 1.5 },
    { label: 'negative', value: -1 },
    { label: 'NaN', value: Number.NaN },
    { label: 'positive Infinity', value: Number.POSITIVE_INFINITY },
    { label: 'negative Infinity', value: Number.NEGATIVE_INFINITY },
    { label: 'u32 overflow', value: 4294967296 },
  ];
  it.each(invalidNumbers)('rejects $label ticks before unsigned conversion', ({ value }) => {
    expect(errors(configText, '7', 4, value)).toEqual([
      { field: 'ticks', message: expect.stringMatching(/finite integer/) },
    ]);
  });
  it.each(invalidNumbers)('rejects $label sampling before unsigned conversion', ({ value }) => {
    expect(errors(configText, '7', value)).toEqual([
      { field: 'sample_every', message: expect.stringMatching(/finite integer/) },
    ]);
  });
  it('accepts zero ticks without consuming worker opportunities', () => {
    const episode = JSON.parse(burrow_access_replay_json(configText, '7', 0, 4));
    expect(episode.episode).toMatchObject({ requested_ticks: 0, completed_ticks: 0, events: [] });
    expect(episode.access).toMatchObject({ first_access: null, observed_opportunities: 0, deadline_censored: true });
  });
  it('rejects zero sampling at the checked boundary', () => {
    expect(errors(configText, '7', 0)).toContainEqual({ field: 'sample_every', message: expect.any(String) });
  });
  it('reports contextual JSON parse errors', () => {
    expect(errors('{invalid', '7', 4)).toEqual([{ field: 'burrow_access_config', message: expect.any(String) }]);
  });
  it('reports invalid goal fields', () => {
    const config = JSON.parse(configText);
    config.task.goal.x = 999;
    expect(errors(JSON.stringify(config), '7', 4))
      .toContainEqual({ field: 'task.goal', message: expect.any(String) });
  });
  it('rejects unsupported fixtures', () => {
    const config = JSON.parse(configText);
    config.lab.fixture = { corridor: { length: 9, workers: 2 } };
    expect(errors(JSON.stringify(config), '7', 4))
      .toContainEqual({ field: 'lab.fixture', message: expect.any(String) });
  });
  it('rejects unknown task coordinate fields contextually', () => {
    const config = JSON.parse(configText);
    config.task.goal.extra = true;
    expect(errors(JSON.stringify(config), '7', 4))
      .toContainEqual({ field: 'burrow_access_config', message: expect.any(String) });
  });
});
