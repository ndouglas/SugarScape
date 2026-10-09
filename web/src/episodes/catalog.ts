import type { StudyDescriptor } from './types';
/** All catalog families have checkpoint-only renderers. */
export function rendererAvailable(descriptor: StudyDescriptor): boolean { return ['game', 'testimony', 'surface'].includes(descriptor.family); }
export type RenderDescriptor = Pick<StudyDescriptor, 'id' | 'family' | 'title' | 'supplied' | 'question'>;
export function renderDescriptor({ id, family, title, supplied, question }: StudyDescriptor): RenderDescriptor { return { id, family, title, supplied, question }; }
