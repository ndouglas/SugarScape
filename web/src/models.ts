// Which model a config is (milestones 9–12), and what each model offers the page.
import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
import type {
  AnasaziInspection,
  AnyInspection,
  CivilConfig,
  CivilInspection,
  ClassesConfig,
  ClassesInspection,
  CultureConfig,
  CultureInspection,
  OpinionsConfig,
  OpinionsInspection,
  ColorMode,
  Config,
  Inspection,
  ModelConfig,
  ModelKind,
  Preset,
  RingInspection,
  SpatialInspection,
  TagsConfig,
  TagsInspection,
} from './types';

export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'opinions'];

/** The presets menu's group labels. */
export const MODEL_LABELS: Record<ModelKind, string> = {
  sugarscape: 'Sugarscape',
  schelling: 'Schelling',
  ring: 'Ring World',
  anasazi: 'Artificial Anasazi',
  civil: 'Civil Violence',
  spatial: 'Spatial Games',
  tags: 'Tag Cooperation',
  culture: 'Axelrod Culture',
  classes: 'Emergence of Classes',
  opinions: 'Bounded Confidence',
};

/** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
export function modelOf(c: ModelConfig): ModelKind {
  const tag = (c as { model?: unknown }).model;
  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'opinions'
    ? tag
    : 'sugarscape';
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

/** A civil violence site's inspection (it lists the agents jailed after arrest there). */
export function isCivilView(v: AnyInspection): v is CivilInspection {
  return 'jailed' in v;
}

/** A spatial games cell's inspection (it names its z). */
export function isSpatialView(v: AnyInspection): v is SpatialInspection {
  return 'z' in v.site;
}

/** A cell of the tags model's diagram (it names its generation). */
export function isTagsView(v: AnyInspection): v is TagsInspection {
  return 'generation' in v;
}

/** A point of the classes model's memory simplexes (it names its simplex). */
export function isClassesView(v: AnyInspection): v is ClassesInspection {
  return 'simplex' in v && 'mix' in v;
}

/** A cell of the bounded-confidence frame (it names its period and lattice site). */
export function isOpinionsView(v: AnyInspection): v is OpinionsInspection {
  return 'period' in v && 'lattice_site' in v;
}

/** A cell of the culture frame (it says whether it is a site or a lane). */
export function isCultureView(v: AnyInspection): v is CultureInspection {
  return 'kind' in v && 'neighbors' in v;
}

/** The calendar year a world of `c` is in at `tick` (the anasazi's), or null for a model without one. */
export function calendarYear(c: ModelConfig, tick: number): number | null {
  return 'model' in c && c.model === 'anasazi' ? c.start_year + tick : null;
}

/**
 * Ticks until a world of `c` at `tick` is finished (the anasazi's end year, the tags model's last
 * generation); Infinity for a model that never finishes.
 */
export function ticksLeft(c: ModelConfig, tick: number): number {
  if ('model' in c && c.model === 'anasazi') return Math.max(0, c.end_year - c.start_year - tick);
  if (modelOf(c) === 'tags' && (c as TagsConfig).end > 0) return Math.max(0, (c as TagsConfig).end - tick);
  return Infinity;
}

/**
 * Whether a world of `c` can finish at a tick nobody knows in advance: civil Model II stopping when
 * a group dies out (its `ticksLeft` is Infinity until then). Compare steps such a pair one tick at a
 * time, so neither world runs past the tick at which the other finished.
 */
export function finishesUnpredictably(c: ModelConfig): boolean {
  const model = modelOf(c);
  if (model === 'culture') return (c as CultureConfig).stop_when_stable && (c as CultureConfig).drift === 0;
  if (model === 'classes') return (c as ClassesConfig).stop_at_equity;
  if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
  if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
  return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
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
    // Axelrod's culture rule (milestone 14); agents are gray under the book's rule.
    ['culture', 'Culture'],
  ],
  schelling: [
    ['color', 'Color'],
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
  // The paper's two screens (Fig. 1) and Model II's groups (all Blue in Model I).
  civil: [
    ['action', 'Action'],
    ['grievance', 'Grievance'],
    ['group', 'Group'],
  ],
  // NM92's four colors (blue C→C, red D→D, yellow C→D, green D→C) first.
  spatial: [
    ['change', 'Change'],
    ['strategy', 'Strategy'],
    ['payoff', 'Payoff'],
  ],
  // The diagram's three shadings of each tag bin.
  tags: [
    ['count', 'Count'],
    ['tolerance', 'Tolerance'],
    ['clones', 'Clones'],
  ],
  // Each culture its color; the paper's Fig. 1 (lanes by similarity); cultural zones.
  culture: [
    ['culture', 'Culture'],
    ['similarity', 'Similarity'],
    ['zones', 'Zones'],
  ],
  // The memory simplexes shaded by best reply (AEY's figures), or the agents colored by payoff.
  classes: [
    ['best_reply', 'Best reply'],
    ['payoff', 'Payoff'],
  ],
  // Lines colored by where each agent started (HK's figures) or where it is now.
  opinions: [
    ['start', 'Start'],
    ['opinion', 'Opinion'],
  ],
};

/** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
  sugarscape: NETWORKS,
  schelling: [],
  ring: [],
  anasazi: VALLEY_OVERLAYS,
  civil: [],
  spatial: [],
  tags: [],
  culture: [],
  classes: [],
  opinions: [],
};
