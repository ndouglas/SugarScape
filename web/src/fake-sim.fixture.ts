// A stand-in for the WASM `Sim` in host, transport and engine tests (Vitest runs in Node, without WASM).
import type { SimLike, SimModule } from './sim-host';

const fieldError = (field: string, message: string): string => JSON.stringify([{ field, message }]);

/** A fake keyframe: copies of the state a restore puts back. */
export class FakeCheckpoint {
  constructor(
    readonly ticks: number,
    readonly agents: Map<number, [number, number]>,
    readonly config: FakeConfig,
    private log: string[],
  ) {}
  free(): void {
    this.log.push('free checkpoint');
  }
}

interface FakeConfig {
  /**
   * Absent for a sugarscape; `'ring'` also answers `ring_sugar`/`ring_agents`, `'anasazi'` the
   * `anasazi_*` overlays.
   */
  model?: 'schelling' | 'ring' | 'anasazi';
  /** The tick at which the world is finished (it steps no further); none by default. */
  finish?: number;
  width: number;
  height: number;
  population: number;
  schedule: { tick: number; set: Record<string, unknown> }[];
  goods: { name: string }[];
  pollution: { enabled: boolean; pollutants: { name: string }[] };
  disease: { enabled: boolean };
  sex: { enabled: boolean };
  lifespan: { enabled: boolean };
  culture: { enabled: boolean };
}

/** The core fills in missing fields; so does the fake (enough for the host and `clampDisplay`). */
function normalize(c: Partial<FakeConfig>): FakeConfig {
  return {
    width: 4,
    height: 3,
    population: 10,
    schedule: [],
    goods: [{ name: 'sugar' }],
    pollution: { enabled: false, pollutants: [] },
    disease: { enabled: false },
    sex: { enabled: false },
    lifespan: { enabled: false },
    culture: { enabled: false },
    ...c,
  };
}

/**
 * A 4 × 3 world (by default) with one agent, #1, starting at (1, 1) and walking one site right per
 * tick. `population > 1000` is a field error; painting a negative capacity "panics" (throws an
 * Error, not a string). Frames are filled with `tick % 256`.
 */
export class FakeSim implements SimLike {
  ticks = 0;
  stepCalls = 0;
  config: FakeConfig;
  agents = new Map<number, [number, number]>([[1, [1, 1]]]);
  /** Whether `checkpoint()` gives a keyframe; set false for a model without them. */
  keyframes = true;
  restores = 0;
  private edited: boolean[];
  private followedId = -1;

  constructor(
    config: Partial<FakeConfig>,
    readonly seed: number,
    private log: string[],
  ) {
    this.config = normalize(config);
    this.edited = this.config.goods.map(() => false);
  }

  step(n: number): void {
    this.stepCalls++;
    const before = this.ticks;
    this.ticks = Math.min(this.ticks + n, this.config.finish ?? Infinity);
    const a = this.agents.get(1);
    if (a) a[0] = (a[0] + this.ticks - before) % this.config.width;
  }
  tick(): number {
    return this.ticks;
  }
  width(): number {
    return this.config.width;
  }
  height(): number {
    return this.config.height;
  }
  population(): number {
    return this.agents.size;
  }
  render(colorMode: string, layer: string): number {
    this.log.push(`render ${colorMode} ${layer}`);
    return 0;
  }
  frame_len(): number {
    return this.config.width * this.config.height * 4;
  }
  stats_latest(): string {
    return JSON.stringify({ tick: this.ticks, population: this.agents.size });
  }
  series_group(namesJson: string, max: number): Float64Array {
    const names = JSON.parse(namesJson) as string[];
    for (const n of names) if (n !== 'population' && n !== 'gini') throw fieldError('edit', `unknown series "${n}"`);
    const k = Math.min(this.ticks + 1, max);
    const ticks = Array.from({ length: k }, (_, i) => (k === 1 ? 0 : Math.round((i * this.ticks) / (k - 1))));
    return Float64Array.from([k, ...ticks, ...names.flatMap((n) => ticks.map((t) => (n === 'gini' ? 0.5 : t)))]);
  }
  lorenz(points: number): Float64Array {
    return new Float64Array(points);
  }
  wealth_hist(bins: number): Float64Array {
    return new Float64Array(bins + 1);
  }
  age_hist(bin: number): Float64Array {
    return Float64Array.of(bin, this.agents.size, 0);
  }
  tag_hist(): Float64Array {
    return Float64Array.of(100, 0);
  }
  good_wealth_hist(good: number, bins: number): Float64Array {
    if (good >= this.config.goods.length) throw fieldError('edit', `there is no good ${good}`);
    return Float64Array.from({ length: bins + 1 }, (_, i) => (i === 0 ? good + 1 : 0));
  }
  lorenz_total(points: number): Float64Array {
    return new Float64Array(points).fill(0.5);
  }
  supply_demand(): Float64Array {
    return Float64Array.of(0, NaN, NaN, NaN, NaN);
  }
  private agentAt(x: number, y: number): number | undefined {
    return [...this.agents].find(([, p]) => p[0] === x && p[1] === y)?.[0];
  }
  inspect(x: number, y: number): string {
    if (x >= this.config.width || y >= this.config.height) throw fieldError('edit', `(${x}, ${y}) is off the grid`);
    const id = this.agentAt(x, y);
    return JSON.stringify({ site: { x, y, resources: [1], capacities: [4], pollution: [] }, agent: id === undefined ? null : { id } });
  }
  locate(id: number): Uint32Array | undefined {
    const p = this.agents.get(id);
    return p && Uint32Array.from(p);
  }
  follow(id: number): void {
    this.followedId = id;
  }
  unfollow(): void {
    this.followedId = -1;
  }
  trail(): Uint32Array {
    const p = this.agents.get(this.followedId);
    return p ? Uint32Array.from(p) : new Uint32Array(0);
  }
  followed(): number {
    return this.followedId;
  }
  paint_capacity(_x: number, _y: number, _radius: number, value: number, good: number): void {
    if (value < 0) throw new Error('unreachable executed');
    if (good >= this.config.goods.length) throw fieldError('edit', `there is no good ${good}`);
    this.edited[good] = true;
    this.log.push(`paint ${good}`);
  }
  set_landscape(good: number, capacities: Uint8Array): void {
    if (capacities.length !== this.config.width * this.config.height) throw fieldError('edit', 'wrong size');
    this.edited[good] = true;
  }
  place_agent(x: number, y: number, overridesJson: string): number {
    if (this.agentAt(x, y) !== undefined) throw fieldError('edit', 'site is occupied');
    const id = Math.max(0, ...this.agents.keys()) + 1;
    this.agents.set(id, [x, y]);
    this.log.push(`place ${overridesJson}`);
    return id;
  }
  remove_agent(x: number, y: number): void {
    const id = this.agentAt(x, y);
    if (id === undefined) throw fieldError('edit', `no agent at (${x}, ${y})`);
    this.agents.delete(id);
  }
  infect(): boolean {
    return true;
  }
  vaccinate(): number {
    return 1;
  }
  set_config(json: string): void {
    const c = JSON.parse(json) as Partial<FakeConfig>;
    if ((c.population ?? 0) > 1000) throw fieldError('population', 'too many');
    this.config = normalize(c);
    this.log.push('setConfig');
  }
  export_config(): string {
    return JSON.stringify(this.config);
  }
  export_landscape(): Uint8Array {
    return new Uint8Array(this.config.width * this.config.height).fill(7);
  }
  landscape_edited(good: number): boolean {
    return this.edited[good] ?? false;
  }
  export_series_csv(): string {
    return `tick,population\n${this.ticks},${this.agents.size}\n`;
  }
  export_agents_csv(): string {
    return 'id\n1\n';
  }
  networks(kind: string): Uint32Array {
    return kind === 'trade' ? Uint32Array.of(0, 0, 1, 1) : new Uint32Array(0);
  }
  credit_graph(): string {
    return '{"agents":[],"loans":[]}';
  }
  disease_list(): string {
    return '[{"id":0,"bits":"01","carriers":2}]';
  }
  ring_sugar(): Float64Array {
    return this.config.model === 'ring' ? new Float64Array(this.config.width).fill(2) : new Float64Array(0);
  }
  ring_agents(): Uint32Array {
    return this.config.model === 'ring' ? Uint32Array.from(this.agents.values(), (p) => p[0]) : new Uint32Array(0);
  }
  anasazi_water(): Uint32Array {
    return this.config.model === 'anasazi' ? Uint32Array.of(0, 0, 2, 1) : new Uint32Array(0);
  }
  anasazi_settlements(): Uint32Array {
    const a = this.agents.get(1);
    return this.config.model === 'anasazi' && a ? Uint32Array.of(a[0], 0, 1) : new Uint32Array(0);
  }
  anasazi_links(): Uint32Array {
    const a = this.agents.get(1);
    return this.config.model === 'anasazi' && a ? Uint32Array.of(a[0], a[1], a[0], 0) : new Uint32Array(0);
  }
  finished(): boolean {
    return this.config.finish !== undefined && this.ticks >= this.config.finish;
  }
  fingerprint(): string {
    const agents = [...this.agents].map(([id, p]) => `${id}@${p[0]},${p[1]}`).join(';');
    return `0x${this.ticks.toString(16)}|${agents}|${this.config.population}`;
  }
  checkpoint(): FakeCheckpoint | undefined {
    if (!this.keyframes) return undefined;
    const agents = new Map([...this.agents].map(([id, p]) => [id, [p[0], p[1]] as [number, number]]));
    return new FakeCheckpoint(this.ticks, agents, structuredClone(this.config), this.log);
  }
  restore(cp: FakeCheckpoint): void {
    if (cp.ticks > this.ticks) throw fieldError('tick', `this world has not reached tick ${cp.ticks}`);
    this.restores++;
    this.ticks = cp.ticks;
    this.agents = new Map([...cp.agents].map(([id, p]) => [id, [p[0], p[1]] as [number, number]]));
    this.config = structuredClone(cp.config);
  }
  latest_value(name: string): number | undefined {
    if (name === 'population') return this.agents.size;
    if (name === 'tick') return this.ticks;
    return undefined;
  }
  free(): void {
    this.log.push('free');
  }
}

/** A module whose worlds are `FakeSim`s (all kept in `sims`, newest last). */
export function fakeModule(log: string[] = [], opts: { keyframes?: boolean } = {}): SimModule & { sims: FakeSim[] } {
  const sims: FakeSim[] = [];
  return {
    sims,
    create(configJson, seed) {
      const config = JSON.parse(configJson) as Partial<FakeConfig>;
      if ((config.population ?? 0) > 1000) throw fieldError('population', 'too many');
      const sim = new FakeSim(config, seed, log);
      sim.keyframes = opts.keyframes ?? true;
      sims.push(sim);
      log.push(`create ${seed}`);
      return sim;
    },
    frameBytes: (_ptr, len) => new Uint8Array(len).fill((sims.at(-1)?.ticks ?? 0) % 256),
  };
}
