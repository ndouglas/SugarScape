// Which model a config is (milestones 9–21), and what each model offers the page.
import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
import type {
  AgreementConfig,
  AgreementInspection,
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
  StructureConfig,
  StructureInspection,
  ColorMode,
  Config,
  DpdConfig,
  DpdInspection,
  NormsConfig,
  NormsInspection,
  EthnoConfig,
  EthnoInspection,
  ImageConfig,
  ImageInspection,
  Inspection,
  ModelConfig,
  ModelKind,
  Preset,
  RingInspection,
  SpatialInspection,
  TagsConfig,
  TagsInspection,
} from './types';

export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image'];

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
  ethno: 'Ethnocentrism',
  opinions: 'Bounded Confidence',
  structure: 'Social Structure',
  dpd: 'Demographic PD',
  norms: 'Norms and Metanorms',
  agreement: 'Relative Agreement',
  image: 'Image Scoring',
};

/** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
export function modelOf(c: ModelConfig): ModelKind {
  const tag = (c as { model?: unknown }).model;
  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image'
    ? tag
    : 'sugarscape';
}

export function isSugar(c: ModelConfig): c is Config {
  return modelOf(c) === 'sugarscape';
}

/** A sugarscape site's inspection (it lists the site's resources). */
export function isSugarView(v: AnyInspection): v is Inspection {
  return 'site' in v && 'resources' in v.site;
}

/** A Ring World site's inspection (it has sugar but no resources list). */
export function isRingView(v: AnyInspection): v is RingInspection {
  return 'site' in v && 'sugar' in v.site;
}

/** A Long House Valley cell's inspection (it names its zone). */
export function isValleyView(v: AnyInspection): v is AnasaziInspection {
  return 'site' in v && 'zone' in v.site;
}

/** A civil violence site's inspection (it lists the agents jailed after arrest there). */
export function isCivilView(v: AnyInspection): v is CivilInspection {
  return 'jailed' in v;
}

/** A spatial games cell's inspection (it names its z). */
export function isSpatialView(v: AnyInspection): v is SpatialInspection {
  return 'site' in v && 'z' in v.site;
}

/** A cell of the tags model's diagram (it names its generation). */
export function isTagsView(v: AnyInspection): v is TagsInspection {
  return 'generation' in v;
}

/** A point of the classes model's memory simplexes (it names its simplex). */
export function isClassesView(v: AnyInspection): v is ClassesInspection {
  return 'simplex' in v && 'mix' in v;
}

/** A cell of the social-structure frame (it names its block cell and plane point). */
export function isStructureView(v: AnyInspection): v is StructureInspection {
  return 'block' in v && 'plane' in v;
}

/** A cell of the bounded-confidence frame (it names its period and lattice site). */
export function isOpinionsView(v: AnyInspection): v is OpinionsInspection {
  return 'period' in v && 'lattice_site' in v;
}

/** A cell of the culture frame (it says whether it is a site or a lane). */
export function isCultureView(v: AnyInspection): v is CultureInspection {
  return 'kind' in v && 'neighbors' in v;
}

/**
 * An ethnocentrism site's inspection (its agent names a kin marker). An empty one
 * (`{ site: { x, y }, agent: null }`) is exactly an empty Schelling site, so the world's model
 * decides as well as the shape.
 */
export function isEthnoView(v: AnyInspection, model: ModelKind): v is EthnoInspection {
  return model === 'ethno' && (v.agent === null || 'kin_marker' in v.agent);
}

/**
 * A demographic PD site's inspection (its agent says whether it is surrounded). An empty one is
 * exactly an empty Schelling or ethnocentrism site, so the world's model decides as well as the shape.
 */
/** A cell of the norms frame (it names its plane levels). */
export function isNormsView(v: AnyInspection): v is NormsInspection {
  return 'level' in v && 'agents' in v;
}

/** A cell of the relative agreement frame (it names its panel). */
export function isAgreementView(v: AnyInspection): v is AgreementInspection {
  return 'panel' in v && 'agents' in v;
}

export function isDpdView(v: AnyInspection, model: ModelKind): v is DpdInspection {
  return model === 'dpd' && (v.agent === null || 'surrounded' in v.agent);
}

/**
 * A cell of the image-scoring frame (it names its cell and group; no other model's inspection has a
 * `cell`). It has no `site`, so the guards above check for one before reading it.
 */
export function isImageView(v: AnyInspection): v is ImageInspection {
  return 'cell' in v && 'group' in v;
}

/** The calendar year a world of `c` is in at `tick` (the anasazi's), or null for a model without one. */
export function calendarYear(c: ModelConfig, tick: number): number | null {
  return 'model' in c && c.model === 'anasazi' ? c.start_year + tick : null;
}

/**
 * Ticks until a world of `c` at `tick` is finished (the anasazi's end year, the tags model's last
 * generation, the ethnocentrism model's last period, the demographic PD's last cycle, image scoring's
 * last generation); Infinity for a model that never finishes.
 */
export function ticksLeft(c: ModelConfig, tick: number): number {
  if ('model' in c && c.model === 'anasazi') return Math.max(0, c.end_year - c.start_year - tick);
  if (modelOf(c) === 'tags' && (c as TagsConfig).end > 0) return Math.max(0, (c as TagsConfig).end - tick);
  if (modelOf(c) === 'ethno' && (c as EthnoConfig).end > 0) return Math.max(0, (c as EthnoConfig).end - tick);
  if (modelOf(c) === 'structure' && (c as StructureConfig).stop_at > 0) return Math.max(0, (c as StructureConfig).stop_at - tick);
  if (modelOf(c) === 'dpd' && (c as DpdConfig).end > 0) return Math.max(0, (c as DpdConfig).end - tick);
  if (modelOf(c) === 'norms' && (c as NormsConfig).stop_at > 0) return Math.max(0, (c as NormsConfig).stop_at - tick);
  // Relative agreement's stop_at is its horizon, or a cap on a run that stops when stable.
  if (modelOf(c) === 'agreement' && (c as AgreementConfig).stop_at > 0) return Math.max(0, (c as AgreementConfig).stop_at - tick);
  if (modelOf(c) === 'image' && (c as ImageConfig).end > 0) return Math.max(0, (c as ImageConfig).end - tick);
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
  if (model === 'agreement') return (c as AgreementConfig).stop_when_stable;
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
  // Strategy first (the paper's question); the core's mode names.
  ethno: [
    ['strategy', 'Strategy'],
    ['tag', 'Tag'],
    ['lineage', 'Lineage'],
    ['ptr', 'PTR'],
  ],
  // Lines colored by where each agent started (HK's figures) or where it is now.
  opinions: [
    ['start', 'Start'],
    ['opinion', 'Opinion'],
  ],
  // Friendliness first (the paper's p); provocability is 1 − q.
  structure: [
    ['friendliness', 'Friendliness'],
    ['provocability', 'Provocability'],
    ['payoff', 'Payoff'],
    ['strategy', 'Strategy'],
  ],
  // Epstein's colors first (cooperators blue, defectors red); the core's mode names.
  dpd: [
    ['strategy', 'Strategy'],
    ['wealth', 'Wealth'],
    ['age', 'Age'],
    ['surrounded', 'Surrounded'],
  ],
  // The plane by how many agents hold each strategy; payoff; group (under dominance).
  norms: [
    ['agents', 'Agents'],
    ['payoff', 'Payoff'],
    ['group', 'Group'],
  ],
  // Confident red to uncertain green (DAWF's figures); the initial extremists; the start.
  agreement: [
    ['uncertainty', 'Uncertainty'],
    ['role', 'Role'],
    ['start', 'Start'],
  ],
  // The strategies first (k on a blue–red scale, each other class its own color); the core's mode names.
  image: [
    ['strategy', 'Strategy'],
    ['score', 'Score'],
    ['payoff', 'Payoff'],
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
  ethno: [],
  opinions: [],
  structure: [],
  dpd: [],
  norms: [],
  agreement: [],
  image: [],
};
