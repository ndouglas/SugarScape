// Mirrors of the JSON shapes produced by sugarscape-core (serde).

export interface URange { min: number; max: number }

export type LandscapeKind = { kind: 'two_peaks' } | { kind: 'flat'; capacity: number };

export type Placement =
  | { kind: 'random' }
  | { kind: 'block'; x: number; y: number; width: number; height: number }
  | { kind: 'tribes'; size: number };

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
  pollution: { enabled: boolean; production: number; consumption: number };
  diffusion: { enabled: boolean; every: number };
  lifespan: { enabled: boolean; max_age: URange };
  replacement: { enabled: boolean };
  sex: { enabled: boolean; fertility_onset: URange; female_end: URange; male_end: URange };
  inheritance: { enabled: boolean };
  culture: { enabled: boolean };
  combat: { enabled: boolean; unlimited: boolean; reward: number };
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
}

export interface SiteView { x: number; y: number; sugar: number; capacity: number; pollution: number }
export interface LinkView { id: number; alive: boolean }
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
}
export interface Inspection { site: SiteView; agent: AgentView | null }

export type ColorMode = 'tribe' | 'wealth' | 'sex' | 'age' | 'vision';
export type Layer = 'sugar' | 'capacity' | 'pollution';

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
