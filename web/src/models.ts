// Which model a config is (milestones 9 and 10), and what each model offers the page.
import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
import type {
  AnasaziInspection,
  AnyInspection,
  ColorMode,
  Config,
  Inspection,
  ModelConfig,
  ModelKind,
  Preset,
  RingInspection,
} from './types';

export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi'];

/** The presets menu's group labels. */
export const MODEL_LABELS: Record<ModelKind, string> = {
  sugarscape: 'Sugarscape',
  schelling: 'Schelling',
  ring: 'Ring World',
  anasazi: 'Artificial Anasazi',
};

/** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
export function modelOf(c: ModelConfig): ModelKind {
  const tag = (c as { model?: unknown }).model;
  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' ? tag : 'sugarscape';
}

export function isSugar(c: ModelConfig): c is Config {
  return modelOf(c) === 'sugarscape';
}

/** A sugarscape site's inspection (it lists the site's resources). */
export function isSugarView(v: AnyInspection): v is Inspection {
  return 'resources' in v.site;
}

/** A Ring World site's inspection (it has sugar but no resources list). */
export function isRingView(v: AnyInspection): v is RingInspection {
  return 'sugar' in v.site;
}

/** A Long House Valley cell's inspection (it names its zone). */
export function isValleyView(v: AnyInspection): v is AnasaziInspection {
  return 'zone' in v.site;
}

/** The calendar year a world of `c` is in at `tick` (the anasazi's), or null for a model without one. */
export function calendarYear(c: ModelConfig, tick: number): number | null {
  return 'model' in c && c.model === 'anasazi' ? c.start_year + tick : null;
}

/** Ticks until a world of `c` at `tick` is finished (the anasazi's end year); Infinity for a model that never finishes. */
export function ticksLeft(c: ModelConfig, tick: number): number {
  return 'model' in c && c.model === 'anasazi' ? Math.max(0, c.end_year - c.start_year - tick) : Infinity;
}

export function presetModel(p: Preset): ModelKind {
  return modelOf(p.config);
}

/** The presets menu's groups: each model with presets, in `MODELS` order, its presets in list order. */
export function presetGroups(presets: Preset[]): { model: ModelKind; label: string; presets: Preset[] }[] {
  return MODELS.map((model) => ({ model, label: MODEL_LABELS[model], presets: presets.filter((p) => presetModel(p) === model) })).filter(
    (g) => g.presets.length > 0,
  );
}

/** The color modes (value, label) the Agents menu offers for each model, in order; the first is the default. */
export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
  sugarscape: [
    ['tribe', 'Tribe'],
    ['wealth', 'Wealth'],
    ['sex', 'Sex'],
    ['age', 'Age'],
    ['vision', 'Vision'],
    ['credit', 'Credit'],
    ['disease', 'Disease'],
    ['lineage', 'Lineage'],
  ],
  schelling: [
    ['color', 'Colour'],
    ['satisfaction', 'Satisfaction'],
    ['preference', 'Preference'],
  ],
  // The ring view and space–time diagram have no color modes.
  ring: [],
  // Occupation first: households show without any overlay on.
  anasazi: [
    ['occupation', 'Occupation'],
    ['zones', 'Zones'],
    ['yield', 'Yield'],
  ],
};

/** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
  sugarscape: NETWORKS,
  schelling: [],
  ring: [],
  anasazi: VALLEY_OVERLAYS,
};
