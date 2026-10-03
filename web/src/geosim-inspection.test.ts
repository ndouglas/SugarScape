import { readFileSync } from 'node:fs';
import { initSync, presets_json } from './wasm-pkg/sugarscape.js';
import { describe, expect, it } from 'vitest';
import { geosimRows, geosimChartCaption, geosimLegend } from './geosim';
import type { GeosimConfig, GeosimInspection, GeosimOutcome, GeosimWar, Preset } from './types';

initSync({module:readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm',import.meta.url))});
const {model:_,...config}=(JSON.parse(presets_json()) as Preset[]).find(p=>p.id==='geosim-paper')!.config as GeosimConfig;
const war:GeosimWar={id:4,parents:[],start_period:1,end_period:7,last_active_period:5,active_periods:3,elapsed_periods:7,raw_severity:2.5,exported_severity:250,participants:[{state:{capital_cell:1,sovereignty_generation:2},last_fighting_period:5}],end_cause:'shadow_expired_or_participants_retired',java_saturated:false,java_subunit_zero:false,fighting_periods:[1,2,5]};
const outcome=(patch:Partial<GeosimOutcome>):GeosimOutcome=>({config,seed:1,rng_mode:'portable_pcg64_mcg',periods:7,attempted_period:7,counting_start:1,valid:true,state_available:true,finish_reason:'horizon',invalid_reason:null,completed_wars:[],censored_wars:[],legacy_visible_wars:[],exporter_backlog:[],merges:[],retired_states:[],sovereign_count:1,states:[],cells:[],fronts:[],resource_updates:[],partial_period_fights:[],ledger:{attacks:0,fighting_front_periods:0,mutual_front_periods:0,conquests:0,collapses:0,disconnections:0,stale_claims:0,locked_claims:0,double_successes:0,path_collisions:0,shocks:0,damage:0,measured_damage:0,capacity_increase:0,capacity_decrease:0,clipping:0,retirement_capacity:0,reemergence_capacity:0,recurrence_residual:0},...patch});

const view = {
  model: 'geosim', cell: { id: 3, owner: { capital_cell: 1, sovereignty_generation: 2 }, last_threshold: 2, next_generation: 0 },
  state: { id: { capital_cell: 1, sovereignty_generation: 2 }, capacity: null, threshold: 2, alert: true, campaign: null, previous_damage: 0, newly_independent: false, extracted_yield: 0, recurrence_residual: null },
  members: [1,3], distance: 1, projection: 0.9, resource_recurrence: null, fronts: [], wars: [], period: 4, periods: 4, attempted_period: 5, counting_start: 1,
  finish_reason: 'invalid', invalidity: 'nonfinite capacity', outcome: null, last_structural_event: {id: 8, period: 4, kind: 'conquest', states: [], cells: [3]}, events: [], events_dropped: 0, agent: null,
} as GeosimInspection;
describe('GeoSim inspection presentation', () => {
  it('distinguishes unavailable capacity from real zero losses and retired generations', () => {
    const rows = geosimRows(view);
    expect(rows).toContainEqual(['Government', 'Cell 3 · capital 1 · generation 2']);
    expect(rows).toContainEqual(['Capacity', 'unavailable · extracted yield 0.000 · previous damage 0.000']);
    expect(rows).toContainEqual(['Invalid reconstruction', 'nonfinite capacity']);
  });
  it('retains structural information independently of disabled event traces', () => {
    expect(geosimRows(view)).toContainEqual(['Last structural event', '#8 · period 4 · conquest · cells 3']);
    expect(geosimRows(view)).toContainEqual(['UI event trace', '0 retained · 0 dropped · bounded trace, not the full war census']);
  });
  it('keeps completion, censoring, FIFO visibility and backlog separate', () => {
    const terminal = {...view, outcome: outcome({completed_wars: [{...war,id:1},{...war,id:2}], censored_wars:[{...war,id:3,end_period:null}], legacy_visible_wars:[{...war,id:1}], exporter_backlog:[{...war,id:2}]}) } as GeosimInspection;
    expect(geosimRows(terminal)).toContainEqual(['Terminal war census', '2 completed · 1 censored · 1 exported · 1 completed awaiting export']);
  });
  it('labels each comparison clock and qualifies artifact identity', () => {
    expect(geosimChartCaption('Wars', [{periods_per_tick:7},{periods_per_tick:1}] as GeosimConfig[])).toBe('Wars · A: 7 source periods/tick · B: 1 source periods/tick');
    expect(geosimLegend({initialization_periods:2, observation_periods:13, periods_per_tick:7, initial_capacity:'artifact_random_100_1'} as GeosimConfig, 'wars', 14)).toContain('artifact identity Unresolved');
  });
});

describe('readable complete terminal census', () => {
  it('attributes open episodes to invalid termination rather than the horizon', () => {
    const terminal={...view,outcome:outcome({valid:false,finish_reason:'invalid',censored_wars:[{...war,end_period:null}]})};
    expect(geosimRows(terminal).find(([label])=>label==='Censored war #4')?.[1]).toContain('open at invalid');
  });
  it('presents all completed and censored records as damage and period summaries', () => {
    const terminal={...view,outcome:outcome({completed_wars:[war],censored_wars:[{...war,id:5,end_period:null}],exporter_backlog:[war]})} as GeosimInspection;
    const rows=geosimRows(terminal);
    expect(rows.find(([label])=>label==='Completed war #4')?.[1]).toContain('raw damage 2.500 · exported 250.000');
    expect(rows.find(([label])=>label==='Censored war #5')?.[1]).toContain('open at horizon');
    expect(rows.find(([label])=>label==='Completed war #4')?.[1]).toContain('end cause shadow_expired_or_participants_retired');
  });
});
