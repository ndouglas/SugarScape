import { describe, it, expect } from 'vitest';
import { spatialDisplay } from './spatial-projection';
import { projectCheckpoint } from './projection';
import { obj, list } from './presentation';
import type { Checkpoint, EpisodeRecord } from './types';
const at: Checkpoint = { index: 0, kind: 'spatial_sample', clock: {completed_ticks:'7'}, public: {arena:{width:'3',height:'2'},nest:[{x:'0',y:'0'}]}, local:{'0':{agent:{id:'0',pos:{x:'0',y:'0'},cargo:{Food:'0'}},knowledge:{cells:[{pos:{x:'1',y:'0'},kind:{KnownSolid:{diggable:true}}}]},capture_at:'7',cell_observed_at:null}}, researcher:{snapshot:{open:[{x:'1',y:'0'}],agents:[{id:'1',pos:{x:'2',y:'0'}}],food:[{food:{id:'0',pos:{x:'2',y:'0'}},state:'Hidden'}],spoil:[{id:'0',pos:{x:'1',y:'1'}}]}} };
const record = {checkpoints:[at],payload:{future:'hidden'}} as unknown as EpisodeRecord;
describe('permitted spatial display',()=>{
 it('Agent projection retains sparse private classifications without physical state',()=>{
  const shown=projectCheckpoint(record,0,{kind:'agent',agent:'0'}), display=obj(spatialDisplay(shown));
  expect(display.researcher).toBeNull();
  expect(display).not.toHaveProperty('global_food');
  expect(list(display.cells).map(obj).find(c=>c.x===1&&c.y===0)?.kind).toEqual({KnownSolid:{diggable:true}});
  expect(list(display.cells).map(obj).find(c=>c.x===2&&c.y===0)?.kind).toBe('Unknown');
  expect(JSON.stringify(display)).not.toContain('"id":"1"');
 });
 it('F2 own state does not add food observations or a topology memory',()=>{
  const shown={...at,researcher:null,local:{'0':{agent:{id:'0',target:{x:'2',y:'1'},find:{site:{x:'2',y:'1'},count:'1'},cargo:'18446744073709551615'}}}};
  const display=obj(spatialDisplay(shown));expect(display.mapKind).toBe('own state');expect(display).not.toHaveProperty('global_food');expect(display.agents).toEqual([{id:'0',state:shown.local['0'].agent,local:shown.local['0']}]);
 });
 it('Burrow W remains an overlay glyph without identity or pile count',()=>{
  const shown={...at,public:{},researcher:{grid:{width:'1',height:'1',cells:[{x:'0',y:'0',glyph:'W'}],overlay_caveat:'overlays hide spoil'}},local:{}};
  const display=obj(spatialDisplay(shown)); expect(display.agents).toEqual([]);expect(list(display.cells)[0]).toEqual({x:0,y:0,kind:'W',items:[]});expect(display.overlayCaveat).toBe('overlays hide spoil');
 });
 it('different samples never construct a route or observation age',()=>{
  const display=obj(spatialDisplay({...at,researcher:null}));expect(display).not.toHaveProperty('paths');expect(display).not.toHaveProperty('observation_age');expect(obj(list(display.agents)[0]).local).toEqual(at.local['0']);
 });
 it('researcher resource labels preserve Food and Spoil namespaces with equal IDs',()=>{
  const display=obj(spatialDisplay(at));expect(list(display.resources).map(obj).map(r=>r.namespace)).toEqual(['Food','Spoil']);expect(list(display.resources).map(obj).map(r=>r.id)).toEqual(['0','0']);
 });
});
