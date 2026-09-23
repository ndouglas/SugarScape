// Mirrors of the JSON shapes produced by sugarscape-core (serde).

export interface URange { min: number; max: number }

export type LandscapeKind = { kind: 'two_peaks' } | { kind: 'flat'; capacity: number };

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
  landscape: LandscapeKind;
  population: number;
  placement: Placement;
  vision: URange;
  metabolism: URange;
  endowment: URange;
  tag_length: number;
  growback: { rate: number; instant: boolean };
  seasons: { enabled: boolean; winter_divisor: number; period: number };
  pollution: { enabled: boolean; production: number; consumption: number; spice_pollutes: boolean };
  diffusion: { enabled: boolean; every: number };
  lifespan: { enabled: boolean; max_age: URange };
  replacement: { enabled: boolean };
  sex: { enabled: boolean; fertility_onset: URange; female_end: URange; male_end: URange };
  inheritance: { enabled: boolean };
  culture: { enabled: boolean };
  combat: { enabled: boolean; unlimited: boolean; reward: number };
  spice: { enabled: boolean; metabolism: URange; endowment: URange };
  trade: { enabled: boolean };
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
}

export interface SiteView { x: number; y: number; sugar: number; capacity: number; pollution: number; spice: number; spice_capacity: number }
export interface LinkView { id: number; alive: boolean }
export interface LoanView { id: number; role: 'lender' | 'borrower'; counterparty: LinkView; due: number; due_tick: number }
export interface DiseaseView { id: number; bits: string; distance: number }
export interface DiseaseEntry { id: number; bits: string; carriers: number }
export interface AgentView {
  id: number;
  x: number;
  y: number;
  sex: 'female' | 'male';
  tribe: 'blue' | 'red';
  tags: string;
  vision: number;
  metabolism: number;
  sugar: number;
  initial_sugar: number;
  age: number;
  max_age: number;
  fertile: boolean;
  fertility_onset: number;
  fertility_end: number;
  born: number;
  parents: LinkView[];
  children: LinkView[];
  spice: number;
  initial_spice: number;
  spice_metabolism: number;
  foresight: number;
  loans: LoanView[];
  immune: string;
  immune_genome: string;
  diseases: DiseaseView[];
  infected_by: LinkView | null;
}
export interface Inspection { site: SiteView; agent: AgentView | null }

export type ColorMode = 'tribe' | 'wealth' | 'sex' | 'age' | 'vision' | 'credit' | 'disease';
export type Layer = 'sugar' | 'capacity' | 'pollution' | 'spice' | 'spice_capacity';

/** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
export function parseErrors(e: unknown): FieldError[] {
  const text = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e);
  try {
    const parsed: unknown = JSON.parse(text);
    if (Array.isArray(parsed)) return parsed as FieldError[];
  } catch {
    // not JSON
  }
  return [{ field: 'config', message: text }];
}
