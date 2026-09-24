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
  culture: { enabled: boolean; groups: TagGroup[] };
  combat: { enabled: boolean; unlimited: boolean; reward: number };
  trade: { enabled: boolean; price: PriceRule };
  credit: { enabled: boolean; duration: number; rate: number };
  foresight: { enabled: boolean; range: URange };
  disease: DiseaseRule;
  schedule: ScheduledChange[];
}

export interface Preset { id: string; name: string; source: string; description: string; config: Config }

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
}

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

export type ColorMode = 'tribe' | 'wealth' | 'sex' | 'age' | 'vision' | 'credit' | 'disease' | 'lineage';
export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}`;

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
