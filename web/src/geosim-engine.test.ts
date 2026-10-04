import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { Engine, finishedNotice } from './engine';
import { geosimRows, geosimParams } from './geosim';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import { wasmSimModule } from './sim-module';
import { Lockstep } from './compare/lockstep';
import { readoutText } from './valley';
import { captureExperimentBase, defaultForm, formToSweep } from './experiments/form';
import { initSync, Sim, aggregate, run_point, sweep_points, presets_json, model_schemas_json } from './wasm-pkg/sugarscape.js';
import type { GeosimConfig, GeosimInspection, GeosimStats, Preset } from './types';
const setup = async (partial: Partial<GeosimConfig>, seed=1) => {
  const wasm = initSync({module:readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url))});
  const presets = JSON.parse(presets_json()) as Preset[];
  const base = presets.find(p=>p.id==='geosim-paper')!.config as GeosimConfig;
  const config = {...base,...partial};
  return Engine.create({config,seed}, {presets, schemas:JSON.parse(model_schemas_json()),transport:new InlineTransport(new SimHost(wasmSimModule(wasm.memory)))});
};
describe('GeoSim through actual shared WASM hosts', () => {
  it('discovers all named controls and roundtrips inactive choices after manual changes', async () => {
    const engine = await setup({width:2,height:2,initial_states:1,initialization_periods:2,observation_periods:13,periods_per_tick:7,damage_incidence:'acting_party',defender_threshold:'same_threshold'});
    expect(engine.model).toBe('geosim');
    expect(engine.config).toMatchObject({damage_incidence:'acting_party',defender_threshold:'same_threshold',campaign_drop_timing:'each_decision'});
    expect(geosimParams(engine.schemas.geosim!).every(p=>Boolean(p.help))).toBe(true);
    expect(engine.schemas.geosim!.map(p=>p.path)).toEqual(expect.arrayContaining(['damage_incidence','defender_threshold','technology_inheritance','completed_export','initial_capacity','war_shadow']));
    expect(engine.presets.filter(p=>'model' in p.config && p.config.model==='geosim')).toHaveLength(5);
    expect(engine.presetId).toBeNull();
    await engine.loadPreset('geosim-paper');
    expect(engine.presetId).toBe('geosim-paper');
    await engine.resetModelWith(c => { const g=c as GeosimConfig; g.width=2; g.height=2; g.initial_states=1; g.initialization_periods=2; g.observation_periods=13; g.periods_per_tick=7; g.damage_incidence='acting_party'; g.defender_threshold='same_threshold'; });
    expect(engine.presetId).toBeNull();
    expect(engine.config).toMatchObject({damage_incidence:'acting_party',defender_threshold:'same_threshold'});
    await engine.select(0,0);
    expect((engine.inspection!.view as GeosimInspection).outcome).toBeNull();
    await engine.advance(3);
    await engine.select(0,0);
    const view=engine.inspection!.view as GeosimInspection;
    expect(view.outcome!.periods).toBe(15);
    expect(geosimRows(view)).toContainEqual(['Terminal war census','0 completed · 0 censored · 0 exported · 0 completed awaiting export']);
    const before=await engine.fingerprint();
    await engine.advance(1);
    expect(await engine.fingerprint()).toBe(before);
    expect((engine.latest as GeosimStats).periods).toBe(15);
  });

  it('keeps real Compare worlds independent and presents their own source clocks', async () => {
    const partial={width:2,height:2,initial_states:1,initialization_periods:2,observation_periods:13};
    const a=await setup({...partial,periods_per_tick:7},1);
    const b=await setup({...partial,periods_per_tick:1},2);
    const standalone=await setup({...partial,periods_per_tick:1},2);
    const pair=new Lockstep([a,b],1);
    await pair.settled();
    await pair.advance(2);
    await standalone.advance(2);
    expect((a.latest as GeosimStats).periods).toBe(14);
    expect((b.latest as GeosimStats).periods).toBe(2);
    expect(readoutText(a,b)).toContain('A 14 periods');
    expect(readoutText(a,b)).toContain('B 2 periods');
    expect(await b.fingerprint()).toBe(await standalone.fingerprint());
    await pair.advance(5);
    expect(a.finished).toBe(true);
    const stable=await a.fingerprint();
    await pair.advance(1);
    expect(await a.fingerprint()).toBe(stable);
    a.close();b.close();standalone.close();
  });
  it('runs a bounded actual WASM experiment from a captured resolved custom horizon', async () => {
    const engine=await setup({width:2,height:2,initial_states:1,initialization_periods:2,observation_periods:13,periods_per_tick:7});
    const captured=captureExperimentBase(engine.presetId,engine.isModified(),engine.baseConfig);
    const form=defaultForm('geosim',engine.baseConfig);form.seeds=1;form.x.values='0';
    const sweep=formToSweep(form,captured).sweep!;
    await engine.resetModelWith(c=>{(c as GeosimConfig).observation_periods=100;});
    expect(sweep.ticks).toBe(3);
    expect('config' in sweep.base && (sweep.base.config as GeosimConfig).observation_periods).toBe(13);
    const points=JSON.parse(sweep_points(JSON.stringify(sweep))) as {index:number}[];
    expect(points).toHaveLength(1);
    const run=JSON.parse(run_point(JSON.stringify(sweep),0));
    expect(run).toMatchObject({point:0,seed:1,value:0});
    engine.close();
  });
  it('renders all five canvas modes without advancing the actual world', async () => {
    initSync({module:readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm',import.meta.url))});
    const base=(JSON.parse(presets_json()) as Preset[]).find(p=>p.id==='geosim-paper')!.config;
    const sim=new Sim(JSON.stringify({...base,width:2,height:2,initial_states:1}),1,null);
    const before=sim.fingerprint();
    for(const mode of ['territory','capacity','technology','alert','wars']) {
      expect(sim.render(mode,'sugar')).toBeGreaterThan(0);
      expect(sim.frame_len()).toBe((2*6)*(2*6)*4);
      expect(sim.fingerprint()).toBe(before);
    }
    sim.free();
  });
  it('presents invalid numerical termination with its attempted and completed clocks', async () => {
    const engine=await setup({width:2,height:2,initial_states:4,initialization_periods:0,observation_periods:10,resource_adjustment:1,damage_fraction:1,attack_probability:1,superiority_threshold:.1});
    await engine.advance(10);
    await engine.select(0,0);
    const view=engine.inspection!.view as GeosimInspection;
    expect(view.outcome!.valid).toBe(false);
    expect(view.outcome!.periods).toBe(3);
    expect(view.outcome!.attempted_period).toBe(4);
    expect(finishedNotice(engine.config,engine.tick,engine.latest)).toContain('attempted period 4');
    expect(geosimRows(view).some(([label,value])=>label==='Capacity' && value.includes('-'))).toBe(true);
    engine.close();
  });
  it('reports an overflowing technology frontier as invalid instead of valid horizon completion', async () => {
    const engine=await setup({width:2,height:2,initial_states:1,initialization_periods:0,observation_periods:2,distance_threshold:1e308,shock_shift:1e308,shock_probability:1});
    await engine.advance(2);
    await engine.select(0,0);
    const view=engine.inspection!.view as GeosimInspection;
    expect(view.outcome!.valid).toBe(false);
    expect(finishedNotice(engine.config,engine.tick,engine.latest)).toContain('Invalid reconstruction');
    engine.close();
  });
  it('excludes an invalid final grouped tick from exploratory completed-run counts', async () => {
    const engine=await setup({width:2,height:2,initial_states:4,initialization_periods:0,observation_periods:4,periods_per_tick:4,resource_adjustment:1,damage_fraction:1,attack_probability:1,superiority_threshold:.1});
    const form=defaultForm('geosim',engine.baseConfig);form.seeds=1;form.x.values='20';
    const sweep=formToSweep(form,captureExperimentBase(null,true,engine.baseConfig)).sweep!;
    const run=JSON.parse(run_point(JSON.stringify(sweep),0));
    expect(run.value).toBeNull();
    const summary=JSON.parse(aggregate(JSON.stringify(sweep),JSON.stringify([run])));
    expect(summary.rows[0]).toMatchObject({n:0,nan:1});
    engine.close();
  });
  it.each(['geosim-paper','geosim-artifact-2017'])('%s matches native every tick across actual shocks, conquests and cluster completions', async id => {
    const wasm=initSync({module:readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm',import.meta.url))});
    const presets=JSON.parse(presets_json()) as Preset[];
    const config={...presets.find(p=>p.id===id)!.config,width:4,height:4,initial_states:4,initialization_periods:0,observation_periods:120,periods_per_tick:7,attack_probability:1,superiority_threshold:1,victory_threshold:1,war_shadow:2,shock_probability:1,shock_shift:20,event_log:true,event_log_limit:1000} as GeosimConfig;
    const scratch=mkdtempSync(`${tmpdir()}/geosim-parity-`);
    try {
      writeFileSync(`${scratch}/config.json`,JSON.stringify(config));
      execFileSync(fileURLToPath(new URL('../../target/release/sugarscape',import.meta.url)),['run','--config',`${scratch}/config.json`,'--seed','1','--ticks','18','--fingerprint-trace',`${scratch}/trace.json`,'--series-csv',`${scratch}/series.csv`],{cwd:fileURLToPath(new URL('../../',import.meta.url)),encoding:'utf8'});
      const trace=JSON.parse(readFileSync(`${scratch}/trace.json`,'utf8')) as {tick:number;fingerprint:string}[];
      const engine=await Engine.create({config,seed:1},{presets,transport:new InlineTransport(new SimHost(wasmSimModule(wasm.memory)))});
      const evidence=fileURLToPath(new URL('../../survey/out/geosim-host-parity/',import.meta.url));
      mkdirSync(evidence,{recursive:true});
      writeFileSync(`${evidence}/${id}-config.json`,JSON.stringify(config,null,2));
      writeFileSync(`${evidence}/${id}-native-trace.json`,JSON.stringify(trace,null,2));
      for (const row of trace) {
        if(row.tick>0) await engine.advance(1);
        const actual=await engine.fingerprint();
        if(actual!==row.fingerprint){
          await engine.select(0,0);
          writeFileSync(`${evidence}/${id}-first-divergence-wasm.json`,JSON.stringify({tick:row.tick,native:row.fingerprint,wasm:actual,config:engine.config,latest:engine.latest,inspection:engine.inspection!.view},null,2));
        }
        expect(actual,`${id} divergence tick ${row.tick}`).toBe(row.fingerprint);
      }
      await engine.select(0,0);
      const view=engine.inspection!.view as GeosimInspection;
      const out=view.outcome!;
      const csv=readFileSync(`${scratch}/series.csv`,'utf8');
      const [header,...rows]=csv.trim().split('\n').map(row=>row.split(','));
      const terminal=Object.fromEntries(header.map((key,i)=>[key,Number(rows.at(-1)![i])]));
      expect(terminal).toMatchObject({periods:120,shocks:out.ledger.shocks,conquests:out.ledger.conquests,completed_wars:out.completed_wars.length});
      writeFileSync(`${evidence}/${id}-native-series.csv`,csv);
      expect(out.valid).toBe(true);
      expect(out.periods).toBe(120);
      expect(out.ledger.shocks).toBeGreaterThan(0);
      expect(out.ledger.conquests).toBeGreaterThan(0);
      expect(out.ledger.collapses).toBeGreaterThan(0);
      expect(out.completed_wars.length).toBeGreaterThan(0);
      const dest=fileURLToPath(new URL('../../survey/out/geosim-host-parity/',import.meta.url));
      mkdirSync(dest,{recursive:true});
      writeFileSync(`${dest}/${id}-config.json`,JSON.stringify(config,null,2));
      writeFileSync(`${dest}/${id}-native-trace.json`,JSON.stringify(trace,null,2));
      writeFileSync(`${dest}/${id}-outcome.json`,JSON.stringify(out,null,2));
    } finally {rmSync(scratch,{recursive:true,force:true});}
  });
});
