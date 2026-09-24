import type { CreditGraph } from './credit';
import { clampDisplay } from './layers';
import {
  chartKey,
  type ChartGroup,
  type DisplayState,
  type HostReply,
  type HostRequest,
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
  private display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };
  /** The next snapshot carries the config: after init, reset, setConfig or a scheduled change. */
  private configDue = false;
  /** The next snapshot carries the edited landscapes: after init, reset, setConfig, paint or import. */
  private landscapesDue = false;
  /** Set by a panic: every later command is answered with it. */
  private dead: string | null = null;
  /** When, and at which history length, each chart group was last sent. */
  private sent = new Map<string, { at: number; length: number }>();

  constructor(
    private module: SimModule,
    private now: () => number = () => performance.now(),
  ) {}

  handle(req: HostRequest): HostReply {
    let result: Result;
    if (this.dead) {
      result = { ok: false, fatal: this.dead };
    } else {
      try {
        result = this.apply(req);
      } catch (e) {
        // The core throws field errors as JSON strings; anything else is a panic or a bug.
        result = typeof e === 'string' ? { ok: false, errors: parseErrors(e) } : { ok: false, fatal: this.fail(e) };
      }
    }
    // A lent buffer the snapshot did not use goes straight back.
    const used = result.ok && result.snapshot?.frame === req.frame;
    return req.frame && !used ? { id: req.id, result, spare: [req.frame] } : { id: req.id, result };
  }

  /** Stops the host for good; returns the message every later command gets. */
  fail(e: unknown): string {
    this.dead = `The simulation stopped: ${e instanceof Error ? e.message : String(e)}`;
    return this.dead;
  }

  private apply(req: HostRequest): Result {
    const { cmd, frame } = req;
    const wants = req.wants ?? {};
    if (cmd.type === 'ready') return { ok: true };
    if (cmd.type === 'init' || cmd.type === 'reset') {
      // Built before the old world is freed: a bad config keeps the world.
      const next = this.module.create(JSON.stringify(cmd.config), cmd.seed, cmd.landscapes);
      this.sim?.free();
      this.sim = next;
      if (cmd.type === 'init') this.display = cmd.display;
      this.configDue = true;
      this.landscapesDue = true;
      // The engine clears its selection on a new world.
      return this.reply(next, { ...wants, select: undefined }, frame);
    }
    const sim = this.sim;
    if (!sim) throw NO_WORLD;
    switch (cmd.type) {
      case 'setConfig':
        sim.set_config(JSON.stringify(cmd.config));
        this.configDue = true;
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'step': {
        const from = sim.tick();
        sim.step(cmd.n);
        this.fired(from, sim.tick());
        return this.reply(sim, wants, frame);
      }
      case 'refresh':
        return this.reply(sim, wants, frame);
      case 'setDisplay':
        this.display = cmd.display;
        return this.reply(sim, wants, frame);
      case 'paint':
        sim.paint_capacity(cmd.x, cmd.y, cmd.radius, cmd.value, cmd.good);
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'importLandscape':
        sim.set_landscape(cmd.good, cmd.capacities);
        this.landscapesDue = true;
        return this.reply(sim, wants, frame);
      case 'place':
        sim.place_agent(cmd.x, cmd.y, JSON.stringify(cmd.overrides));
        return this.reply(sim, wants, frame);
      case 'erase':
        sim.remove_agent(cmd.x, cmd.y);
        return this.reply(sim, wants, frame);
      case 'infect':
        sim.infect(cmd.x, cmd.y, cmd.disease);
        return this.reply(sim, wants, frame);
      case 'vaccinate':
        sim.vaccinate(cmd.x, cmd.y, cmd.radius, cmd.disease);
        return this.reply(sim, wants, frame);
      case 'follow':
        if (cmd.id === null) sim.unfollow();
        else sim.follow(cmd.id);
        return this.reply(sim, { ...wants, trail: cmd.id !== null }, frame);
      case 'inspect': {
        const { target } = cmd;
        const at = 'agentId' in target ? sim.locate(target.agentId) : Uint32Array.of(target.x, target.y);
        return this.reply(sim, wants, frame, at ? this.selectAt(sim, at[0], at[1]) : null);
      }
      case 'seriesCsv':
        return { ok: true, value: sim.export_series_csv() };
      case 'agentsCsv':
        return { ok: true, value: sim.export_agents_csv() };
      case 'fingerprint':
        return { ok: true, value: sim.fingerprint() };
    }
  }

  /** An entry at tick t fires when the step from t to t + 1 starts. */
  private fired(from: number, to: number): void {
    if (this.config?.schedule.some((c) => c.tick >= from && c.tick < to)) this.configDue = true;
  }

  private reply(sim: SimLike, wants: Wants, frame?: ArrayBuffer, selected?: Selected | null): Result {
    return { ok: true, snapshot: this.snapshot(sim, wants, frame, selected) };
  }

  /** `selected` is an `inspect` command's answer; it replaces `wants.select`. */
  private snapshot(sim: SimLike, wants: Wants, frame: ArrayBuffer | undefined, selected?: Selected | null): WorldSnapshot {
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
      const charts = this.charts(sim, wants.charts.groups, wants.charts.max);
      if (charts) s.charts = charts;
    }
    if (wants.lorenz) s.lorenz = sim.lorenz(101);
    if (wants.wealthHist) s.wealthHist = sim.wealth_hist(20);
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

  /** The groups with news (Decision 4); undefined when none has any. */
  private charts(sim: SimLike, groups: string[][], max: number): Record<string, ChartGroup> | undefined {
    const length = sim.tick() + 1;
    const now = this.now();
    let out: Record<string, ChartGroup> | undefined;
    for (const names of groups) {
      const key = chartKey(names);
      const last = this.sent.get(key);
      if (last && (length === last.length || (now - last.at < CHART_MS && length <= last.length * (1 + CHART_GROWTH)))) continue;
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
