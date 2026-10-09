import { describe,it,expect } from 'vitest';
import { declaredSpatialAxis,spatialMatchedInputs,spatialComparisonKey } from './spatial-comparison';
import { spatialCatalog } from './catalog';
import type { Json,StudyDescriptor } from './types';
const input:Json={study:'foraging_construction',seed:'18446744073709551615',ticks:40,sample_every:7,setup:{parameters:{p_search:0.5,p_return:0.25,lambda_fidelity:1},workers:[{x:1,y:0},{x:0,y:0}],food:[{id:'0',pos:{x:3,y:0}}],open:[{x:0,y:0}]}};
const descriptor={id:'foraging_construction',family:'spatial',controls:{comparisons:[{id:'p_search',path:['setup','parameters','p_search'],values:[0,1]}]}} as unknown as StudyDescriptor;
describe('declared spatial matching',()=>{
 it('deep clones one field while preserving all unrelated settings',()=>{const axis=declaredSpatialAxis(descriptor,'p_search'),candidates=spatialMatchedInputs(input,descriptor,'p_search');expect(candidates).toHaveLength(2);for(const next of candidates){expect(spatialComparisonKey(next,axis)).toBe(spatialComparisonKey(input,axis));expect(next).not.toBe(input);}expect(JSON.stringify(input)).toContain('"p_search":0.5');});
 it('rejects undeclared axes and paths into geometry or worker order',()=>{expect(()=>declaredSpatialAxis(descriptor,'open')).toThrow();for(const path of [['setup','open'],['setup','workers'],['seed'],['setup','parameters','__proto__']])expect(()=>declaredSpatialAxis({...descriptor,controls:{comparisons:[{id:'p_search',path,values:[0,1]}]}},'p_search')).toThrow();});
 it('missing axis input produces no compatible candidates',()=>{expect(spatialMatchedInputs({study:'foraging_construction',setup:{}},descriptor,'p_search')).toEqual([]);});
 it('custom and malformed inputs survive recipe generation unchanged',()=>{const malformed={...input as object,seed:'bad',extra:'retain'} as Json;const before=JSON.stringify(malformed);spatialMatchedInputs(malformed,descriptor,'p_search');expect(JSON.stringify(malformed)).toBe(before);});
 it('catalog explicitly declares all original supported CPFA axes with zero and one',()=>{const d=spatialCatalog({...descriptor,default_input:input,title:'',supplied:'',question:''});expect((d.controls as Record<string,Json>).comparisons).toEqual(expect.arrayContaining([{id:'lambda_publish',path:['setup','parameters','lambda_publish'],values:[0,1]}]));});
});
