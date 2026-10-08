import type { StudyDescriptor } from './types';
/** Task7 enables the surface renderer; never imply a fabricated trace exists. */
export function rendererAvailable(descriptor: StudyDescriptor): boolean { return descriptor.family !== 'surface'; }
export type RenderDescriptor = Pick<StudyDescriptor, 'id' | 'family' | 'title' | 'supplied' | 'question'>;
export function renderDescriptor({ id, family, title, supplied, question }: StudyDescriptor): RenderDescriptor { return { id, family, title, supplied, question }; }
