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
  genome_mutation: number;
  disease_mutation: number;
  outbreaks: Outbreak[];
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
  lifespan: { enabled: boolean; max_age: URange };
  replacement: { enabled: boolean };
  sex: { enabled: boolean; fertility_onset: URange; female_end: URange; male_end: URange };
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
  schedule: ScheduledChange[];
}

/** The models the playground runs (milestones 9–13). */
export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'opinions';

/** A fraction range (Schelling's preferences). */
export interface FRange { min: number; max: number }

/** The book's Schelling variant (animations VI-4 to VI-7). */
export interface SchellingConfig {
  model: 'schelling';
  width: number;
  height: number;
  population: number;
  preference: FRange;
  residence: { enabled: boolean; min: number; max: number };
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
 * Hegselmann and Krause's bounded confidence (milestone 16): agents move to the mean of the opinions
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

export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | OpinionsConfig;

export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }

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
  /** Shown only while the string field `path` equals `equals` (Model II's population fields). */
  show_if?: { path: string; equals: string };
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

export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | OpinionsStats;

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
}
export interface Inspection { site: SiteView; agent: AgentView | null }

export interface SchellingAgentView {
  id: number;
  color: 'red' | 'blue';
  preference: number;
  satisfied: boolean;
  /** Like-colored and all occupied von Neumann neighbors. */
  like: number;
  neighbors: number;
  age: number;
  /** The age at which it leaves; null with residence off. */
  residence: number | null;
}
export interface SchellingInspection { site: { x: number; y: number }; agent: SchellingAgentView | null }
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

export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | OpinionsInspection;

/**
 * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
 * `occupation`, `zones`, `yield`, or (civil violence) `action`, `grievance`, `group`, or (tags)
 * `count`, `tolerance`, `clones`.
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
  | 'culture'
  | 'similarity'
  | 'zones'
  | 'best_reply'
  | 'payoff'
  | 'start'
  | 'opinion';
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
