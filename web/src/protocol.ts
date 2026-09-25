// Messages between the engine (page) and the SimHost (simulation worker, or the page as a fallback).
import type { CreditGraph } from './credit';
import type { AnyInspection, ColorMode, DiseaseEntry, FieldError, Layer, ModelConfig, ModelStats } from './types';

export type Overlay = 'trade' | 'credit' | 'disease' | 'neighbors' | 'friends' | 'family';
export const OVERLAYS: Overlay[] = ['trade', 'credit', 'disease', 'neighbors', 'friends', 'family'];

/** Every overlay off (a fresh object each call). */
export function noOverlays(): Record<Overlay, boolean> {
  return Object.fromEntries(OVERLAYS.map((k) => [k, false])) as Record<Overlay, boolean>;
}

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

/** How the host renders frames and which network overlays are drawn. */
export interface DisplayState { colorMode: ColorMode; layer: Layer; overlays: Record<Overlay, boolean> }

/** The selection to report on: a site, or an agent (tracked while it lives; x, y are its last known site). */
export interface SelectQuery { x: number; y: number; agentId: number | null }

/** The selected site as the host last saw it; `alive` is false once a selected agent has died (or none was selected). */
export interface Selected { x: number; y: number; agentId: number | null; alive: boolean; view: AnyInspection }

/** Several series downsampled onto one tick axis: `columns[k][i]` is series k at `ticks[i]` (NaN = no value). */
export interface ChartGroup { ticks: Float64Array; columns: Float64Array[] }

/** Points per line in a chart group. */
export const CHART_POINTS = 2000;

/** What a snapshot should carry besides the always-present fields (Decision 2). */
export interface Wants {
  select?: SelectQuery;
  trail?: boolean;
  networks?: Overlay[];
  charts?: { groups: string[][]; max: number };
  lorenz?: boolean;
  wealthHist?: boolean;
  ageHist?: boolean;
  tagHist?: boolean;
  lorenzTotal?: boolean;
  goodWealthHists?: boolean;
  supplyDemand?: boolean;
  creditGraph?: boolean;
  diseaseList?: boolean;
  /** Ring World's sugar per site and agents' sites (the ring view). */
  ring?: boolean;
}

/** Ring World's state for the ring view: sugar per site (site 0 first) and each agent's site. */
export interface RingState { sugar: Float64Array; agents: Uint32Array }

export interface WorldSnapshot {
  /** RGBA pixels, when the request lent a buffer (transferred back). */
  frame?: ArrayBuffer;
  width: number;
  height: number;
  tick: number;
  population: number;
  latest: ModelStats;
  /** The followed agent's id (alive or not), or null. */
  followed: number | null;
  followedAlive: boolean;
  /** The normalized live config: after init, reset, setConfig and a scheduled change. */
  config?: ModelConfig;
  /** Each good's map where it differs from the generated one: after init, reset, setConfig, paint and import. */
  editedLandscapes?: (Uint8Array | null)[];
  /** The display, when the host had to clamp it to the config. */
  display?: DisplayState;
  /** The selection (from `wants.select` or an `inspect` command); null when an inspect by id found no agent. */
  inspection?: Selected | null;
  trail?: Uint32Array;
  networks?: Partial<Record<Overlay, Uint32Array>>;
  /** Chart groups, by `chartKey`, that have news since they were last sent. */
  charts?: Record<string, ChartGroup>;
  lorenz?: Float64Array;
  wealthHist?: Float64Array;
  /** `[bin, count_0, …]`: living agents' ages in 5-tick bins (Animation III-1). */
  ageHist?: Float64Array;
  /** The percentage of agents with a 0 at each tag position, position 0 first (Animation III-7). */
  tagHist?: Float64Array;
  /** The Lorenz curve of total wealth (every good's holdings summed), 101 points. */
  lorenzTotal?: Float64Array;
  /** Each good's wealth histogram `[binWidth, counts…]` (20 bins), in good order. */
  goodWealthHists?: Float64Array[];
  supplyDemand?: Float64Array;
  creditGraph?: CreditGraph;
  diseaseList?: DiseaseEntry[];
  ring?: RingState;
  /** Edits still to replay: after every init and reset, and whenever it changes. */
  replayLeft?: number;
  /** A page edit just dropped the edits still to replay (the session branched here). */
  forked?: true;
}

export type Command =
  | { type: 'ready' }
  | { type: 'init'; config: ModelConfig; seed: number; landscapes: (Uint8Array | null)[]; display: DisplayState; log?: LogEntry[] }
  | { type: 'reset'; config: ModelConfig; seed: number; landscapes: (Uint8Array | null)[]; log?: LogEntry[] }
  | { type: 'setConfig'; config: ModelConfig }
  | { type: 'step'; n: number }
  | { type: 'refresh' }
  | { type: 'setDisplay'; display: DisplayState }
  | { type: 'paint'; x: number; y: number; radius: number; value: number; good: number }
  | { type: 'importLandscape'; good: number; capacities: Uint8Array }
  | { type: 'place'; x: number; y: number; overrides: PlaceOverrides }
  | { type: 'erase'; x: number; y: number }
  | { type: 'infect'; x: number; y: number; disease: number }
  | { type: 'vaccinate'; x: number; y: number; radius: number; disease: number }
  | { type: 'follow'; id: number | null }
  | { type: 'inspect'; target: { x: number; y: number } | { agentId: number } }
  | { type: 'seriesCsv' }
  | { type: 'agentsCsv' }
  | { type: 'fingerprint' }
  | { type: 'session' }
  | { type: 'endReplay' }
  | { type: 'run' }
  | { type: 'stop' }
  | { type: 'frame' };

/** The commands that change the world: logged with the tick they were applied at, and replayed (Decision 1). */
export type EditCommand = Extract<
  Command,
  { type: 'setConfig' | 'paint' | 'importLandscape' | 'place' | 'erase' | 'infect' | 'vaccinate' }
>;

/** One edit of a session: applied after tick `tick` was computed, before tick + 1 is. */
export interface LogEntry { tick: number; cmd: EditCommand }

/** What a world was built from and every edit since: replaying it rebuilds the world exactly. */
export interface Session { config: ModelConfig; seed: number; landscapes: (Uint8Array | null)[]; log: LogEntry[] }

/**
 * The `session` command's answer: the log (the entries applied so far, then those still to
 * replay), whether it overflowed `LOG_CAP`, and the tick when it was taken.
 */
export interface SessionLog { log: LogEntry[]; full: boolean; tick: number }

export interface HostRequest { id: number; cmd: Command; wants?: Wants; frame?: ArrayBuffer }

export type Result =
  | { ok: true; snapshot?: WorldSnapshot; value?: string; session?: SessionLog }
  | { ok: false; errors: FieldError[] }
  | { ok: false; fatal: string };

export interface HostReply { id: number; result: Result; spare?: ArrayBuffer[] }

/** A reply, or something the host sends on its own: a Max-speed snapshot, or news that it died. */
export type HostMessage = HostReply | { id: null; post: WorldSnapshot } | { id: null; fatal: string };

export function chartKey(names: string[]): string {
  return names.join('|');
}

const FLAGS = [
  'trail',
  'lorenz',
  'wealthHist',
  'ageHist',
  'tagHist',
  'lorenzTotal',
  'goodWealthHists',
  'supplyDemand',
  'creditGraph',
  'diseaseList',
  'ring',
] as const;

/** Combines wants: flags OR, networks and chart groups are unioned, the first selection wins. */
export function mergeWants(parts: Wants[]): Wants {
  const out: Wants = {};
  const networks = new Set<Overlay>();
  const groups = new Map<string, string[]>();
  let max = 0;
  for (const w of parts) {
    if (w.select && !out.select) out.select = w.select;
    for (const flag of FLAGS) if (w[flag]) out[flag] = true;
    w.networks?.forEach((k) => networks.add(k));
    if (w.charts) {
      max = Math.max(max, w.charts.max);
      for (const g of w.charts.groups) groups.set(chartKey(g), g);
    }
  }
  if (networks.size > 0) out.networks = OVERLAYS.filter((k) => networks.has(k));
  if (groups.size > 0) out.charts = { groups: [...groups.values()], max };
  return out;
}

/** The buffers a message carries, to transfer rather than copy. */
export function transfers(m: HostRequest | HostMessage): ArrayBuffer[] {
  if ('cmd' in m) return m.frame ? [m.frame] : [];
  if (m.id === null) return 'post' in m && m.post.frame ? [m.post.frame] : [];
  const out = [...(m.spare ?? [])];
  if (m.result.ok && m.result.snapshot?.frame) out.push(m.result.snapshot.frame);
  return out;
}
