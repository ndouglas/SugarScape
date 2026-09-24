// A stand-in for the WASM `Sim` in host, transport and engine tests (Vitest runs in Node, without WASM).
import type { SimLike, SimModule } from './sim-host';

const fieldError = (field: string, message: string): string => JSON.stringify([{ field, message }]);

interface FakeConfig {
  width: number;
  height: number;
  population: number;
  schedule: { tick: number; set: Record<string, unknown> }[];
  goods: { name: string }[];
  pollution: { enabled: boolean; pollutants: { name: string }[] };
  disease: { enabled: boolean };
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
    this.ticks += n;
    const a = this.agents.get(1);
    if (a) a[0] = (a[0] + n) % this.config.width;
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
  fingerprint(): string {
    return `0x${this.ticks.toString(16)}`;
  }
  free(): void {
    this.log.push('free');
  }
}

/** A module whose worlds are `FakeSim`s (all kept in `sims`, newest last). */
export function fakeModule(log: string[] = []): SimModule & { sims: FakeSim[] } {
  const sims: FakeSim[] = [];
  return {
    sims,
    create(configJson, seed) {
      const config = JSON.parse(configJson) as Partial<FakeConfig>;
      if ((config.population ?? 0) > 1000) throw fieldError('population', 'too many');
      const sim = new FakeSim(config, seed, log);
      sims.push(sim);
      log.push(`create ${seed}`);
      return sim;
    },
    frameBytes: (_ptr, len) => new Uint8Array(len).fill((sims.at(-1)?.ticks ?? 0) % 256),
  };
}
