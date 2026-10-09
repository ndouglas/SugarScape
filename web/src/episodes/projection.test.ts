import { it, expect } from 'vitest';
import { projectCheckpoint, resultPayload } from './projection';
import { record } from './record.fixture';
it('Agent excludes researcher truth at every index',()=>{for(let i=0;i<record.checkpoints.length;i++){const s=projectCheckpoint(record,i,{kind:'agent',agent:'0'});expect(s.researcher).toBeNull();expect(s.local).toEqual({'0':record.checkpoints[i].local['0']});expect(JSON.stringify(s)).not.toContain('future')}});
it('different Agent gets own projection or unavailable',()=>{expect(projectCheckpoint(record,1,{kind:'agent',agent:'1'}).local).toEqual({'1':{seen:11}});expect(projectCheckpoint(record,1,{kind:'agent',agent:'absent'}).local).toEqual({absent:null})});
it('terminal payload gated until final or Researcher',()=>{expect(resultPayload(record,0,{kind:'agent',agent:'0'})).toBeNull();expect(resultPayload(record,2,{kind:'agent',agent:'0'})).toBe(record.payload);expect(resultPayload(record,0,{kind:'researcher'})).toBe(record.payload)});
