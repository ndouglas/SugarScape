// Which model a config is (milestones 9–21), and what each model offers the page.
import { MINDS_OVERLAYS, NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
import type {
  HoardConfig,
  HoardInspection,
  LineInspection,
  TippingInspection,
  FirmsConfig,
  FirmsInspection,
  ZiInspection,
  BaliConfig,
  BaliInspection,
  PunishmentConfig,
  PunishmentInspection,
  RetirementConfig,
  RetirementInspection,
  ThresholdsConfig,
  ThresholdsInspection,
  AntsConfig,
  AntsInspection,
  FarolConfig,
  FarolInspection,
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

export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi', 'bali', 'line', 'tipping', 'hoard', 'firms'];

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
  farol: 'El Farol and the Minority Game',
  ants: 'Ants and Recruitment',
  thresholds: 'Threshold Models',
  retirement: 'The Timing of Retirement',
  punishment: 'Altruistic Punishment',
  zi: 'Zero-Intelligence Traders',
  bali: 'Balinese Water Temples',
  line: "Schelling's line",
  tipping: "Schelling's tipping",
  hoard: 'The evolution of hoarding',
  firms: 'The Emergence of Firms',
};

/** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
export function modelOf(c: ModelConfig): ModelKind {
  const tag = (c as { model?: unknown }).model;
  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi' || tag === 'bali' || tag === 'line' || tag === 'tipping' || tag === 'hoard' || tag === 'firms'
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

/** A cell of the bali frame (a panel, a `subak` and a `dam`); check it first. */
export function isBaliView(v: AnyInspection): v is BaliInspection {
  return 'panel' in v && 'subak' in v && 'dam' in v;
}

/** A column of the hoard frame: the run's generation, day and bout, and an agent by index. */
export function isHoardView(v: AnyInspection): v is HoardInspection {
  return 'bout' in v && 'public' in v;
}

/** A cell of the firms frame (a panel, a `firm` and a `member`); check it before the others with a panel. */
export function isFirmsView(v: AnyInspection): v is FirmsInspection {
  return 'panel' in v && 'firm' in v && 'member' in v;
}

/** A point of Schelling's tipping plane. */
export function isTippingView(v: AnyInspection): v is TippingInspection {
  return 'red_content' in v && 'blue_content' in v;
}

/** A person in Schelling's line (or the row past its end). */
export function isLineView(v: AnyInspection): v is LineInspection {
  return 'place' in v && 'agent' in v;
}

/** A cell of the zi frame (a panel, a `trade` and a step's `supply`); check it first. */
export function isZiView(v: AnyInspection): v is ZiInspection {
  return 'panel' in v && 'trade' in v && 'supply' in v;
}

/** A cell of the punishment frame (a panel, an agent and its group, and a period's `punishment`); check it first. */
export function isPunishmentView(v: AnyInspection): v is PunishmentInspection {
  return 'panel' in v && 'punishment' in v;
}

/** A cell of the retirement frame (a panel, an agent as `member`, and an age's `exposed`); check it first. */
export function isRetirementView(v: AnyInspection): v is RetirementInspection {
  return 'panel' in v && 'exposed' in v;
}

/** A cell of the thresholds frame (a panel, an actor as `member`, and Figure 1's `cdf`); check it first. */
export function isThresholdsView(v: AnyInspection): v is ThresholdsInspection {
  return 'panel' in v && 'cdf' in v;
}

/** A cell of the ants frame (a panel, a grid ant as `member`, and each source's `shares`). */
export function isAntsView(v: AnyInspection): v is AntsInspection {
  return 'panel' in v && 'shares' in v;
}

/** A cell of the El Farol frame (it names its panel and its grid agent, `member`); check `isAntsView` first. */
export function isFarolView(v: AnyInspection): v is FarolInspection {
  return 'panel' in v && 'member' in v;
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
  if (modelOf(c) === 'farol' && (c as FarolConfig).stop_at > 0) return Math.max(0, (c as FarolConfig).stop_at - tick);
  if (modelOf(c) === 'ants' && (c as AntsConfig).stop_at > 0) return Math.max(0, (c as AntsConfig).stop_at - tick);
  if (modelOf(c) === 'thresholds' && (c as ThresholdsConfig).stop_at > 0) return Math.max(0, (c as ThresholdsConfig).stop_at - tick);
  if (modelOf(c) === 'retirement' && (c as RetirementConfig).stop_at > 0) return Math.max(0, (c as RetirementConfig).stop_at - tick);
  if (modelOf(c) === 'punishment' && (c as PunishmentConfig).stop_at > 0) return Math.max(0, (c as PunishmentConfig).stop_at - tick);
  if (modelOf(c) === 'bali' && (c as BaliConfig).stop_at > 0) {
    const b = c as BaliConfig;
    return Math.max(0, b.stop_at * (b.watershed === 'two_node' ? b.node_periods : 12) - tick);
  }
  if (modelOf(c) === 'hoard') {
    // The core finishes the season in progress even if `generations` is lowered below it live, so
    // count to the later of the last generation's end and the current season's end.
    const h = c as HoardConfig;
    const s = hoardSeasonTicks(h);
    return Math.max(h.generations, Math.max(1, Math.ceil(tick / s))) * s - tick;
  }
  if (modelOf(c) === 'firms' && (c as FirmsConfig).stop_at > 0) return Math.max(0, (c as FirmsConfig).stop_at - tick);
  return Infinity;
}

/** A hoard season's bouts (ticks): generation g's are (g − 1)·days·bouts + 1 to g·days·bouts. */
export function hoardSeasonTicks(c: HoardConfig): number {
  return c.days * c.bouts;
}

/**
 * Whether a world of `c` can finish at a tick nobody knows in advance: civil Model II stopping when
 * a group dies out (its `ticksLeft` is Infinity until then). Compare steps such a pair one tick at a
 * time, so neither world runs past the tick at which the other finished. A hoard run can also die
 * out in any season, but that is rare and it ends the run for good, so it isn't counted here: Compare
 * steps a hoard pair in batches, and a pair whose world died out stops at the end of that batch.
 */
export function finishesUnpredictably(c: ModelConfig): boolean {
  const model = modelOf(c);
  if (model === 'culture') return (c as CultureConfig).stop_when_stable && (c as CultureConfig).drift === 0;
  if (model === 'classes') return (c as ClassesConfig).stop_at_equity;
  if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
  if (model === 'agreement') return (c as AgreementConfig).stop_when_stable;
  if (model === 'retirement') return (c as RetirementConfig).stop_at_norm;
  if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
  return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
}

/** What the presets menu shows for a preset: its plain title. */
export function presetOptionLabel(p: Preset): string {
  return p.title;
}

/** Where a preset comes from and which rules it runs, shown with its description. */
export function presetReference(p: Preset): string {
  return `${p.source} · ${p.name}`;
}

export function presetModel(p: Preset): ModelKind {
  return modelOf(p.config);
}

/**
 * A sugarscape preset's chapter of the book, from its source ("Animation II-2", "Figure III-6",
 * "Chapter IV, footnote 7"), its appendix, "Minds" for the decision-engine experiments built on it,
 * or "Other sources" (the docking study).
 */
export function sugarscapeChapter(p: Preset): string {
  if (/\bMinds\b/.test(p.source)) return 'Minds';
  const m = /\b(VI|IV|V|III|II)(?=[-\s,/]|$)/.exec(p.source);
  if (m) return `Chapter ${m[1]}`;
  const appendix = /\bAppendix ([A-Z])\b/.exec(p.source);
  return appendix ? `Appendix ${appendix[1]}` : 'Other sources';
}

/**
 * Whether `c` is a Minds world (docs/studies/2026-09-27-minds.md): a sugarscape config that uses any rule
 * the Minds experiments added — a decision other than rule M, walking, memory, walls, truffles, caching,
 * a carrying limit, central-place foraging, a winter everywhere at once, or theft (a chance to find caches
 * or any cheaters), or watching.
 * The Minds run on the sugarscape model but have their own entry in the model menu.
 */
export function usesMinds(c: ModelConfig): boolean {
  if (!isSugar(c)) return false;
  const s = c as Config;
  return (
    (s.decision?.rule ?? 'book') !== 'book' ||
    s.movement?.mode === 'walk' ||
    (s.memory?.span ?? 0) > 0 ||
    (s.walls?.length ?? 0) > 0 ||
    (s.truffles?.share ?? 0) > 0 ||
    (s.caching?.rule ?? 'none') !== 'none' ||
    s.caching?.mixed === true ||
    (s.caching?.capacity ?? 0) > 0 ||
    s.central?.enabled === true ||
    s.seasons?.mode === 'global' ||
    (s.theft?.find ?? 0) > 0 ||
    (s.theft?.cheaters ?? 0) > 0 ||
    s.watching?.on === true
  );
}

/** Minds 5: caching is on (a rule that buries, mixed rules, or a carrying limit), as the core's `Caching::is_on`. */
export const cachingOn = (c: Config): boolean =>
  (c.caching?.rule ?? 'none') !== 'none' || c.caching?.mixed === true || (c.caching?.capacity ?? 0) > 0;

/** Minds 6: theft is on (a chance to find caches, or any cheaters), as the core's `Theft::is_on`. */
export const theftOn = (c: Config): boolean => (c.theft?.find ?? 0) > 0 || (c.theft?.cheaters ?? 0) > 0;

/** Minds 8: watching is on. */
export const watchingOn = (c: Config): boolean => c.watching?.on === true;

/** Minds 6 and 8: caches get pilfered, by theft or by watchers' raids (the core's `Config::pilfering_on`). */
export const pilferingOn = (c: Config): boolean => theftOn(c) || watchingOn(c);

/** Minds 5–6: whether `c` can hold caches (caching on, or a central world's larders). */
export const hasCaches = (c: Config): boolean => cachingOn(c) || c.central?.enabled === true;

/**
 * Minds 5–6: whether the page draws a `MindsView` for `c` — caches or larders, a winter everywhere
 * at once, or a lab.
 */
export const mindsShown = (c: Config): boolean =>
  cachingOn(c) || c.central?.enabled === true || (c.seasons?.enabled === true && c.seasons.mode === 'global') || (c.lab ?? null) !== null;

/** An entry of the model menu: a model, or the Minds (sugarscape worlds using the Minds rules). */
export type MenuKind = ModelKind | 'minds';

/** Models of their own that belong to the Minds entry (Minds 7's hoarding), not an entry of their own. */
export const MINDS_MODELS: ModelKind[] = ['hoard'];

/** The model menu's entries in order: the Minds right after the Sugarscape. */
export const MENUS: MenuKind[] = ['sugarscape', 'minds', ...MODELS.filter((m) => m !== 'sugarscape' && !MINDS_MODELS.includes(m))];

export const MENU_LABELS: Record<MenuKind, string> = { ...MODEL_LABELS, minds: 'Minds' };

/** The model menu's entry for config `c` by its rules alone. */
export function menuOf(c: ModelConfig): MenuKind {
  return usesMinds(c) || MINDS_MODELS.includes(modelOf(c)) ? 'minds' : modelOf(c);
}

/**
 * A preset's entry: Minds if its source names a Minds milestone (some Minds baselines run the book's
 * rule M on a Minds world) or it uses a Minds rule.
 */
export function presetMenu(p: Preset): MenuKind {
  return /\bMinds \d/.test(p.source) ? 'minds' : menuOf(p.config);
}

/** A world's entry: its preset's if it came from a Minds preset, else by its rules (share links, custom setups). */
export function worldMenu(config: ModelConfig, preset: Preset | undefined): MenuKind {
  return preset && presetMenu(preset) === 'minds' ? 'minds' : menuOf(config);
}

/** The Minds milestones' groups in the presets menu. */
const MINDS_TITLES: Record<string, string> = {
  '1': 'Minds 1: the utility mind',
  '2': 'Minds 2: walking',
  '3': 'Minds 3: memory',
  '4': 'Minds 4: planning',
  '5': 'Minds 5: caching',
  '6': 'Minds 6: theft',
  '7': 'Minds 7: evolution of hoarding',
  '8': 'Minds 8: watching',
};

/**
 * The preset menu's groups within one menu entry: the sugarscape's by chapter (in the order they first
 * appear), the Minds by milestone (in order), and every other model's as one unlabeled list in list order.
 */
export function presetSubgroups(menu: MenuKind, presets: Preset[]): { label: string | null; presets: Preset[] }[] {
  const mine = presets.filter((p) => presetMenu(p) === menu);
  if (menu === 'minds') {
    const groups = new Map<string, Preset[]>();
    for (const p of mine) {
      const n = /\bMinds (\d+)/.exec(p.source)?.[1] ?? '';
      groups.set(n, [...(groups.get(n) ?? []), p]);
    }
    return [...groups]
      .sort(([a], [b]) => Number(a) - Number(b))
      .map(([n, presets]) => ({ label: MINDS_TITLES[n] ?? (n ? `Minds ${n}` : 'Minds'), presets }));
  }
  if (menu !== 'sugarscape') return mine.length > 0 ? [{ label: null, presets: mine }] : [];
  const groups = new Map<string, Preset[]>();
  for (const p of mine) {
    const chapter = sugarscapeChapter(p);
    groups.set(chapter, [...(groups.get(chapter) ?? []), p]);
  }
  return [...groups].map(([label, presets]) => ({ label, presets }));
}

/** The model menu's entries with presets, in `MENUS` order, each with its presets in list order. */
export function presetGroups(presets: Preset[]): { model: MenuKind; label: string; presets: Preset[] }[] {
  return MENUS.map((model) => ({ model, label: MENU_LABELS[model], presets: presets.filter((p) => presetMenu(p) === model) })).filter(
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
    // Minds 6: hoarder or cheater; Minds 5: each agent's caching rule. See `defaultColorMode`.
    ['strategy', 'Strategy'],
    ['caching_rule', 'Caching rule'],
    // Minds 8: watchers who bury, scroungers, others.
    ['watching', 'Watching'],
    // Minds 3: whether each agent remembers.
    ['memory', 'Memory'],
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
  // Who went this round; winnings; the active predictor's family (or who switched); memory.
  farol: [
    ['choice', 'Choice'],
    ['gain', 'Gain'],
    ['strategy', 'Strategy'],
    ['memory', 'Memory'],
  ],
  // Each ant's source; the ants that never herd; how many each can meet.
  ants: [
    ['source', 'Source'],
    ['independent', 'Independent'],
    ['degree', 'Degree'],
  ],
  // Who acts (and the seed); each actor's threshold; how many it watches; its crowd.
  thresholds: [
    ['state', 'State'],
    ['threshold', 'Threshold'],
    ['degree', 'Degree'],
    ['crowd', 'Crowd'],
  ],
  // Working by type, or retired; each agent's type; its threshold; its sub-population.
  retirement: [
    ['status', 'Status'],
    ['type', 'Type'],
    ['threshold', 'Threshold'],
    ['group', 'Group'],
  ],
  // Each agent's type (blended for continuous traits); what it did this period; its payoff; its group's share of defectors.
  punishment: [
    ['type', 'Type'],
    ['acts', 'Acts'],
    ['payoff', 'Payoff'],
    ['group', 'Group'],
  ],
  // Buyers and sellers; each trader's profit against its equilibrium profit; ZIP margins.
  zi: [
    ['side', 'Side'],
    ['profit', 'Profit'],
    ['margin', 'Margin'],
  ],
  // Plans and start months first; the temples to compare them with; the year's harvest; this month's state.
  bali: [
    ['plan', 'Plan'],
    ['temple', 'Temple'],
    ['harvest', 'Harvest'],
    ['pests', 'Pests'],
    ['water', 'Water'],
    ['crop', 'Crop'],
  ],
  // Schelling's line: stars Red, zeros Blue; the discontented yellow.
  line: [
    ['color', 'Color'],
    ['satisfaction', 'Satisfaction'],
  ],
  // His plane: Red inside across, Blue inside up; where each color is content tinted.
  tipping: [['plane', 'Plane']],
  // One column per agent: status, L and D, larder up and scatter down (the population panel).
  hoard: [['agents', 'Agents']],
  // Axtell's red founders and blue members first; then preference, effort and pay.
  firms: [
    ['founder', 'Founder'],
    ['theta', 'θ (income)'],
    ['effort', 'Effort'],
    ['income', 'Income'],
  ],
};

/** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
  sugarscape: [...NETWORKS, ...MINDS_OVERLAYS],
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
  farol: [],
  ants: [],
  thresholds: [],
  retirement: [],
  punishment: [],
  zi: [],
  bali: [],
  line: [],
  tipping: [],
  hoard: [],
  firms: [],
};
