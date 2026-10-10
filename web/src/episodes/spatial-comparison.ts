import type { Json, StudyDescriptor, SpatialAxis } from './types';
import { obj, list } from './presentation';
import { canonical } from './comparison';
const cachingDomains:Record<string,Record<string,Json[]>>={protection_recaching:{policy:['off','selective','indiscriminate','erased']},deception_gestures:{sender:['ordinary','matched_neutral','sham'],view:['ambiguous','clear'],display_seen:[false,true],effort_cost:[0,3]}};
const common = ['p_search','p_return','lambda_fidelity','lambda_publish','lambda_waypoint'];
/** Catalog declarations are constrained to original family controls, never arbitrary paths. */
export function validSpatialPath(study:string,id:string,path:string[]):boolean {
  const expected = cachingDomains[study]?.[id] ? ['lab',id] : study === 'burrow_excavation' && ['transport','cue'].includes(id) ? ['config',id]
    : study === 'burrow_access' && ['transport','cue'].includes(id) ? ['config','lab',id]
    : study === 'burrow_access' && id === 'objective' ? ['config','task',id]
    : ['foraging_fixed','foraging_passage','foraging_construction'].includes(study) && [...common,...(study === 'foraging_fixed' ? ['omega','lambda_informed'] : [])].includes(id) ? ['setup','parameters',id] : null;
  return expected !== null && canonical(expected) === canonical(path);
}
export function declaredSpatialAxis(descriptor:StudyDescriptor,axisId:string):SpatialAxis {
  const rows=list(obj(descriptor.controls).comparisons).map(obj).filter(a=>a.id===axisId);
  if(descriptor.family!=='spatial'||rows.length!==1) throw new Error('Unknown spatial comparison axis');
  const row=rows[0],path=list(row.path),values=list(row.values);
  if(!path.every(p=>typeof p==='string')||!validSpatialPath(descriptor.id,axisId,path as string[])||!values.length) throw new Error('Invalid spatial comparison path');
  const allowed=cachingDomains[descriptor.id]?.[axisId] ?? (descriptor.id.startsWith('foraging_') ? [0,1] : axisId==='transport' ? ['direct','relay'] : axisId==='cue' ? ['blind','responsive'] : ['explore','known_goal']);
  if(!values.every(v=>allowed.includes(v as never))) throw new Error('Invalid spatial comparison values');
  return {id:axisId,path:path as string[],values:structuredClone(values)};
}
function parent(input:Json,path:string[]):Record<string,Json>|null {
  let value=input;
  for(const key of path.slice(0,-1)) { const o=obj(value);if(!(key in o)) return null;value=o[key]; }
  return value!==null&&typeof value==='object'&&!Array.isArray(value)?value:null;
}
export function spatialComparisonKey(input:Json,axis:SpatialAxis):string {
  if(!validSpatialPath(String(obj(input).study),axis.id,axis.path)) throw new Error('Invalid spatial comparison path');
  const copy=structuredClone(input),p=parent(copy,axis.path);if(p) delete p[axis.path.at(-1)!];return canonical(copy);
}
export function spatialMatchedInputs(input:Json,descriptor:StudyDescriptor,axisId:string):Json[] {
  const axis=declaredSpatialAxis(descriptor,axisId);
  if(obj(input).study!==descriptor.id) throw new Error('Comparison study differs');
  const p=parent(input,axis.path),key=axis.path.at(-1)!;
  if(!p||!(key in p)) return [];
  const seen=new Set<string>();return axis.values.flatMap(value=>{
    const copy=structuredClone(input);parent(copy,axis.path)![key]=structuredClone(value);
    const encoded=canonical(copy);if(encoded===canonical(input)||seen.has(encoded))return [];
    seen.add(encoded);if(spatialComparisonKey(copy,axis)!==spatialComparisonKey(input,axis))throw new Error('Comparison inputs differ');return [copy];
  });
}
export function spatialAxisValue(input:Json,axis:SpatialAxis):Json {return parent(input,axis.path)?.[axis.path.at(-1)!]??null;}
