import init, { Sim, presets_json } from './wasm-pkg/sugarscape.js';
import type { ColorMode, Config, DiseaseEntry, FieldError, Inspection, Layer, Preset } from './types';
import { parseErrors } from './types';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit';

export interface Selection { x: number; y: number; agentId: number | null }

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

export interface InitialState { config: Config; seed: number; landscape?: Uint8Array }

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
  layer: Layer = 'sugar';
  overlays: Record<Overlay, boolean> = { trade: false, credit: false, disease: false };
  selection: Selection | null = null;
  presetId: string | null;
  /** Painted or shared capacities, carried across resets that keep the landscape shape. */
  private customLandscape: Uint8Array | undefined;
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
    landscape?: Uint8Array,
  ) {
    this.config = structuredClone(baseConfig);
    this.customLandscape = landscape;
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
      sim = new Sim(JSON.stringify(config), seed, initial?.landscape);
    } catch (e) {
      throw new Error(parseErrors(e).map((x) => `${x.field}: ${x.message}`).join('; '));
    }
    return new Engine(wasm.memory, presets, sim, normalized(sim), seed, initial?.landscape);
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
   * Rebuilds the world, keeping a painted/shared landscape unless `config`
   * changes the landscape kind or the grid size. On error the current world is
   * kept and errors returned.
   */
  reset(config: Config = this.baseConfig, seed: number = this.seed): FieldError[] | null {
    const sameShape =
      config.width === this.baseConfig.width &&
      config.height === this.baseConfig.height &&
      JSON.stringify(config.landscape) === JSON.stringify(this.baseConfig.landscape);
    return this.rebuild(config, seed, sameShape ? this.customLandscape : undefined);
  }

  private rebuild(config: Config, seed: number, landscape: Uint8Array | undefined): FieldError[] | null {
    let next: Sim;
    try {
      next = new Sim(JSON.stringify(config), seed, landscape);
    } catch (e) {
      return parseErrors(e);
    }
    this.sim.free();
    this.sim = next;
    this.baseConfig = normalized(next);
    this.config = structuredClone(this.baseConfig);
    this.customLandscape = landscape;
    this.seed = seed;
    this.selection = null;
    this.presetId = this.matchPreset();
    this.clearDiseaseDisplayIfDiseaseIsOff();
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
    this.clearDiseaseDisplayIfDiseaseIsOff();
    this.emit('config');
    return null;
  }

  loadPreset(id: string): FieldError[] | null {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    const errors = this.rebuild(structuredClone(preset.config), this.seed, undefined);
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
    return !preset || this.customLandscape !== undefined || JSON.stringify(preset.config) !== JSON.stringify(this.baseConfig);
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
   * When disease is off, a `disease` color mode or network overlay has
   * nothing to show: fall back to tribe coloring and hide the overlay.
   * Called whenever the config is (re)applied, so display state never
   * outlives the rule it depicts.
   */
  private clearDiseaseDisplayIfDiseaseIsOff(): void {
    if (this.config.disease.enabled) return;
    let changed = false;
    if (this.colorMode === 'disease') {
      this.colorMode = 'tribe';
      changed = true;
    }
    if (this.overlays.disease) {
      this.overlays.disease = false;
      changed = true;
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

  paint(x: number, y: number, radius: number, value: number): FieldError[] | null {
    return this.edit(() => {
      this.sim.paint_capacity(x, y, radius, value, 0);
      this.customLandscape = this.sim.export_landscape(0);
    });
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
