import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json, experiment_validate_json, experiment_recorded_json } from '../wasm-pkg/sugarscape.js';
import type { StudyDescriptor } from './types';

initSync({ module: readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const cwd = fileURLToPath(new URL('../../../', import.meta.url));
function native(op: string, text?: string): unknown {
  const scratch = mkdtempSync(`${tmpdir()}/episode-parity-`);
  try {
    const args = ['experiment-view', op];
    if (text !== undefined) {
      const path = `${scratch}/input.json`;
      writeFileSync(path, text);
      args.push('--input', path);
    }
    const options = { cwd, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024 + 1 };
    return JSON.parse(execFileSync(`${cwd}/target/release/sugarscape`, args, options));
  } finally { rmSync(scratch, { recursive: true, force: true }); }
}
function errors(call: () => string): { field: string; message: string }[] {
  try { call(); } catch (error) { return JSON.parse(String(error)); }
  throw new Error('expected checked boundary rejection');
}
const completeCatalog = JSON.parse(experiment_catalog_json()) as StudyDescriptor[];
// Keep all original eight families and their complete import/parity contract.
const catalog = completeCatalog.filter(study => study.family !== 'spatial');
describe('complete native and actual WASM experiment records', () => {
  it('exports the same eight original study descriptors', () => {
    expect(catalog).toEqual((native('catalog') as StudyDescriptor[]).filter(study => study.family !== 'spatial'));
    expect(catalog.map(study => study.id)).toEqual(['wink', 'testimony', 'testimony_game', 'strategic_reporting', 'strategy_inference', 'adversarial_audit', 'shared_surface', 'active_surface']);
  });
  for (const study of catalog) {
    it(`${study.id} default preserves every native payload and checkpoint field`, () => {
      const input = JSON.stringify(study.default_input);
      const text = experiment_run_json(input);
      const record = JSON.parse(text);
      const nativeRecord=native('run', input);
      // Rust compares the complete structure and only the explicitly approved legacy float paths.
      expect(JSON.parse(experiment_validate_json(JSON.stringify(nativeRecord)))).toEqual(record);
      if(study.id!=='testimony_game')expect(record).toEqual(nativeRecord);
      expect(record.checkpoints.length).toBeLessThanOrEqual(4096);
      expect(new TextEncoder().encode(text).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
    }, 120_000);
  }
  it('preserves the max u64 seed and imports by complete fresh reconstruction', () => {
    const input = JSON.stringify({ study: 'wink', seed: '18446744073709551615', policy: 'passive', mode: 'ordinary' });
    const text = experiment_run_json(input);
    expect(JSON.parse(text)).toEqual(native('run', input));
    expect(JSON.parse(experiment_validate_json(text))).toEqual(native('validate', text));
  });
  it('exports both complete retained search measurements with every curve', () => {
    const text = experiment_recorded_json();
    expect(JSON.parse(text)).toEqual(native('recorded'));
    expect(new TextEncoder().encode(text).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
  }, 30_000);
  it.each(['{', '{"study":"testimony","fixture":"transfer","extra":true}', '{"study":"wink","seed":"18446744073709551616","policy":"passive","mode":"ordinary"}', ' '.repeat(65_537)])('rejects malformed or oversized raw input %#', text => {
    expect(errors(() => experiment_run_json(text))).toContainEqual({ field: 'input', message: expect.any(String) });
  });
  it('rejects oversized records before parsing', () => {
    expect(errors(() => experiment_validate_json(' '.repeat(16 * 1024 * 1024 + 1)))).toContainEqual({ field: 'episode', message: expect.any(String) });
  });
  it('cannot import arbitrary true flags, unknown versions, zero fractions or checkpoint overflow', () => {
    const original = JSON.parse(experiment_run_json('{"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}'));
    const flag = structuredClone(original); flag.passed = true;
    const version = structuredClone(original); version.version = 2;
    const fraction = structuredClone(original); fraction.payload.reference_posterior.denominator = '0';
    const checkpoints = structuredClone(original); checkpoints.checkpoints = Array(4097).fill(original.checkpoints[0]);
    for (const [record, field] of [[flag, 'episode'], [version, 'version'], [fraction, 'episode'], [checkpoints, 'checkpoints']] as const) {
      expect(errors(() => experiment_validate_json(JSON.stringify(record)))).toContainEqual({ field, message: expect.any(String) });
    }
  });
});

// Captured before the authorized sampler correction, from the immutable copied native64 boundary.
const nativeReference = JSON.parse(readFileSync(`${cwd}/crates/sugarscape-core/src/browser_experiments/fixtures/wink-portability.json`, 'utf8')) as { native_reference_cases: { input: unknown; fingerprint: string; checkpoints: number }[] };
describe('representative native64 Wink preservation and real WASM portability', () => {
  for (const reference of nativeReference.native_reference_cases) {
    it(JSON.stringify(reference.input), () => {
      const text=JSON.stringify(reference.input);
      const record=JSON.parse(experiment_run_json(text));
      expect(record.payload.fingerprint).toBe(reference.fingerprint);
      expect(record.checkpoints.length).toBe(reference.checkpoints);
      expect(record).toEqual(native('run',text));
    },30_000);
  }
});
