import type { CreditGraph } from './credit';
import { clampDisplay } from './layers';
import {
  chartKey,
  noOverlays,
  transfers,
  type ChartGroup,
  type DisplayState,
  type EditCommand,
  type HostMessage,
  type HostReply,
  type HostRequest,
  type LogEntry,
  type Overlay,
  type Result,
  type SelectQuery,
  type Selected,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { parseErrors, type Config, type DiseaseEntry, type Inspection, type Snapshot } from './types';

/** The part of the WASM `Sim` a host uses. The real class satisfies it; tests pass `FakeSim`. */
export interface SimLike {
  step(n: number): void;
  tick(): number;
  width(): number;
  height(): number;
  population(): number;
  render(colorMode: string, layer: string): number;
  frame_len(): number;
  stats_latest(): string;
  series_group(namesJson: string, max: number): Float64Array;
  lorenz(points: number): Float64Array;
  wealth_hist(bins: number): Float64Array;
  age_hist(bin: number): Float64Array;
  tag_hist(): Float64Array;
  good_wealth_hist(good: number, bins: number): Float64Array;
  lorenz_total(points: number): Float64Array;
  supply_demand(): Float64Array;
  inspect(x: number, y: number): string;
  locate(id: number): Uint32Array | undefined;
  follow(id: number): void;
  unfollow(): void;
  trail(): Uint32Array;
  followed(): number;
  paint_capacity(x: number, y: number, radius: number, value: number, good: number): void;
  set_landscape(good: number, capacities: Uint8Array): void;
  place_agent(x: number, y: number, overridesJson: string): number;
  remove_agent(x: number, y: number): void;
  infect(x: number, y: number, disease: number): boolean;
  vaccinate(x: number, y: number, radius: number, disease: number): number;
  set_config(json: string): void;
  export_config(): string;
  export_landscape(good: number): Uint8Array;
  landscape_edited(good: number): boolean;
  export_series_csv(): string;
  export_agents_csv(): string;
  networks(kind: string): Uint32Array;
  credit_graph(): string;
  disease_list(): string;
  fingerprint(): string;
  free(): void;
}

/** Makes worlds and reads their frames; `wasmSimModule` (sim-module.ts) is the real one. */
export interface SimModule {
  create(configJson: string, seed: number, landscapes: (Uint8Array | null)[]): SimLike;
  /** The `len` bytes `render` just wrote at `ptr` (a view into WASM memory: copy it at once). */
  frameBytes(ptr: number, len: number): Uint8Array;
}

/** A chart group is sent again once this long has passed… */
export const CHART_MS = 250;
/** …or once its history has grown by more than this fraction. */
export const CHART_GROWTH = 0.01;

/** Max speed: a batch steps for about this long… */
export const BATCH_MS = 16;
/** …and a snapshot is posted about this often (while the host holds a free buffer). */
export const POST_MS = 33;

/** Animation III-1's age histogram bins are this many ticks wide. */
export const AGE_BIN = 5;

/** The edit log holds at most this many entries; past it, edits still apply but are not recorded (Decision 1). */
export const LOG_CAP = 50_000;

const NO_WORLD = JSON.stringify([{ field: 'world', message: 'no world yet' }]);

/**
 * An extra the world cannot give — a chart line or a selected site that a config change just
 * removed — is left out of the snapshot instead of failing the command. Only field errors (thrown
 * strings) are dropped; anything else is a panic and propagates.
 */
function optional<T>(get: () => T): T | undefined {
  try {
    return get();
  } catch (e) {
    if (typeof e === 'string') return undefined;
    throw e;
  }
}

/**
 * Holds the simulation and answers one command at a time, in arrival order. It runs in the
 * simulation worker (sim-worker.ts) or, in tests and as a fallback, on the page (InlineTransport).
 */
export class SimHost {
  private sim: SimLike | null = null;
  private config: Config | null = null;
  private display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() };
  /** The next snapshot carries the config: after init, reset, setConfig or a scheduled change. */
  private configDue = false;
  /** The next snapshot carries the edited landscapes: after init, reset, setConfig, paint or import. */
  private landscapesDue = false;
  /** Set by a panic: every later command is answered with it. */
  private dead: string | null = null;
  /** When, and at which history length, each chart group was last sent. */
  private sent = new Map<string, { at: number; length: number }>();
  /** Max speed: the wants to answer with, the buffers to post frames in, when it last posted; null when not running. */
  private max: { wants: Wants; pool: ArrayBuffer[]; posted: number } | null = null;
  /** Edits applied to this world since it was built, with their ticks: the session's log (Decision 1). */
  private log: LogEntry[] = [];
  /** Set once the log reached LOG_CAP: later edits apply but are not recorded. */
  private logFull = false;
  /** A session's entries still to replay: `pending[cursor…]`, in log order (Decision 2). */
  private pending: LogEntry[] = [];
  private cursor = 0;
  /** The `replayLeft` last sent; -1 sends it with the next snapshot. */
  private replaySent = -1;
  /** The next snapshot says a page edit dropped the pending entries (Decision 3). */
  private forkDue = false;

  constructor(
    private module: SimModule,
    private now: () => number = () => performance.now(),
  ) {}

  get running(): boolean {
    return this.max !== null;
  }

  handle(req: HostRequest): HostReply {
    if (this.max && req.wants) this.max.wants = req.wants;
    const spare: ArrayBuffer[] = [];
    let result: Result;
    if (this.dead) {
      result = { ok: false, fatal: this.dead };
    } else {
      try {
        result = this.apply(req, spare);
      } catch (e) {
        // The core throws field errors as JSON strings; anything else is a panic or a bug.
        result = typeof e === 'string' ? { ok: false, errors: parseErrors(e) } : { ok: false, fatal: this.fail(e) };
      }
    }
    // A lent buffer the snapshot did not use and the Max loop did not keep goes straight back.
    const frame = req.frame;
    const used = result.ok && result.snapshot?.frame === frame;
    if (frame && !used && !this.max?.pool.includes(frame)) spare.push(frame);
    return spare.length > 0 ? { id: req.id, result, spare } : { id: req.id, result };
  }

  /** Stops the host for good; returns the message every later command gets. */
  fail(e: unknown): string {
    this.max = null;
    this.dead = `The simulation stopped: ${e instanceof Error ? e.message : String(e)}`;
    return this.dead;
  }

  /**
   * One Max-speed batch (Decision 11): steps for about `BATCH_MS`, but never past the next
   * post deadline while a buffer is free to post into, so posts land close to every `POST_MS`
   * instead of drifting toward 2 × BATCH_MS when POST_MS falls between two batch-lengths. With no
   * buffer pooled there is nothing to post regardless, so the deadline is not applied — capping it
   * anyway would leave `posted` unmoved while every batch (and its deferred round trip) stepped
   * just one tick, throttling Max to roughly the scheduler's minimum delay. Returns a snapshot to
   * post, or null.
   */
  batch(): WorldSnapshot | null {
    const max = this.max;
    const sim = this.sim;
    if (!max || !sim || this.dead) return null;
    const start = this.now();
    const cap = max.pool.length > 0 ? Math.min(start + BATCH_MS, max.posted + POST_MS) : start + BATCH_MS;
    // One tick at a time, applying any edit due at each (Decision 2).
    do this.advance(sim, 1);
    while (this.now() < cap);
    const now = this.now();
    const frame = now - max.posted >= POST_MS ? max.pool.pop() : undefined;
    if (!frame) return null;
    max.posted = now;
    return this.snapshot(sim, max.wants, frame);
  }

  private apply(req: HostRequest, spare: ArrayBuffer[]): Result {
    const { cmd, frame } = req;
    const wants = req.wants ?? {};
    if (cmd.type === 'ready') return { ok: true };
    // init, reset and setConfig keep Max running on the (possibly new) world if it was running;
    // in practice the engine quiesces (sends `stop`) before any of them, so this doesn't happen.
    if (cmd.type === 'init' || cmd.type === 'reset') {
      // Built before the old world is freed: a bad config keeps the world (and its log).
      const next = this.module.create(JSON.stringify(cmd.config), cmd.seed, cmd.landscapes);
      this.sim?.free();
      this.sim = next;
      if (cmd.type === 'init') this.display = cmd.display;
      this.configDue = true;
      this.landscapesDue = true;
      // A new world starts a new log; a session's log is replayed into it (Decision 2).
      this.log = [];
      this.logFull = false;
      this.pending = cmd.log ?? [];
      this.cursor = 0;
      this.replaySent = -1;
      this.forkDue = false;
      this.replayDue(next);
      // The engine clears its selection on a new world.
      return this.reply(next, { ...wants, select: undefined }, frame);
    }
    const sim = this.sim;
    if (!sim) throw NO_WORLD;
    switch (cmd.type) {
      case 'setConfig':
      case 'paint':
      case 'importLandscape':
      case 'place':
      case 'erase':
      case 'infect':
      case 'vaccinate':
        this.edit(sim, cmd);
        // Only an edit that succeeded branches a replay (Decision 3).
        this.fork();
        return this.reply(sim, wants, frame);
      case 'step':
        this.advance(sim, cmd.n);
        return this.reply(sim, wants, frame);
      case 'refresh':
        // The only command exempt from the charts throttle: it is already paced client-side by
        // REFRESH_MS (Engine.pump), and it is the only path the paused catch-up needs. Every other
        // command — including drag-driven ones like paint, which carry no debounce of their own —
        // stays throttled, or a drag would resend full chart groups on nearly every pointermove.
        return this.reply(sim, wants, frame, undefined, false);
      case 'setDisplay':
        this.display = cmd.display;
        return this.reply(sim, wants, frame);
      case 'follow': {
        if (cmd.id === null) sim.unfollow();
        else sim.follow(cmd.id);
        // At send time the engine's own wants still describe the old follow state (it only
        // learns the new one from this reply), so while Max is running the loop's own copy is
        // patched here too — or its posts would keep reporting the trail as it was before this.
        if (this.max) this.max.wants = { ...this.max.wants, trail: cmd.id !== null };
        return this.reply(sim, { ...wants, trail: cmd.id !== null }, frame);
      }
      case 'inspect': {
        const { target } = cmd;
        const at = 'agentId' in target ? sim.locate(target.agentId) : Uint32Array.of(target.x, target.y);
        const selected = at ? this.selectAt(sim, at[0], at[1]) : null;
        // Same reasoning as `follow`: the new selection is this command's own result, not
        // yet reflected in `wants.select`, so the loop's copy is patched from it directly.
        if (this.max) {
          this.max.wants = {
            ...this.max.wants,
            select: selected ? { x: selected.x, y: selected.y, agentId: selected.agentId } : undefined,
          };
        }
        return this.reply(sim, wants, frame, selected);
      }
      case 'seriesCsv':
        return { ok: true, value: sim.export_series_csv() };
      case 'agentsCsv':
        return { ok: true, value: sim.export_agents_csv() };
      case 'fingerprint':
        return { ok: true, value: sim.fingerprint() };
      case 'session':
        // Applied entries, then those still to replay: a link made mid-replay carries the whole session.
        return {
          ok: true,
          session: { log: [...this.log, ...this.pending.slice(this.cursor)], full: this.logFull, tick: sim.tick() },
        };
      case 'endReplay':
        this.pending = [];
        this.cursor = 0;
        return this.reply(sim, wants, frame);
      case 'run':
        // A second `run` while already running keeps the pooled buffers (and adds this one, if
        // any) instead of replacing the pool and losing them; `handle` has already applied this
        // request's `wants` to the loop, same as any other command.
        if (this.max) {
          if (frame) this.max.pool.push(frame);
        } else {
          this.max = { wants, pool: frame ? [frame] : [], posted: this.now() };
        }
        return { ok: true };
      case 'frame':
        if (this.max && frame) this.max.pool.push(frame);
        return { ok: true };
      case 'stop': {
        const max = this.max;
        this.max = null;
        if (!max) return this.reply(sim, wants);
        // A pooled buffer is preferred; with none pooled, the buffer this `stop` itself just lent
        // (if any) is used instead of being left unrendered and handed straight back as spare.
        const last = max.pool.pop() ?? frame;
        spare.push(...max.pool);
        return this.reply(sim, max.wants, last);
      }
    }
  }

  /** Applies a world-changing command and records it with the tick (Decision 1); throws the core's field errors. */
  private edit(sim: SimLike, cmd: EditCommand): void {
    switch (cmd.type) {
      case 'setConfig':
        sim.set_config(JSON.stringify(cmd.config));
        this.configDue = true;
        this.landscapesDue = true;
        break;
      case 'paint':
        sim.paint_capacity(cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good);
        this.landscapesDue = true;
        break;
      case 'importLandscape':
        sim.set_landscape(cmd.good, cmd.capacities);
        this.landscapesDue = true;
        break;
      case 'place':
        sim.place_agent(cmd.x, cmd.y, JSON.stringify(cmd.overrides));
        break;
      case 'erase':
        sim.remove_agent(cmd.x, cmd.y);
        break;
      case 'infect':
        sim.infect(cmd.x, cmd.y, cmd.disease);
        break;
      case 'vaccinate':
        sim.vaccinate(cmd.x, cmd.y, cmd.radius, cmd.disease);
        break;
    }
    if (this.log.length < LOG_CAP) this.log.push({ tick: sim.tick(), cmd });
    else this.logFull = true;
  }

  /** A page edit while entries are pending drops them: the session branches here (Decision 3). */
  private fork(): void {
    if (this.cursor >= this.pending.length) return;
    this.pending = [];
    this.cursor = 0;
    this.forkDue = true;
  }

  /** Applies every pending entry whose tick has been reached, in log order (Decision 2). */
  private replayDue(sim: SimLike): void {
    const tick = sim.tick();
    while (this.cursor < this.pending.length && this.pending[this.cursor].tick <= tick) {
      const { cmd } = this.pending[this.cursor++];
      try {
        this.edit(sim, cmd);
      } catch (e) {
        // An entry the world rejects (only a hand-made link has one) is skipped; a panic is fatal.
        if (typeof e !== 'string') throw e;
      }
    }
    if (this.pending.length > 0 && this.cursor >= this.pending.length) {
      this.pending = [];
      this.cursor = 0;
    }
  }

  /**
   * Steps `n` ticks. While entries are pending it stops at each entry's tick and applies it
   * (`step(k)` is the same world as k single steps, so it steps straight there).
   */
  private advance(sim: SimLike, n: number): void {
    let left = n;
    while (left > 0) {
      const next = this.pending[this.cursor]?.tick;
      const k = next === undefined ? left : Math.min(left, Math.max(1, next - sim.tick()));
      const from = sim.tick();
      sim.step(k);
      this.fired(from, sim.tick());
      this.replayDue(sim);
      left -= k;
    }
  }

  /** An entry at tick t fires when the step from t to t + 1 starts. */
  private fired(from: number, to: number): void {
    if (this.config?.schedule.some((c) => c.tick >= from && c.tick < to)) this.configDue = true;
  }

  /**
   * `throttleCharts` bounds chart resends to at most 4/s (or 1 % of history): true (the default)
   * for every command except `refresh` (see its case above), so an undebounced drag (paint, erase,
   * vaccinate, …) cannot resend full chart groups on every pointermove.
   */
  private reply(sim: SimLike, wants: Wants, frame?: ArrayBuffer, selected?: Selected | null, throttleCharts = true): Result {
    return { ok: true, snapshot: this.snapshot(sim, wants, frame, selected, throttleCharts) };
  }

  /** `selected` is an `inspect` command's answer; it replaces `wants.select`. */
  private snapshot(
    sim: SimLike,
    wants: Wants,
    frame: ArrayBuffer | undefined,
    selected?: Selected | null,
    throttleCharts = true,
  ): WorldSnapshot {
    const id = sim.followed();
    const s: WorldSnapshot = {
      width: sim.width(),
      height: sim.height(),
      tick: sim.tick(),
      population: sim.population(),
      latest: JSON.parse(sim.stats_latest()) as Snapshot,
      followed: id < 0 ? null : id,
      followedAlive: id >= 0 && sim.locate(id) !== undefined,
    };
    const left = this.pending.length - this.cursor;
    if (left !== this.replaySent) {
      s.replayLeft = left;
      this.replaySent = left;
    }
    if (this.forkDue) {
      s.forked = true;
      this.forkDue = false;
    }
    let config = this.config;
    if (this.configDue || !config) {
      config = JSON.parse(sim.export_config()) as Config;
      this.config = config;
      s.config = config;
      this.configDue = false;
      // Chart lines follow the config: send every group afresh.
      this.sent.clear();
    }
    if (this.landscapesDue) {
      s.editedLandscapes = config.goods.map((_, i) => (sim.landscape_edited(i) ? sim.export_landscape(i) : null));
      this.landscapesDue = false;
    }
    const display = clampDisplay(this.display, config);
    if (display !== this.display) {
      this.display = display;
      s.display = display;
    }
    if (frame) s.frame = this.render(sim, frame);
    const select = wants.select;
    if (selected !== undefined) s.inspection = selected;
    else if (select) s.inspection = optional(() => this.track(sim, select));
    if (wants.trail) s.trail = sim.trail();
    if (wants.networks) {
      const networks: Partial<Record<Overlay, Uint32Array>> = {};
      for (const kind of wants.networks) networks[kind] = sim.networks(kind);
      s.networks = networks;
    }
    if (wants.charts) {
      const charts = this.charts(sim, wants.charts.groups, wants.charts.max, throttleCharts);
      if (charts) s.charts = charts;
    }
    if (wants.lorenz) s.lorenz = sim.lorenz(101);
    if (wants.wealthHist) s.wealthHist = sim.wealth_hist(20);
    if (wants.ageHist) s.ageHist = sim.age_hist(AGE_BIN);
    if (wants.tagHist) s.tagHist = sim.tag_hist();
    if (wants.lorenzTotal) s.lorenzTotal = sim.lorenz_total(101);
    if (wants.goodWealthHists) s.goodWealthHists = config.goods.map((_, i) => sim.good_wealth_hist(i, 20));
    if (wants.supplyDemand) s.supplyDemand = sim.supply_demand();
    if (wants.creditGraph) s.creditGraph = JSON.parse(sim.credit_graph()) as CreditGraph;
    if (wants.diseaseList) s.diseaseList = JSON.parse(sim.disease_list()) as DiseaseEntry[];
    return s;
  }

  /** Copies the rendered frame into the lent buffer, or a new one when the grid size changed. */
  private render(sim: SimLike, frame: ArrayBuffer): ArrayBuffer {
    const ptr = sim.render(this.display.colorMode, this.display.layer);
    const bytes = this.module.frameBytes(ptr, sim.frame_len());
    const out = frame.byteLength === bytes.length ? frame : new ArrayBuffer(bytes.length);
    new Uint8Array(out).set(bytes);
    return out;
  }

  private selectAt(sim: SimLike, x: number, y: number): Selected {
    const view = JSON.parse(sim.inspect(x, y)) as Inspection;
    const agentId = view.agent?.id ?? null;
    return { x, y, agentId, alive: agentId !== null, view };
  }

  /** The selection now: a selected agent's current site while it lives, else the selected site. */
  private track(sim: SimLike, q: SelectQuery): Selected {
    const at = q.agentId === null ? undefined : sim.locate(q.agentId);
    const x = at ? at[0] : q.x;
    const y = at ? at[1] : q.y;
    return { x, y, agentId: q.agentId, alive: at !== undefined, view: JSON.parse(sim.inspect(x, y)) as Inspection };
  }

  /**
   * The groups with news (Decision 4); undefined when none has any. `throttle` applies the 4/s,
   * 1 % bandwidth cap: without it, only a group truly unchanged since the last send is skipped.
   */
  private charts(sim: SimLike, groups: string[][], max: number, throttle: boolean): Record<string, ChartGroup> | undefined {
    const length = sim.tick() + 1;
    const now = this.now();
    let out: Record<string, ChartGroup> | undefined;
    for (const names of groups) {
      const key = chartKey(names);
      const last = this.sent.get(key);
      const unchanged = last && length === last.length;
      const withinBudget = throttle && last && now - last.at < CHART_MS && length <= last.length * (1 + CHART_GROWTH);
      if (unchanged || withinBudget) continue;
      const flat = optional(() => sim.series_group(JSON.stringify(names), max));
      if (!flat) continue;
      const n = flat[0];
      (out ??= {})[key] = {
        ticks: flat.slice(1, 1 + n),
        columns: names.map((_, k) => flat.slice(1 + n * (k + 1), 1 + n * (k + 2))),
      };
      this.sent.set(key, { at: now, length });
    }
    return out;
  }
}

/**
 * Wires a host to a message channel: each request is answered in order, its buffers transferred;
 * while Max runs, batches are scheduled with `defer` so queued requests are handled between them.
 */
export function serve(
  host: SimHost,
  send: (message: HostMessage, transfer: Transferable[]) => void,
  defer: (fn: () => void) => void,
): (req: HostRequest) => void {
  let looping = false;
  const loop = (): void => {
    if (!host.running) {
      looping = false;
      return;
    }
    let message: HostMessage | null = null;
    try {
      const post = host.batch();
      if (post) message = { id: null, post };
    } catch (e) {
      message = { id: null, fatal: host.fail(e) };
    }
    if (message) send(message, transfers(message));
    defer(loop);
  };
  return (req) => {
    const reply = host.handle(req);
    send(reply, transfers(reply));
    if (host.running && !looping) {
      looping = true;
      defer(loop);
    }
  };
}

/** Runs `fn` as a new task without `setTimeout`'s 4 ms clamp; messages already queued run first or between. */
export function channelDefer(): (fn: () => void) => void {
  const channel = new MessageChannel();
  const queue: (() => void)[] = [];
  channel.port1.onmessage = () => queue.shift()?.();
  return (fn) => {
    queue.push(fn);
    channel.port2.postMessage(null);
  };
}
