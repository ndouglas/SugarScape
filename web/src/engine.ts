import init, { Sim, presets_json } from './wasm-pkg/sugarscape.js';
import type { ColorMode, Config, DiseaseEntry, FieldError, Inspection, Layer, Preset } from './types';
import { parseErrors } from './types';
import { validLayer } from './layers';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit';

export interface Selection { x: number; y: number; agentId: number | null }

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

export interface InitialState { config: Config; seed: number; landscapes?: (Uint8Array | null)[] }

export type Overlay = 'trade' | 'credit' | 'disease';

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** The config as Rust sees it: serde defaults filled in for any missing fields. */
function normalized(sim: Sim): Config {
  return JSON.parse(sim.export_config()) as Config;
}

/** Owns the WASM simulation and the playground's run/display/selection state. */
export class Engine {
  running = false;
  stepsPerFrame = 1;
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'resource:0';
  overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };
  selection: Selection | null = null;
  presetId: string | null;
  /** Painted or shared maps per good (null = generated), kept across resets that keep that good's map. */
  private customLandscapes: (Uint8Array | null)[];
  private listeners = new Map<EngineEvent, Set<() => void>>();

  /**
   * The live config: what the running world uses now, including scheduled
   * changes that have fired. The Rules panel shows this one.
   */
  config: Config;

  /**
   * @param baseConfig The setup as chosen (preset, share link, reset): what a
   * reset rebuilds and a share link carries; preset matching and
   * `isModified()` compare against it.
   */
  private constructor(
    private memory: WebAssembly.Memory,
    readonly presets: Preset[],
    public sim: Sim,
    public baseConfig: Config,
    public seed: number,
    landscapes: (Uint8Array | null)[],
  ) {
    this.config = structuredClone(baseConfig);
    this.customLandscapes = landscapes;
    this.presetId = this.matchPreset();
  }

  /** Throws parsed FieldError[] (as an Error message) if `initial` is invalid. */
  static async create(initial?: InitialState): Promise<Engine> {
    const wasm = await init();
    const presets = JSON.parse(presets_json()) as Preset[];
    const fallback = presets.find((p) => p.id === 'ii-2-unit') ?? presets[0];
    const config = initial?.config ?? structuredClone(fallback.config);
    const seed = initial?.seed ?? randomSeed();
    let sim: Sim;
    try {
      sim = new Sim(JSON.stringify(config), seed, initial?.landscapes ?? null);
    } catch (e) {
      throw new Error(parseErrors(e).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    return new Engine(wasm.memory, presets, sim, normalized(sim), seed, initial?.landscapes ?? []);
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

  size(): { width: number; height: number } {
    return { width: this.sim.width(), height: this.sim.height() };
  }

  /**
   * Rebuilds the world, keeping each good's painted/shared map unless the
   * grid size, the number of goods or that good's map changes. On error the
   * current world is kept and errors returned.
   */
  reset(config: Config = this.baseConfig, seed: number = this.seed): FieldError[] | null {
    const same =
      config.width === this.baseConfig.width &&
      config.height === this.baseConfig.height &&
      config.goods.length === this.baseConfig.goods.length;
    // A changed number of goods drops every painted map: Sim needs one entry per good (or none).
    const kept = same
      ? this.customLandscapes
          .slice(0, config.goods.length)
          .map((l, i) => (JSON.stringify(config.goods[i]?.map) === JSON.stringify(this.baseConfig.goods[i]?.map) ? l : null))
      : [];
    return this.rebuild(config, seed, kept);
  }

  private rebuild(config: Config, seed: number, landscapes: (Uint8Array | null)[]): FieldError[] | null {
    let next: Sim;
    try {
      next = new Sim(JSON.stringify(config), seed, landscapes);
    } catch (e) {
      return parseErrors(e);
    }
    this.sim.free();
    this.sim = next;
    this.baseConfig = normalized(next);
    this.config = structuredClone(this.baseConfig);
    this.customLandscapes = landscapes;
    this.seed = seed;
    this.selection = null;
    this.presetId = this.matchPreset();
    this.clampDisplay();
    this.emit('reset');
    return null;
  }

  /**
   * Applies a rule/parameter change to the running world: `mutate` edits a
   * copy of the live config for the world and, on success, a copy of the base
   * config too, so scheduled changes that already fired are not undone.
   */
  applyConfig(mutate: (c: Config) => void): FieldError[] | null {
    const next = structuredClone(this.config);
    mutate(next);
    try {
      this.sim.set_config(JSON.stringify(next));
    } catch (e) {
      return parseErrors(e);
    }
    const base = structuredClone(this.baseConfig);
    mutate(base);
    this.baseConfig = base;
    this.config = normalized(this.sim);
    this.clampDisplay();
    this.emit('config');
    return null;
  }

  loadPreset(id: string): FieldError[] | null {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    const errors = this.rebuild(structuredClone(preset.config), this.seed, []);
    if (!errors) this.presetId = id;
    return errors;
  }

  /** The preset whose config equals the base config, if any. */
  private matchPreset(): string | null {
    const json = JSON.stringify(this.baseConfig);
    return this.presets.find((p) => JSON.stringify(p.config) === json)?.id ?? null;
  }

  /** True when the base config differs from the last chosen preset or the landscape is custom. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || this.customLandscapes.some((l) => l !== null) || JSON.stringify(preset.config) !== JSON.stringify(this.baseConfig);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  advance(n: number = this.stepsPerFrame): void {
    const from = this.sim.tick();
    this.sim.step(n);
    const to = this.sim.tick();
    // An entry at tick t fires when the step from t to t + 1 starts.
    if (this.baseConfig.schedule.some((c) => c.tick >= from && c.tick < to)) {
      this.config = normalized(this.sim);
      this.emit('config');
    }
    this.emit('tick');
  }

  /** A fresh view over the rendered RGBA frame (views die when memory grows). */
  frame(): Uint8ClampedArray<ArrayBuffer> {
    const ptr = this.sim.render(this.colorMode, this.layer);
    return new Uint8ClampedArray(this.memory.buffer, ptr, this.sim.frame_len());
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer; overlays?: Partial<Record<Overlay, boolean>> }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    if (d.overlays) Object.assign(this.overlays, d.overlays);
    this.emit('display');
  }

  /**
   * Keeps display state valid for the config: a layer the world lacks falls
   * back to good 0's level, and with disease off the disease color mode and
   * overlay fall back to tribe coloring / hidden.
   */
  private clampDisplay(): void {
    let changed = false;
    const layer = validLayer(this.layer, this.config);
    if (layer !== this.layer) {
      this.layer = layer;
      changed = true;
    }
    if (!this.config.disease.enabled) {
      if (this.colorMode === 'disease') {
        this.colorMode = 'tribe';
        changed = true;
      }
      if (this.overlays.disease) {
        this.overlays.disease = false;
        changed = true;
      }
    }
    if (changed) this.emit('display');
  }

  inspect(x: number, y: number): Inspection {
    return JSON.parse(this.sim.inspect(x, y)) as Inspection;
  }

  select(x: number, y: number): void {
    this.selection = { x, y, agentId: this.inspect(x, y).agent?.id ?? null };
    this.emit('select');
  }

  follow(id: number): void {
    const p = this.sim.locate(id);
    if (p) this.select(p[0], p[1]);
  }

  /** Keeps the selection on a followed agent as it moves. */
  trackSelection(): void {
    const s = this.selection;
    if (s?.agentId == null) return;
    const p = this.sim.locate(s.agentId);
    if (p) {
      s.x = p[0];
      s.y = p[1];
    }
  }

  private edit(fn: () => void): FieldError[] | null {
    try {
      fn();
    } catch (e) {
      return parseErrors(e);
    }
    this.emit('edit');
    return null;
  }

  /** Keeps good `good`'s current capacities as its custom map (share links, export, reset). */
  private keepLandscape(good: number): void {
    const next = [...this.customLandscapes];
    while (next.length <= good) next.push(null);
    next[good] = this.sim.export_landscape(good);
    this.customLandscapes = next;
  }

  paint(x: number, y: number, radius: number, value: number, good = 0): FieldError[] | null {
    return this.edit(() => {
      this.sim.paint_capacity(x, y, radius, value, good);
      this.keepLandscape(good);
    });
  }

  /** Replaces good `good`'s capacity map (one byte per site, 0–10). */
  importLandscape(good: number, capacities: Uint8Array): FieldError[] | null {
    return this.edit(() => {
      this.sim.set_landscape(good, capacities);
      this.keepLandscape(good);
    });
  }

  /** Each good's map where it differs from the generated one (for share links). */
  editedLandscapes(): (Uint8Array | null)[] | undefined {
    const maps = this.config.goods.map((_, i) => (this.sim.landscape_edited(i) ? this.sim.export_landscape(i) : null));
    return maps.some((m) => m !== null) ? maps : undefined;
  }

  place(x: number, y: number, overrides: PlaceOverrides): FieldError[] | null {
    return this.edit(() => void this.sim.place_agent(x, y, JSON.stringify(overrides)));
  }

  erase(x: number, y: number): FieldError[] | null {
    return this.edit(() => this.sim.remove_agent(x, y));
  }

  diseaseList(): DiseaseEntry[] {
    return JSON.parse(this.sim.disease_list()) as DiseaseEntry[];
  }

  /** `disease` −1 infects with a brand-new random disease. */
  infect(x: number, y: number, disease: number): FieldError[] | null {
    return this.edit(() => void this.sim.infect(x, y, disease));
  }

  vaccinate(x: number, y: number, radius: number, disease: number): FieldError[] | null {
    return this.edit(() => void this.sim.vaccinate(x, y, radius, disease));
  }
}
