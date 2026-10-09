import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { initSync, experiment_catalog_json, experiment_run_json } from '../wasm-pkg/sugarscape.js';
import { projectCheckpoint } from './projection';
import { surfaceDisplay, surfaceSummary } from './surface-projection';
import { obj, list } from './presentation';
import type { EpisodeRecord, Json, StudyDescriptor } from './types';
initSync({module:readFileSync(new URL('../wasm-pkg/sugarscape_bg.wasm',import.meta.url))});
const catalog = JSON.parse(experiment_catalog_json()) as StudyDescriptor[];
function run(study:string, patch:Record<string,Json> = {}):EpisodeRecord {
 const input=structuredClone(obj(catalog.find(d=>d.id===study)!.default_input));
 const protocol={...obj(input.protocol),...obj(patch.protocol)};
 return JSON.parse(experiment_run_json(JSON.stringify({...input,...patch,protocol})));
}
const active=run('active_surface');
it('accepted own write is not a received message and peer action stays hidden',()=>{
 const index=active.checkpoints.findIndex(c=>JSON.stringify(obj(c.researcher).event).includes('Accepted'));
 const a=obj(surfaceDisplay(projectCheckpoint(active,index,{kind:'agent',agent:'Agent-A'})));
 const b=obj(surfaceDisplay(projectCheckpoint(active,index,{kind:'agent',agent:'Agent-B'})));
 expect(a.researcher).toBeNull();
 expect(obj(list(a.agents)[0]).observation).toBeNull();
 expect(JSON.stringify(b)).not.toContain('Accepted');
 expect(list(b.agents).map(v=>obj(v).id)).toEqual(['Agent-B']);
});
it('only actual prefix events appear and public stop adds no paid waits',()=>{
 const at=active.checkpoints.find(c=>obj(c.public).probe_stop==='1')!;
 const local=obj(at.local['Agent-A']);
 const display=obj(surfaceDisplay(projectCheckpoint(active,at.index,{kind:'agent',agent:'Agent-A'})));
 expect(obj(list(display.agents)[0]).events).toEqual(obj(local.prefix).entries);
 expect(display.probeStop).toBe('1');
});
it('Researcher preserves stored inversion and distinct original write lineage',()=>{
 const r=run('active_surface',{environment:'DataFlip'});
 const at=r.checkpoints.find(c=>list(obj(c.researcher).fields).some(f=>obj(f).lineage!==null&&obj(f).symbol!==obj(obj(f).lineage).symbol))!;
 expect(obj(surfaceDisplay(at)).researcher).toEqual(at.researcher);
});
it('reset projection uses exact native field clocks while Agent retains dated observation',()=>{
 const r=run('active_surface',{environment:{InFamily:'SharedResetting'}});
 const at=r.checkpoints.find(c=>list(obj(c.researcher).fields).every(f=>obj(f).symbol==='Blank')&&obj(obj(c.local['Agent-A']).surface).last_observation!==null)!;
 const d=obj(surfaceDisplay(at));
 expect(obj(d.researcher).fields).toEqual(obj(at.researcher).fields);
 expect(obj(list(d.agents)[0]).observation).toEqual(obj(obj(at.local['Agent-A']).surface).last_observation);
});
it('unsupported history keeps A5/B7 costs and unavailable terminal values',()=>{
 const r=run('active_surface',{environment:'DataFlip',protocol:{experimenter:'B',policy:'InspectOnly'}});
 expect(surfaceSummary(r.payload)).toEqual({status:'unsupported history',spent:['5','7'],net:[null,null],reward:[null,null],correct:[null,null]});
});
it('supported wrong inversion remains complete with wrong realized performance',()=>{
 const r=run('active_surface',{environment:'DataFlip'});
 expect(surfaceSummary(r.payload)).toMatchObject({status:'complete',net:['-26','-26'],correct:['0','0']});
});
it('names native model masses in the original catalog order without numerical conversion', async()=>{
 const {surfaceBelief}=await import('./surface-projection');
 const belief={models:[{numerator:'1',denominator:'3'}, {numerator:'1',denominator:'6'}, {numerator:'1',denominator:'2'}, {numerator:'0',denominator:'1'}],target:null};
 expect(surfaceBelief(belief)).toEqual({SharedPersistent:belief.models[0],SharedResetting:belief.models[1],PrivatePersistent:belief.models[2],Inert:belief.models[3],target:null});
 expect(surfaceBelief(null)).toBeNull();
});
