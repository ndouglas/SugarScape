import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { Engine } from './engine';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import { wasmSimModule } from './sim-module';
import { initSync, presets_json } from './wasm-pkg/sugarscape.js';
import type { PolarityConfig, PolarityInspection, PolarityStats, Preset } from './types';
describe('polarity through the shared WASM host', () => {
  it.each([
    ['polarity-original', '0x489849cf252544a3'],
    ['polarity-defense', '0x6c55f6f41448e503'],
    ['polarity-alliances', '0x93f744fba37cb327'],
    ['polarity-pra', '0xd1b184350e70d728'],
    ['polarity-two-level', '0x999de30e70e8da90'],
    ['polarity-overextension', '0x5a75f9a4214f4e17'],
  ])('%s matches its native 200-tick fingerprint', async (id, expected) => {
    const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
    const presets = JSON.parse(presets_json()) as Preset[];
    const config = structuredClone(presets.find(p => p.id === id)!.config);
    const engine = await Engine.create({ config, seed: 1 }, { presets, transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) });
    await engine.advance(200);
    expect(await engine.fingerprint()).toBe(expected);
  });
  it('exports resolved source choices after a partial config create and reset', async () => {
    const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
    const presets = JSON.parse(presets_json()) as Preset[];
    const config = { model: 'polarity', width: 2, height: 2, predator_share: 0 } as PolarityConfig;
    const engine = await Engine.create({ config, seed: 1 }, { presets, transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) });
    expect((await engine.session()).session.config).toMatchObject({ source_profile: 'chapter4', action_memory: 'previous_action', horizon: 1000 });
    await engine.reset({ ...config, variant: 'two_level' } as PolarityConfig);
    expect((await engine.session()).session.config).toMatchObject({ source_profile: 'chapter5', variant: 'two_level' });
  });
  it('finishes at the partial economic tick and keeps rendering and inspection read only', async () => {
    const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
    const presets = JSON.parse(presets_json()) as Preset[];
    const preset = presets.find((p) => p.id === 'polarity-original');
    expect(preset).toBeDefined();
    const config = { ...preset!.config as PolarityConfig, width: 2, height: 2, predator_share: 0, horizon: 15, periods_per_tick: 7 };
    const engine = await Engine.create({ config, seed: 1 }, { presets, transport: new InlineTransport(new SimHost(wasmSimModule(wasm.memory))) });
    await engine.advance(2);
    expect([engine.tick, engine.finished, (engine.latest as PolarityStats).periods]).toEqual([2, false, 14]);
    await engine.advance(8);
    expect([engine.tick, engine.finished, (engine.latest as PolarityStats).periods, (engine.latest as PolarityStats).last_tick_periods]).toEqual([3, true, 15, 1]);
    const endpoint = await engine.fingerprint();
    expect((await engine.session()).session.config).toMatchObject({ model: 'polarity', source_profile: 'chapter4', horizon: 15, periods_per_tick: 7 });
    for (const colorMode of ['territory', 'resources', 'strategy', 'coalitions'] as const) {
      engine.setDisplay({ colorMode });
      await engine.select(0, 0);
      expect((engine.inspection!.view as PolarityInspection).capital).toBe(0);
    }
    await engine.advance(1);
    expect([engine.tick, (engine.latest as PolarityStats).sovereign_count]).toEqual([3, 4]);
    expect(await engine.fingerprint()).toBe(endpoint);
  });
});
