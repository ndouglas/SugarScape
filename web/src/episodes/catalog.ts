import type { Json, StudyDescriptor } from './types';
import cachingScenes from '../../../crates/sugarscape-core/src/browser_experiments/fixtures/caching-scenes.json';
import scenes from '../../../crates/sugarscape-core/src/browser_experiments/fixtures/spatial-scenes.json';
import { obj, list } from './presentation';
import { validSpatialPath } from './spatial-comparison';
/** All catalog families have checkpoint-only renderers. */
export function rendererAvailable(descriptor: StudyDescriptor): boolean { return ['game', 'testimony', 'surface', 'spatial'].includes(descriptor.family); }
export type RenderDescriptor = Pick<StudyDescriptor, 'id' | 'family' | 'title' | 'supplied' | 'question'>;
export function renderDescriptor({ id, family, title, supplied, question }: StudyDescriptor): RenderDescriptor { return { id, family, title, supplied, question }; }
/** Viewer fixtures are source-bound examples. Native field definitions remain unchanged. */
export function spatialCatalog(descriptor:StudyDescriptor):StudyDescriptor {
  if(descriptor.family!=='spatial') return descriptor;
  const fixtures = ['protection_recaching','deception_gestures'].includes(descriptor.id) ? cachingScenes : scenes;
  const definition=obj(obj(fixtures as unknown as Json).studies)[descriptor.id];
  const comparisons=list(obj(definition).comparisons).map(obj).map(row=>{
    const path=list(row.path);if(!path.every(p=>typeof p==='string')||!validSpatialPath(descriptor.id,String(row.id),path as string[]))throw new Error('Invalid catalog spatial axis');
    return {...row,values:descriptor.id.startsWith('foraging_')?[0,1]:row.values};
  });
  return {...descriptor,controls:{...obj(descriptor.controls),comparisons,scenes:list(obj(fixtures as unknown as Json).scenes).filter(row=>obj(obj(row).input).study===descriptor.id)}};
}
