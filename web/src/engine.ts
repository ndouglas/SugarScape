import { fieldErrorsMessage } from './errors';
import {
  chartKey,
  mergeWants,
  NETWORKS,
  noOverlays,
  type ChartGroup,
  type Command,
  type DisplayState,
  type LogEntry,
  type Overlay,
  type PlaceOverrides,
  type Result,
  type RingState,
  type Selected,
  type SelectQuery,
  type Session,
  type ValleyState,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { calendarYear, isSugar, modelOf, ticksLeft } from './models';
import { MAX_TICKS, SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport, startWorker, type Transport } from './transport';
import type { ColorMode, Config, FieldError, Layer, ModelConfig, ModelKind, ModelStats, Param, Preset } from './types';
import init, { model_schemas_json, presets_json } from './wasm-pkg/sugarscape.js';

export type { Overlay, PlaceOverrides } from './protocol';

export type EngineEvent =
  | 'reset'
  | 'tick'
  | 'config'
  | 'run'
  | 'select'
  | 'display'
  | 'edit'
  | 'follow'
  | 'snapshot'
  | 'crash'
  | 'replay'
  | 'fork'
  | 'full'
  | 'finished';

/** What the page says when a world reaches `MAX_TICKS` (the engine pauses and fires 'full'). */
export const FULL_NOTICE = 'This world has reached 1,000,000 ticks, the most its history holds here — export its data, or Reset to start again';

/** What the page says when a world has run its course (the engine pauses and fires 'finished'). */
export function finishedNotice(config: ModelConfig, tick: number): string {
  if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
  const year = calendarYear(config, tick);
  return `This run has reached its end year${year === null ? '' : ` (AD ${year})`} — Reset to run it again`;
}

export interface Selection { x: number; y: number; agentId: number | null }

/** A world to build: its setup, starting maps and (a session's) edits to replay. */
export interface InitialState { config: ModelConfig; seed: number; landscapes?: (Uint8Array | null)[]; log?: LogEntry[] }

/**
 * What an engine runs on: the presets (every model's), each non-sugarscape model's Rules-panel
 * schema, and a transport to a SimHost (tests pass fakes).
 */
export interface EngineDeps { presets: Preset[]; schemas?: Partial<Record<ModelKind, Param[]>>; transport: Transport }

/**
 * What a panel needs in the next snapshot. It is called before every request, so it must not
 * change state: a panel records what it received when the snapshot arrives.
 */
export type WantsProvider = (now: number) => Wants;

/**
 * Ticks per animation frame, or 'max': the host steps flat out and posts about 30 snapshots a
 * second. Below 1 it is a fraction of a tick per frame, paced by the clock rather than by frames
 * (so 1/60 is one tick a second however fast the display refreshes).
 */
export type Speed = number | 'max';

/** The frame rate a speed below 1 is a fraction of. */
const NOMINAL_FPS = 60;

/**
 * Paces a speed below 1: `due` says whether a tick is due at `now`. It keeps to the schedule
 * across frames that land a little late, but never saves up ticks (after a pause, say).
 */
export class SlowPacer {
  private next = -Infinity;

  due(speed: number, now: number): boolean {
    if (now < this.next) return false;
    const interval = 1000 / (NOMINAL_FPS * speed);
    // A frame a little late keeps the schedule; one a whole interval late starts it afresh.
    this.next = (now - this.next < interval ? this.next : now) + interval;
    return true;
  }
}

/** What the toolbar's Play, Step and speed drive: one engine, or Compare's lockstep (Decision 9). */
export interface RunControls {
  readonly running: boolean;
  readonly speed: Speed;
  setRunning(on: boolean): void;
  setSpeed(speed: Speed): void;
  advance(n?: number): Promise<void>;
  on(event: 'run', fn: () => void): () => void;
}

/** While paused, extras a panel wants are fetched at most this often. */
const REFRESH_MS = 250;
/** Frame buffers kept for reuse besides the one on screen. */
const MAX_SPARE = 4;
const NO_CELLS: Uint32Array = new Uint32Array(0);

/** A sugarscape Rules-panel edit, refused (thrown) on another model's config. */
function sugarOnly(mutate: (c: Config) => void): (c: ModelConfig) => void {
  return (c) => {
    if (!isSugar(c)) throw new Error(`this world is a ${modelOf(c)} world, not a sugarscape`);
    mutate(c);
  };
}

/** Runs `mutate` on `c`; a throw (an unknown path, the wrong model) becomes a field error. */
function tryMutate(mutate: (c: ModelConfig) => void, c: ModelConfig): FieldError[] | null {
  try {
    mutate(c);
    return null;
  } catch (e) {
    return [{ field: 'config', message: e instanceof Error ? e.message : String(e) }];
  }
}

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** A failed result's errors in the core's shape (a dead simulation is one error), or null on success. */
function failure(result: Result): FieldError[] | null {
  if (result.ok) return null;
  return 'errors' in result ? result.errors : [{ field: 'simulation', message: result.fatal }];
}

/** A write that must change the world failed, or (a host bug) succeeded without a snapshot. */
function writeFailure(result: Result): FieldError[] | null {
  if (result.ok && !result.snapshot) return [{ field: 'simulation', message: 'the simulation sent no snapshot' }];
  return failure(result);
}

/** What an engine keeps after handing its world over: every request fails, and closing does nothing. */
function deadTransport(message: string): Transport {
  return {
    onPost: null,
    onFatal: null,
    request: async () => ({ id: 0, result: { ok: false, fatal: message } }),
    close: () => {},
  };
}

/**
 * The simulation runs in a worker; the page keeps its own WASM instance for presets, Experiments
 * and, if a module worker cannot start, the simulation itself (Decision 10).
 */
async function defaultDeps(): Promise<EngineDeps> {
  const wasm = await init();
  const presets = JSON.parse(presets_json()) as Preset[];
  const schemas = JSON.parse(model_schemas_json()) as Partial<Record<ModelKind, Param[]>>;
  let transport: Transport;
  try {
    transport = await startWorker();
  } catch (e) {
    console.warn('The simulation worker could not start; simulating on the page instead.', e);
    transport = new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));
  }
  return { presets, schemas, transport };
}

/**
 * The playground's view of the simulation, which runs behind a transport: reads are served from
 * the latest snapshot, writes return promises, and events fire when replies arrive.
 */
export class Engine {
  running = false;
  speed: Speed = 1;
  private readonly slow = new SlowPacer();
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'resource:0';
  overlays: Record<Overlay, boolean> = noOverlays();
  selection: Selection | null = null;
  /** The selected site (and agent) as of the latest snapshot. */
  inspection: Selected | null = null;
  presetId: string | null = null;
  /**
   * The setup as chosen (preset, share link, reset): what a reset rebuilds and a share link
   * carries; preset matching and `isModified()` compare against it.
   */
  baseConfig!: ModelConfig;
  /**
   * The live config: what the running world uses now, including scheduled changes that have
   * fired. The Rules panel shows this one.
   */
  config!: ModelConfig;
  tick = 0;
  population = 0;
  latest: ModelStats | null = null;
  /** Ring World's sugar and agents as of the latest snapshot (the ring view); null in other models. */
  ring: RingState | null = null;
  /** The anasazi's water, settlements and links as of the latest snapshot; null in other models. */
  valley: ValleyState | null = null;
  /** The world has run its course (the anasazi's end year) as of the latest snapshot. */
  finished = false;
  /**
   * The latest snapshot; its extras are there only when something wanted them. Its frame is left
   * out: that buffer is lent back to the host later (read frames through `frame()`).
   */
  last: WorldSnapshot | null = null;
  /** Why the simulation stopped for good, or null. */
  crashed: string | null = null;
  /** Edits still to replay (the toolbar's chip counts them down). */
  replayLeft = 0;
  /** What the world was last built from: a session's config, seed and starting maps (Decision 4). */
  private origin!: { config: ModelConfig; seed: number; landscapes: (Uint8Array | null)[] };
  /** `replayLeft` changed in the snapshot being adopted: announce 'replay'. */
  private replayMoved = false;
  private width = 0;
  private height = 0;
  private followedId: number | null = null;
  private followedLive = false;
  private trailCells = NO_CELLS;
  private edges: Partial<Record<Overlay, Uint32Array>> = {};
  /** The last group received per `chartKey` (the host re-sends a group only when its history grew). */
  private charts = new Map<string, ChartGroup>();
  /**
   * Bumped on every selection change and on every new world: a reply to a request sent under an
   * older generation carries a stale selection, which is ignored (so it cannot undo the change).
   */
  private selectionGen = 0;
  /** The selection generation each received snapshot's request was sent under. */
  private sentUnder = new WeakMap<WorldSnapshot, number>();
  /** Bumped by every `setDisplay`: a display the host clamped for an older choice is ignored. */
  private displayGen = 0;
  /** The display generation each received snapshot's request was sent under. */
  private displayUnder = new WeakMap<WorldSnapshot, number>();
  /** `setDisplay` requests sent and not yet answered. */
  private displaysPending = 0;
  /** The generation of the request whose reply last set `selection`. */
  private selectedUnder = -1;
  /**
   * The selection `wants()` reports while an `inspect`/`select` issued after the confirmed
   * `selection` hasn't had its own reply resolve yet: set the moment the request is sent, so a
   * request sent before that reply lands (e.g. Max's `frame`) does not carry the stale selection
   * and clobber the host's own patch. `undefined` means no override is pending; `null` means the
   * override is "report no selection at all" — used for a pending site-only select, where the
   * target may turn out to have an agent on it the host would track more precisely than the site
   * alone, so a guess is omitted rather than sent and self-corrects once the reply lands (an
   * agentId-based select always knows its target exactly, so it never needs this: see
   * `inspectTarget`).
   */
  private pendingSelect: SelectQuery | null | undefined;
  /** Guards `pendingSelect` against a superseded inspect's own resolution clearing a newer one. */
  private pendingSelectSeq = 0;
  /** Same idea as `pendingSelect`, for `wants().trail` while a `follow`/`unfollow` is outstanding. */
  private pendingTrail: boolean | undefined;
  private pendingTrailSeq = 0;
  /** Each good's map where it differs from the generated one (share links; kept across resets). */
  private landscapes: (Uint8Array | null)[] = [];
  /** The frame on screen, and buffers free to lend to the host (Decision 6). */
  private shown: ArrayBuffer | null = null;
  private spare: ArrayBuffer[] = [];
  private providers = new Set<WantsProvider>();
  private listeners = new Map<EngineEvent, Set<() => void>>();
  /** The frame loop's step or refresh while it is outstanding. */
  private inFlight: Promise<void> | null = null;
  /** Writes that need a quiet world hold the frame loop while they run (Decision 8). */
  private holds = 0;
  /** Whether the host's Max loop should be running (a `run` was sent and no `stop` since). */
  private maxOn = false;
  /** The pending `stop`, while one is. */
  private stopping: Promise<void> | null = null;
  /**
   * The quiet writes (and Steps), one after another: each starts only once the one before has
   * settled, so it is built from the state that write left (two quick edits both take effect).
   */
  private writes: Promise<unknown> = Promise.resolve();
  /** Resets sent and not yet answered: requests sent meanwhile carry no selection. */
  private resetting = 0;
  private lastRefresh = -Infinity;
  /** The config the sugarscape panels show while another model runs: the last sugarscape config seen. */
  private lastSugar!: Config;

  private constructor(
    private transport: Transport,
    readonly presets: Preset[],
    readonly schemas: Partial<Record<ModelKind, Param[]>>,
    public seed: number,
  ) {
    transport.onFatal = (message) => this.crash(message);
    transport.onPost = (s) => this.onPost(s);
  }

  /** Throws the core's errors (as an Error message) if `initial` is invalid. */
  static async create(initial?: InitialState, deps?: EngineDeps): Promise<Engine> {
    const { presets, schemas = {}, transport } = deps ?? (await defaultDeps());
    const fallback = presets.find((p) => p.id === 'ii-2-unit') ?? presets[0];
    const engine = new Engine(transport, presets, schemas, initial?.seed ?? randomSeed());
    const firstSugar = presets.map((p) => p.config).find(isSugar);
    if (!firstSugar) throw new Error('no sugarscape preset');
    engine.lastSugar = structuredClone(firstSugar);
    const config = initial?.config ?? structuredClone(fallback.config);
    // Until the init snapshot brings the normalized config, `wants()` reads the model from this one.
    engine.config = config;
    const landscapes = initial?.landscapes ?? [];
    const log = initial?.log ?? [];
    const result = await engine.send(
      { type: 'init', config, seed: engine.seed, landscapes, display: engine.displayState(), log },
      true,
    );
    if (!result.ok || !result.snapshot) {
      transport.close();
      throw new Error((failure(result) ?? []).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    engine.origin = { config, seed: engine.seed, landscapes };
    engine.adopt(result.snapshot);
    engine.replayMoved = false;
    engine.baseConfig = structuredClone(engine.config);
    engine.presetId = engine.matchPreset();
    return engine;
  }

  /** The model of the world running now. */
  get model(): ModelKind {
    return modelOf(this.config);
  }

  /** Ticks until the world is finished (the anasazi's end year); Infinity for a model that never finishes. */
  get ticksLeft(): number {
    return ticksLeft(this.config, this.tick);
  }

  /**
   * The config the sugarscape panels (Rules, Display, tools, Inspect, Credit, Charts) read: the
   * live config while the world is a sugarscape, otherwise the last sugarscape config this engine
   * ran (those panels are hidden then, Decision 6). Never write through it.
   */
  get sugar(): Config {
    return isSugar(this.config) ? this.config : this.lastSugar;
  }

  on(event: EngineEvent, fn: () => void): () => void {
    let set = this.listeners.get(event);
    if (!set) this.listeners.set(event, (set = new Set()));
    set.add(fn);
    return () => set.delete(fn);
  }

  private emit(event: EngineEvent): void {
    this.listeners.get(event)?.forEach((fn) => fn());
  }

  /** Registers what a panel needs in snapshots (Decision 9); returns its removal. */
  want(provider: WantsProvider): () => void {
    this.providers.add(provider);
    return () => {
      this.providers.delete(provider);
    };
  }

  size(): { width: number; height: number } {
    return { width: this.width, height: this.height };
  }

  /** The frame on screen (RGBA), or null before the first one. */
  frame(): Uint8ClampedArray<ArrayBuffer> | null {
    return this.shown ? new Uint8ClampedArray(this.shown) : null;
  }

  /** Called every animation frame: asks for the next frame's steps, or (paused) for extras a panel still wants. */
  pump(now: number = performance.now()): void {
    if (this.crashed || this.inFlight || this.holds > 0) return;
    let next: Promise<void> | null = null;
    if (this.running) {
      // At Max the host runs its own loop (Decision 11); below 1× a frame steps only when a tick is due.
      if (this.speed !== 'max' && (this.speed >= 1 || this.slow.due(this.speed, now))) next = this.stepNow(Math.max(1, this.speed));
    } else if (now - this.lastRefresh >= REFRESH_MS && this.providersWant(now)) {
      next = this.refresh();
    }
    if (next) this.inFlight = next.finally(() => (this.inFlight = null));
  }

  /** A snapshot with the current wants, without changing the world. */
  async refresh(): Promise<void> {
    this.lastRefresh = performance.now();
    const result = await this.send({ type: 'refresh' });
    if (result.ok && result.snapshot) this.accept(result.snapshot);
  }

  /**
   * Rebuilds the world, keeping each good's painted/shared map unless the grid size, the number
   * of goods or that good's map changes. On error the current world is kept and errors returned.
   */
  reset(config?: ModelConfig, seed?: number): Promise<FieldError[] | null> {
    // The defaults are read when the reset runs, after any write queued before it.
    return this.quiet(() => {
      const next = config ?? this.baseConfig;
      return this.rebuild(next, seed ?? this.seed, this.keptLandscapes(next));
    });
  }

  /**
   * A reset to the base config as changed by `mutate` (a reset-requiring rule edit), built when it
   * runs so that an earlier change still in flight is kept. The sugarscape Rules panel's; it fails
   * when the world is another model.
   */
  resetWith(mutate: (c: Config) => void, seed?: number): Promise<FieldError[] | null> {
    return this.resetModelWith(sugarOnly(mutate), seed);
  }

  /** `resetWith` for a config of any model (the schema-driven Rules panel's). */
  resetModelWith(mutate: (c: ModelConfig) => void, seed?: number): Promise<FieldError[] | null> {
    return this.quiet(() => {
      const next = structuredClone(this.baseConfig);
      const refused = tryMutate(mutate, next);
      if (refused) return Promise.resolve(refused);
      return this.rebuild(next, seed ?? this.seed, this.keptLandscapes(next));
    });
  }

  /**
   * Applies a rule/parameter change to the running world: `mutate` edits a copy of the live
   * config for the world and, on success, a copy of the base config too, so scheduled changes
   * that already fired are not undone. The sugarscape Rules panel's; it fails when the world is
   * another model.
   */
  applyConfig(mutate: (c: Config) => void): Promise<FieldError[] | null> {
    return this.applyModelConfig(sugarOnly(mutate));
  }

  /** `applyConfig` for a config of any model (the schema-driven Rules panel's live fields). */
  applyModelConfig(mutate: (c: ModelConfig) => void): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const next = structuredClone(this.config);
      const refused = tryMutate(mutate, next);
      if (refused) return refused;
      const result = await this.send({ type: 'setConfig', config: next }, true);
      if (!result.ok || !result.snapshot) return writeFailure(result);
      const base = structuredClone(this.baseConfig);
      mutate(base);
      this.baseConfig = base;
      // The reply carries the new config, so accepting it fires 'config'.
      this.accept(result.snapshot);
      void this.refresh();
      return null;
    });
  }

  /** Rebuilds the world as preset `id`, at `seed` (default: the world's seed). */
  async loadPreset(id: string, seed?: number): Promise<FieldError[] | null> {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    return this.quiet(() => this.rebuild(structuredClone(preset.config), seed ?? this.seed, [], { presetId: id }));
  }

  /** True when the base config differs from the last chosen preset or a landscape is custom. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || this.landscapes.some((l) => l !== null) || JSON.stringify(preset.config) !== JSON.stringify(this.baseConfig);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
    this.syncMax();
  }

  setSpeed(speed: Speed): void {
    this.speed = speed;
    this.syncMax();
  }

  /** Steps `n` ticks, after the frame loop's step and any queued write (and before later writes). */
  advance(n = 1): Promise<void> {
    return this.quiet(() => this.stepNow(n));
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<Record<Overlay, boolean>> }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    if (d.overlays) this.overlays = { ...this.overlays, ...d.overlays };
    this.emit('display');
    // Any reply to a request sent before this one carries the host's clamp of an older choice
    // (a reset's, say): it must not overwrite this choice (see `adopt`).
    this.displayGen++;
    this.displaysPending++;
    void this.send({ type: 'setDisplay', display: this.displayState() }, true).then((result) => {
      this.displaysPending--;
      if (result.ok && result.snapshot) this.accept(result.snapshot);
    });
  }

  select(x: number, y: number): Promise<void> {
    return this.inspectTarget({ x, y });
  }

  /** Selects agent `id` if it is alive. */
  selectAgent(id: number): Promise<void> {
    return this.inspectTarget({ agentId: id });
  }

  /** Records and draws agent `id`'s trail from now on (a new trail replaces any other; reset clears it). */
  followAgent(id: number): Promise<void> {
    return this.follow(id);
  }

  unfollow(): Promise<void> {
    return this.follow(null);
  }

  /** The followed agent's id (alive or not), or null. */
  followed(): number | null {
    return this.followedId;
  }

  followedAlive(): boolean {
    return this.followedLive;
  }

  /** The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first. */
  trail(): Uint32Array {
    return this.trailCells;
  }

  /** Edges `[x1, y1, x2, y2, …]` of an overlay that is on (empty until its first snapshot). */
  networks(kind: Overlay): Uint32Array {
    return this.edges[kind] ?? NO_CELLS;
  }

  /** The latest downsampled chart group for these series (Decision 4), or undefined until one arrives. */
  chartGroup(names: string[]): ChartGroup | undefined {
    return this.charts.get(chartKey(names));
  }

  /** Each good's map where it differs from the generated one (for share links). */
  editedLandscapes(): (Uint8Array | null)[] | undefined {
    return this.landscapes.some((m) => m !== null) ? this.landscapes : undefined;
  }

  paint(x: number, y: number, radius: number, value: number, good = 0): Promise<FieldError[] | null> {
    return this.edit({ type: 'paint', x, y, radius, value, good });
  }

  /** Replaces good `good`'s capacity map (one byte per site, 0–10). */
  importLandscape(good: number, capacities: Uint8Array): Promise<FieldError[] | null> {
    return this.edit({ type: 'importLandscape', good, capacities });
  }

  place(x: number, y: number, overrides: PlaceOverrides): Promise<FieldError[] | null> {
    return this.edit({ type: 'place', x, y, overrides });
  }

  erase(x: number, y: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'erase', x, y });
  }

  /** `disease` −1 infects with a brand-new random disease. */
  infect(x: number, y: number, disease: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'infect', x, y, disease });
  }

  vaccinate(x: number, y: number, radius: number, disease: number): Promise<FieldError[] | null> {
    return this.edit({ type: 'vaccinate', x, y, radius, disease });
  }

  seriesCsv(): Promise<string> {
    return this.value({ type: 'seriesCsv' });
  }

  agentsCsv(): Promise<string> {
    return this.value({ type: 'agentsCsv' });
  }

  /** The world's fingerprint as `0x…` (the golden tests' format). */
  fingerprint(): Promise<string> {
    return this.value({ type: 'fingerprint' });
  }

  /**
   * The world's session — the config, seed and starting maps it was built from, and its edit log
   * (including edits still to replay) — whether the log overflowed, and the tick (Decision 4).
   */
  session(): Promise<{ session: Session; full: boolean; tick: number }> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'session' });
      if (!result.ok || !result.session) {
        const errors = failure(result) ?? [{ field: 'simulation', message: 'the simulation sent no session' }];
        throw new Error(fieldErrorsMessage(errors));
      }
      const { log, full, tick } = result.session;
      const { config, seed, landscapes } = this.origin;
      const session: Session = { config: structuredClone(config), seed, landscapes: landscapes.map((l) => (l ? l.slice() : null)), log };
      return { session, full, tick };
    });
  }

  /**
   * Reset with the same seed: rebuilds the session's world and replays its log from the start,
   * keeping the setup (base config and preset). With a full log it is a plain reset (Decision 4).
   */
  replay(): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'session' });
      if (!result.ok) return failure(result);
      if (!result.session || result.session.full) {
        return this.rebuild(this.baseConfig, this.seed, this.keptLandscapes(this.baseConfig));
      }
      const { config, seed, landscapes } = this.origin;
      return this.rebuild(config, seed, landscapes, { log: result.session.log, keepSetup: true });
    });
  }

  /** Builds a session opened on this page (a session file) and replays its log. */
  open(state: InitialState): Promise<FieldError[] | null> {
    return this.quiet(() => this.rebuild(state.config, state.seed, state.landscapes ?? [], { log: state.log ?? [] }));
  }

  /**
   * Drops the edits still to replay, keeping the world as it is. Queued behind any write already
   * pending — Reset (replay) then the chip's ✕ in quick succession — so a `replay()` still queued
   * cannot reach the host after this and re-arm the replay it was meant to end.
   */
  endReplay(): Promise<void> {
    return this.quiet(async () => {
      const result = await this.send({ type: 'endReplay' });
      if (result.ok && result.snapshot) this.accept(result.snapshot);
    });
  }

  /**
   * Takes over `other`'s world — its transport (worker), session and state — as Compare's
   * "Keep B" does (Decision 8). Both must be paused; `other` is dead afterwards. Panels bound to
   * this engine redraw from the events that follow.
   */
  async takeWorld(other: Engine): Promise<void> {
    if (other === this) throw new Error('an engine cannot take over its own world');
    const dead = () => {
      if (other.crashed) throw new Error(`there is no world to take over: ${other.crashed}`);
    };
    dead();
    await this.quiet(() =>
      other.quiet(async () => {
        // It may have died while the two were going quiet.
        dead();
        const mine = this.transport;
        mine.onFatal = null;
        mine.onPost = null;
        mine.close();
        this.transport = other.transport;
        this.transport.onFatal = (message) => this.crash(message);
        this.transport.onPost = (s) => this.onPost(s);
        // `other` lets go of the transport: its later requests fail, and closing it is harmless.
        other.crashed = 'This world now runs in another engine.';
        other.transport = deadTransport(other.crashed);
        other.running = false;
        other.maxOn = false;
        this.seed = other.seed;
        this.presetId = other.presetId;
        this.baseConfig = other.baseConfig;
        this.config = other.config;
        this.lastSugar = other.lastSugar;
        this.ring = other.ring;
        this.valley = other.valley;
        this.finished = other.finished;
        this.tick = other.tick;
        this.population = other.population;
        this.latest = other.latest;
        this.last = other.last;
        this.width = other.width;
        this.height = other.height;
        this.followedId = other.followedId;
        this.followedLive = other.followedLive;
        this.trailCells = other.trailCells;
        this.edges = { ...other.edges };
        this.charts = new Map(other.charts);
        this.landscapes = other.landscapes;
        this.shown = other.shown;
        this.spare = [...other.spare];
        this.selection = other.selection;
        this.inspection = other.inspection;
        this.replayLeft = other.replayLeft;
        this.origin = other.origin;
        this.colorMode = other.colorMode;
        this.layer = other.layer;
        this.overlays = { ...other.overlays };
        // Announced below, not by the next snapshot.
        this.replayMoved = false;
        // An inspect or follow still outstanding was sent to the old world: its reply must not
        // clear (or be mistaken for) anything in this one.
        this.pendingSelect = undefined;
        this.pendingSelectSeq++;
        this.pendingTrail = undefined;
        this.pendingTrailSeq++;
        // Nothing sent before this belongs to the new world.
        this.selectionGen++;
        this.displayGen++;
        this.lastRefresh = -Infinity;
      }),
    );
    for (const event of ['reset', 'follow', 'display', 'replay'] as const) this.emit(event);
    if (this.inspection) this.emit('select');
    this.emit('snapshot');
  }

  /** Stops this engine for good and closes its transport (Compare's "Keep A" discards B this way). */
  close(): void {
    this.crashed ??= 'This world was closed.';
    this.running = false;
    this.maxOn = false;
    this.transport.onFatal = null;
    this.transport.onPost = null;
    this.transport.close();
  }

  private displayState(): DisplayState {
    return { colorMode: this.colorMode, layer: this.layer, overlays: { ...this.overlays } };
  }

  /** The engine's own wants (Decision 9) merged with every provider's. */
  private wants(now: number): Wants {
    const own: Wants = {};
    // pendingSelect/pendingTrail stand in for the confirmed selection/follow state
    // while their own request's reply hasn't landed yet, so a request sent in that window (Max's
    // `frame`, in particular) reports the new target instead of the one it is replacing.
    const select = this.pendingSelect !== undefined ? this.pendingSelect : this.selection;
    // While a reset is outstanding the selection belongs to the old world.
    if (select && this.resetting === 0) own.select = { ...select };
    const trail = this.pendingTrail !== undefined ? this.pendingTrail : this.followedId !== null;
    if (trail) own.trail = true;
    // Networks exist only in a sugarscape; the ring view needs Ring World's state with every
    // snapshot, and the valley's overlays the anasazi's.
    const networks = this.model === 'sugarscape' ? NETWORKS.filter((k) => this.overlays[k]) : [];
    if (networks.length > 0) own.networks = networks;
    if (this.model === 'ring') own.ring = true;
    if (this.model === 'anasazi') own.valley = true;
    return mergeWants([own, ...Array.from(this.providers, (p) => p(now))]);
  }

  private providersWant(now: number): boolean {
    return Array.from(this.providers).some((p) => Object.keys(p(now)).length > 0);
  }

  /** Sends a command with the current wants, lending a frame buffer when it changes what is drawn. */
  private async send(cmd: Command, withFrame = false): Promise<Result> {
    if (this.crashed) return { ok: false, fatal: this.crashed };
    const gen = this.selectionGen;
    const displayGen = this.displayGen;
    const reply = await this.transport.request(cmd, {
      wants: this.wants(performance.now()),
      frame: withFrame ? this.takeBuffer() : undefined,
    });
    for (const b of reply.spare ?? []) this.recycle(b);
    if (reply.result.ok && reply.result.snapshot) {
      this.sentUnder.set(reply.result.snapshot, gen);
      this.displayUnder.set(reply.result.snapshot, displayGen);
    }
    return reply.result;
  }

  /** A spare buffer of the grid's size, or a new one. */
  private takeBuffer(): ArrayBuffer {
    const size = this.width * this.height * 4;
    for (let b = this.spare.pop(); b; b = this.spare.pop()) if (b.byteLength === size) return b;
    return new ArrayBuffer(size);
  }

  /** Keeps a returned buffer for reuse while it still fits the grid. */
  private recycle(b: ArrayBuffer): void {
    if (b.byteLength > 0 && b.byteLength === this.width * this.height * 4 && this.spare.length < MAX_SPARE) this.spare.push(b);
  }

  /**
   * Takes in a snapshot: counters, the frame, and whatever extras it carries. Returns whether it
   * changed the display (the host clamped it to the config).
   */
  private adopt(s: WorldSnapshot): boolean {
    const { frame: _frame, ...rest } = s;
    this.last = rest;
    this.width = s.width;
    this.height = s.height;
    this.tick = s.tick;
    this.population = s.population;
    this.latest = s.latest;
    if (s.replayLeft !== undefined && s.replayLeft !== this.replayLeft) {
      this.replayLeft = s.replayLeft;
      this.replayMoved = true;
    }
    this.followedId = s.followed;
    this.followedLive = s.followedAlive;
    if (s.frame) {
      if (this.shown) this.recycle(this.shown);
      this.shown = s.frame;
    }
    if (s.config) {
      this.config = s.config;
      if (isSugar(s.config)) this.lastSugar = s.config;
    }
    // A world of another model has no ring; Ring World sends its state with every snapshot (and
    // the anasazi its valley's).
    this.ring = s.ring ?? (this.model === 'ring' ? this.ring : null);
    this.valley = s.valley ?? (this.model === 'anasazi' ? this.valley : null);
    this.finished = s.finished === true;
    // All null (nothing edited) is the same as none.
    if (s.editedLandscapes) this.landscapes = s.editedLandscapes.some((m) => m !== null) ? s.editedLandscapes : [];
    // A clamp of a display chosen before the latest `setDisplay` is stale: the host clamps that
    // newer choice itself when it gets to it, and says so in that reply if it has to.
    const clamped = s.display !== undefined && this.displayUnder.get(s) === this.displayGen;
    if (clamped && s.display) {
      this.colorMode = s.display.colorMode;
      this.layer = s.display.layer;
      this.overlays = { ...s.display.overlays };
    }
    // Checked here, not when the reply arrives: a reset's reply may be adopted in between.
    const gen = this.sentUnder.get(s);
    if (s.inspection && gen === this.selectionGen) {
      this.selectedUnder = gen;
      this.inspection = s.inspection;
      this.selection = { x: s.inspection.x, y: s.inspection.y, agentId: s.inspection.agentId };
    }
    this.trailCells = s.trail ?? (s.followed === null ? NO_CELLS : this.trailCells);
    if (s.networks) Object.assign(this.edges, s.networks);
    // The host sends every group afresh after a config change (its lines may have changed),
    // except when a civil schedule entry or ramp only moved values.
    if (s.config && !s.sameCharts) this.charts.clear();
    for (const [key, group] of Object.entries(s.charts ?? {})) this.charts.set(key, group);
    return clamped;
  }

  /**
   * Fires `'config'` if the snapshot carries a config (except for a new world, which fires
   * `'reset'` instead), then `events`, then `'replay'` if `replayLeft` moved and `'fork'` if the
   * session branched, then `'display'` if the host clamped the display, then `'snapshot'`. This is
   * the only place `'config'` fires: the host sends the config once, in whichever reply or post
   * comes next after it changed (a scheduled change at Max may arrive in a paint's or a click's
   * reply), so every command's reply is checked here.
   */
  private announce(s: WorldSnapshot, events: EngineEvent[], clamped: boolean): void {
    if (s.config && !events.includes('reset')) this.emit('config');
    for (const e of events) this.emit(e);
    if (this.replayMoved) {
      this.replayMoved = false;
      this.emit('replay');
    }
    if (s.forked) this.emit('fork');
    if (clamped) this.emit('display');
    this.emit('snapshot');
  }

  private accept(s: WorldSnapshot, events: EngineEvent[] = []): void {
    this.announce(s, events, this.adopt(s));
  }

  private async stepNow(n: number): Promise<void> {
    const result = await this.send({ type: 'step', n }, true);
    if (result.ok && result.snapshot) this.accept(result.snapshot, ['tick']);
    // A step to the cap, or one refused there; a step to the end year, or one after it.
    if (this.crashed) return;
    if (this.tick >= MAX_TICKS) this.full();
    else if (this.finished) this.finish();
  }

  /** The world is at `MAX_TICKS`: pause and say so. */
  private full(): void {
    if (this.running) this.setRunning(false);
    this.emit('full');
  }

  /** The world has run its course: pause and say so. */
  private finish(): void {
    if (this.running) this.setRunning(false);
    this.emit('finished');
  }

  /**
   * Runs `fn` after the writes queued before it and the frame loop's step have settled, holding
   * the loop until `fn` finishes (Decision 8): Max is stopped (and its acknowledgement awaited)
   * before `fn` runs, and restarts afterwards if still wanted (Decision 11).
   */
  private async quiet<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    const run = this.writes.then(async () => {
      await this.stopMax();
      await this.inFlight;
      return fn();
    });
    this.writes = run.catch(() => undefined);
    try {
      return await run;
    } finally {
      this.holds--;
      this.syncMax();
    }
  }

  /** Starts or stops the host's Max loop to match `running`, `speed`, holds and crashes. */
  private syncMax(): void {
    const want = this.running && this.speed === 'max' && this.holds === 0 && !this.crashed;
    if (want && !this.maxOn) {
      this.maxOn = true;
      void this.send({ type: 'run' }, true);
    } else if (!want && this.maxOn) {
      void this.stopMax();
    }
  }

  /** Stops the Max loop; resolves once the stop is acknowledged (so every earlier post has been handled). */
  private stopMax(): Promise<void> {
    if (!this.maxOn) return this.stopping ?? Promise.resolve();
    this.maxOn = false;
    // Captured by reference: Pause, Play, Pause in quick succession
    // starts a second stop while the first is still outstanding; the first's `.then` must not clear
    // the second's still-pending `this.stopping` out from under it.
    const stopping: Promise<void> = this.send({ type: 'stop' }).then((result) => {
      if (this.stopping === stopping) this.stopping = null;
      if (result.ok && result.snapshot) this.accept(result.snapshot, ['tick']);
    });
    this.stopping = stopping;
    return stopping;
  }

  /** A Max-speed snapshot: adopt it, then hand the displaced buffer straight back with fresh wants. */
  private onPost(s: WorldSnapshot): void {
    // Posts never go through send(), so they carry no sentUnder entry: adopt()'s selection
    // generation check would otherwise always drop s.inspection, leaving the Inspect panel and the
    // grid's selection frozen for as long as Max runs. Tag it with the
    // current generation before accepting — but only while no inspect/select is pending, so a post
    // that happens to be in flight under the same generation cannot win over that request's own
    // reply once it lands. (The trail and `followed`/`followedAlive` need no such fix: adopt() sets
    // them unconditionally from every snapshot, not gated by a generation.)
    if (this.pendingSelect === undefined) this.sentUnder.set(s, this.selectionGen);
    // Likewise for the display: a post made before the host took an outstanding `setDisplay`
    // clamps the older choice.
    if (this.displaysPending === 0) this.displayUnder.set(s, this.displayGen);
    this.accept(s, ['tick']);
    // The host's Max loop ends at the cap, or when the world is finished, with this post.
    if (s.tick >= MAX_TICKS) this.full();
    else if (s.finished) this.finish();
    if (this.maxOn) void this.send({ type: 'frame' }, true);
  }

  private keptLandscapes(config: ModelConfig): (Uint8Array | null)[] {
    const base = this.baseConfig;
    // Only a sugarscape has maps, and only a sugarscape's maps carry over.
    if (!isSugar(config) || !isSugar(base)) return [];
    const same = config.width === base.width && config.height === base.height && config.goods.length === base.goods.length;
    // A changed number of goods drops every painted map: Sim needs one entry per good (or none).
    return same
      ? this.landscapes
          .slice(0, config.goods.length)
          .map((l, i) => (JSON.stringify(config.goods[i]?.map) === JSON.stringify(base.goods[i]?.map) ? l : null))
      : [];
  }

  /**
   * Builds a new world. `log` is replayed into it (a session); `keepSetup` keeps the base config
   * and preset (a replay rewinds the same setup rather than choosing a new one — Decision 4).
   */
  private async rebuild(
    config: ModelConfig,
    seed: number,
    landscapes: (Uint8Array | null)[],
    opts: { presetId?: string; log?: LogEntry[]; keepSetup?: boolean } = {},
  ): Promise<FieldError[] | null> {
    // Replies to requests sent before this carry the old world's selection; a selection made
    // after it (a click while it is outstanding) is kept.
    const gen = ++this.selectionGen;
    this.resetting++;
    let result: Result;
    try {
      result = await this.send({ type: 'reset', config, seed, landscapes, log: opts.log }, true);
    } finally {
      this.resetting--;
    }
    if (!result.ok || !result.snapshot) return writeFailure(result);
    this.seed = seed;
    this.origin = { config, seed, landscapes };
    // Unless a selection made after the reset was sent has already arrived.
    if (this.selectedUnder < gen) {
      this.selection = null;
      this.inspection = null;
    }
    const clamped = this.adopt(result.snapshot);
    if (!opts.keepSetup) {
      this.baseConfig = structuredClone(this.config);
      this.presetId = opts.presetId ?? this.matchPreset();
    }
    this.announce(result.snapshot, ['reset'], clamped);
    // Panels' wants may have changed with the config (new chart lines, say).
    void this.refresh();
    return null;
  }

  /** The preset whose config equals the base config, if any. */
  private matchPreset(): string | null {
    const json = JSON.stringify(this.baseConfig);
    return this.presets.find((p) => JSON.stringify(p.config) === json)?.id ?? null;
  }

  private async inspectTarget(target: { x: number; y: number } | { agentId: number }): Promise<void> {
    // Replies to requests already sent carry the old selection.
    this.selectionGen++;
    // Reflect this target in wants() immediately, so a request sent before this
    // command's own reply lands does not carry the selection it is replacing.
    const seq = ++this.pendingSelectSeq;
    this.pendingSelect =
      'agentId' in target
        ? { x: this.selection?.x ?? 0, y: this.selection?.y ?? 0, agentId: target.agentId }
        : null; // site-only: which agent (if any) is there is not known until the reply
    const result = await this.send({ type: 'inspect', target });
    // Only clear it if a later inspect/select hasn't already replaced it with its own pending value.
    if (this.pendingSelectSeq === seq) this.pendingSelect = undefined;
    // An agent that has died gives no inspection: the selection stays, the snapshot is still news.
    if (result.ok && result.snapshot) this.accept(result.snapshot, result.snapshot.inspection ? ['select'] : []);
  }

  private async follow(id: number | null): Promise<void> {
    // Same reasoning as inspectTarget, for the trail flag.
    const seq = ++this.pendingTrailSeq;
    this.pendingTrail = id !== null;
    const result = await this.send({ type: 'follow', id });
    if (this.pendingTrailSeq === seq) this.pendingTrail = undefined;
    if (result.ok && result.snapshot) this.accept(result.snapshot, ['follow']);
  }

  private async edit(cmd: Command): Promise<FieldError[] | null> {
    const result = await this.send(cmd, true);
    if (!result.ok) return failure(result);
    if (result.snapshot) this.accept(result.snapshot, ['edit']);
    return null;
  }

  private async value(cmd: Command): Promise<string> {
    const result = await this.send(cmd);
    if (!result.ok) throw new Error((failure(result) ?? []).map((e) => `${e.field}: ${e.message}`).join('; '));
    return result.value ?? '';
  }

  /** The host died (Decision 7): stop for good and tell the page once. */
  private crash(message: string): void {
    if (this.crashed) return;
    this.crashed = message;
    this.maxOn = false;
    console.error(message);
    this.running = false;
    this.emit('run');
    this.emit('crash');
  }
}
