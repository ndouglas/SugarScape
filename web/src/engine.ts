import {
  chartKey,
  mergeWants,
  OVERLAYS,
  type ChartGroup,
  type Command,
  type DisplayState,
  type Overlay,
  type PlaceOverrides,
  type Result,
  type Selected,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport, startWorker, type Transport } from './transport';
import type { ColorMode, Config, FieldError, Layer, Preset, Snapshot } from './types';
import init, { presets_json } from './wasm-pkg/sugarscape.js';

export type { Overlay, PlaceOverrides } from './protocol';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit' | 'follow' | 'snapshot' | 'crash';

export interface Selection { x: number; y: number; agentId: number | null }

export interface InitialState { config: Config; seed: number; landscapes?: (Uint8Array | null)[] }

/** What an engine runs on: the presets and a transport to a SimHost (tests pass fakes). */
export interface EngineDeps { presets: Preset[]; transport: Transport }

/**
 * What a panel needs in the next snapshot. It is called before every request, so it must not
 * change state: a panel records what it received when the snapshot arrives.
 */
export type WantsProvider = (now: number) => Wants;

/** While paused, extras a panel wants are fetched at most this often. */
const REFRESH_MS = 250;
/** Frame buffers kept for reuse besides the one on screen. */
const MAX_SPARE = 4;
const NO_CELLS: Uint32Array = new Uint32Array(0);

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

/**
 * The simulation runs in a worker; the page keeps its own WASM instance for presets, Experiments
 * and, if a module worker cannot start, the simulation itself (Decision 10).
 */
async function defaultDeps(): Promise<EngineDeps> {
  const wasm = await init();
  const presets = JSON.parse(presets_json()) as Preset[];
  let transport: Transport;
  try {
    transport = await startWorker();
  } catch (e) {
    console.warn('The simulation worker could not start; simulating on the page instead.', e);
    transport = new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));
  }
  return { presets, transport };
}

/**
 * The playground's view of the simulation, which runs behind a transport: reads are served from
 * the latest snapshot, writes return promises, and events fire when replies arrive.
 */
export class Engine {
  running = false;
  stepsPerFrame = 1;
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'resource:0';
  overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };
  selection: Selection | null = null;
  /** The selected site (and agent) as of the latest snapshot. */
  inspection: Selected | null = null;
  presetId: string | null = null;
  /**
   * The setup as chosen (preset, share link, reset): what a reset rebuilds and a share link
   * carries; preset matching and `isModified()` compare against it.
   */
  baseConfig!: Config;
  /**
   * The live config: what the running world uses now, including scheduled changes that have
   * fired. The Rules panel shows this one.
   */
  config!: Config;
  tick = 0;
  population = 0;
  latest: Snapshot | null = null;
  /**
   * The latest snapshot; its extras are there only when something wanted them. Its frame is left
   * out: that buffer is lent back to the host later (read frames through `frame()`).
   */
  last: WorldSnapshot | null = null;
  /** Why the simulation stopped for good, or null. */
  crashed: string | null = null;
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
  /** The generation of the request whose reply last set `selection`. */
  private selectedUnder = -1;
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
  /**
   * The quiet writes (and Steps), one after another: each starts only once the one before has
   * settled, so it is built from the state that write left (two quick edits both take effect).
   */
  private writes: Promise<unknown> = Promise.resolve();
  /** Resets sent and not yet answered: requests sent meanwhile carry no selection (PF7). */
  private resetting = 0;
  private lastRefresh = -Infinity;

  private constructor(
    private transport: Transport,
    readonly presets: Preset[],
    public seed: number,
  ) {
    transport.onFatal = (message) => this.crash(message);
  }

  /** Throws the core's errors (as an Error message) if `initial` is invalid. */
  static async create(initial?: InitialState, deps?: EngineDeps): Promise<Engine> {
    const { presets, transport } = deps ?? (await defaultDeps());
    const fallback = presets.find((p) => p.id === 'ii-2-unit') ?? presets[0];
    const engine = new Engine(transport, presets, initial?.seed ?? randomSeed());
    const config = initial?.config ?? structuredClone(fallback.config);
    const landscapes = initial?.landscapes ?? [];
    const result = await engine.send({ type: 'init', config, seed: engine.seed, landscapes, display: engine.displayState() }, true);
    if (!result.ok || !result.snapshot) {
      transport.close();
      throw new Error((failure(result) ?? []).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    engine.adopt(result.snapshot);
    engine.baseConfig = structuredClone(engine.config);
    engine.presetId = engine.matchPreset();
    return engine;
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
    if (this.running) next = this.stepNow(this.stepsPerFrame);
    else if (now - this.lastRefresh >= REFRESH_MS && this.providersWant(now)) next = this.refresh();
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
  reset(config?: Config, seed?: number): Promise<FieldError[] | null> {
    // The defaults are read when the reset runs, after any write queued before it.
    return this.quiet(() => {
      const next = config ?? this.baseConfig;
      return this.rebuild(next, seed ?? this.seed, this.keptLandscapes(next));
    });
  }

  /**
   * A reset to the base config as changed by `mutate` (a reset-requiring rule edit), built when it
   * runs so that an earlier change still in flight is kept.
   */
  resetWith(mutate: (c: Config) => void, seed?: number): Promise<FieldError[] | null> {
    return this.quiet(() => {
      const next = structuredClone(this.baseConfig);
      mutate(next);
      return this.rebuild(next, seed ?? this.seed, this.keptLandscapes(next));
    });
  }

  /**
   * Applies a rule/parameter change to the running world: `mutate` edits a copy of the live
   * config for the world and, on success, a copy of the base config too, so scheduled changes
   * that already fired are not undone.
   */
  applyConfig(mutate: (c: Config) => void): Promise<FieldError[] | null> {
    return this.quiet(async () => {
      const next = structuredClone(this.config);
      mutate(next);
      const result = await this.send({ type: 'setConfig', config: next }, true);
      if (!result.ok || !result.snapshot) return writeFailure(result);
      const base = structuredClone(this.baseConfig);
      mutate(base);
      this.baseConfig = base;
      this.accept(result.snapshot, ['config']);
      void this.refresh();
      return null;
    });
  }

  async loadPreset(id: string): Promise<FieldError[] | null> {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    return this.quiet(() => this.rebuild(structuredClone(preset.config), this.seed, [], id));
  }

  /** True when the base config differs from the last chosen preset or a landscape is custom. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || this.landscapes.some((l) => l !== null) || JSON.stringify(preset.config) !== JSON.stringify(this.baseConfig);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  /** Steps `n` ticks, after the frame loop's step and any queued write (and before later writes). */
  advance(n: number = this.stepsPerFrame): Promise<void> {
    return this.quiet(() => this.stepNow(n));
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<Record<Overlay, boolean>> }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    if (d.overlays) this.overlays = { ...this.overlays, ...d.overlays };
    this.emit('display');
    void this.send({ type: 'setDisplay', display: this.displayState() }, true).then((result) => {
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

  private displayState(): DisplayState {
    return { colorMode: this.colorMode, layer: this.layer, overlays: { ...this.overlays } };
  }

  /** The engine's own wants (Decision 9) merged with every provider's. */
  private wants(now: number): Wants {
    const own: Wants = {};
    // While a reset is outstanding the selection belongs to the old world.
    if (this.selection && this.resetting === 0) own.select = { ...this.selection };
    if (this.followedId !== null) own.trail = true;
    const networks = OVERLAYS.filter((k) => this.overlays[k]);
    if (networks.length > 0) own.networks = networks;
    return mergeWants([own, ...Array.from(this.providers, (p) => p(now))]);
  }

  private providersWant(now: number): boolean {
    return Array.from(this.providers).some((p) => Object.keys(p(now)).length > 0);
  }

  /** Sends a command with the current wants, lending a frame buffer when it changes what is drawn. */
  private async send(cmd: Command, withFrame = false): Promise<Result> {
    if (this.crashed) return { ok: false, fatal: this.crashed };
    const gen = this.selectionGen;
    const reply = await this.transport.request(cmd, {
      wants: this.wants(performance.now()),
      frame: withFrame ? this.takeBuffer() : undefined,
    });
    for (const b of reply.spare ?? []) this.recycle(b);
    if (reply.result.ok && reply.result.snapshot) this.sentUnder.set(reply.result.snapshot, gen);
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

  /** Takes in a snapshot: counters, the frame, and whatever extras it carries. */
  private adopt(s: WorldSnapshot): void {
    const { frame: _frame, ...rest } = s;
    this.last = rest;
    this.width = s.width;
    this.height = s.height;
    this.tick = s.tick;
    this.population = s.population;
    this.latest = s.latest;
    this.followedId = s.followed;
    this.followedLive = s.followedAlive;
    if (s.frame) {
      if (this.shown) this.recycle(this.shown);
      this.shown = s.frame;
    }
    if (s.config) this.config = s.config;
    // All null (nothing edited) is the same as none.
    if (s.editedLandscapes) this.landscapes = s.editedLandscapes.some((m) => m !== null) ? s.editedLandscapes : [];
    if (s.display) {
      this.colorMode = s.display.colorMode;
      this.layer = s.display.layer;
      this.overlays = { ...s.display.overlays };
    }
    // Checked here, not when the reply arrives: a reset's reply may be adopted in between (PF7).
    const gen = this.sentUnder.get(s);
    if (s.inspection && gen === this.selectionGen) {
      this.selectedUnder = gen;
      this.inspection = s.inspection;
      this.selection = { x: s.inspection.x, y: s.inspection.y, agentId: s.inspection.agentId };
    }
    this.trailCells = s.trail ?? (s.followed === null ? NO_CELLS : this.trailCells);
    if (s.networks) Object.assign(this.edges, s.networks);
    // The host sends every group afresh after a config change (its lines may have changed).
    if (s.config) this.charts.clear();
    for (const [key, group] of Object.entries(s.charts ?? {})) this.charts.set(key, group);
  }

  /** Fires `events`, then `'display'` if the host clamped the display, then `'snapshot'`. */
  private announce(s: WorldSnapshot, events: EngineEvent[]): void {
    for (const e of events) this.emit(e);
    if (s.display) this.emit('display');
    this.emit('snapshot');
  }

  private accept(s: WorldSnapshot, events: EngineEvent[] = []): void {
    this.adopt(s);
    this.announce(s, events);
  }

  private async stepNow(n: number): Promise<void> {
    const result = await this.send({ type: 'step', n }, true);
    if (!result.ok || !result.snapshot) return;
    const events: EngineEvent[] = result.snapshot.config ? ['config', 'tick'] : ['tick'];
    this.accept(result.snapshot, events);
  }

  /**
   * Runs `fn` after the writes queued before it and the frame loop's step have settled, holding
   * the loop until `fn` finishes (Decision 8).
   */
  private async quiet<T>(fn: () => Promise<T>): Promise<T> {
    this.holds++;
    const run = this.writes.then(async () => {
      while (this.inFlight) await this.inFlight;
      return fn();
    });
    this.writes = run.catch(() => undefined);
    try {
      return await run;
    } finally {
      this.holds--;
    }
  }

  private keptLandscapes(config: Config): (Uint8Array | null)[] {
    const base = this.baseConfig;
    const same = config.width === base.width && config.height === base.height && config.goods.length === base.goods.length;
    // A changed number of goods drops every painted map: Sim needs one entry per good (or none).
    return same
      ? this.landscapes
          .slice(0, config.goods.length)
          .map((l, i) => (JSON.stringify(config.goods[i]?.map) === JSON.stringify(base.goods[i]?.map) ? l : null))
      : [];
  }

  private async rebuild(
    config: Config,
    seed: number,
    landscapes: (Uint8Array | null)[],
    presetId?: string,
  ): Promise<FieldError[] | null> {
    // Replies to requests sent before this carry the old world's selection; a selection made
    // after it (a click while it is outstanding) is kept (PF7).
    const gen = ++this.selectionGen;
    this.resetting++;
    let result: Result;
    try {
      result = await this.send({ type: 'reset', config, seed, landscapes }, true);
    } finally {
      this.resetting--;
    }
    if (!result.ok || !result.snapshot) return writeFailure(result);
    this.seed = seed;
    // Unless a selection made after the reset was sent has already arrived.
    if (this.selectedUnder < gen) {
      this.selection = null;
      this.inspection = null;
    }
    this.adopt(result.snapshot);
    this.baseConfig = structuredClone(this.config);
    this.presetId = presetId ?? this.matchPreset();
    this.announce(result.snapshot, ['reset']);
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
    const result = await this.send({ type: 'inspect', target });
    // An agent that has died gives no inspection: the selection stays, the snapshot is still news.
    if (result.ok && result.snapshot) this.accept(result.snapshot, result.snapshot.inspection ? ['select'] : []);
  }

  private async follow(id: number | null): Promise<void> {
    const result = await this.send({ type: 'follow', id });
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
    console.error(message);
    this.running = false;
    this.emit('run');
    this.emit('crash');
  }
}
