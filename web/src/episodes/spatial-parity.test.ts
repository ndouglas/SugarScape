import { execFileSync } from 'node:child_process';
import { env } from 'node:process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json, experiment_validate_json } from '../wasm-pkg/sugarscape.js';
import { rendererAvailable } from './catalog';
import type { EpisodeRecord, Json, StudyDescriptor } from './types';

type ObjectJson = { [key: string]: Json };
type Path = (string | number)[];
interface InspectionLeaf { path: Path; category: 'target' | 'inspection_float' | 'snapshot_bytes'; value: Json }
const cwd = fileURLToPath(new URL('../../../', import.meta.url));
const wasmBytes = readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm', import.meta.url));
initSync({ module: wasmBytes });
async function hash(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), value => value.toString(16).padStart(2, '0')).join('');
}
const catalog = JSON.parse(experiment_catalog_json()) as StudyDescriptor[];
const rawNativeRecords = new WeakMap<EpisodeRecord, string>();
const scenes = JSON.parse(readFileSync(`${cwd}/crates/sugarscape-core/src/browser_experiments/fixtures/spatial-scenes.json`, 'utf8')) as { scenes: { id: string; input: ObjectJson }[] };

function object(value: Json): ObjectJson {
  expect(value).not.toBeNull(); expect(typeof value).toBe('object'); expect(Array.isArray(value)).toBe(false);
  return value as ObjectJson;
}
function array(value: Json): Json[] { expect(Array.isArray(value)).toBe(true); return value as Json[]; }
function native(op: 'run' | 'validate', text: string): EpisodeRecord {
  const scratch = mkdtempSync(`${tmpdir()}/spatial-parity-`);
  try {
    const path = `${scratch}/input.json`; writeFileSync(path, text);
    const options = { cwd, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024 + 1 };
    const raw = execFileSync(`${cwd}/target/release/sugarscape`, ['experiment-view', op, '--input', path], options);
    const record = JSON.parse(raw) as EpisodeRecord; rawNativeRecords.set(record, raw);
    return record;
  } finally { rmSync(scratch, { recursive: true, force: true }); }
}
function runNativeFixture(input: Json): EpisodeRecord { return native('run', JSON.stringify(input)); }
function errors(call: () => string): { field: string; message: string }[] {
  try { call(); } catch (error) { return JSON.parse(String(error)); }
  throw new Error('expected checked boundary rejection');
}
function sourcePrefix(record: EpisodeRecord): string {
  const match = record.rules_identity.match(/^(foraging_(?:fixed|passage|construction):spatial-adapter-1:source-sha256:[a-f0-9]{64}):target:([^:]+)$/);
  expect(match, 'CPFA source/adapter identity must be recognized before target diagnostics').not.toBeNull();
  expect(match![1].startsWith(`${record.study}:`)).toBe(true);
  expect(object(object(record.payload).capture).target).toBe(match![2]);
  return match![1];
}

/** Diagnostic only: clone the complete record and separate only declared leaves.
 * Never used by imports; same-target full reconstruction remains the authority.
 */
function canonicalSpatialSemantics(record: EpisodeRecord, leaves: InspectionLeaf[]): Json {
  const clone = structuredClone(record) as unknown as ObjectJson;
  if (!record.study.startsWith('foraging_')) return clone; // Burrow has no exclusions.
  const prefix = sourcePrefix(record);
  function separate(path: Path, category: InspectionLeaf['category']) {
    let parent: Json = clone;
    for (const key of path.slice(0, -1)) parent = (parent as ObjectJson)[key];
    const key = path[path.length - 1]; const value = (parent as ObjectJson)[key];
    expect(value).not.toBeUndefined();
    if (category === 'inspection_float') { expect(typeof value).toBe('number'); expect(Number.isFinite(value)).toBe(true); }
    if (category === 'snapshot_bytes') expect(value).toMatch(/^(0|[1-9][0-9]*)$/);
    leaves.push({ path, category, value });
    delete (parent as ObjectJson)[key];
  }
  separate(['rules_identity'], 'target'); clone.rules_identity = prefix;
  separate(['payload', 'capture', 'target'], 'target');
  separate(['payload', 'native', 'snapshot_bytes'], 'snapshot_bytes');
  function snapshot(value: Json, path: Path) {
    const captured = object(value);
    array(captured.waypoints).forEach((waypoint, index) => {
      expect(object(waypoint).strength).not.toBeUndefined();
      separate([...path, 'waypoints', index, 'strength'], 'inspection_float');
    });
    if (record.study === 'foraging_fixed') array(captured.agents).forEach((agent, index) => {
      expect(object(agent).heading).not.toBeUndefined();
      separate([...path, 'agents', index, 'heading'], 'inspection_float');
    });
  }
  array(object(object(record.payload).native).snapshots).forEach((value, index) => snapshot(value, ['payload', 'native', 'snapshots', index]));
  record.checkpoints.forEach((checkpoint, index) => {
    snapshot(object(checkpoint.researcher).snapshot, ['checkpoints', index, 'researcher', 'snapshot']);
    if (record.study === 'foraging_fixed') Object.entries(checkpoint.local).forEach(([id, value]) => {
      expect(object(object(value).agent).heading).not.toBeUndefined();
      separate(['checkpoints', index, 'local', id, 'agent', 'heading'], 'inspection_float');
    });
  });
  return clone;
}
function diagnostic(nativeRecord: EpisodeRecord, browserRecord: EpisodeRecord) {
  // Source identity is compared BEFORE target/float separation, even if the targets differ.
  expect(browserRecord.study).toBe(nativeRecord.study);
  if (nativeRecord.study.startsWith('foraging_')) expect(sourcePrefix(browserRecord)).toBe(sourcePrefix(nativeRecord));
  else expect(browserRecord.rules_identity).toBe(nativeRecord.rules_identity);
  const nativeLeaves: InspectionLeaf[] = []; const wasmLeaves: InspectionLeaf[] = [];
  const nativeSemantics = canonicalSpatialSemantics(nativeRecord, nativeLeaves);
  const browserSemantics = canonicalSpatialSemantics(browserRecord, wasmLeaves);
  expect(wasmLeaves.map(({ path, category }) => ({ path, category }))).toEqual(nativeLeaves.map(({ path, category }) => ({ path, category })));
  const separated = nativeLeaves.map((leaf, index) => ({
    path: leaf.path, category: leaf.category, native: leaf.value, wasm: wasmLeaves[index].value,
    equal: Object.is(leaf.value, wasmLeaves[index].value),
    delta: typeof leaf.value === 'number' && typeof wasmLeaves[index].value === 'number' ? (wasmLeaves[index].value as number) - leaf.value : null,
  }));
  return { nativeSemantics, browserSemantics, separated };
}
async function retain(id: string, input: Json, nativeRecord: EpisodeRecord, browserRecord: EpisodeRecord, wasmJson: string, separated: unknown) {
  const dir = env.TASK5_EVIDENCE_DIR;
  if (!dir) return;
  mkdirSync(dir, { recursive: true });
  writeFileSync(`${dir}/${id}.json`, JSON.stringify({ input, nativeRecord, browserRecord, native_json: rawNativeRecords.get(nativeRecord), wasm_json: wasmJson, separated, artifacts: {
    native_sha256: await hash(readFileSync(new URL('../../../target/release/sugarscape', import.meta.url))), wasm_sha256: await hash(wasmBytes),
  } }, null, 2));
}
const fixtures = scenes.scenes.map(scene => ({ id: scene.id, input: structuredClone(scene.input) }));
for (const study of catalog.filter(study => study.family === 'spatial')) {
  const max = structuredClone(study.default_input) as ObjectJson; max.seed = '18446744073709551615';
  if (max.config) {
    const config = object(max.config); object(config.lab ?? config).freshness_window = '18446744073709551615';
  }
  fixtures.push({ id: `${study.id}-max-u64`, input: max });
  const off = structuredClone(study.default_input) as ObjectJson; off.ticks = 8; off.sample_every = 20;
  fixtures.push({ id: `${study.id}-off-cadence`, input: off });
  const expensive = structuredClone(study.default_input) as ObjectJson;
  if (study.id.startsWith('burrow_')) { expensive.ticks = 256; expensive.sample_every = 256; }
  else {
    const setup = object(expensive.setup);
    setup.width = study.id === 'foraging_fixed' ? 32 : 64;
    setup.height = setup.width;
    if (study.id === 'foraging_fixed') setup.agents = 16;
    expensive.ticks = study.id === 'foraging_fixed' ? 512 : 7200;
    expensive.sample_every = expensive.ticks;
  }
  fixtures.push({ id: `${study.id}-expensive-bounded`, input: expensive });
  if (study.id.startsWith('burrow_')) { const zero = structuredClone(study.default_input) as ObjectJson; zero.ticks = 0; fixtures.push({ id: `${study.id}-zero`, input: zero }); }
}

describe('five spatial bridges and actual native/WASM complete records', () => {
  it('catalog extends eight original studies to thirteen runnable entries with spatial renderer available', () => {
    expect(catalog).toHaveLength(13);
    expect(catalog.filter(study => study.family === 'spatial').map(study => study.id)).toEqual(['burrow_excavation', 'burrow_access', 'foraging_fixed', 'foraging_passage', 'foraging_construction']);
    for (const descriptor of catalog.filter(study => study.family === 'spatial')) expect(rendererAvailable(descriptor)).toBe(true);
  });
  for (const fixture of fixtures) it(`${fixture.id} retains the full records and every declared diagnostic leaf`, async () => {
    const nativeRecord = runNativeFixture(fixture.input);
    const text = experiment_run_json(JSON.stringify(fixture.input));
    const browserRecord = JSON.parse(text) as EpisodeRecord;
    // Reconstruction compares original complete records including ALL target floats.
    expect(native('validate', JSON.stringify(nativeRecord))).toEqual(nativeRecord);
    expect(JSON.parse(experiment_validate_json(text))).toEqual(browserRecord);
    const { nativeSemantics, browserSemantics, separated } = diagnostic(nativeRecord, browserRecord);
    await retain(fixture.id, fixture.input, nativeRecord, browserRecord, text, separated);
    expect(browserSemantics).toEqual(nativeSemantics);
    expect(browserRecord.checkpoints.length).toBeLessThanOrEqual(4096);
    expect(new TextEncoder().encode(text).byteLength).toBeLessThanOrEqual(16 * 1024 * 1024);
    if (String(fixture.input.study).startsWith('burrow_')) { expect(separated).toEqual([]); expect(browserRecord).toEqual(nativeRecord); }
    else expect(errors(() => experiment_validate_json(JSON.stringify(nativeRecord)))).toContainEqual({ field: 'rules_identity', message: expect.stringContaining('incompatible CPFA target') });
  }, 120_000);
  it('diagnostics never separate input parameters, other float paths, work, identity, clocks or nulls', () => {
    const record = runNativeFixture(scenes.scenes.find(scene => scene.id === 'foraging-fixed-information')!.input);
    const leaves: InspectionLeaf[] = []; const semantics = canonicalSpatialSemantics(record, leaves);
    const tests: Path[] = [['input', 'setup', 'parameters', 'omega'], ['payload', 'native', 'snapshots', 0, 'agents', 0, 'work', 'opportunities'], ['payload', 'native', 'snapshots', 0, 'agents', 0, 'id'], ['payload', 'native', 'snapshots', 0, 'agents', 0, 'cargo'], ['checkpoints', 0, 'clock', 'completed_ticks']];
    for (const path of tests) {
      const changed = structuredClone(record) as unknown as ObjectJson; let parent: Json = changed;
      for (const key of path.slice(0, -1)) parent = (parent as ObjectJson)[key];
      (parent as ObjectJson)[path[path.length - 1]] = 'forged';
      expect(canonicalSpatialSemantics(changed as unknown as EpisodeRecord, [])).not.toEqual(semantics);
    }
    const unknown = structuredClone(record); object(unknown.payload).invented_heading = 1.234;
    expect(canonicalSpatialSemantics(unknown, [])).not.toEqual(semantics);
    const forgedSource = structuredClone(record); forgedSource.rules_identity = forgedSource.rules_identity.replace(/source-sha256:[a-f0-9]{64}/, `source-sha256:${'0'.repeat(64)}`);
    expect(() => diagnostic(record, forgedSource)).toThrow();
    expect(record).toEqual(runNativeFixture(record.input));
  });
});


describe('spatial import diagnostics preserve contextual source and target failures', () => {
  for (const study of catalog.filter(study => study.id.startsWith('foraging_'))) {
    it(`${study.id} distinguishes source forgery from a foreign target`, () => {
      const record = JSON.parse(experiment_run_json(JSON.stringify(study.default_input))) as EpisodeRecord;
      const prefix = sourcePrefix(record);
      const foreign = structuredClone(record); foreign.rules_identity = `${prefix}:target:foreign-os`;
      expect(errors(() => experiment_validate_json(JSON.stringify(foreign)))).toContainEqual({ field: 'rules_identity', message: expect.stringContaining('incompatible CPFA target') });
      const source = structuredClone(record); source.rules_identity = source.rules_identity.replace('source-sha256:', 'source-sha256:forged');
      expect(errors(() => experiment_validate_json(JSON.stringify(source)))).toContainEqual({ field: 'rules_identity', message: 'engine source or rules version mismatch' });
      const tiny = structuredClone(record);
      if (study.id === 'foraging_fixed') {
        const own = object(array(object(array(object(object(tiny.payload).native).snapshots)[0]).agents)[0]);
        own.heading = Number(own.heading) + 1e-13;
      } else {
        object(object(object(object(tiny.payload).native).setup).parameters).p_search = 1 - 1e-13;
      }
      expect(errors(() => experiment_validate_json(JSON.stringify(tiny)))).toContainEqual({ field: 'episode', message: expect.any(String) });
    });
  }
});

// Source bindings must authenticate the workspace used to build these artifacts.
// Keep this metadata guard independent of native/WASM episode reconstruction.
describe('spatial source identity integrity', () => {
  const identities = JSON.parse(readFileSync(`${cwd}/crates/sugarscape-core/src/browser_experiments/fixtures/spatial-identities.json`, 'utf8')) as Record<string, { source_files: Record<string, string>; source_sha256: string }>;
  for (const [study, identity] of Object.entries(identities)) {
    it(`${study} authenticates every declared source and the canonical digest`, async () => {
      const mismatches: string[] = [];
      let canonical = '';
      for (const [path, expected] of Object.entries(identity.source_files).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) {
        const actual = await hash(readFileSync(new URL(path, new URL('../../../', import.meta.url))));
        if (actual !== expected) mismatches.push(path);
        canonical += `${path}\0${actual}\n`;
      }
      expect({ mismatches, source_sha256: await hash(new TextEncoder().encode(canonical)) }).toEqual({ mismatches: [], source_sha256: identity.source_sha256 });
    });
  }
});
