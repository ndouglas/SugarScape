import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { env } from 'node:process';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json, experiment_validate_json, protection_episode_json, deception_episode_json } from '../wasm-pkg/sugarscape.js';
import { projectCheckpoint } from './projection';
import { rendererAvailable } from './catalog';
import type { EpisodeRecord, Json, StudyDescriptor } from './types';
type Obj = { [key: string]: Json };
const root = fileURLToPath(new URL('../../../', import.meta.url));
const preservation = JSON.parse(readFileSync(`${root}/web/src/episodes/caching-preservation.json`, 'utf8')) as { exports: { name: string; sha256: string; seed: string; lab: Obj; study: string }[]; scientific_files: Record<string, string>; guides: { files: Record<string, string>; prefix_bytes: Record<string, number> } };
const wasmBytes = readFileSync(new URL(`file://${root}/web/src/wasm-pkg/sugarscape_bg.wasm`));
initSync({ module: wasmBytes });
const bytes = (path: string) => readFileSync(new URL(`file://${path}`));
async function sha(value: string | Uint8Array<ArrayBuffer>): Promise<string> {
  const input = typeof value === 'string' ? new TextEncoder().encode(value) : value;
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', input)), n => n.toString(16).padStart(2, '0')).join('');
}
const obj = (v: Json) => v as Obj;
const catalog = JSON.parse(experiment_catalog_json()) as StudyDescriptor[];
const scenes = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/caching-scenes.json`, 'utf8')) as { scenes: { id: string; input: Obj }[] };
const seeds = ['7', '18446744073709551615'];
/** Tokenize strings first; integer atoms become decimal strings before parsing
 * can round a u64. Float tokens stay floats, matching the Rust wire codec. */
function losslessNative(text: string): Json {
  return JSON.parse(text.replace(/"(?:[^"\\]|\\.)*"|-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/g,
    token => token.startsWith('"') || /[.eE]/.test(token) ? token : JSON.stringify(token))) as Json;
}
function native(op: 'run' | 'validate', text: string): string {
  const dir = mkdtempSync(`${tmpdir()}/caching-parity-`);
  try {
    writeFileSync(`${dir}/input.json`, text);
    const options = { cwd: root, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024 + 1, stdio: 'pipe' as const };
    return execFileSync(`${root}/target/release/sugarscape`, ['experiment-view', op, '--input', `${dir}/input.json`], options);
  } finally { rmSync(dir, { recursive: true, force: true }); }
}
const reject = (call: () => string) => expect(call).toThrow();
const fixtures = [...scenes.scenes];
for (const scene of scenes.scenes.filter(s => ['protection-observed-selective', 'deception-sham-ambiguous'].includes(s.id))) {
  fixtures.push({ id: `${scene.id}-default`, input: structuredClone(catalog.find(d => d.id === scene.input.study)!.default_input) as Obj });
  const mirrored = structuredClone(scene.input); obj(mirrored.lab).mirrored = true;
  fixtures.push({ id: `${scene.id}-mirror`, input: mirrored });
  if (scene.input.study === 'protection_recaching') {
    const max = structuredClone(scene.input); obj(max.lab).exposure_span = seeds[1];
    fixtures.push({ id: 'protection-max-span', input: max });
  }
}
for (const [id, changes] of [
  ['deception-clear-zero', { view: 'clear', effort_cost: 0 }],
  ['deception-on-route', { layout: 'on_route' }],
  ['deception-ordinary', { sender: 'ordinary' }],
] as const) {
  const input = structuredClone(scenes.scenes.find(s => s.input.study === 'deception_gestures')!.input);
  Object.assign(obj(input.lab), changes); fixtures.push({ id, input });
}
describe('caching complete native/WASM parity without tolerance', () => {
  it('retains fifteen runnable entries and seven spatial renderers', () => {
    expect(catalog).toHaveLength(15); expect(catalog.filter(d => d.family === 'spatial')).toHaveLength(7);
    for (const d of catalog) expect(rendererAvailable(d)).toBe(true);
  });
  it('lossless parser preserves max integers, strings and floats', () => {
    expect(losslessNative('{"seed":18446744073709551615,"float":0.0,"string":"123\\\"45","negative":-1}')).toEqual({ seed: seeds[1], float: 0, string: '123"45', negative: '-1' });
  });
  for (const fixture of fixtures) for (const seed of seeds) it(`${fixture.id} seed ${seed} compares complete records and projections`, async () => {
    const input: Obj = { ...fixture.input, seed };
    const nativeText = native('run', JSON.stringify(input)); const wasmText = experiment_run_json(JSON.stringify(input));
    const expected = JSON.parse(nativeText) as EpisodeRecord; const actual = JSON.parse(wasmText) as EpisodeRecord;
    if (env.CACHING_PARITY_EVIDENCE_DIR) {
      mkdirSync(env.CACHING_PARITY_EVIDENCE_DIR, { recursive: true });
      writeFileSync(`${env.CACHING_PARITY_EVIDENCE_DIR}/${fixture.id}-${seed}.json`, JSON.stringify({ input, native_json: nativeText, wasm_json: wasmText, native_sha256: await sha(bytes(`${root}/target/release/sugarscape`)), wasm_sha256: await sha(wasmBytes) }, null, 2));
    }
    expect(actual).toEqual(expected);
    expect(JSON.parse(native('validate', wasmText))).toEqual(expected);
    expect(JSON.parse(experiment_validate_json(nativeText))).toEqual(expected);
    expect(experiment_run_json(JSON.stringify(input))).toBe(wasmText); expect(native('run', JSON.stringify(input))).toBe(nativeText);
    expect(actual.checkpoints.length).toBeLessThanOrEqual(65); expect(new TextEncoder().encode(wasmText).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
    for (let i = 0; i < actual.checkpoints.length; i++) {
      expect(projectCheckpoint(actual, i, { kind: 'researcher' })).toEqual(projectCheckpoint(expected, i, { kind: 'researcher' }));
      for (const agent of Object.keys(actual.checkpoints[i].local)) expect(projectCheckpoint(actual, i, { kind: 'agent', agent })).toEqual(projectCheckpoint(expected, i, { kind: 'agent', agent }));
    }
    const lab = obj(input.lab);
    // Build the original numeric LabConfig as text, never round max-span in JS.
    const bare = JSON.stringify(lab).replace(/"exposure_span":"([0-9]+)"/, '"exposure_span":$1');
    const raw = input.study === 'protection_recaching' ? protection_episode_json(bare, seed) : deception_episode_json(bare, seed);
    expect(losslessNative(raw)).toEqual(obj(actual.payload).native);
  });
});
describe('caching import authority and strict bounds', () => {
  for (const study of ['protection_recaching', 'deception_gestures']) it(`${study} rejects individually forged paths`, () => {
    const input = scenes.scenes.find(s => s.input.study === study)!.input;
    const record = JSON.parse(experiment_run_json(JSON.stringify(input))) as EpisodeRecord;
    const mutations: ((r: EpisodeRecord) => void)[] = [
      r => { obj(r as unknown as Json).version = 99; }, r => { r.rules_identity += ':foreign'; }, r => { obj(r.input).seed = seeds[1]; },
      r => { obj(r.payload).invented_terminal = true; }, r => { obj(obj(r.payload).native).seed = '0'; }, r => { obj(obj(r.checkpoints[1].researcher).frame).fingerprint = 'forged'; },
      r => { obj(r.checkpoints[1].clock).native_tick = '01'; },
    ];
    const excessive = structuredClone(record);
    excessive.checkpoints = Array.from({ length: 4097 }, (_, index) => ({ index, clock: null, kind: 'caching_initial', public: null, local: {}, researcher: null }));
    const excessiveText = JSON.stringify(excessive);
    expect(new TextEncoder().encode(excessiveText).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
    try { experiment_validate_json(excessiveText); throw new Error('expected checkpoint rejection'); }
    catch (error) { expect(JSON.parse(String(error))).toContainEqual({ field: 'checkpoints', message: expect.any(String) }); }
    try { native('validate', excessiveText); throw new Error('expected checkpoint rejection'); }
    catch (error) { expect(String((error as { stderr?: string }).stderr)).toMatch(/^checkpoints:/m); }
    const index = record.checkpoints.findIndex(c => Object.values(c.local).some(v => v !== null)); const agent = Object.keys(record.checkpoints[index].local)[0];
    mutations.push(r => { obj(r.checkpoints[index].local[agent]).seen = ['forged']; });
    if (study === 'deception_gestures') mutations.push(r => { obj(r.checkpoints[index].local[agent]).received_observations = ['forged']; });
    for (const mutate of mutations) { const forged = structuredClone(record); mutate(forged); reject(() => experiment_validate_json(JSON.stringify(forged))); reject(() => native('validate', JSON.stringify(forged))); }
    for (const seed of ['07', '+7', ' 7', '18446744073709551616', 7]) reject(() => experiment_run_json(JSON.stringify({ ...input, seed })));
    const unknown = structuredClone(input); obj(unknown.lab).invented = true; reject(() => experiment_run_json(JSON.stringify(unknown)));
    const missing = structuredClone(input); delete obj(missing.lab).mirrored; reject(() => experiment_run_json(JSON.stringify(missing)));
    const numeric = structuredClone(record); const own = obj(numeric.checkpoints[index].local[agent]); obj(own.agent).holdings = Number(obj(own.agent).holdings) + 1e-13;
    reject(() => experiment_validate_json(JSON.stringify(numeric)));
    const bad = structuredClone(input); obj(bad.lab)[study === 'deception_gestures' ? 'effort_cost' : 'reburial_cost'] = study === 'deception_gestures' ? 1 : -1; reject(() => experiment_run_json(JSON.stringify(bad)));
  });
  it('rejects malformed fixture fields and P3 probability/cost domains', () => {
    const original = scenes.scenes[0].input;
    for (const [key, value] of [['discovery', -0.1], ['discovery', 1.1], ['reburial_cost', -1], ['exposure_span', '01'], ['observer_span', -1]] as const) {
      const input = structuredClone(original); obj(input.lab)[key] = value; reject(() => experiment_run_json(JSON.stringify(input)));
    }
    const extra = structuredClone(original); obj(obj(extra.lab).fixture).extra = true; reject(() => experiment_run_json(JSON.stringify(extra)));
    const missing = structuredClone(original); delete obj(obj(missing.lab).fixture).initial_observed; reject(() => experiment_run_json(JSON.stringify(missing)));
    reject(() => experiment_run_json(JSON.stringify(original).replace('0.25', 'NaN')));
  });
  it('checks raw byte limits before parsing', () => {
    reject(() => experiment_run_json(' '.repeat(64 * 1024 + 1))); reject(() => experiment_validate_json(' '.repeat(16 * 1024 * 1024 + 1)));
  });
});
// Test-only historical source authentication; production episode identities stay frozen.
async function cachingIdentityBytes(path: string, live: Uint8Array<ArrayBuffer>): Promise<Uint8Array<ArrayBuffer>> {
  if (path !== 'crates/sugarscape-core/src/lib.rs') return live;
  const historical = bytes(`${root}/web/src/episodes/test-fixtures/pre-war-core-lib.rs.txt`);
  expect(await sha(historical)).toBe('6ac710e9fe3521e6591dc8edc02b884e2aaae55a36298ab7313d9a9fb4c85f4f');
  const nativeGate = '\n#[cfg(all(feature = "war-benchmarks", not(target_arch = "wasm32")))]\npub mod war;\n';
  // Compare every byte, including whitespace; no stripping or hash exemption.
  const gateBytes = new TextEncoder().encode(nativeGate);
  const expectedLive = new Uint8Array(historical.length + gateBytes.length);
  expectedLive.set(historical);
  expectedLive.set(gateBytes, historical.length);
  expect(new Uint8Array(live), 'live lib.rs differs beyond the authorized native W1 gate').toEqual(expectedLive);
  return historical;
}
describe('caching source identity integrity', () => {
  it('rejects arbitrary live lib changes despite authentic historical bytes', async () => {
    const path = 'crates/sugarscape-core/src/lib.rs';
    const live = new TextDecoder().decode(bytes(`${root}/${path}`));
    for (const changed of [
      `${live}\n// unrelated delta\n`,
      live.replace('not(target_arch = "wasm32")', 'target_arch = "wasm32"'),
      live.replace('pub mod world;', '// removed world'),
      live.replace('feature = "war-benchmarks"', 'feature = "other-feature"'),
    ]) {
      expect(changed).not.toBe(live);
      await expect(cachingIdentityBytes(path, new TextEncoder().encode(changed))).rejects.toThrow();
    }
  });
  const identities = JSON.parse(readFileSync(`${root}/crates/sugarscape-core/src/browser_experiments/fixtures/caching-identities.json`, 'utf8')) as Record<string, { source_files: Record<string, string>; source_sha256: string }>;
  for (const [study, identity] of Object.entries(identities)) it(`${study} authenticates paths, files and canonical digest`, async () => {
    let canonical = '';
    for (const [path, digest] of Object.entries(identity.source_files).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) {
      expect(path).toMatch(/^(Cargo\.lock|crates\/sugarscape-core\/src\/.+\.rs)$/); expect(path.split('/')).not.toContain('..');
      const actual = await sha(await cachingIdentityBytes(path, bytes(`${root}/${path}`))); expect(actual, path).toBe(digest); canonical += `${path}\0${actual}\n`;
    }
    expect(await sha(canonical)).toBe(identity.source_sha256);
  });
});
describe('preimplementation exports and guide preservation', () => {
  for (const saved of preservation.exports) it(saved.name, async () => {
    const raw = saved.study === 'protection_recaching' ? protection_episode_json(JSON.stringify(saved.lab), saved.seed) : deception_episode_json(JSON.stringify(saved.lab), saved.seed);
    expect(await sha(raw)).toBe(saved.sha256);
    expect(obj(losslessNative(raw)).seed).toBe(saved.seed);
  });
  it('preserves original scientific reports and published settings', async () => {
    for (const [path, digest] of Object.entries(preservation.scientific_files)) expect(await sha(bytes(`${root}/${path}`)), path).toBe(digest);
  });
  it('preserves frozen guide prefixes', async () => {
    for (const [path, digest] of Object.entries(preservation.guides.files)) expect(await sha(bytes(`${root}/${path}`).subarray(0, preservation.guides.prefix_bytes[path])), path).toBe(digest);
  });
});
