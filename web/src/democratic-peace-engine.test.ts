import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { Engine } from './engine';
import { Lockstep } from './compare/lockstep';
import { captureExperimentBase, defaultForm, formToSweep } from './experiments/form';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import { wasmSimModule } from './sim-module';
import { readoutText } from './valley';
import { initSync, presets_json, model_schemas_json, aggregate, run_point } from './wasm-pkg/sugarscape.js';
import type { DemocraticPeaceConfig, DemocraticPeaceStats, Preset } from './types';
import type { RunResult } from './experiments/types';
const setup = async (partial: Partial<DemocraticPeaceConfig> = {}, seed = 9123) => {
  const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
  const presets = JSON.parse(presets_json()) as Preset[];
  const base = presets.find((preset) => preset.id === 'democratic-peace-printed-2001');
  expect(base, 'the production WASM catalog discovers democratic peace').toBeDefined();
  return Engine.create({ config: { ...base!.config, width: 4, height: 4, horizon_periods: 14, periods_per_tick: 3, ...partial } as DemocraticPeaceConfig, seed }, { presets, schemas: JSON.parse(model_schemas_json()), transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) });
};
describe('Democratic peace through production WASM hosts', () => {
  it('discovers all six presets and reading controls', async () => {
    const engine = await setup();
    expect(engine.presets.filter((preset) => 'model' in preset.config && preset.config.model === 'democratic_peace')).toHaveLength(6);
    expect(engine.schemas.democratic_peace!.map((param) => param.path)).toEqual(expect.arrayContaining(['mechanism', 'probability_direction', 'zero_ratio', 'assignment', 'capital_capture', 'claim_locking', 'security_scope', 'clustering_weights']));
  });
  it('finishes its partial final tick and inspection/rendering never consumes randomness', async () => {
    const engine = await setup();
    await engine.advance(5);
    expect(engine.latest).toMatchObject({ periods: 14, attempted_period: 14, last_tick_periods: 2, finish_reason: 'complete' });
    const before = await engine.fingerprint();
    for (const color of ['territory', 'governing_regime', 'latent_regime', 'resources', 'alliances', 'pariahs'] as const) {
      engine.setDisplay({ colorMode: color });
      await engine.select(0, 0);
      expect(engine.inspection!.view).toMatchObject({ model: 'democratic_peace' });
      expect(await engine.fingerprint()).toBe(before);
    }
    await engine.advance(1);
    expect(await engine.fingerprint()).toBe(before);
  });
  it('keeps real Compare worlds independent with different period groupings', async () => {
    const a = await setup({ periods_per_tick: 3 }, 11);
    const b = await setup({ periods_per_tick: 2 }, 12);
    const standalone = await setup({ periods_per_tick: 2 }, 12);
    const pair = new Lockstep([a, b], 1);
    await pair.settled();
    await pair.advance(1);
    await standalone.advance(1);
    expect(readoutText(a, b)).toContain('A 3 periods');
    expect(readoutText(a, b)).toContain('B 2 periods');
    expect(await b.fingerprint()).toBe(await standalone.fingerprint());
  });
  it('captures the authoritative config and includes valid zero and undefined clustering histories', async () => {
    const engine = await setup({ initial_democratic_share: 0, periods_per_tick: 3 });
    const captured = captureExperimentBase(engine.presetId, true, engine.config);
    const form = defaultForm('democratic_peace', engine.config);
    form.x.values = '0';
    form.seeds = 1;
    const sweep = formToSweep(form, captured).sweep!;
    const spec = JSON.stringify(sweep);
    const run = JSON.parse(run_point(spec, 0)) as RunResult;
    expect(run).toMatchObject({ value: 0, democratic_peace: { status: 'complete', completed_periods: 14, clustering_reason: 'undefined_initial_density' } });
    expect(JSON.parse(aggregate(spec, JSON.stringify([run]))).rows[0]).toMatchObject({ n: 1, nan: 0, mean: 0 });
    const clusterSpec = JSON.stringify({ ...sweep, metric: { kind: 'final', series: 'clustering_ratio' } });
    const cluster = JSON.parse(run_point(clusterSpec, 0));
    expect(cluster).toMatchObject({ value: null, democratic_peace: { status: 'complete', clustering_reason: 'undefined_initial_density' } });
    await engine.resetModelWith((config) => { (config as DemocraticPeaceConfig).horizon_periods = 100; });
    expect((captured as { config: DemocraticPeaceConfig }).config.horizon_periods).toBe(14);
  });
  it('excludes every metric sample of an invalid attempted history', async () => {
    const engine = await setup({ initial_resourced_share: 0, zero_ratio: 'reject_zero_denominator' });
    const form = defaultForm('democratic_peace', engine.config);
    form.x.values = '0.1';
    form.seeds = 1;
    const sweep = formToSweep(form, { config: engine.config }).sweep!;
    const spec = JSON.stringify(sweep);
    const run = JSON.parse(run_point(spec, 0));
    expect(run).toMatchObject({ value: null, democratic_peace: { status: 'invalid', completed_periods: 0, attempted_period: 1 } });
    expect(JSON.parse(aggregate(spec, JSON.stringify([run]))).rows[0]).toMatchObject({ n: 0, nan: 1 });
    await engine.advance(1);
    expect((engine.latest as DemocraticPeaceStats).invalidity).toBeTruthy();
  });
});

for (const [index, partial] of [
  { mechanism: 'tagging', probability_direction: 'printed_decreasing' },
  { mechanism: 'alliances', probability_direction: 'prose_increasing' },
  { mechanism: 'collective_security', assignment: 'rounded_quota', distance_metric: 'territorial_path', capital_capture: 'capture_and_fragment' },
] .entries()) {
  it(`matches actual native/WASM states and RNG through representative small history ${index + 1}`, async () => {
    const engine = await setup({ ...partial, width: 6, height: 6, horizon_periods: 60, periods_per_tick: 7, initial_democratic_share: 0.3, initial_resourced_share: 0.2 } as Partial<DemocraticPeaceConfig>, 914000 + index);
    const scratch = mkdtempSync(`${tmpdir()}/democratic-peace-host-parity-`);
    const root = fileURLToPath(new URL('../../', import.meta.url));
    try {
      writeFileSync(`${scratch}/config.json`, JSON.stringify(engine.config));
      execFileSync(`${root}/target/release/sugarscape`, ['run', '--config', `${scratch}/config.json`, '--seed', String(914000 + index), '--ticks', '100', '--fingerprint-trace', `${scratch}/trace.json`, '--state-out', `${scratch}/state.json`, '--outcome-out', `${scratch}/outcome.json`], { cwd: root, encoding: 'utf8', stdio: 'pipe' });
      const trace = JSON.parse(readFileSync(`${scratch}/trace.json`, 'utf8')) as { tick: number; fingerprint: string }[];
      for (const row of trace) {
        if (row.tick > 0) await engine.advance(1);
        expect(await engine.fingerprint()).toBe(row.fingerprint);
      }
      expect(JSON.parse(await engine.modelJson())).toEqual(JSON.parse(readFileSync(`${scratch}/state.json`, 'utf8')));
      const outcome = JSON.parse(readFileSync(`${scratch}/outcome.json`, 'utf8'));
      expect(outcome).toMatchObject({ valid: true, completed_periods: 60 });
      expect(outcome.census.initiated_fronts).toBeGreaterThan(0);
    } finally {
      engine.close();
      rmSync(scratch, { recursive: true, force: true });
    }
  });
}
