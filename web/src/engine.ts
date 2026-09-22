import init, { Sim, presets_json } from './wasm-pkg/sugarscape.js';
import type { ColorMode, Config, FieldError, Inspection, Layer, Preset } from './types';
import { parseErrors } from './types';

export type EngineEvent = 'reset' | 'tick' | 'config' | 'run' | 'select' | 'display' | 'edit';

export interface Selection { x: number; y: number; agentId: number | null }

export interface PlaceOverrides { sex?: 'female' | 'male'; tribe?: 'blue' | 'red' }

export interface InitialState { config: Config; seed: number; landscape?: Uint8Array }

export function randomSeed(): number {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

/** Owns the WASM simulation and the playground's run/display/selection state. */
export class Engine {
  running = false;
  stepsPerFrame = 1;
  colorMode: ColorMode = 'tribe';
  layer: Layer = 'sugar';
  selection: Selection | null = null;
  presetId: string | null;
  private listeners = new Map<EngineEvent, Set<() => void>>();

  private constructor(
    private memory: WebAssembly.Memory,
    readonly presets: Preset[],
    public sim: Sim,
    public config: Config,
    public seed: number,
  ) {
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
    return new Engine(wasm.memory, presets, sim, config, seed);
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

  /** Rebuilds the world. On error the current world is kept and errors returned. */
  reset(config: Config = this.config, seed: number = this.seed, landscape?: Uint8Array): FieldError[] | null {
    let next: Sim;
    try {
      next = new Sim(JSON.stringify(config), seed, landscape);
    } catch (e) {
      return parseErrors(e);
    }
    this.sim.free();
    this.sim = next;
    this.config = config;
    this.seed = seed;
    this.selection = null;
    this.presetId = this.matchPreset();
    this.emit('reset');
    return null;
  }

  /** Applies rule/parameter changes to the running world. */
  applyConfig(config: Config): FieldError[] | null {
    try {
      this.sim.set_config(JSON.stringify(config));
    } catch (e) {
      return parseErrors(e);
    }
    this.config = config;
    this.emit('config');
    return null;
  }

  loadPreset(id: string): FieldError[] | null {
    const preset = this.presets.find((p) => p.id === id);
    if (!preset) return [{ field: 'preset', message: `unknown preset ${id}` }];
    const errors = this.reset(structuredClone(preset.config));
    if (!errors) this.presetId = id;
    return errors;
  }

  /** The preset whose config equals the current one, if any. */
  private matchPreset(): string | null {
    const json = JSON.stringify(this.config);
    return this.presets.find((p) => JSON.stringify(p.config) === json)?.id ?? null;
  }

  /** True when the config differs from the last chosen preset. */
  isModified(): boolean {
    const preset = this.presets.find((p) => p.id === this.presetId);
    return !preset || JSON.stringify(preset.config) !== JSON.stringify(this.config);
  }

  setRunning(on: boolean): void {
    this.running = on;
    this.emit('run');
  }

  advance(n: number = this.stepsPerFrame): void {
    this.sim.step(n);
    this.emit('tick');
  }

  /** A fresh view over the rendered RGBA frame (views die when memory grows). */
  frame(): Uint8ClampedArray<ArrayBuffer> {
    const ptr = this.sim.render(this.colorMode, this.layer);
    return new Uint8ClampedArray(this.memory.buffer, ptr, this.sim.frame_len());
  }

  setDisplay(d: { colorMode?: ColorMode; layer?: Layer }): void {
    if (d.colorMode) this.colorMode = d.colorMode;
    if (d.layer) this.layer = d.layer;
    this.emit('display');
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
    return this.edit(() => this.sim.paint_capacity(x, y, radius, value));
  }

  place(x: number, y: number, overrides: PlaceOverrides): FieldError[] | null {
    return this.edit(() => void this.sim.place_agent(x, y, JSON.stringify(overrides)));
  }

  erase(x: number, y: number): FieldError[] | null {
    return this.edit(() => this.sim.remove_agent(x, y));
  }
}
