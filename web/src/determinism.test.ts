import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { copyWorld, Lockstep } from './compare/lockstep';
import { Engine, type Speed } from './engine';
import { modelOf } from './models';
import type { NetworkOverlay } from './protocol';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport } from './transport';
import { decodeShare, encodeShare } from './share';
import type { AnasaziStats, CivilConfig, CivilStats, Preset, Snapshot } from './types';
import { MODEL_CHARTS } from './ui/series-data';
import { config_series_names, initSync, presets_json, run_point, sweep_points } from './wasm-pkg/sugarscape.js';

// Built by `npm run build` (wasm-pack) before `npm test`.
const wasm = initSync({ module: readFileSync(new URL('./wasm-pkg/sugarscape_bg.wasm', import.meta.url)) });
const presets = JSON.parse(presets_json()) as Preset[];
const unit = presets.find((p) => p.id === 'ii-2-unit')!;
/** crates/sugarscape-core/tests/golden.rs: ii-2-unit after 200 ticks from seed 1. */
const GOLDEN = '0x75b93943813545e4';

const inline = () => new InlineTransport(new SimHost(wasmSimModule(wasm.memory)));

async function engine(): Promise<Engine> {
  return Engine.create({ config: structuredClone(unit.config), seed: 1 }, { presets, transport: inline() });
}

describe('determinism through the engine', () => {
  it('reproduces the golden ii-2-unit fingerprint', async () => {
    const e = await engine();
    await e.advance(200);
    expect(e.tick).toBe(200);
    expect(await e.fingerprint()).toBe(GOLDEN);
  });

  it('does not depend on how ticks are split into frames', async () => {
    for (const chunks of [Array.from({ length: 40 }, () => 5), [1, 2, 5, 10, 25, 100, 57]]) {
      const e = await engine();
      for (const n of chunks) await e.advance(n);
      expect(await e.fingerprint()).toBe(GOLDEN);
    }
  });

  it('is unchanged by rendering, inspection, overlays and charts', async () => {
    const e = await engine();
    e.setDisplay({ colorMode: 'wealth', overlays: { trade: true } });
    await e.select(10, 10);
    const networks: NetworkOverlay[] = ['trade', 'credit'];
    e.want(() => ({ charts: { groups: [['population', 'gini']], max: 50 }, lorenz: true, wealthHist: true, networks }));
    for (let i = 0; i < 20; i++) await e.advance(10);
    expect(await e.fingerprint()).toBe(GOLDEN);
  });
});

describe('Chapter VI views', () => {
  const everything = presets.find((p) => p.id === 'vi-1-everything')!;
  const create = () => Engine.create({ config: structuredClone(everything.config), seed: 1 }, { presets, transport: inline() });
  /** crates/sugarscape-core/src/render.rs: FOUNDER (dark gray) and BORN (green). */
  const has = (frame: Uint8ClampedArray, rgb: [number, number, number]) => {
    for (let i = 0; i < frame.length; i += 4) if (frame[i] === rgb[0] && frame[i + 1] === rgb[1] && frame[i + 2] === rgb[2]) return true;
    return false;
  };

  it('draw lineage colors and fetch networks, histograms and wealth views without changing the run', async () => {
    const plain = await create();
    await plain.advance(100);
    const watched = await create();
    watched.setDisplay({ colorMode: 'lineage', overlays: { neighbors: true, friends: true, family: true } });
    watched.want(() => ({ ageHist: true, tagHist: true, lorenzTotal: true, goodWealthHists: true }));
    for (let i = 0; i < 10; i++) await watched.advance(10);
    expect(watched.networks('neighbors').length).toBeGreaterThan(0);
    expect(watched.networks('friends').length).toBeGreaterThan(0);
    expect(watched.networks('family').length).toBeGreaterThan(0);
    expect(watched.last?.ageHist?.[0]).toBe(5);
    expect(watched.last?.tagHist).toHaveLength(11);
    expect(watched.last?.goodWealthHists).toHaveLength(2);
    expect(watched.last?.lorenzTotal).toHaveLength(101);
    expect((watched.latest as Snapshot).gini_total).toBeGreaterThan(0);
    const frame = watched.frame()!;
    expect(has(frame, [0x5a, 0x5a, 0x5a])).toBe(true);
    expect(has(frame, [0x3d, 0xd6, 0x6b])).toBe(true);
    expect(await watched.fingerprint()).toBe(await plain.fingerprint());
  });
});

describe('sessions replay exactly', () => {
  const endemic = presets.find((p) => p.id === 'v-2-endemic')!;
  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
  const create = (seed: number) => Engine.create({ config: structuredClone(endemic.config), seed }, { presets, transport: inline() });

  /**
   * A Max run's pace depends on the machine, so waits here poll for progress instead of sleeping a
   * fixed time: a generous cap still fails the test with a clear message if `e` ever gets stuck
   * rather than hanging until Vitest's own timeout.
   */
  const PROGRESS_CAP_MS = 5000;

  /** Loops `wait(0)` until `e.tick` has moved past `tick`, capped so a stuck run fails clearly. */
  async function waitForTickPast(e: Engine, tick: number): Promise<void> {
    const start = performance.now();
    while (e.tick <= tick) {
      if (performance.now() - start > PROGRESS_CAP_MS) {
        throw new Error(`timed out after ${PROGRESS_CAP_MS}ms waiting for tick to pass ${tick} (stuck at ${e.tick})`);
      }
      await wait(0);
    }
  }

  /** Runs `e` at `speed` for a few animation frames (Max: until it has made real progress), then pauses it and waits until it is quiet. */
  async function run(e: Engine, speed: Speed, frames = 4): Promise<void> {
    e.setSpeed(speed);
    e.setRunning(true);
    if (speed === 'max') await waitForTickPast(e, e.tick);
    else
      for (let i = 0; i < frames; i++) {
        e.pump();
        await wait(0);
      }
    e.setRunning(false);
    // Waits for the Max stop (or the frame's step) to be answered.
    await e.session();
  }

  /** Steps `e` to `tick` in uneven requests. */
  async function reach(e: Engine, tick: number): Promise<void> {
    for (const n of [1, 7, 100, 3]) if (e.tick < tick) await e.advance(Math.min(n, tick - e.tick));
    while (e.tick < tick) await e.advance(Math.min(250, tick - e.tick));
  }

  /**
   * The first unoccupied site at or after (`x`, `y`), scanning row-major and wrapping once around
   * the grid: deterministic under a fixed seed since it only runs while `e` isn't at Max (no tick
   * elapses between sites while scanning, so occupancy cannot change mid-scan). Used so `place`
   * always lands on an empty site instead of failing "site is occupied" depending on the seed.
   */
  async function emptySite(e: Engine, x: number, y: number): Promise<{ x: number; y: number }> {
    const { width, height } = e.size();
    let cx = x;
    let cy = y;
    for (let n = 0; n < width * height; n++) {
      await e.select(cx, cy);
      if (e.inspection?.agentId == null) return { x: cx, y: cy };
      if (++cx >= width) {
        cx = 0;
        cy = (cy + 1) % height;
      }
    }
    throw new Error(`no empty site found scanning the whole grid from (${x}, ${y})`);
  }

  it(
    'replays a session recorded at mixed speeds, through a share link, to the same world at the same tick',
    async () => {
      const live = await create(5);
      expect(await live.paint(10, 10, 2, 0, 0)).toBeNull(); // tick 0
      // Erase the agent just placed, before any tick runs: guaranteed present (an agent placed
      // and then left to a run may wander or die, so erasing it later is not deterministic under
      // any seed).
      const eraseSpot = await emptySite(live, 3, 3);
      expect(await live.place(eraseSpot.x, eraseSpot.y, {})).toBeNull();
      expect(await live.erase(eraseSpot.x, eraseSpot.y)).toBeNull();
      await run(live, 1);
      // Same trick for infect: place fresh (a guaranteed-empty site), then infect it immediately.
      const infectSpot = await emptySite(live, 3, 3);
      expect(await live.place(infectSpot.x, infectSpot.y, {})).toBeNull();
      expect(await live.infect(infectSpot.x, infectSpot.y, -1)).toBeNull();
      await run(live, 25);
      const femaleSpot = await emptySite(live, 0, 0);
      expect(await live.place(femaleSpot.x, femaleSpot.y, { sex: 'female' })).toBeNull();
      expect(await live.applyConfig((c) => void (c.growback.rate = 2))).toBeNull();
      await run(live, 'max');
      expect(await live.vaccinate(20, 20, 3, 0)).toBeNull();
      // Edits while Max runs land between batches: wait for real progress on each side (not a
      // fixed sleep) so they land regardless of how fast Max runs on this machine. `place` stays
      // out of this window (occupancy can't be checked race-free while Max keeps ticking);
      // `paint` never depends on occupancy, so it proves the same "lands mid-run" point safely.
      live.setSpeed('max');
      live.setRunning(true);
      await waitForTickPast(live, live.tick);
      expect(await live.paint(30, 30, 1, 4, 0)).toBeNull();
      await waitForTickPast(live, live.tick);
      live.setRunning(false);
      await run(live, 100, 2);
      const { session, full, tick } = await live.session();
      expect(full).toBe(false);
      // Every edit above is asserted to succeed, so all nine are logged, spread across five
      // distinct ticks (0, after the first run, after the second, at the Max stop and mid-Max).
      expect(session.log.length).toBe(9);
      expect(new Set(session.log.map((e) => e.tick)).size).toBe(5);
      const expected = await live.fingerprint();

      const replayed = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
      await reach(replayed, tick);
      expect(replayed.tick).toBe(tick);
      expect(replayed.replayLeft).toBe(0);
      expect(await replayed.fingerprint()).toBe(expected);
      expect((await replayed.session()).session.log).toEqual(session.log);

      // Reset with the same seed replays it again on the same engine.
      expect(await live.replay()).toBeNull();
      expect(live.tick).toBe(0);
      await reach(live, tick);
      expect(await live.fingerprint()).toBe(expected);
    },
    20_000,
  );

  it(
    'replays at Max to the same world as the live run',
    async () => {
      const live = await create(8);
      await live.advance(2);
      const spot = await emptySite(live, 4, 4);
      expect(await live.place(spot.x, spot.y, {})).toBeNull();
      expect(await live.infect(spot.x, spot.y, -1)).toBeNull();
      await live.advance(3);
      expect(await live.paint(12, 30, 3, 1, 0)).toBeNull();
      expect(await live.applyConfig((c) => void (c.growback.rate = 3))).toBeNull();
      await live.advance(4);
      const { session } = await live.session();
      const lastLoggedTick = session.log[session.log.length - 1].tick;
      // The wire round trip (encode/decode a share link) is proven together with the Max replay.
      const replayed = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
      replayed.setSpeed('max');
      replayed.setRunning(true);
      // Run until replay is past the last logged tick, not for a fixed time: only then can every
      // edit in the log have replayed.
      await waitForTickPast(replayed, lastLoggedTick);
      replayed.setRunning(false);
      // Waits for the Max stop to be answered.
      await replayed.session();
      expect(replayed.replayLeft).toBe(0);
      // Bring whichever is behind up to the other: after the log ends, both just step.
      const at = Math.max(replayed.tick, live.tick);
      await reach(replayed, at);
      await reach(live, at);
      expect(await replayed.fingerprint()).toBe(await live.fingerprint());
    },
    20_000,
  );
  it('copies a world for Compare: B reaches A’s tick with A’s fingerprint', async () => {
    const a = await create(9);
    await a.advance(5);
    const spot = await emptySite(a, 3, 3);
    expect(await a.place(spot.x, spot.y, {})).toBeNull();
    expect(await a.infect(spot.x, spot.y, -1)).toBeNull();
    await a.advance(20);
    expect(await a.paint(8, 8, 2, 1, 0)).toBeNull();
    await a.advance(15);
    const { session, tick } = await a.session();
    expect(session.log.map((e) => e.cmd.type)).toEqual(['place', 'infect', 'paint']);
    const b = await copyWorld(session, tick, (s) => Engine.create(s, { presets, transport: inline() }));
    expect(b.tick).toBe(tick);
    expect(await b.fingerprint()).toBe(await a.fingerprint());
    expect((await b.session()).session.log).toEqual(session.log);
  });
});

describe('other models through the engine', () => {
  /** crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: each preset after 200 ticks from seed 1. */
  const GOLDEN_MODELS: [string, string][] = [
    ['vi-4-schelling-25', '0x7a7072c3433f5f6f'],
    ['vi-5-schelling-25-residence', '0x9abe1c25e873debd'],
    ['vi-6-schelling-50-residence', '0x637412f7af91f684'],
    ['vi-7-schelling-mixed', '0x79346d2a338108cf'],
    ['vi-8-ring-world', '0x1c341361c466db90'],
    ['vi-9-ring-megagroup', '0x430d0c3b19b6e58e'],
    ['cv-run-1-no-movement', '0x49637b8b34864721'],
    ['cv-run-2-punctuated', '0x7888f03e0d511f6d'],
    ['cv-run-3-salami', '0x51664e9ecc568140'],
    ['cv-run-4-one-jump', '0xc8ad285446786559'],
    ['cv-run-5-cop-reductions', '0x5713c0cafe4898dc'],
    ['cv-run-6-coexistence', '0x1ce4fc6300e993ee'],
    ['cv-run-8-nasty-regime', '0x5ce734d905ba0c5d'],
    ['cv-netlogo', '0x87a92345c017b0ae'],
  ];

  it.each(GOLDEN_MODELS)('%s reproduces its golden fingerprint, whatever is watched', async (id, golden) => {
    const preset = presets.find((p) => p.id === id)!;
    const e = await Engine.create({ config: structuredClone(preset.config), seed: 1 }, { presets, transport: inline() });
    // Sugarscape-only wishes are ignored; charts, the ring and inspection change nothing.
    e.want(() => ({ charts: { groups: [['population']], max: 50 }, lorenz: true, networks: ['trade'] }));
    await e.select(3, 3);
    for (const n of [1, 9, 40, 150]) await e.advance(n);
    expect(e.tick).toBe(200);
    expect(await e.fingerprint()).toBe(golden);
  });
});

describe('model charts', () => {
  it('draw only series their model records', () => {
    for (const [model, charts] of Object.entries(MODEL_CHARTS)) {
      const preset = presets.find((p) => modelOf(p.config) === model)!;
      const names = JSON.parse(config_series_names(JSON.stringify(preset.config))) as string[];
      for (const c of charts) for (const line of c.lines) expect(names, `${model}: ${c.title}`).toContain(line.key);
    }
  });
});

describe('sweeps over other models', () => {
  const spec = {
    name: 'Schelling population',
    base: { preset: 'vi-4-schelling-25' },
    x: { path: 'population', values: [500, 1500] },
    seeds: { from: 1, count: 1 },
    ticks: 10,
    metric: { kind: 'final', series: 'segregation' },
  };

  it('run over a Schelling base, its paths and statistics checked against Schelling', () => {
    const json = JSON.stringify(spec);
    expect(JSON.parse(sweep_points(json))).toHaveLength(2);
    const run = JSON.parse(run_point(json, 1)) as { value: number };
    expect(run.value).toBeGreaterThan(0.5);
    const wrongPath = JSON.stringify({ ...spec, x: { path: 'vision.max', values: [1] } });
    expect(() => sweep_points(wrongPath)).toThrow('unknown field vision.max');
    const wrongSeries = JSON.stringify({ ...spec, metric: { kind: 'final', series: 'gini' } });
    expect(() => sweep_points(wrongSeries)).toThrow('no statistics series');
  });

  it('refuse an anasazi sweep longer than the valley’s years, so no worker aborts', () => {
    const valley = {
      name: 'Valley',
      base: { preset: 'lhv-published' },
      x: { path: 'harvest_adjustment', values: [0.56] },
      seeds: { from: 1, count: 1 },
      ticks: 551,
      metric: { kind: 'final', series: 'fit' },
    };
    const json = JSON.stringify(valley);
    // The Experiments view shows these errors (sweep_points) before it starts any worker.
    expect(() => sweep_points(json)).toThrow('must be ≤ 550');
    expect(() => run_point(json, 0)).toThrow('must be ≤ 550');
    expect(JSON.parse(sweep_points(JSON.stringify({ ...valley, ticks: 550 })))).toHaveLength(1);
  });
});

describe('the anasazi through the engine', () => {
  const lhv = presets.find((p) => p.id === 'lhv-published')!;

  it('reproduces the golden lhv-published fingerprint and stops at AD 1350', async () => {
    const e = await Engine.create({ config: structuredClone(lhv.config), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'zones', overlays: { water: true, settlements: true, links: true } });
    await e.advance(200);
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
    expect(await e.fingerprint()).toBe('0x3b357e6f0cc5f74a');
    expect((e.latest as AnasaziStats).year).toBe(1000);
    expect(e.valley?.links.length).toBe(4 * e.population);
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1000);
    expect([e.tick, e.finished, ends]).toEqual([550, true, 1]);
    expect((e.latest as AnasaziStats).year).toBe(1350);
  });
});

describe('civil violence through the engine', () => {
  it('stops run 7 when a group is gone, once', async () => {
    const run7 = presets.find((p) => p.id === 'cv-run-7-cleansing')!;
    const e = await Engine.create({ config: structuredClone(run7.config), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'grievance' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1000);
    const s = e.latest as CivilStats;
    expect([e.finished, ends, s.extinction]).toEqual([true, 1, e.tick]);
    expect(e.tick).toBeLessThan(1000);
    expect(Math.min(s.blue, s.green)).toBe(0);
    await e.advance(10);
    expect(s.extinction).toBe(e.tick);
  });

  it('keeps ethnic cleansing and peacekeepers in step in Compare until one dies out', async () => {
    const make = (id: string) =>
      Engine.create({ config: structuredClone(presets.find((p) => p.id === id)!.config), seed: 1 }, { presets, transport: inline() });
    const [a, b] = [await make('cv-run-7-cleansing'), await make('cv-safe-havens')];
    const lock = new Lockstep([a, b], 'max', () => 0); // batches double each frame
    await lock.settled();
    lock.setRunning(true);
    for (let i = 0; i < 200 && lock.running; i++) {
      lock.pump(i);
      await lock.settled();
    }
    expect(lock.running).toBe(false);
    expect(a.tick).toBe(b.tick);
    expect(lock.finishedWorld()).not.toBe(-1);
    const [ended, other] = lock.finishedWorld() === 0 ? [a, b] : [b, a];
    expect((ended.latest as CivilStats).extinction).toBe(ended.tick);
    expect(other.tick).toBe(ended.tick);
  });
});

describe('civil violence’s schedule and ramps reach the page', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const create = (id: string) => Engine.create({ config: structuredClone(preset(id).config), seed: 1 }, { presets, transport: inline() });
  const legitimacy = (e: Engine) => (e.config as CivilConfig).legitimacy;

  it('shows one jump’s legitimacy after t = 77 and keeps it through an unrelated live edit', async () => {
    const e = await create('cv-run-4-one-jump');
    await e.advance(80);
    expect(legitimacy(e)).toBe(0.7);
    expect(await e.applyModelConfig((c) => void ((c as CivilConfig).quirks.jailed_stay = true))).toBeNull();
    expect(legitimacy(e)).toBe(0.7);
    await e.advance(1);
    expect((e.latest as CivilStats).legitimacy).toBe(0.7);
    expect(legitimacy(e)).toBe(0.7);
  });

  it('shows salami tactics’ ramped legitimacy mid-ramp', async () => {
    const e = await create('cv-run-3-salami');
    e.want(() => ({ charts: { groups: [['legitimacy']], max: 2000 } }));
    await e.advance(100);
    // The step from t = 99 moved it last: 0.9 − 0.7 × 22 / 70.
    expect(legitimacy(e)).toBeCloseTo(0.68, 12);
    expect(legitimacy(e)).toBe((e.latest as CivilStats).legitimacy);
    // Within the charts throttle, a ramped change keeps the group the page has.
    await e.advance(1);
    expect([e.last?.config !== undefined, e.last?.charts]).toEqual([true, undefined]);
    expect(e.chartGroup(['legitimacy'])?.ticks).toHaveLength(101);
    await e.advance(4);
    expect(legitimacy(e)).toBeCloseTo(0.63, 12);
  });

  it('keeps a same-value live edit mid-ramp from changing the run (cop reductions)', async () => {
    const plain = await create('cv-run-5-cop-reductions');
    await plain.advance(150);
    const live = await create('cv-run-5-cop-reductions');
    await live.advance(100);
    // Sets threshold to the value it already has: with the live config right, the cops stay as the ramp left them.
    expect(await live.applyModelConfig((c) => void ((c as CivilConfig).threshold = (c as CivilConfig).threshold))).toBeNull();
    await live.advance(50);
    expect(await live.fingerprint()).toBe(await plain.fingerprint());
  });

  it('replays one jump with a live edit after t = 77 through a share link to the same world', async () => {
    const live = await create('cv-run-4-one-jump');
    await live.advance(80);
    expect(await live.applyModelConfig((c) => void ((c as CivilConfig).quirks.active_counts_twice = true))).toBeNull();
    await live.advance(40);
    const expected = await live.fingerprint();
    const { session, tick } = await live.session();
    expect(session.log).toHaveLength(1);
    expect((session.log[0].cmd as { config: CivilConfig }).config.legitimacy).toBe(0.7);
    const replayed = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await replayed.advance(tick);
    expect(replayed.replayLeft).toBe(0);
    expect(await replayed.fingerprint()).toBe(expected);
    expect([legitimacy(replayed), (replayed.latest as CivilStats).legitimacy]).toEqual([0.7, 0.7]);
  });
});

