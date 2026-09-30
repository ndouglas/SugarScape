// Mirrors of the JSON shapes produced by sugarscape-core (serde).

export interface URange { min: number; max: number }

export type Transform =
  | 'identity'
  | 'rotate_90'
  | 'rotate_180'
  | 'rotate_270'
  | 'mirror_x'
  | 'mirror_y'
  | 'transpose'
  | 'anti_transpose';

export type PriceRule = 'geometric_mean' | 'random';

export interface Peak { x: number; y: number; radius: number; height: number }

export type GoodMap =
  | { kind: 'two_peaks'; transform: Transform }
  | { kind: 'peaks'; peaks: Peak[] }
  | { kind: 'flat'; capacity: number }
  | { kind: 'noise'; seed: number; scale: number; octaves: number; height: number };

export interface Good { name: string; color: string; map: GoodMap; metabolism: URange; endowment: URange }

export interface Pollutant { name: string; production: number[]; consumption: number[]; devalues: boolean[] }

/** A tag group (tribe): agents whose tags hold a number of zeros in `zeros`. */
export interface TagGroup { name: string; color: string; zeros: URange }

export type Placement =
  | { kind: 'random' }
  | { kind: 'block'; x: number; y: number; width: number; height: number }
  | { kind: 'tribes'; size: number };

export interface ScheduledChange { tick: number; set: Record<string, unknown> }

export interface Outbreak { tick: number; agents: number; length?: URange | null }

export interface DiseaseRule {
  enabled: boolean;
  count: number;
  length: URange;
  initial: number;
  immune_length: number;
  fee: number;
  flips_per_tick: number;
  learning: 'per_agent' | 'per_disease';
  cure: 'next_tick' | 'immediate';
  genome_mutation: number;
  disease_mutation: number;
  outbreaks: Outbreak[];
}

/**
 * Minds 1's decision seam (absent from older configs: the book's rule M). Minds 4 adds `goap`
 * (planning) and `mvt` (the marginal-value rule); both need `movement.mode: 'walk'`.
 */
export interface Decision {
  rule: 'book' | 'utility' | 'goap' | 'mvt';
  travel: number;
  crowding: number;
  idle: 'stay' | 'wander';
}

/** Minds 2's movement seam (absent from older configs: the book's rule M jumps at speed 1). */
export interface Movement {
  mode: 'jump' | 'walk';
  speed: number;
}

/** Minds 2: a rectangle of impassable sites. `opaque` walls (stone) also block sight; fences (wood) don't. */
export interface Wall {
  x: number;
  y: number;
  width: number;
  height: number;
  opaque: boolean;
}

/** Minds 3: how an agent reasons about a remembered site it can't currently see. */
export type Belief = 'recall' | 'project';

/**
 * Minds 3's memory seam (absent from older configs: the book, no memory). `span` and `share` are
 * reset-only (`span > 0` needs `movement.mode: 'walk'`); `belief` applies live.
 */
export interface Memory {
  span: number;
  share: number;
  belief: Belief;
  /** Minds 4: what a founder starts knowing (reset-only; absent from older configs: `none`); founders start knowing the map, children start with empty memory. */
  prior?: 'none' | 'map';
}

/**
 * Minds 4's GOAP search (absent from older configs: k 8, horizon 10, shortlist rate); all live.
 * `k` known sites (1–12) ranked by `shortlist`; a goal of `horizon` ticks of food (1–100).
 */
export interface Goap {
  k: number;
  horizon: number;
  shortlist: 'rate' | 'value';
}

/** Minds 4's marginal-value rule (absent from older configs: α 0.05); `alpha` in (0, 1], live. */
export interface Mvt {
  alpha: number;
}

/**
 * Minds 3's truffles (absent from older configs: no truffles). `share` and `seed` are reset-only;
 * `value` and `regrow` apply live.
 */
export interface Truffles {
  share: number;
  value: number;
  regrow: number;
  seed: number;
}

export interface Config {
  width: number;
  height: number;
  population: number;
  placement: Placement;
  vision: URange;
  tag_length: number;
  goods: Good[];
  growback: { rate: number; instant: boolean };
  seasons: { enabled: boolean; winter_divisor: number; period: number };
  pollution: { enabled: boolean; pollutants: Pollutant[] };
  diffusion: { enabled: boolean; every: number };
  lifespan: { enabled: boolean; max_age: URange; founders?: 'newborn' | 'random' };
  replacement: { enabled: boolean };
  sex: {
    enabled: boolean; fertility_onset: URange; female_end: URange; male_end: URange;
    fertile_wealth?: 'each_good' | 'total' | 'welfare' | 'sugar';
  };
  inheritance: { enabled: boolean };
  /**
   * Rule K. `rule`, `features`, `traits` and `stop_when_settled` are milestone 14's Axelrod option
   * (absent from older configs: the book's `flip`).
   */
  culture: { enabled: boolean; groups: TagGroup[]; rule?: 'flip' | 'axelrod'; features?: number; traits?: number; stop_when_settled?: boolean };
  combat: { enabled: boolean; unlimited: boolean; reward: number };
  trade: { enabled: boolean; price: PriceRule };
  credit: { enabled: boolean; duration: number; rate: number };
  foresight: { enabled: boolean; range: URange };
  disease: DiseaseRule;
  decision?: Decision;
  movement?: Movement;
  walls?: Wall[];
  memory?: Memory;
  truffles?: Truffles;
  goap?: Goap;
  mvt?: Mvt;
  schedule: ScheduledChange[];
}

/** The models the playground runs (milestones 9–13). */
export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi' | 'bali' | 'line' | 'tipping';

/** A fraction range (Schelling's preferences). */
export interface FRange { min: number; max: number }

/** A demand table: for n occupied neighbors, the least (`min[n]`) and most (`max[n]`) alike wanted; empty uses the share. */
export interface Demand { min: number[]; max: number[] }

/** Schelling's checkerboard (1971) by default; Epstein & Axtell's variant (VI-4 to VI-7) as presets. */
export interface SchellingConfig {
  model: 'schelling';
  width: number;
  height: number;
  population: number;
  preference: FRange;
  residence: { enabled: boolean; min: number; max: number };
  neighborhood: 'moore' | 'von_neumann';
  radius: number;
  edges: 'bounded' | 'torus';
  movement: 'nearest' | 'random';
  order: 'rounds' | 'random';
  sweep: 'reading' | 'center_out';
  red_share: number;
  exact: boolean;
  red_demand: Demand;
  blue_demand: Demand;
}

/** Ring World (animations VI-8 and VI-9). */
export interface RingConfig {
  model: 'ring';
  sites: number;
  agents: number;
  vision: URange;
  capacity: number;
  growback: number;
  start: 'random' | 'megagroup';
}

/** The published replication's departures from the written Artificial Anasazi (all on: the replication). */
export interface Quirks {
  age_before_death_check: boolean;
  no_farm_water_check: boolean;
  uplands_single_class: boolean;
  wrap_edges: boolean;
  initial_corn_per_slot: boolean;
  fission_fresh_endowment: boolean;
  fission_needs_free_farm: boolean;
  initial_eligibility_ignores_adjustment: boolean;
  occupancy_leak: boolean;
  single_harvest_variance: boolean;
}

/** Artificial Anasazi, the Long House Valley (milestone 10). A tick is a year from `start_year`. */
export interface AnasaziConfig {
  model: 'anasazi';
  start_year: number;
  end_year: number;
  initial_households: number;
  initial_corn: FRange;
  need: number;
  fertility_start: number;
  fertility_end: number;
  death_age: number;
  fission_probability: number;
  child_endowment: number;
  storage_years: number;
  harvest_adjustment: number;
  spatial_sd: number;
  annual_sd: number;
  quirks: Quirks;
}

/** NetLogo Rebellion's departures from the paper (all off: the paper's rules). */
export interface CivilQuirks {
  floor_ratio: boolean;
  active_counts_twice: boolean;
  cop_moves_to_arrest: boolean;
  jailed_stay: boolean;
  netlogo_jail_term: boolean;
}

/** A numeric live field moving linearly from its value when tick `start` begins to `to` when tick `end` begins; while it runs, a scheduled or live change to that field lasts only until its next tick. */
export interface CivilRamp { path: string; start: number; end: number; to: number }

/** Epstein's civil violence (milestone 11): Model I (`rebellion`) or Model II (`ethnic`). */
export interface CivilConfig {
  model: 'civil';
  variant: 'rebellion' | 'ethnic';
  width: number;
  height: number;
  agent_density: number;
  cop_density: number;
  legitimacy: number;
  threshold: number;
  k: number;
  vision: { agent: number; cop: number };
  movement: boolean;
  jail: { max: number; infinite: boolean };
  clone_probability: number;
  max_age: number;
  stop_at_extinction: boolean;
  outburst_threshold: number;
  quirks: CivilQuirks;
  schedule: ScheduledChange[];
  ramps: CivilRamp[];
}

/** Nowak & May's spatial Prisoner's Dilemma and its variants (milestone 13). */
export interface SpatialConfig {
  model: 'spatial';
  lattice: 'square' | 'cube' | 'random';
  width: number;
  height: number;
  neighborhood: 'moore' | 'von_neumann';
  boundary: 'fixed' | 'periodic';
  occupancy: number;
  radius: number;
  b: number;
  epsilon: number;
  self_weight: number;
  update: 'synchronous' | 'asynchronous';
  winning: 'deterministic' | 'probabilistic';
  m: number;
  start: 'random' | 'single_defector';
  defectors: number;
  schedule: ScheduledChange[];
}

/**
 * A config of any model. A sugarscape `Config` carries no `model` key (every config, link, session
 * and sweep written before milestone 9 is one); the others carry theirs. Narrow with `isSugar` /
 * `modelOf` (models.ts).
 */
/**
 * Riolo, Cohen & Axelrod's tag-based donation (milestone 12), with Edmonds & Hales' and Roberts &
 * Sherratt's departures as switches. A tick is a generation.
 */
export interface TagsConfig {
  model: 'tags';
  agents: number;
  pairings: number;
  cost: number;
  benefit: number;
  initial_tolerance: 'uniform' | { fixed: number };
  tag_mutation: number;
  tolerance_mutation: number;
  tolerance_sd: number;
  /** The last generation (0: never). */
  end: number;
  tie_rule: 'random' | 'current' | 'other';
  donation_test: 'at_most' | 'below';
  tolerance_floor: number;
  tag_noise: number;
  selection: 'tournament' | 'adopt';
}

/** A strategy as Inspect and `allowed` name it (`allowed` lists only E, H, S and T). */
export type EthnoStrategy = 'E' | 'H' | 'S' | 'T' | 'kin' | 'nonkin' | 'mixed';

/** Hammond & Axelrod's ethnocentrism model and its critics' variants (milestone 16). A tick is a period. */
export interface EthnoConfig {
  model: 'ethno';
  width: number;
  colors: number;
  start: 'empty' | 'random' | 'selfish';
  immigration: number;
  base_ptr: number;
  cost: number;
  benefit: number;
  death: number;
  mutation: number;
  /** The tag's own mutation rate; null = `mutation`. */
  tag_mutation: number | null;
  pair_play: 'once' | 'twice';
  discrimination: 'same_other' | 'none' | 'each_color';
  misperception: number;
  offspring: 'adjacent' | 'anywhere';
  /** Not on the Rules panel: presets, files and links set it. */
  allowed: ('E' | 'H' | 'S' | 'T')[];
  kin_strategies: boolean;
  kin_basis: 'mutates' | 'fixed';
  kin_mutation: number;
  /** The last period (0: never). */
  end: number;
  schedule: ScheduledChange[];
}

/**
 * Epstein's demographic Prisoner's Dilemma (milestone 19): agents with fixed strategies move, play
 * their neighbors, clone and die by their accumulated payoffs, with the working paper's rule, Radax
 * and Rengs' timing choices, soup and metabolism as switches. A tick is a cycle.
 */
export interface DpdConfig {
  model: 'dpd';
  width: number;
  agents: number;
  initial_cooperators: number;
  initial_wealth: number;
  /** Payoffs: T to a defector against a cooperator, R to two cooperators, P to two defectors, S to a cooperator against a defector (any finite values). */
  t: number;
  r: number;
  p: number;
  s: number;
  fission_wealth: number;
  endowment: number;
  /** 0: no maximum. */
  max_age: number;
  metabolism: number;
  metabolism_per: 'cycle' | 'interaction';
  mutation: number;
  vision: number;
  play: 'each_neighbor' | 'random_neighbor';
  pairing: 'space' | 'soup';
  death_timing: 'immediate' | 'own_turn';
  removal: 'immediate' | 'end_of_cycle';
  endowment_from: 'parent' | 'granted';
  newborn_age: 'random' | 'zero';
  updating: 'asynchronous' | 'synchronous';
  shuffle: 'swaps' | 'full';
  newborns_act: 'next_cycle' | 'this_cycle';
  /** The last cycle (0: never). */
  end: number;
  schedule: ScheduledChange[];
}

/** An image-scoring strategy class (milestone 21). */
export type ImageClass = 'k' | 'h' | 'and' | 'or' | 'own_only' | 'binary' | 'standing' | 'q';

/**
 * One image-scoring strategy as the core writes it: `{k: 0}`, `{h: 1}`, `{and: {k, h}}`,
 * `{or: {k, h}}`, `{own_only: h}`, `{binary: k}`, `"standing"` or `{q: Δq in hundredths}`.
 */
export type ImageStrategy =
  | { k: number }
  | { h: number }
  | { and: { k: number; h: number } }
  | { or: { k: number; h: number } }
  | { own_only: number }
  | { binary: number }
  | 'standing'
  | { q: number };

/**
 * Nowak and Sigmund's image scoring (milestone 21), with Leimar and Hammerstein's island model,
 * errors, standing and q strategies: g groups of n, each generation m random donor–recipient pairs
 * per group, offspring in proportion to payoff. A tick is a generation.
 */
export interface ImageConfig {
  model: 'image';
  groups: number;
  group_size: number;
  /** p: the chance an offspring's parent comes from its own group. */
  local: number;
  /** m: rounds per group per generation (fixed, or the mean). */
  rounds: number;
  rounds_kind: 'fixed' | 'random';
  b: number;
  c: number;
  u0: number;
  /** `both`: c added to donor and recipient each round (LH01 on NS98). */
  offset: 'both' | 'none';
  /** Scores stay in −clamp … +clamp (0: unbounded). */
  clamp: number;
  information: 'perfect' | 'observers';
  /** With observers: the mean number of members besides the pair who see an interaction. */
  observers: number;
  /** What an observer writes: its own tally ± 1 (FAIR23), or the donor's new score. */
  records: 'tally' | 'score';
  execution_error: number;
  perception_error: number;
  mutation: number;
  /** The classes allowed (presets, files and links set them; the Rules panel does not). */
  strategies: ImageClass[];
  /** `"uniform"`, or everyone playing `only` but the first round(share × n) of each group playing `invader`. */
  initial: 'uniform' | { only: ImageStrategy; invader?: ImageStrategy; share?: number };
  /** The last generation (0: never). */
  end: number;
  schedule: ScheduledChange[];
}


/**
 * Axelrod's culture model (milestone 14): sites with F features of q traits copying a neighbor's
 * trait with probability equal to their similarity, with Axtell et al.'s and later departures.
 */
export interface CultureConfig {
  model: 'culture';
  width: number;
  height: number;
  features: number;
  traits: number;
  neighborhood: 'von_neumann' | 'moore' | 'diamond' | 'soup';
  boundary: 'bounded' | 'torus';
  activation: 'random' | 'sweep';
  changes: 'active' | 'neighbor';
  drift: number;
  stop_when_stable: boolean;
}

/**
 * Axtell, Epstein and Young's bargaining society (milestone 15): agents best-reply to their memory of
 * opponents' demands in the Nash demand game, with Poza et al.'s departures.
 */
export interface ClassesConfig {
  model: 'classes';
  agents: number;
  memory: number;
  noise: number;
  tags: boolean;
  tag_memory: 'per_tag' | 'shared';
  decision: 'expected' | 'mode';
  low: number;
  start: 'random' | 'fractious' | 'progressive' | 'classes';
  interaction: 'random' | 'lattice';
  lattice: { width: number; height: number; neighborhood: 'moore' | 'von_neumann'; layout: 'random' | 'four_zones' | 'two_zones' };
  stop_at_equity: boolean;
}

/**
 * Hegselmann and Krause's bounded confidence (milestone 17): agents move to the mean of the opinions
 * within their reach, with the paper's asymmetric and opinion-dependent confidence and its unfigured
 * claims (serial updating, lattice neighborhoods) as switches.
 */
export interface OpinionsConfig {
  model: 'opinions';
  agents: number;
  start: 'random' | 'regular';
  confidence: 'symmetric' | 'asymmetric' | 'opinion_dependent';
  epsilon: number;
  epsilon_left: number;
  epsilon_right: number;
  bias: number;
  updating: 'simultaneous' | 'serial_shuffled' | 'serial_random';
  interaction: 'all' | 'lattice';
  lattice: { width: number; height: number; neighborhood: 'moore' | 'von_neumann' };
  stop_when_stable: boolean;
}

/**
 * Cohen, Riolo and Axelrod's population of adaptive agents playing short iterated Prisoner's
 * Dilemmas under a social structure (milestone 18), with the paper's two readings of its method as
 * switches.
 */
export interface StructureConfig {
  model: 'structure';
  agents: number;
  structure: 'rwr' | 'torus' | 'frne' | 'frn';
  substitution: number;
  partners: number;
  moves: number;
  judge_error: number;
  mutation: number;
  mutation_sd: number;
  noise_on: 'always' | 'copy';
  start: 'grid' | 'random';
  high: number;
  stop_at: number;
}

/**
 * Axelrod's norms and metanorms games (milestone 20): agents evolving boldness and vengefulness,
 * with Galán and Izquierdo's departures and readings as switches.
 */
export interface NormsConfig {
  model: 'norms';
  agents: number;
  metanorms: boolean;
  rounds: number;
  temptation: number;
  hurt: number;
  punishment: number;
  enforcement: number;
  meta_punishment: number;
  meta_enforcement: number;
  mutation: number;
  selection: 'axelrod' | 'tournament' | 'roulette' | 'average';
  refill: 'random' | 'ranked';
  all_equal: 'drift' | 'keep';
  groups: { enabled: boolean; strong: number; weak: number; strong_punishment: number };
  stop_at: number;
}

/**
 * Deffuant et al.'s relative agreement (milestone 22): random pairs meet and move each other's
 * opinions and uncertainties; extremists, three bounded-confidence rules, networks, and the readings
 * the papers leave open (Meadows and Cliff's, the 2013 reply's, eq. 11's window) as switches.
 */
export interface AgreementConfig {
  model: 'agreement';
  agents: number;
  rule: 'ra' | 'bc' | 'bc_averaging' | 'bc_variance';
  window: 'influencer' | 'listener';
  mu: number;
  alpha: number;
  uncertainty: number;
  extremists: number;
  extremist_uncertainty: number;
  delta: number;
  placement: 'drawn' | 'bounds' | 'band';
  band: number;
  extreme_margin: number;
  pair_update: 'simultaneous' | 'sequential' | 'one_way';
  network: 'all' | 'lattice' | 'small_world' | 'scale_free';
  lattice: { width: number; height: number; neighborhood: 'moore' | 'von_neumann' };
  small_world: { substrate: 'ring' | 'grid'; degree: number; rewire: number };
  scale_free: { links: number };
  pairing: 'edge' | 'node';
  stop_when_stable: boolean;
  stop_at: number;
}

export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig | BaliConfig | LineConfig | TippingConfig;

/**
 * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
 * and Ottino's scoring, bias and random baseline, and the minority game's Darwinian variant.
 */
export interface FarolConfig {
  model: 'farol';
  game: 'el_farol' | 'minority';
  agents: number;
  strategies: number;
  behavior: 'inductive' | 'random';
  /** Null: the game's own (60 % of N, or (N − 1)/2). */
  capacity: number | null;
  scoring: 'error' | 'payoff';
  decay: number;
  at_capacity: 'stay' | 'go';
  shared: boolean;
  memory: number;
  mixed_memory: { enabled: boolean; min: number; max: number };
  payoff: 'step' | 'inverse';
  rounding: 'nearest' | 'exact';
  bias: number;
  information: 'true' | 'random';
  evolution: { enabled: boolean; every: number; strategy_mutation: number; memory_mutation: number };
  stop_at: number;
}

export interface FarolStats {
  tick: number;
  attendance: number;
  crowded: number;
  /** (A − c)² over the last 100 rounds, over N; and the same for coin-flippers. */
  fluctuation: number;
  random_fluctuation: number;
  success: number;
  mean_gain: number;
  switching: number;
  forecast_above: number;
  mean_memory: number;
}

/** A strategy or predictor of an inspected agent: its score, and what it says for next round. */
export interface FarolStrategyView { label: string; score: number; forecast: number | null; attend: boolean | null; active: boolean }
export interface FarolAgentView { id: number; memory: number; went: boolean; gain: number; switches: number; strategies: FarolStrategyView[] }
/**
 * A cell of the El Farol frame: a round of the time panel, a row of the histogram, or an agent of
 * the grid (`member`). `agent` is always null: cells are read where they are.
 */
export interface FarolInspection {
  site: { x: number; y: number };
  panel: 'time' | 'histogram' | 'agents' | null;
  round: number | null;
  attendance: number | null;
  count: number | null;
  member: FarolAgentView | null;
  agent: null;
}

/**
 * Kirman's ants and recruitment (milestone 24), with Becker's majority pull, more sources, meetings
 * over a network, and Alfarano and Milaković's agent rule.
 */
export interface AntsConfig {
  model: 'ants';
  ants: number;
  rule: 'kirman' | 'alfarano';
  epsilon: number;
  delta: number;
  conversion: 'kirman' | 'footnote';
  meetings: number;
  sources: number;
  pull: number;
  a: number;
  lambda: number;
  network: 'complete' | 'ring' | 'small_world' | 'random' | 'scale_free';
  degree: number;
  link: number;
  independent: number;
  start: 'random' | 'one';
  stop_at: number;
}

export interface AntsStats {
  tick: number;
  /** z: the share at the first source; the largest source's share. */
  share: number;
  top_share: number;
  /** Var[z] so far, and what theory gives (null when neither Kirman's chain nor the mean field applies). */
  variance: number;
  theory_variance: number | null;
  flips: number;
  residence: number;
  extreme: number;
}

export interface AntView { id: number; source: number; independent: boolean; degree: number; elsewhere: number }
/**
 * A cell of the ants frame: a step of the time panel, a row of the histogram, or an ant of the grid
 * (`member`). `agent` is always null: cells are read where they are.
 */
export interface AntsInspection {
  site: { x: number; y: number };
  panel: 'time' | 'histogram' | 'ants' | null;
  step: number | null;
  shares: number[] | null;
  share: number | null;
  count: number | null;
  theory: number | null;
  member: AntView | null;
  agent: null;
}

/**
 * Granovetter's threshold models (milestone 25): crowds, friends, sampled crowds, clusters and
 * ceilings, with Watts's cascades on random networks.
 */
export interface ThresholdsConfig {
  model: 'thresholds';
  actors: number;
  distribution: 'uniform' | 'perturbed' | 'normal' | 'fixed';
  mean: number;
  sd: number;
  crowd: 'quantiles' | 'sampled';
  rounding: 'exact' | 'floor' | 'nearest';
  population: 'fixed' | 'city';
  network: 'everyone' | 'random' | 'power_law';
  degree: number;
  counts_self: boolean;
  friends: { enabled: boolean; acquaintance: number; weight: number; symmetric: boolean };
  trigger: 'instigators' | 'random' | 'hub';
  zero: 'acts' | 'when_reached';
  update: 'synchronous' | 'asynchronous';
  ceilings: { share: number; at: number };
  clusters: { enabled: boolean; count: number; movement: number };
  repeat: boolean;
  global: number;
  max_steps: number;
  stop_at: number;
}

export interface ThresholdsStats {
  tick: number;
  acting: number;
  step: number;
  episodes: number;
  last_size: number;
  mean_size: number;
  global_share: number;
  /** Granovetter's continuous equilibrium share (a normal crowd seen whole), or null. */
  theory: number | null;
  recent_mean: number;
  swing: number;
}

export interface ActorView {
  id: number;
  threshold: number | null;
  ceiling: number | null;
  degree: number | null;
  sees: number;
  of: number;
  acting: boolean;
  seed: boolean;
  crowd: number;
}
/**
 * A cell of the thresholds frame: a step of the time panel, a point of Granovetter's Figure 1, a
 * histogram row, or an actor of the grid (`member`). `agent` is always null.
 */
export interface ThresholdsInspection {
  site: { x: number; y: number };
  panel: 'time' | 'figure' | 'histogram' | 'actors' | null;
  step: number | null;
  crowds: number[] | null;
  share: number | null;
  cdf: number | null;
  count: number | null;
  member: ActorView | null;
  agent: null;
}

/**
 * Axtell and Epstein's timing of retirement (milestone 26): cohorts, rationals, randoms and
 * imitators in transient social networks, the policy switch and two coupled sub-populations.
 */
export interface RetirementConfig {
  model: 'retirement';
  per_cohort: number;
  rational: number;
  random: number;
  p: number;
  threshold: number;
  spread: number;
  size: { min: number; max: number };
  extent: number;
  counts: 'eligible' | 'all';
  renewal: 'slot' | 'replace';
  order: 'by_cohort' | 'shuffled';
  initial_deaths: 'literal' | 'survivors';
  eligibility: number;
  mandatory: number;
  policy: { enabled: boolean; to: number };
  groups: { enabled: boolean; coupling: number };
  norm: number;
  stop_at_norm: boolean;
  stop_at: number;
}

export interface RetirementStats {
  tick: number;
  retired: number;
  retired_a: number;
  retired_b: number;
  /** The period the norm set in, and periods from the policy switch to the new norm (null before). */
  transition: number | null;
  transition_new: number | null;
  /** The period each group reached the norm (both `transition` without groups). */
  transition_a: number | null;
  transition_b: number | null;
  modal_age: number | null;
  mean_age: number | null;
  rational_share: number;
  eligibility: number;
}

export interface RetireeView {
  id: number;
  age: number;
  kind: 'rational' | 'random' | 'imitator';
  threshold: number;
  death_age: number;
  group: number;
  network: number;
  eligible: number;
  retired_members: number;
  retired: boolean;
  retired_at: number | null;
}
/**
 * A cell of the retirement frame: an agent of the population, an age's retirement bar, or a period
 * of the time panel. `agent` is always null.
 */
export interface RetirementInspection {
  site: { x: number; y: number };
  panel: 'population' | 'ages' | 'time' | null;
  age: number | null;
  retirements: number | null;
  exposed: number | null;
  period: number | null;
  retired: number | null;
  member: RetireeView | null;
  agent: null;
}

/**
 * Boyd, Gintis, Bowles and Richerson's altruistic punishment (milestone 27): groups of contributors,
 * defectors and punishers, payoff-biased imitation, intergroup conflict and mutation.
 */
export interface PunishmentConfig {
  model: 'punishment';
  groups: number;
  size: number;
  cost: number;
  punish_cost: number;
  fine: number;
  punishing: 'variable' | 'fixed';
  fixed_cost: number;
  benefit: number;
  baseline: number;
  error: number;
  mixing: number;
  mutation: number;
  conflict: number;
  pairing: 'paired' | 'either' | 'challenge';
  victory: 'defectors' | 'payoff' | 'tanh';
  sensitivity: number;
  counted: 'types' | 'acts';
  erring: 'others' | 'none' | 'self';
  imitation: 'together' | 'in_turn';
  refill: 'copy' | 'split';
  traits: 'discrete' | 'continuous';
  structure: 'groups' | 'ring';
  start: 'one_punisher_group' | 'all_defectors';
  window: number;
  stop_at: number;
}

export interface PunishmentStats {
  tick: number;
  cooperation: number;
  contributors: number;
  punishers: number;
  defectors: number;
  punishment: number;
  /** This period's share cooperating and mean payoff (null before the first period). */
  acts: number | null;
  payoff: number | null;
  conflicts: number;
  extinctions: number;
  spread: number;
  /** The mean cooperation over the long-run window so far (null before it). */
  long_run: number | null;
}

export interface PunisherView {
  id: number;
  group: number;
  kind: 'contributor' | 'defector' | 'punisher' | null;
  cooperate: number;
  punish: number;
  cooperated: boolean;
  punished: boolean;
  payoff: number;
}

export interface PunishmentGroupView {
  index: number;
  contributors: number;
  punishers: number;
  defectors: number;
  acts: number;
  payoff: number;
  last_conflict: number | null;
  lost: boolean;
}

/** A cell of the punishment frame: an agent and its group, or a period of the time strip. */
export interface PunishmentInspection {
  site: { x: number; y: number };
  panel: 'groups' | 'time' | null;
  group: PunishmentGroupView | null;
  agent: PunisherView | null;
  period: number | null;
  cooperation: number | null;
  punishment: number | null;
}

/**
 * Gode and Sunder's zero-intelligence traders (milestone 28), with Cliff's critique, mechanism and
 * ZIP traders.
 */
export interface ZiConfig {
  model: 'zi';
  market: 'gs1' | 'gs2' | 'gs3' | 'gs4' | 'gs5' | 'symmetric' | 'flat_supply' | 'excess_demand' | 'excess_supply' | 'retail' | 'custom';
  buyers: number[][];
  sellers: number[][];
  price_max: number;
  strategy: 'zi_u' | 'zi_c' | 'zip';
  mechanism: 'book' | 'cliff';
  nyse: boolean;
  turns: 'trader' | 'side';
  sellers_only: boolean;
  period_end: 'shouts' | 'sessions' | 'failures';
  shouts: number;
  sessions: number;
  momentum: 'code' | 'text';
  shift: 'none' | 'demand' | 'supply';
  shift_at: number;
  stop_at: number;
}

export interface ZiStats {
  tick: number;
  /** This shout's trade price (null if it did not trade). */
  price: number | null;
  /** This period so far (the mean price and rmsd null before a trade). */
  mean_price: number | null;
  volume: number;
  efficiency: number | null;
  rmsd: number | null;
  alpha: number | null;
  dispersion: number;
  period: number;
  p0: number;
  /** The last completed period, and means over the completed periods (null before one). */
  last_price: number | null;
  last_efficiency: number | null;
  last_alpha: number | null;
  last_dispersion: number | null;
  avg_price: number | null;
  avg_efficiency: number | null;
  avg_dispersion: number | null;
}

export interface ZiTrade { period: number; tick: number; price: number; buyer: number; seller: number; value: number; cost: number }

export interface ZiTraderView {
  id: number;
  buyer: boolean;
  limits: number[];
  traded: number;
  profit: number;
  equilibrium_profit: number;
  margin: number | null;
}

/** A cell of the zi frame: a step of the schedules, a trade, or a trader. */
export interface ZiInspection {
  site: { x: number; y: number };
  panel: 'schedules' | 'prices' | 'traders' | null;
  unit: number | null;
  demand: number | null;
  supply: number | null;
  trade: ZiTrade | null;
  trader: ZiTraderView | null;
  /** Always null: cells are read where they are. */
  agent: null;
}

/**
 * Lansing and Kremer's Balinese water temples (milestone 29), with Janssen's reanalysis, on Janssen's
 * data for the Oos and Petanu. One tick is a month.
 */
export interface BaliConfig {
  model: 'bali';
  watershed: 'bali' | 'two_node';
  plans: 'random' | 'traditional' | 'hyv' | 'temples' | 'search';
  level: number;
  decision: 'imitate' | 'generalized' | 'adaptive' | 'fixed';
  growth: number;
  dispersal: number;
  rain: 'low' | 'middle' | 'high' | 'random';
  rain_scale: number;
  perturb: { enabled: boolean; at: number; growth: number; dispersal: number; damage: number; rain: number };
  routing: 'network' | 'janssen_code';
  dam_columns: 'code' | 'physical';
  pest_form: 'shortcut' | 'diffusion';
  pest_reset: boolean;
  gamma_p: number;
  gamma_w: number;
  innovation: number;
  m_w: number;
  m_p: number;
  remove_links: number;
  add_links: number;
  node_rain: number;
  node_periods: number;
  score_from: number;
  stop_at: number;
}

/** A month's statistics; the year's are held from its end (null before the first, or where they do not apply). */
export interface BaliStats {
  tick: number;
  harvest: number | null;
  spread: number | null;
  scored: number | null;
  changing: number;
  water_stress: number | null;
  pest_loss: number | null;
  patches: number | null;
  strategies: number | null;
  temple_match: number | null;
  network_match: number | null;
  year: number;
}

export interface BaliSubakView {
  id: number;
  area: number;
  masceti: number;
  source: number;
  ret: number;
  plan: number;
  start: number;
  crop: number;
  harvest: number;
  pests: number;
  water: number;
  neighbors: number;
}

export interface BaliDamView { id: number; inflow: number; demand: number; stress: number }

/** A cell of the bali frame: a subak or a dam on the map, or a month of the water strip. */
export interface BaliInspection {
  site: { x: number; y: number };
  panel: 'map' | 'strip' | null;
  subak: BaliSubakView | null;
  dam: BaliDamView | null;
  month: number | null;
  stress: number | null;
  /** Always null: cells are read where they are. */
  agent: null;
}

/** A preset: `title` is the menu's plain headline; `source` and `name` are its figure or paper and its rules. */
export interface Preset { id: string; title: string; name: string; source: string; description: string; config: ModelConfig }

/** One field of a model's Rules panel (the core's `schema::Param`). */
export interface Param {
  path: string;
  label: string;
  kind: 'integer' | 'number' | 'range' | 'bool' | 'choice';
  min?: number;
  max?: number;
  step?: number;
  choices?: { value: string; label: string }[];
  apply: 'live' | 'reset';
  group: string;
  /** A one-line explanation shown under the control (the anasazi's quirks). */
  help?: string;
  /** Shown only while the field `path` equals `equals` (a bool as `'true'`/`'false'`): Model II's population fields, the kin fields. */
  show_if?: { path: string; equals: string };
  /** A second condition that must also hold. */
  also_if?: { path: string; equals: string };
  /** A number field that may be empty: null shows as an empty box, and an empty box sends null. */
  nullable?: true;
  /** For a nullable field, the path its slider follows while null (else it follows `min`). */
  fallback?: string;
}

export interface FieldError { field: string; message: string }

export interface Snapshot {
  tick: number;
  population: number;
  gini: number;
  mean_wealth: number;
  mean_vision: number;
  mean_metabolism: number;
  blue_fraction: number;
  births: number;
  deaths: number;
  mean_log_price: number;
  sd_log_price: number;
  trade_volume: number;
  sugar_traded: number;
  loans_made: number;
  amount_lent: number;
  defaults: number;
  debt_outstanding: number;
  mean_foresight: number;
  mean_spice: number;
  mean_spice_metabolism: number;
  infected_fraction: number;
  mean_diseases: number;
  diseases_in_circulation: number;
  new_infections: number;
  trade_pairs: number;
  gini_total: number;
  goods: { mean_holding: number; mean_metabolism: number; traded: number }[];
  pollution: number[];
  groups: number[];
  /** Under Axelrod's culture rule (milestone 14). */
  axelrod?: { distinct_cultures: number; settled: boolean };
  /** Minds 1's patch counts, on peaks maps with two or more peaks. */
  patches?: { on_first: number; on_other: number; off: number };
}

export interface SchellingStats {
  tick: number;
  population: number;
  unsatisfied: number;
  segregation: number;
  moves: number;
  red_share: number;
  quiet: number;
}

export interface RingStats {
  tick: number;
  population: number;
  flocks: number;
  mean_flock: number;
  largest_flock: number;
  mean_distance: number;
}

export interface AnasaziStats {
  tick: number;
  year: number;
  households: number;
  historical: number;
  fit: number;
  capacity: number;
  mean_corn: number;
  births: number;
  moves: number;
  departures: number;
}

export interface CivilStats {
  tick: number;
  population: number;
  active: number;
  quiet: number;
  jailed: number;
  cops: number;
  legitimacy: number;
  mean_grievance: number;
  tension: number;
  outbursts: number;
  /** Null until an outburst has followed another. */
  mean_wait: number | null;
  /** Null until an outburst has ended. */
  mean_activation: number | null;
  blue: number;
  green: number;
  killed: number;
  /** The tick a group was first gone (Model II), else the current tick. */
  extinction: number;
}

export interface SpatialStats {
  tick: number;
  fraction_c: number;
  changed: number;
  c_to_d: number;
  d_to_c: number;
  /** Null with no player of that strategy. */
  mean_payoff_c: number | null;
  mean_payoff_d: number | null;
  players: number;
}


export interface TagsStats {
  tick: number;
  population: number;
  donation_rate: number;
  mean_tolerance: number;
  cluster_share: number;
  relatedness: number;
  cluster_tolerance: number;
  zero_tolerance_share: number;
  distinct_tags: number;
  takeovers: number;
}

/** A period's statistics. Shares are null on an empty lattice; the interaction ratios null with a zero denominator (and at t = 0). */
export interface EthnoStats {
  tick: number;
  population: number;
  ethnocentric: number | null;
  humanitarian: number | null;
  selfish: number | null;
  traitorous: number | null;
  kin: number | null;
  nonkin: number | null;
  mixed: number | null;
  cooperation: number | null;
  same_tag: number | null;
  relatives: number | null;
  kin_help: number | null;
  tag_given_relative: number | null;
  relative_given_tag: number | null;
}

/**
 * A generation's statistics: the rounds it played and the strategies that played them (tick 0: the
 * first generation before it plays, so its help rate is null). A share or mean with nobody to count
 * is null.
 */
export interface ImageStats {
  tick: number;
  /** Helps ÷ rounds played. */
  help_rate: number | null;
  /** The mean k over agents whose strategy has one (k, AND, OR, binary). */
  mean_k: number | null;
  /** The share whose strategy helps at a generation's start (k ≤ 0 for the k strategies). */
  cooperative: number | null;
  mean_payoff: number | null;
  mean_score: number | null;
  k_cooperative: number | null;
  k_defective: number | null;
  h: number | null;
  own_only: number | null;
  and: number | null;
  or: number | null;
  standing: number | null;
  binary_c: number | null;
  binary_x: number | null;
  binary_d: number | null;
  q: number | null;
  /** Helps given this generation. */
  helps: number;
}

/** A cycle's statistics, of the agents alive at its end. The share and mean wealths are null with nobody to count. */
export interface DpdStats {
  tick: number;
  cooperators: number;
  defectors: number;
  population: number;
  cooperator_share: number | null;
  /** Cooperators all eight of whose Moore neighbors are cooperators. */
  surrounded: number;
  wealth_c: number | null;
  wealth_d: number | null;
  births: number;
  deaths: number;
}

/** The latest statistics of a world of any model. */
export interface CultureStats {
  tick: number;
  regions: number;
  zones: number;
  cultures: number;
  largest_region: number;
  mean_similarity: number;
  active_bonds: number;
  changes: number;
  stable_at: number;
}

export interface ClassesStats {
  tick: number;
  mean_payoff: number;
  m_share: number;
  outcome_mm: number;
  outcome_hl: number;
  outcome_fail: number;
  outcome_waste: number;
  /** 0 mixed, 1 equity, 2 fractious, 3 classes, 4 equity between types only, 5 equity above and division below. */
  regime: number;
  segregated: number;
  equity_at: number;
  first_attractor: number;
  payoff_dark: number;
  payoff_light: number;
  payoff_inter: number;
  realized_noise: number;
}

export interface OpinionsStats {
  tick: number;
  /** Surviving opinions: groups of opinions within 10⁻⁶ of each other. */
  clusters: number;
  largest: number;
  second: number;
  mean_opinion: number;
  median_opinion: number;
  range: number;
  splits: number;
  one_sided_splits: number;
  max_change: number;
  stable_at: number;
}

export interface StructureStats {
  tick: number;
  mean_payoff: number;
  cooperation: number;
  mean_y: number;
  mean_p: number;
  mean_q: number;
  high: number;
  /** The first period at or above the threshold, else −1. */
  attained_high: number;
  share_high_since: number;
  copied: number;
  partner_p_slope: number;
}

export interface NormsStats {
  tick: number;
  mean_boldness: number;
  mean_vengefulness: number;
  mean_payoff: number;
  defections: number;
  punishments: number;
  metapunishments: number;
  /** Galán & Izquierdo's regions: 1 when the generation is in them. */
  established: number;
  collapsed: number;
  strong_boldness: number;
  weak_boldness: number;
  strong_vengefulness: number;
  weak_vengefulness: number;
  copied_equal: number;
}

export interface AgreementStats {
  tick: number;
  /** DAWF's indicator: the squared shares of moderates turned extremist at each end, summed. */
  y: number;
  p_plus: number;
  p_minus: number;
  /** 0 central, 1 both extremes, 2 single extreme, 3 intermediate. */
  outcome: number;
  clusters: number;
  major: number;
  isolated: number;
  largest: number;
  second: number;
  dispersion: number;
  unmoved: number;
  mean_opinion: number;
  mean_uncertainty: number;
  max_change: number;
  stable_at: number;
}

export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats | BaliStats;

export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
export interface LinkView { id: number; alive: boolean }
export interface LoanView { id: number; role: 'lender' | 'borrower'; counterparty: LinkView; good: number; due: number; due_tick: number }
export interface DiseaseView { id: number; bits: string; distance: number }
export interface DiseaseEntry { id: number; bits: string; carriers: number }
export interface AgentView {
  id: number;
  x: number;
  y: number;
  sex: 'female' | 'male';
  tribe: 'blue' | 'red';
  group: number;
  tags: string;
  vision: number;
  holdings: number[];
  initial: number[];
  metabolism: number[];
  age: number;
  max_age: number;
  fertile: boolean;
  fertility_onset: number;
  fertility_end: number;
  born: number;
  parents: LinkView[];
  children: LinkView[];
  foresight: number;
  loans: LoanView[];
  immune: string;
  immune_genome: string;
  diseases: DiseaseView[];
  infected_by: LinkView | null;
  /**
   * Minds 2: null until the agent first moves; `path` is empty under jump, once it has arrived, or
   * when no path was found. `walked`: it walked (or tried to) rather than jumped.
   */
  plan?: { target_x: number; target_y: number; path: [number, number][]; walked: boolean } | null;
  /**
   * Minds 3: whether the agent remembers, and how many sites and truffle spots it holds in
   * memory. `null` while memory is off (`span` 0).
   */
  memory?: { remembers: boolean; sites: number; spots: number } | null;
  /**
   * Minds 4: the GOAP plan, under `decision.rule: 'goap'` while the agent holds one: the targets
   * left in order (empty just after the plan finishes), what it was to gather in all, and its goal
   * G. `null` otherwise.
   */
  goap?: GoapView | null;
  /** Minds 4: the running intake-rate estimate ρ, under `decision.rule: 'mvt'` only. */
  rate?: number | null;
}
export interface GoapView { steps: [number, number][]; gathers: number; goal: number }
export interface Inspection { site: SiteView; agent: AgentView | null }

export interface SchellingAgentView {
  id: number;
  color: 'red' | 'blue';
  preference: number;
  satisfied: boolean;
  /** Like-colored and all occupied neighbors. */
  like: number;
  neighbors: number;
  age: number;
  /** The age at which it leaves; null with residence off. */
  residence: number | null;
}
export interface SchellingInspection { site: { x: number; y: number }; agent: SchellingAgentView | null }

/** Schelling's line (1971): a row of stars (Red) and zeros (Blue) with no gaps. */
export interface LineConfig {
  model: 'line';
  length: number;
  red_share: number;
  exact: boolean;
  radius: number;
  preference: number;
  reach: number;
  fallback: number;
  wrap: number;
}
/** A person in the line: its place, color and how many of its neighbors are alike. */
export interface LinePersonView { id: number; place: number; color: 'red' | 'blue'; like: number; neighbors: number; satisfied: boolean }
export interface LineInspection { place: number; agent: LinePersonView | null }

/** A tolerance schedule (Schelling 1971, pp. 168–180). */
export type Schedule = { shape: 'line'; intercept: number } | { shape: 'hyperbola'; k: number } | { shape: 'tiers'; tiers: [number, number][] };
/** Schelling's bounded neighborhood (1971): one area, each person with a tolerance for the other color. */
export interface TippingConfig {
  model: 'tipping';
  red: number;
  blue: number;
  red_schedule: Schedule;
  blue_schedule: Schedule;
  intolerant_red: number;
  intolerant_blue: number;
  draws: 'schedule' | 'random';
  start: { kind: 'given'; red: number; blue: number } | { kind: 'random'; chance: number };
  speed_red: number;
  speed_blue: number;
  order: 'alternate' | 'blue_first' | 'simultaneous';
  entry: 'counting_self' | 'as_is';
  limit_red: number;
  limit_blue: number;
  limit_total: number;
}
/** A point of his plane: Red and Blue inside, and whether the most tolerant of each would all be content there. */
export interface TippingInspection { red_in: number; blue_in: number; red_content: boolean; blue_content: boolean; now: boolean; agent: null }
export interface RingInspection { site: { x: number; sugar: number; capacity: number }; agent: { id: number; vision: number } | null }
/** A Long House Valley cell: its zone, this year's PDSI class and yields, water and occupants. */
export interface ValleyCellView {
  x: number;
  y: number;
  zone: string;
  zone_name: string;
  pdsi: number;
  /** 0 (≤ −3) to 4 (≥ 3). */
  pdsi_class: number;
  zone_yield: number;
  quality: number;
  base_yield: number;
  water: boolean;
  water_near: boolean;
  habitable: boolean;
  farmed_by: number | null;
  residents: number[];
}
export interface HouseholdView {
  id: number;
  age: number;
  /** Newest first. */
  corn: number[];
  stock: number;
  harvest: number;
  expected: number;
  farm: [number, number];
  home: [number, number];
}
export interface AnasaziInspection { site: ValleyCellView; agent: HouseholdView | null }

/** A civil agent: its state, H, R, G, the arrest probability it estimates here, N = R·P, and in Model II its group and age. */
export interface CitizenView {
  id: number;
  state: 'quiet' | 'active' | 'jailed';
  hardship: number;
  risk_aversion: number;
  grievance: number;
  arrest_probability: number;
  net_risk: number;
  /** Ticks of jail left; null while free or for a term that never ends. */
  jail_left: number | null;
  jail_life: boolean;
  group: 'blue' | 'green' | null;
  age: number | null;
  death_age: number | null;
}
/** A civil site: its free agent, its cop, and the agents jailed after arrest here. */
export interface CivilInspection { site: { x: number; y: number }; agent: CitizenView | null; cop: { id: number } | null; jailed: CitizenView[] }

export interface SpatialCandidate { x: number; y: number; z: number; strategy: 'C' | 'D'; score: number }
/** A spatial player: its strategy now and a generation ago, its score, and who could take its site (itself first). */
export interface PlayerView {
  id: number;
  strategy: 'C' | 'D';
  previous: 'C' | 'D';
  score: number;
  candidates: SpatialCandidate[];
  /** The next strategy under deterministic winning; null when winning is probabilistic. */
  next: 'C' | 'D' | null;
  /** Eq. 1's P(C) under probabilistic winning (null when deterministic, or when every score is 0). */
  p_c: number | null;
}
/** A spatial cell (in a cube, of the slice on screen); no player on an empty cell of a random array. */
export interface SpatialInspection { site: { x: number; y: number; z: number }; agent: PlayerView | null }

/** An agent of the current generation: its traits, and this generation's score and donations. */
export interface TaggerView { id: number; parent: number; tag: number; tolerance: number; score: number; given: number; received: number }
/**
 * A cell of the tag × generation diagram: its generation (null above the first) and tag bin, what
 * that bin held, and its agents when it is the current generation. `agent` is always null: agents
 * live one generation, so there is nobody to follow.
 */
export interface TagsInspection {
  site: { x: number; y: number };
  generation: number | null;
  from: number;
  to: number;
  count: number;
  distinct: number;
  tolerance: { min: number; mean: number; max: number } | null;
  given: number;
  received: number;
  agents: TaggerView[];
  agent: null;
}

/** An occupied neighbor of an ethnocentrism agent (up, left, right, down): whether they share a founding immigrant, and this period's helps each way. */
export interface EthnoNeighborView { x: number; y: number; tag: number; strategy: EthnoStrategy; related: boolean; helped: number; helped_by: number }
/** An ethnocentrism agent: its traits, this period's PTR and helps, its lineage, kin marker and age, and its neighbors. */
export interface EthnoAgentView {
  id: number;
  tag: number;
  strategy: EthnoStrategy;
  /** What same/other is judged by: the tag, or (kin strategies) the kin marker. */
  basis: 'tag' | 'kin';
  ptr: number;
  given: number;
  received: number;
  /** The founding immigrant's id. */
  lineage: number;
  /** The family founder's id. */
  kin_marker: number;
  age: number;
  neighbors: EthnoNeighborView[];
}
/** An ethnocentrism site. Empty, it looks exactly like an empty Schelling site: `isEthnoView` asks the model. */
export interface EthnoInspection { site: { x: number; y: number }; agent: EthnoAgentView | null }

/** An occupied neighbor of a demographic PD agent (up, left, right, down), with one game's payoff to each under the current payoffs. An infinite payoff serializes as null. */
export interface DpdNeighborView { x: number; y: number; id: number; strategy: 'C' | 'D'; payoff: number | null; their_payoff: number | null }
/** A demographic PD agent: its strategy, wealth and age, whether it is surrounded, this cycle's income and games, and its neighbors. Wealth and income serialize as null if ever infinite. */
export interface DpdAgentView {
  id: number;
  strategy: 'C' | 'D';
  wealth: number | null;
  age: number;
  /** The maximum age (0: none). */
  max_age: number;
  /** A cooperator all eight of whose Moore neighbors are cooperators. */
  surrounded: boolean;
  /** Payoffs received and games played this cycle. */
  income: number | null;
  games: number;
  neighbors: DpdNeighborView[];
}
/** A demographic PD site. Empty, it looks exactly like an empty Schelling site: `isDpdView` asks the model. */
export interface DpdInspection { site: { x: number; y: number }; agent: DpdAgentView | null }

/** An image-scoring agent in the generation that last played. */
export interface ImageAgentView {
  id: number;
  group: number;
  /** "k = 0", "k = 0, h = 1 (AND)", "standing", "Δq = 0.25" … */
  strategy: string;
  class: ImageClass;
  /** Whether the strategy helps at a generation's start. */
  cooperative: boolean;
  score: number;
  /** Good standing (as everyone would judge it without perception errors). */
  standing: boolean;
  /**
   * With private records: the members who have seen it act this generation, and their mean record
   * of its score (null when none has); both null with perfect information.
   */
  known: number | null;
  mean_view: number | null;
  payoff: number;
  /** Helps given and received this generation. */
  given: number;
  received: number;
}
/**
 * A cell of the image-scoring frame: its group's tile (null in a gap) and the agent there (null in a
 * gap or a tile's unused cell). It has `cell`, not `site`: the guards that read `site` check that
 * there is one.
 */
export interface ImageInspection { cell: { x: number; y: number }; group: number | null; agent: ImageAgentView | null }

/** What a world of any model says about a site. */
/** A culture site: its position, traits, and the sizes of its region and zone. */
export interface CultureSiteView { x: number; y: number; traits: number[]; region_size: number; zone_size: number }
/**
 * A cell of the culture frame: a site (with what each neighbor shares) or a lane between two sites
 * (with what they share). `agent` is always null: sites do not move.
 */
export interface CultureInspection {
  site: { x: number; y: number };
  kind: 'site' | 'lane';
  a: CultureSiteView;
  b: CultureSiteView | null;
  shared: number | null;
  neighbors: { x: number; y: number; shared: number }[];
  agent: null;
}

/** An agent whose memory plots at an inspected simplex point. */
export interface BargainerView { id: number; tag: 'dark' | 'light' | null; memory: [number, number, number]; last_demand: 'L' | 'M' | 'H' | null; mean_payoff: number }
/**
 * A point of a memory simplex: which simplex, the mix of L, M and H remembered there, the best reply,
 * and the agents there. `agent` is always null: agents have no place to follow.
 */
export interface ClassesInspection {
  site: { x: number; y: number };
  simplex: 'one' | 'intra' | 'inter' | null;
  mix: [number, number, number] | null;
  best_reply: 'L' | 'M' | 'H' | null;
  agents: BargainerView[];
  agent: null;
}

/** An agent whose line passes an inspected point, or a lattice site's agent. */
export interface OpinionAgent { id: number; start: number; opinion: number; epsilon_left: number; epsilon_right: number; reaches: number }
/**
 * A cell of the opinion × time diagram (its period and opinion, and the agents whose lines pass
 * within a cell) or of the lattice to its right. `agent` is always null: agents have no place to follow.
 */
export interface OpinionsInspection {
  site: { x: number; y: number };
  period: number | null;
  opinion: number | null;
  lattice_site: { x: number; y: number } | null;
  agents: OpinionAgent[];
  agent: null;
}

/** An agent played this period: its p as played, its score and how many games. */
export interface PartnerView { id: number; p: number; score: number; games: number }
/** An agent: its strategy, class, this period's score, whom it copied and (in the block) its partners. */
export interface StructureAgentView {
  id: number;
  y: number;
  p: number;
  q: number;
  class: 'tft' | 'alld' | 'allc' | 'other';
  score: number;
  copied: number | null;
  partners: PartnerView[];
}
/** A cell of the agents' block (its agent) or of the p–q plane (its (p, q) and the agents there). */
export interface StructureInspection {
  site: { x: number; y: number };
  block: { x: number; y: number } | null;
  plane: [number, number] | null;
  agents: StructureAgentView[];
  agent: StructureAgentView | null;
}

/** An agent of the generation that played last: its strategy, payoff and what happened to it. */
export interface NormAgentView {
  id: number;
  group: 'strong' | 'weak' | null;
  bits: string;
  boldness: number;
  vengefulness: number;
  payoff: number;
  defections: number;
  punished: number;
  punishments: number;
  metapunishments: number;
  metapunished: number;
  parent: number | null;
}
/** A cell of the boldness–vengefulness plane (its levels and agents) or of the agent strip. */
export interface NormsInspection {
  site: { x: number; y: number };
  level: [number, number] | null;
  agents: NormAgentView[];
  agent: NormAgentView | null;
}

/** An agent at an inspected cell: its opinion and uncertainty there, where it started, and its meetings. */
export interface AgreementAgent {
  id: number;
  role: 'plus' | 'minus' | 'moderate';
  start: number;
  opinion: number;
  uncertainty: number;
  degree: number;
  meetings: number;
  moves: number;
}
/**
 * A cell of the opinion × time diagram, the start-against-now panel or the torus, and the agents
 * there. `agent` is always null: a clicked cell is read again each period.
 */
export interface AgreementInspection {
  site: { x: number; y: number };
  panel: 'diagram' | 'scatter' | 'torus' | null;
  period: number | null;
  opinion: number | null;
  agents: AgreementAgent[];
  agent: null;
}

export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection | BaliInspection | LineInspection | TippingInspection;

/**
 * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
 * `occupation`, `zones`, `yield`, or (civil violence) `action`, `grievance`, `group`, or (tags)
 * `count`, `tolerance`, `clones`, or (ethnocentrism) `strategy`, `tag`, `lineage`, `ptr`, or (the
 * demographic PD) `strategy`, `wealth`, `age`, `surrounded`, or (image scoring) `strategy`, `score`,
 * `payoff`.
 */
export type ColorMode =
  | 'tribe'
  | 'wealth'
  | 'sex'
  | 'age'
  | 'vision'
  | 'credit'
  | 'disease'
  | 'lineage'
  | 'color'
  | 'satisfaction'
  | 'preference'
  | 'occupation'
  | 'zones'
  | 'yield'
  | 'action'
  | 'grievance'
  | 'group'
  | 'change'
  | 'strategy'
  | 'payoff'
  | 'count'
  | 'tolerance'
  | 'clones'
  | 'tag'
  | 'ptr'
  | 'culture'
  | 'similarity'
  | 'zones'
  | 'best_reply'
  | 'payoff'
  | 'start'
  | 'opinion'
  | 'friendliness'
  | 'provocability'
  | 'strategy'
  | 'surrounded'
  | 'agents'
  | 'uncertainty'
  | 'role'
  | 'score'
  | 'choice'
  | 'gain'
  | 'memory'
  | 'source'
  | 'independent'
  | 'degree'
  | 'state'
  | 'threshold'
  | 'crowd'
  | 'status'
  | 'type'
  | 'group'
  | 'acts'
  | 'side'
  | 'profit'
  | 'margin'
  | 'plan'
  | 'temple'
  | 'harvest'
  | 'pests'
  | 'water'
  | 'crop'
  | 'plane';
export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;

/** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
export function parseErrors(e: unknown, field = 'config'): FieldError[] {
  const text = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
  try {
    const parsed: unknown = JSON.parse(text);
    if (Array.isArray(parsed)) return parsed as FieldError[];
  } catch {
    // not JSON
  }
  return [{ field, message: text }];
}
