import { readFileSync } from 'node:fs';
import { describe, expect, it, vi } from 'vitest';
import { comparePresetStates, COMPARE_PRESETS } from './compare-presets';
import { copyWorld, Lockstep } from './compare/lockstep';
import { defaultForm, formToSweep } from './experiments/form';
import { Engine, type Speed } from './engine';
import { isDpdView, isEthnoView, isImageView, modelOf, presetMenu } from './models';
import type { NetworkOverlay } from './protocol';
import { paramShown } from './schema-form';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport } from './transport';
import { decodeShare, encodeShare } from './share';
import type {
  AgreementConfig,
  ZiConfig,
  BaliConfig,
  BaliInspection,
  BaliStats,
  ZiInspection,
  ZiStats,
  PunishmentConfig,
  PunishmentInspection,
  PunishmentStats,
  RetirementConfig,
  RetirementInspection,
  RetirementStats,
  ThresholdsConfig,
  ThresholdsInspection,
  ThresholdsStats,
  AntsConfig,
  AntsInspection,
  AntsStats,
  FarolConfig,
  FarolInspection,
  FarolStats,
  AgreementInspection,
  AgreementStats,
  AnasaziStats,
  CivilConfig,
  CivilStats,
  ClassesConfig,
  ClassesInspection,
  ClassesStats,
  CultureInspection,
  CultureStats,
  DpdConfig,
  DpdInspection,
  DpdStats,
  EthnoConfig,
  EthnoInspection,
  EthnoStats,
  ImageConfig,
  ImageInspection,
  ImageStats,
  OpinionsConfig,
  OpinionsInspection,
  OpinionsStats,
  StructureConfig,
  StructureInspection,
  StructureStats,
  NormsConfig,
  NormsInspection,
  NormsStats,
  Param,
  Preset,
  Snapshot,
  TagsConfig,
  TagsInspection,
  TagsStats,
} from './types';
import { InspectPanel } from './ui/inspect-panel';
import { MODEL_CHARTS } from './ui/series-data';
import { config_series_names, initSync, model_schemas_json, presets_json, run_point, sweep_points } from './wasm-pkg/sugarscape.js';

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

  it('loads a theft world in Strategy with every cache shown, and drawing them all reaches the native golden', async () => {
    const e = await engine();
    expect(await e.loadPreset('theft-winter-half', 1)).toBeNull();
    expect(e.colorMode).toBe('strategy');
    expect(e.overlays.caches).toBe(true);
    e.want(() => ({ charts: { groups: [['population']], max: 50 } }));
    await e.advance(150);
    expect(e.minds?.winter).toBe(true);
    expect(e.cacheSites.length).toBeGreaterThan(0);
    const [x, y, total] = e.cacheSites;
    await e.select(x, y);
    const view = e.inspection!.view as { site: { caches: { amount: number }[] } };
    expect(view.site.caches.reduce((sum, c) => sum + c.amount, 0)).toBeCloseTo(total);
    e.setDisplay({ colorMode: 'caching_rule' });
    await e.advance(25);
    e.setDisplay({ colorMode: 'memory' });
    await e.advance(25);
    expect(e.tick).toBe(200);
    expect(e.cacheSites.length).toBeGreaterThan(0);
    // crates/sugarscape-core/tests/golden.rs: theft-winter-half after 200 ticks from seed 1.
    expect(await e.fingerprint()).toBe('0xd0ee29237c28952f');
  });

  it('picks each Minds world’s default color mode and keeps the caches overlay off once turned off by hand', async () => {
    const e = await engine();
    expect(await e.loadPreset('cache-winter-mixed', 1)).toBeNull();
    expect(e.colorMode).toBe('caching_rule');
    expect(await e.loadPreset('cache-winter-even', 1)).toBeNull();
    expect(e.colorMode).toBe('memory');
    expect(await e.loadPreset('theft-winter', 1)).toBeNull();
    expect(e.colorMode).toBe('memory');
    expect(e.overlays.caches).toBe(true);
    e.setDisplay({ overlays: { caches: false } });
    expect(await e.loadPreset('theft-winter-half', 1)).toBeNull();
    expect([e.colorMode, e.overlays.caches]).toEqual(['strategy', false]);
    expect(e.cacheSites.length).toBe(0);
    expect(await e.loadPreset('ii-2-unit', 1)).toBeNull();
    expect([e.colorMode, e.overlays.caches]).toEqual(['tribe', false]);
  });

  it('seeks through keyframes to the golden world, with edits on both sides of the target', async () => {
    const reference = await engine();
    await reference.advance(60);
    await reference.place(3, 3, {});
    await reference.advance(140);
    const want = await reference.fingerprint();

    const e = await engine();
    await e.advance(60);
    await e.place(3, 3, {});
    await e.advance(240);
    for (const t of [200, 61, 60, 59, 0]) {
      await e.seek(t);
      expect(e.tick).toBe(t);
    }
    await e.seek(0);
    await e.advance(200); // replays the edit at 60
    expect(await e.fingerprint()).toBe(want);
    await e.seek(150);
    await e.seek(200);
    expect(await e.fingerprint()).toBe(want);
  });

  it('shares a session ended mid-replay: the branch dropped by endReplay stays dropped through a share link', async () => {
    const e = await engine();
    await e.advance(60);
    await e.place(3, 3, {});
    await e.advance(140);
    await e.seek(20);
    await e.endReplay(); // drops the place at 60 and everything reached past tick 20
    await e.advance(130);
    await e.seek(120);
    const { session } = await e.session();
    expect(session.log).toEqual([]);
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(120);
    expect(await opened.fingerprint()).toBe(await e.fingerprint());
  });

  it('shares a session taken after a seek back', async () => {
    const e = await engine();
    await e.advance(50);
    await e.place(4, 4, {});
    await e.advance(100);
    await e.seek(20);
    const { session } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(150);
    await e.advance(130);
    expect(await opened.fingerprint()).toBe(await e.fingerprint());
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
    'shares a session recorded at mixed speeds, seeked back and branched with a new edit, through a share link',
    async () => {
      const live = await create(6);
      await run(live, 1);
      await run(live, 5, 6);
      const midTick = live.tick;
      await run(live, 2, 8);
      expect(live.tick).toBeGreaterThan(midTick);
      // Seek back into the run just made, then branch it with a fresh edit: what follows here
      // differs from what `live` logged the first time past `midTick`.
      expect(await live.seek(midTick)).toBeNull();
      const spot = await emptySite(live, 3, 3);
      expect(await live.place(spot.x, spot.y, {})).toBeNull();
      await run(live, 3, 5);
      await run(live, 'max');
      const finalTick = live.tick;
      const expected = await live.fingerprint();

      const { session } = await live.session();
      const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
      await reach(opened, finalTick);
      expect(opened.tick).toBe(finalTick);
      expect(await opened.fingerprint()).toBe(expected);
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
    ['nm-1a-static', '0x5a72fde83bfb7283'],
    ['nm-1b-chaos', '0x8360c871dcead14f'],
    ['nm-2a-universal', '0x76ee3f7ba596e8ce'],
    ['nm-3-kaleidoscope', '0xedd9930343121ef9'],
    ['nm-no-self', '0x8feb7f382db6cbb5'],
    ['nm-four-neighbors', '0xfd39a3d611f99c43'],
    ['hg-async-kaleidoscope', '0xef13c172a34a980d'],
    ['nbm-probabilistic', '0x79a048f606d28d74'],
    ['nbm-discrete', '0x2befee9be89b26d9'],
    ['nbm-continuous', '0x186717b420e5af31'],
    ['nbm-random-array', '0xcf2c74041806d530'],
    ['nbm-cube', '0x96baab61504e7923'],
    ['rca-published', '0x1c83900b9b9f0b94'],
    ['rca-literal', '0x8cc0a69cf4d14caa'],
    ['rca-published-p2', '0x3f91b546c9734d3d'],
    ['rca-literal-p2', '0x54380d2809b3fd62'],
    ['rca-strict', '0x7b729c41a4436372'],
    ['rs-no-forced-clones', '0x9dffef2174ca7415'],
    ['eh-clones-only', '0x1bd933620c915c0e'],
    ['eh-no-exact-clones', '0xafe94d6c0bb3b8ab'],
    ['rca-adopt-p1', '0x0c3f75d53d237a89'],
    ['ac-sample-run', '0xeb302b62eb20f85d'],
    ['ac-many-regions', '0xcfca4ee65fe3946c'],
    ['ac-torus', '0x8a21fc496bea71c3'],
    ['ac-random-activation-20', '0xf0c8269aa3a3f7d2'],
    ['ac-sweep-activation', '0x986f6c9a01898253'],
    ['ac-neighbor-changes', '0xb12313a2dedfda7e'],
    ['ac-soup', '0xe15b8cab349e25fa'],
    ['ac-drift', '0xf254ab408f46810f'],
    ['aey-equity', '0x6aa634df2b9c3944'],
    ['aey-fractious', '0xd6b378e369c0066e'],
    ['aey-tags', '0x1455ea172db78c68'],
    ['aey-classes', '0xe2a0783a5fc6734c'],
    ['pvplh-small-tags', '0x03ca181d96a1c667'],
    ['pvplh-mode', '0xb90f0d0cab7966b9'],
    ['pvplh-progressive', '0xca60c420d61a7ffc'],
    ['pvplh-lattice', '0x7975f5afc2d06c10'],
    ['ha-standard', '0xf07433e56417f07c'],
    ['ha-figure-1', '0x843632b62ddf7a6b'],
    ['ha-appendix-mutation', '0xae8c7eda9113dae8'],
    ['ha-appendix-double-play', '0xac2c2167fec9c326'],
    ['ha-java-five-colors', '0xdc78c1e27b9ab453'],
    ['ha-java-archive', '0xde2cff652c758fe7'],
    ['ha-egoist-start', '0xabfdf5c9e1ccdb45'],
    ['ha-cost-2', '0x41ba53998a8ee613'],
    ['ha-cost-2-blind', '0x699aa05497139005'],
    ['ha-misperception', '0x5567187174fd1c15'],
    ['ha-each-color', '0x9ad570c3ea183419'],
    ['jansson-offspring-anywhere', '0xcad22f8e7abafbfe'],
    ['jansson-tag-mutation-30', '0xf9dbf338238a8f1b'],
    ['jansson-kin', '0x265998639eacfbd0'],
    ['jansson-kin-fixed', '0x3fac090571612879'],
    ['hks-no-ethnocentrics', '0xbe867e7210bad2d2'],
    ['hk-lattice', '0xe33359f120b204d9'],
    ['cra-rwr', '0xc7f45f59d9b25490'],
    ['cra-2dk', '0x3b8c19aab4aae805'],
    ['cra-frne', '0xdf2fc96965742a81'],
    ['cra-frn', '0x924d4b2fe686ae18'],
    ['cra-ffr-01', '0x424eda2182150e01'],
    ['cra-ffr-03', '0xbc09206184dc7003'],
    ['cra-ffr-05', '0xf9573021f9848025'],
    ['cra-random-start', '0x1871afd34df774a9'],
    ['cra-copy-noise', '0xd8871da3505ee758'],
    ['dpd-run-1', '0x3d64b053fbfee4f6'],
    ['dpd-run-2', '0xe0198124ac5f4789'],
    ['dpd-run-3', '0x1c819cc85b7bb351'],
    ['dpd-run-4', '0xe6b5d66dee22ce07'],
    ['dpd-run-5', '0x2f5ae2bdc6bd257a'],
    ['dpd-working-paper', '0xaba834120f15810b'],
    ['dpd-closest', '0xd1c7475297f9864c'],
    ['dpd-soup', '0x0db64ad16a49146b'],
    ['dpd-shifted', '0x39b59d546b2bb2e8'],
    ['dpd-metabolism', '0x7b580a83419a3ea4'],
    ['dpd-footnote-27', '0xd1adeadce6881068'],
    ['dpd-rr-best', '0xe8fdc4ce027dd236'],
    ['dpd-coordination', '0x47f8c68504a9157a'],
    ['gi-metanorms-long', '0xf80d7b08d057046f'],
    ['gi-low-mutation', '0x2c37f146a39f0d51'],
    ['gi-mild-metanorms', '0xb7435abcb67c192b'],
    ['gi-temptation-10', '0xa191ff11a4f9ee68'],
    ['gi-tournament', '0x95ea76458cee1a46'],
    // Relative agreement presets still running at period 200 on seed 1 (the others settle sooner).
    ['dnaw-lattice', '0x60f9414e1157482d'],
    ['dnaw-lattice-clusters', '0x459f1c2e1f67651b'],
    ['ra-central', '0xcd795ff8a5bd9e44'],
    ['ra-literal', '0x2a130dc7026094f7'],
    ['ra-deffuant-2013', '0x4b9cf5817bc155d6'],
    ['ra-bc-extremists', '0xc304b87400a0bb46'],
    ['ad-moore', '0xd39d77107d873634'],
    ['ad-small-world', '0xd29498f3ac055d5d'],
    ['w-scale-free', '0xed9e58b01a7987d8'],
    ['ef-arthur', '0x21d68cd385f4c107'],
    ['ef-payoff', '0xfc1e2e95d6261a60'],
    ['ef-random', '0x0bf2299aeb9dfc6a'],
    ['mg-m6', '0x3aa1d7ced39f99c8'],
    ['mg-inverse', '0xf9b094733c6a48aa'],
    ['mg-arms-race', '0x4c1e9852373241c9'],
    ['cmo-binary', '0x2081105c24039d0c'],
    ['ants-2b', '0xf9258e5dd9d1d673'],
    ['gr-normal-sampled', '0xa9bca3821a0f4774'],
    ['gr-city', '0xa587dcb16521cf3c'],
    ['gr-friends', '0x37ae8bba4d07be7c'],
    ['gr-ceilings', '0x1cc528db96973a5a'],
    ['gr-clusters', '0xd493a3251cde5fbe'],
    ['watts-middle', '0x1ed3157ee5e9ac61'],
    ['ants-becker', '0x4ebaae97020b8902'],
    ['ants-three', '0x4a2871368f4ab782'],
    ['am-ring', '0x0104a02f3c5f5011'],
    ['am-random', '0x3e7b4009dc784788'],
    ['am-independent', '0x023127da92f6506f'],
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

describe('image scoring’s golden fingerprints', () => {
  // crates/sugarscape-core/tests/golden.rs IMAGE_GOLDEN (Decision 17): one group 200 generations,
  // islands 20. The WASM prints 16 hex digits, so ns-fig-2 and ns-fig-4b keep their leading zero.
  const IMAGE_GOLDEN: [string, number, string][] = [
    ['ns-fig-1', 200, '0x98875bd71738cf05'],
    ['ns-fig-2', 200, '0x091995405bae5fd9'],
    ['ns-fig-3-n20', 200, '0xb4bd11bc229673c4'],
    ['ns-fig-3-n50', 200, '0x5907ce5cb47602cf'],
    ['ns-fig-3-n100', 200, '0xaf771beff4611d0d'],
    ['ns-fig-4a', 200, '0xe7d6cefc18bd8d6a'],
    ['ns-fig-4b', 200, '0x0a4c19c5fa5fcfe3'],
    ['ns-fig-4c', 200, '0x64755eafb526a816'],
    ['ns-fig-4d', 200, '0x72ca0b1d80947f44'],
    ['ns-own-only', 200, '0x20d6768b1fbd7081'],
    ['ns-no-offset', 200, '0xc595349de0c0bf65'],
    ['lh-fig-1a', 20, '0x46669de796924497'],
    ['lh-fig-1b', 20, '0x2c3d0dea6af0c894'],
    ['lh-fig-2a', 200, '0x56250416634c6ed9'],
    ['lh-fig-2b', 20, '0x566a697764e9cdd9'],
    ['lh-fig-2c', 20, '0x963ec9f09d2fadbe'],
    ['lh-fig-3a', 20, '0x46059fbe6bab2056'],
    ['lh-fig-3b', 20, '0x4d6575c59b0da89a'],
    ['lh-fig-4a', 20, '0xd5654fa4b600eb57'],
    ['lh-fig-4b', 20, '0x4bfe8ea9f3e8598b'],
    ['lh-fig-4c', 20, '0x6371653b544f6387'],
  ];

  it('covers every image-scoring preset', () => {
    expect(IMAGE_GOLDEN.map(([id]) => id)).toEqual(presets.filter((p) => modelOf(p.config) === 'image').map((p) => p.id));
  });

  it.each(IMAGE_GOLDEN)('%s reproduces its golden fingerprint after %i generations, whatever is watched', async (id, ticks, golden) => {
    const preset = presets.find((p) => p.id === id)!;
    const e = await Engine.create({ config: structuredClone(preset.config), seed: 1 }, { presets, transport: inline() });
    e.want(() => ({ charts: { groups: [['help_rate', 'cooperative']], max: 50 }, lorenz: true, networks: ['trade'] }));
    e.setDisplay({ colorMode: 'score' });
    await e.select(0, 0);
    for (const n of [1, ticks / 2 - 1, ticks / 2]) await e.advance(n);
    expect(e.tick).toBe(ticks);
    expect(await e.fingerprint()).toBe(golden);
  });
});

describe('preset titles', () => {
  it('reach the page for every preset, and none is a figure number or rule notation', () => {
    expect(presets.length).toBeGreaterThan(180);
    for (const p of presets) {
      expect(p.title, p.id).toBeTruthy();
      expect(p.title, p.id).not.toMatch(/^(Animation|Fig\.|Figure|Table )|\(\{/);
    }
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

describe('the norms model through the engine', () => {
  it('stops at its last generation and inspects an agent and a strategy', async () => {
    const r = presets.find((p) => p.id === 'ax-metanorms')!;
    const e = await Engine.create({ config: structuredClone(r.config as NormsConfig), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'payoff' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as NormsStats;
    expect([e.finished, ends, e.tick, s.tick]).toEqual([true, 1, 100, 100]);
    // The first strip row (x 102 onward) is agent 1.
    await e.select(110, 1);
    const v = e.inspection!.view as NormsInspection;
    expect(v.agent!.id).toBe(1);
    expect(v.agent!.bits).toMatch(/^[01]{6}$/);
  });
});

describe('the social-structure model through the engine', () => {
  it('stops at its last period and inspects an agent and its partners', async () => {
    const r = presets.find((p) => p.id === 'cra-2dk')!;
    const config = { ...structuredClone(r.config as StructureConfig), stop_at: 30 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'strategy' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as StructureStats;
    expect([e.finished, ends, e.tick, s.tick]).toEqual([true, 1, 30, 30]);
    // The torus: every agent played its four neighbors twice.
    await e.select(1, 1);
    const v = e.inspection!.view as StructureInspection;
    expect(v.block).toEqual({ x: 0, y: 0 });
    expect(v.agent!.partners.map((p) => p.games)).toEqual([2, 2, 2, 2]);
  });

  it('labels an empty block cell of a non-square population instead of falling through to the gap text', async () => {
    const r = presets.find((p) => p.id === 'cra-rwr')!;
    const config = { ...structuredClone(r.config as StructureConfig), agents: 250, stop_at: 5 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'strategy' });
    await e.advance(1);
    // block side 16, cell 6 px: block cell (15, 15) is index 255, past the 250th agent.
    await e.select(90, 90);
    const v = e.inspection!.view as StructureInspection;
    expect(v.block).toEqual({ x: 15, y: 15 });
    expect(v.agent).toBeNull();

    // A minimal document, just enough for `h()` to build <tr><th>/<td> rows.
    const stubElement = (tag: string) => {
      const el: { tag: string; children: unknown[]; setAttribute: () => void; addEventListener: () => void; append: (...items: unknown[]) => void } = {
        tag,
        children: [],
        setAttribute: () => {},
        addEventListener: () => {},
        append(...items: unknown[]) {
          el.children.push(...items);
        },
      };
      Object.defineProperty(el, 'textContent', {
        get: () => el.children.map((c) => (typeof c === 'string' ? c : (c as { textContent: string }).textContent)).join(''),
      });
      return el;
    };
    vi.stubGlobal('document', { createElement: stubElement });
    try {
      const structureRows = (InspectPanel.prototype as unknown as { structureRows(view: StructureInspection): { textContent: string }[] }).structureRows;
      const text = structureRows(v)
        .map((row) => row.textContent)
        .join(' | ');
      expect(text).toContain('Block cell');
      expect(text).toContain('no agent here');
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

describe('the Minds menu over the real presets', () => {
  it('holds every preset whose source names a Minds milestone, and no other', () => {
    const minds = presets.filter((p) => /\bMinds \d/.test(p.source)).map((p) => p.id);
    expect(minds.length).toBeGreaterThanOrEqual(28);
    expect(presets.filter((p) => presetMenu(p) === 'minds').map((p) => p.id)).toEqual(minds);
  });
});

describe('the bali model through the engine', () => {
  it('stops after its last year and inspects a subak, and a dam in the water strip', async () => {
    const r = presets.find((p) => p.id === 'lk-random')!;
    const config = { ...structuredClone(r.config as BaliConfig), stop_at: 2 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'temple' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as BaliStats;
    expect([e.finished, ends, e.tick, s.year]).toEqual([true, 1, 24, 2]);
    expect(s.harvest).toBeGreaterThan(10);
    // Subak 6 sits at (−17, −1): pixel ((−17 + 23 + 3) × 8, (28 + 1 + 3) × 8).
    await e.select(72, 256);
    const v = e.inspection!.view as BaliInspection;
    expect([v.panel, v.subak?.id, v.subak?.masceti]).toEqual(['map', 6, 10]);
    // The strip starts 8 pixels below the 512-pixel map; its first row is dam 0.
    await e.select(5, 521);
    const w = e.inspection!.view as BaliInspection;
    expect([w.panel, w.dam?.id]).toEqual(['strip', 0]);
  });
});

describe('the zi model through the engine', () => {
  it('stops after its last period and inspects a step, a trade and a trader', async () => {
    const r = presets.find((p) => p.id === 'gs-1')!;
    const config = { ...structuredClone(r.config as ZiConfig), shouts: 200, stop_at: 3 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'profit' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as ZiStats;
    expect([e.finished, ends, e.tick, s.period]).toEqual([true, 1, 600, 4]);
    expect(s.last_efficiency).toBeGreaterThan(50);
    await e.select(0, 100);
    const v = e.inspection!.view as ZiInspection;
    expect([v.panel, v.unit, v.demand, v.supply]).toEqual(['schedules', 1, 102, 34]);
    await e.select(5, 240);
    expect((e.inspection!.view as ZiInspection).trader?.id).toBe(1);
  });
});

describe('the punishment model through the engine', () => {
  it('stops at its last period and inspects an agent, its group and a period', async () => {
    const r = presets.find((p) => p.id === 'bg-base')!;
    const config = { ...structuredClone(r.config as PunishmentConfig), groups: 16, size: 8, stop_at: 50, window: 20 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'acts' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as PunishmentStats;
    expect([e.finished, ends, e.tick]).toEqual([true, 1, 50]);
    expect(s.long_run).not.toBeNull();
    expect(s.contributors + s.punishers + s.defectors).toBeCloseTo(1, 9);
    // 16 groups of 8: six groups a row, 3 × 3 cells of 8 pixels, the first at (2, 2).
    await e.select(3, 3);
    const v = e.inspection!.view as PunishmentInspection;
    expect([v.panel, v.agent?.id, v.group?.index]).toEqual(['groups', 1, 0]);
    await e.select(10, 100);
    expect((e.inspection!.view as PunishmentInspection).panel).toBe('time');
  });
});

describe('the retirement model through the engine', () => {
  it('stops at the norm and inspects an agent, an age and a period', async () => {
    const r = presets.find((p) => p.id === 'ae-rapid')!;
    const config = { ...structuredClone(r.config as RetirementConfig), stop_at_norm: true };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'type' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as RetirementStats;
    expect([e.finished, ends, s.transition]).toEqual([true, 1, e.tick]);
    expect(s.retired).toBeGreaterThanOrEqual(0.95);
    // Row 45 (2 pixels an age) is age 65.
    await e.select(0, 90);
    const v = e.inspection!.view as RetirementInspection;
    expect([v.panel, v.age]).toEqual(['population', 65]);
    expect(e.inspection!.agentId).toBeNull();
    await e.select(410, 90);
    expect((e.inspection!.view as RetirementInspection).panel).toBe('ages');
  });
});

describe('the thresholds model through the engine', () => {
  it('stops at its last step and inspects an actor, a step and Figure 1', async () => {
    const r = presets.find((p) => p.id === 'gr-uniform')!;
    const config = { ...structuredClone(r.config as ThresholdsConfig), stop_at: 50 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'threshold' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as ThresholdsStats;
    expect([e.finished, ends, e.tick, s.tick, s.acting]).toEqual([true, 1, 50, 50, 0.5]);
    // The actor grid starts at x 518; actor 1 (threshold 0) is its top-left cell.
    await e.select(518, 0);
    const v = e.inspection!.view as ThresholdsInspection;
    expect([v.panel, v.member!.id, v.member!.threshold, v.member!.acting]).toEqual(['actors', 1, 0, true]);
    expect(e.inspection!.agentId).toBeNull();
    await e.select(459, 100);
    const f = e.inspection!.view as ThresholdsInspection;
    expect([f.panel, f.cdf]).toEqual(['figure', 0.51]);
  });
});

describe('the ants model through the engine', () => {
  it('stops at its last step and inspects an ant, a step and a histogram row', async () => {
    const r = presets.find((p) => p.id === 'am-ring')!;
    const config = { ...structuredClone(r.config as AntsConfig), stop_at: 30 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'degree' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as AntsStats;
    expect([e.finished, ends, e.tick, s.tick]).toEqual([true, 1, 30, 30]);
    expect(s.theory_variance).toBeGreaterThan(0);
    // The ant grid starts at x 518; ant 1 is its top-left cell.
    await e.select(518, 0);
    const v = e.inspection!.view as AntsInspection;
    expect([v.panel, v.member!.id, v.member!.degree]).toEqual(['ants', 1, 10]);
    expect(e.inspection!.agentId).toBeNull();
    await e.select(29, 0);
    const t = e.inspection!.view as AntsInspection;
    expect([t.step, t.shares!.length]).toEqual([30, 2]);
    await e.select(409, 100);
    expect((e.inspection!.view as AntsInspection).panel).toBe('histogram');
  });
});

describe('the El Farol model through the engine', () => {
  it('stops at its last round and inspects an agent and a round', async () => {
    const r = presets.find((p) => p.id === 'ef-arthur')!;
    const config = { ...structuredClone(r.config as FarolConfig), stop_at: 30 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'strategy' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as FarolStats;
    expect([e.finished, ends, e.tick, s.tick]).toEqual([true, 1, 30, 30]);
    // The agent grid starts at x 358; agent 1 is its top-left cell.
    await e.select(358, 0);
    const v = e.inspection!.view as FarolInspection;
    expect([v.panel, v.member!.id, v.member!.strategies.length]).toEqual(['agents', 1, 12]);
    expect(v.member!.strategies.filter((x) => x.active).length).toBe(1);
    expect(e.inspection!.agentId).toBeNull();
    await e.select(29, 0);
    expect((e.inspection!.view as FarolInspection).round).toBe(30);
  });

  const farolPresets = presets.filter((p) => modelOf(p.config) === 'farol');
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).farol;

  it('starts an Experiments sweep every game accepts', () => {
    expect(farolPresets).toHaveLength(16);
    for (const p of farolPresets) {
      const { sweep, errors } = formToSweep(defaultForm('farol', p.config), { preset: p.id });
      expect(errors, p.id).toEqual([]);
      expect(() => sweep_points(JSON.stringify(sweep)), p.id).not.toThrow();
    }
  });

  it('shows each game’s fields only under that game', () => {
    for (const p of farolPresets) {
      const game = (p.config as FarolConfig).game;
      for (const f of schema.filter((f) => paramShown(f, p.config))) {
        if (f.group === 'El Farol') expect(game, `${p.id}: ${f.path}`).toBe('el_farol');
        if (f.group === 'Minority game' || f.group === 'Evolution') expect(game, `${p.id}: ${f.path}`).toBe('minority');
      }
    }
  });

  it('keeps the plain minority game when the number of players changes', async () => {
    const mg = presets.find((p) => p.id === 'mg-m6')!;
    expect((mg.config as FarolConfig).capacity).toBeNull();
    const config = { ...structuredClone(mg.config as FarolConfig), agents: 501 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    await e.advance(200);
    // Coin-flippers' σ²/N of the plain game is ¼, centered on N/2.
    expect((e.latest as FarolStats).random_fluctuation).toBeCloseTo(0.25, 6);
  });
});

describe('the relative agreement model through the engine', () => {
  it('stops once when stable, matches the native golden entry and inspects a column and a dot', async () => {
    const r = presets.find((p) => p.id === 'ra-single')!;
    const e = await Engine.create({ config: structuredClone(r.config as AgreementConfig), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'role' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as AgreementStats;
    // Fig. 7's stated parameters: both extremes (outcome 1), not the figure's single extreme.
    expect([e.finished, ends, e.tick, s.stable_at, s.outcome]).toEqual([true, 1, 71, 71, 1]);
    // crates/sugarscape-core/tests/golden.rs: ra-single stops at period 71 of its 200.
    expect(await e.fingerprint()).toBe('0x769843f421a9f63e');
    await e.select(0, 100);
    const v = e.inspection!.view as AgreementInspection;
    expect([v.panel, v.period, v.opinion]).toEqual(['diagram', 0, 0]);
    expect(e.inspection!.agentId).toBeNull();
    await e.select(249 + 100, 100);
    expect((e.inspection!.view as AgreementInspection).panel).toBe('scatter');
  });

  it('stops at Meadows and Cliff’s horizon', async () => {
    const r = presets.find((p) => p.id === 'ra-meadows-cliff')!;
    const e = await Engine.create({ config: structuredClone(r.config as AgreementConfig), seed: 1 }, { presets, transport: inline() });
    await e.advance(1_000_000);
    const s = e.latest as AgreementStats;
    expect([e.finished, e.tick, s.y]).toEqual([true, 200, 0]);
  });
});

describe('the bounded-confidence model through the engine', () => {
  it('stops once when stable, matches the native golden entry and inspects a line', async () => {
    const r = presets.find((p) => p.id === 'hk-regular-50')!;
    const e = await Engine.create({ config: structuredClone(r.config as OpinionsConfig), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'opinion' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as OpinionsStats;
    expect([e.finished, ends, e.tick, s.stable_at, s.clusters]).toEqual([true, 1, 8, 8, 2]);
    // crates/sugarscape-core/tests/golden.rs: hk-regular-50 stops at period 8 of its 200.
    expect(await e.fingerprint()).toBe('0xbfd3a4ebe6714f04');
    // Agent 1 starts at 0: the diagram's bottom row, in the column of the oldest kept period.
    await e.select(0, 200);
    const v = e.inspection!.view as OpinionsInspection;
    expect([v.period, v.opinion, v.agents[0].id, v.agents[0].start]).toEqual([0, 0, 1, 0]);
    expect(e.inspection!.agentId).toBeNull();
  });
});

describe('the classes model through the engine', () => {
  it('stops once at equity and inspects a simplex point', async () => {
    const t = presets.find((p) => p.id === 'aey-transition')!;
    const e = await Engine.create({ config: structuredClone(t.config as ClassesConfig), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'payoff' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(1_000_000);
    const s = e.latest as ClassesStats;
    expect([e.finished, ends, s.equity_at]).toEqual([true, 1, e.tick]);
    // Everyone remembering mostly M sits near the M vertex, bottom left.
    let found: ClassesInspection | null = null;
    for (let x = 0; x < 30 && !found; x++) {
      await e.select(x, 104);
      const v = e.inspection!.view as ClassesInspection;
      if (v.agents.length > 0) found = v;
    }
    expect(found!.simplex).toBe('one');
    expect(found!.best_reply).toBe('M');
    expect(e.inspection!.agentId).toBeNull();
  });
});

describe('the culture model through the engine', () => {
  it('stops once, at the tick the lattice becomes stable, and inspects sites and lanes', async () => {
    const sample = presets.find((p) => p.id === 'ac-sample-run')!;
    const e = await Engine.create({ config: structuredClone(sample.config), seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'similarity' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(100_000);
    const s = e.latest as CultureStats;
    expect([e.finished, ends, s.stable_at, s.active_bonds]).toEqual([true, 1, e.tick, 0]);
    expect(s.regions).toBe(s.zones);
    await e.select(0, 0);
    const site = e.inspection!.view as CultureInspection;
    expect([site.kind, site.a.traits.length, site.neighbors.length]).toEqual(['site', 5, 2]);
    await e.select(2, 0);
    expect((e.inspection!.view as CultureInspection).kind).toBe('lane');
    expect(e.inspection!.agentId).toBeNull();
  });

  it('runs the activation Compare pair until both lattices are stable, not just the first', async () => {
    const make = (id: string) =>
      Engine.create({ config: structuredClone(presets.find((p) => p.id === id)!.config), seed: 1 }, { presets, transport: inline() });
    const [a, b] = [await make('ac-random-activation-20'), await make('ac-sweep-activation')];
    const lock = new Lockstep([a, b], 'max', () => 0);
    await lock.settled();
    await lock.advance(6000);
    expect([a.tick, b.tick]).toEqual([6000, 6000]);
    const [sa, sb] = [a.latest as CultureStats, b.latest as CultureStats];
    expect(sa.stable_at).toBeLessThan(6000);
    expect(sb.stable_at).toBeLessThan(6000);
    expect(sa.stable_at).not.toBe(sb.stable_at);
  });

  it('reproduces the docking presets’ golden fingerprints in the Sugarscape', async () => {
    // crates/sugarscape-core/tests/golden.rs, GOLDEN.
    for (const [id, golden] of [
      ['dock-mobility-15', '0x3361a01b1a7cd6a3'],
      ['dock-mobility-30', '0x19cfa4ca0800089e'],
    ]) {
      const preset = presets.find((p) => p.id === id)!;
      const e = await Engine.create({ config: structuredClone(preset.config), seed: 1 }, { presets, transport: inline() });
      e.setDisplay({ colorMode: 'culture' });
      await e.advance(200);
      expect(await e.fingerprint()).toBe(golden);
      expect((e.latest as Snapshot).axelrod?.distinct_cultures).toBeGreaterThan(0);
    }
  });
});

describe('the tags model through the engine', () => {
  const published = presets.find((p) => p.id === 'rca-published')!;

  it('stops at its last generation, once, and shows the newest row’s agents', async () => {
    const config = { ...structuredClone(published.config as TagsConfig), end: 60 };
    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
    e.setDisplay({ colorMode: 'tolerance' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(100);
    expect([e.tick, e.finished, ends]).toEqual([60, true, 1]);
    const s = e.latest as TagsStats;
    expect(s.donation_rate).toBeGreaterThan(0);
    expect(s.cluster_share).toBeGreaterThan(0);
    let found: TagsInspection | null = null;
    for (let x = 0; x < 100 && !found; x++) {
      await e.select(x, 199);
      const v = e.inspection!.view as TagsInspection;
      if (v.count > 0) found = v;
    }
    expect(found!.generation).toBe(60);
    expect(found!.agents).toHaveLength(found!.count);
    expect(e.inspection!.agentId).toBeNull();
  });
});

describe('the ethnocentrism model through the engine', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const create = (id: string, edit: (c: EthnoConfig) => void = () => {}) => {
    const config = structuredClone(preset(id).config) as EthnoConfig;
    edit(config);
    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
  };
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).ethno;
  const field = (path: string) => schema.find((p) => p.path === path)!;

  it('builds a Rules panel without `allowed`, with a nullable tag mutation and the kin fields shown only with kin strategies', () => {
    expect(schema.some((p) => p.path === 'allowed')).toBe(false);
    expect(schema.filter((p) => p.nullable).map((p) => p.path)).toEqual(['tag_mutation']);
    expect(field('misperception').show_if).toEqual({ path: 'discrimination', equals: 'same_other' });
    for (const path of ['kin_basis', 'kin_mutation']) {
      const p = field(path);
      expect([paramShown(p, preset('jansson-kin').config), paramShown(p, preset('ha-standard').config)]).toEqual([true, false]);
    }
  });

  it('keeps a restricted `allowed` through live edits, resets and a share link', async () => {
    const e = await create('hks-no-ethnocentrics');
    const allowed = () => (e.config as EthnoConfig).allowed;
    expect(allowed()).toEqual(['H', 'S', 'T']);
    await e.advance(20);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).cost = 0.012))).toBeNull();
    expect(allowed()).toEqual(['H', 'S', 'T']);
    expect(await e.resetModelWith((c) => void ((c as EthnoConfig).colors = 3))).toBeNull();
    expect(allowed()).toEqual(['H', 'S', 'T']);
    await e.advance(60);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).tag_mutation = 0.3))).toBeNull();
    await e.advance(20);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).tag_mutation = null))).toBeNull();
    expect((e.config as EthnoConfig).tag_mutation).toBeNull();
    await e.advance(20);
    expect((e.latest as EthnoStats).ethnocentric).toBe(0);
    const { session, tick } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(tick);
    expect((opened.config as EthnoConfig).allowed).toEqual(['H', 'S', 'T']);
    expect(await opened.fingerprint()).toBe(await e.fingerprint());
  });

  it('stops at its last period, once, and inspects agents and empty sites', async () => {
    const e = await create('jansson-kin', (c) => (c.end = 300));
    e.setDisplay({ colorMode: 'ptr' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(400);
    expect([e.tick, e.finished, ends]).toEqual([300, true, 1]);
    expect((e.latest as EthnoStats).population).toBeGreaterThan(0);
    const seen = { agent: 0, empty: 0 };
    for (let x = 0; x < 50; x++) {
      await e.select(x, 25);
      const v = e.inspection!.view;
      expect(isEthnoView(v, e.model)).toBe(true);
      const a = (v as EthnoInspection).agent;
      if (a) {
        seen.agent++;
        expect(e.inspection!.agentId).toBe(a.id);
        expect(a.neighbors.length).toBeLessThanOrEqual(4);
      } else seen.empty++;
    }
    expect(seen.agent).toBeGreaterThan(0);
    expect(seen.empty).toBeGreaterThan(0);
  });

  it('opens its three Compare entries and sweeps its default form', () => {
    for (const id of ['ha-four-vs-five', 'ha-adjacent-vs-anywhere', 'ha-tags-vs-kin']) {
      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
    }
    const { sweep, errors } = formToSweep(defaultForm('ethno'), { preset: 'ha-standard' });
    expect(errors).toEqual([]);
    // Eleven costs × the form's three seeds (the built-in ha-cost runs ten).
    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(33);
  });
});

describe('the demographic PD through the engine', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const dpdPresets = presets.filter((p) => modelOf(p.config) === 'dpd');
  const create = (id: string, edit: (c: DpdConfig) => void = () => {}) => {
    const config = structuredClone(preset(id).config) as DpdConfig;
    edit(config);
    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
  };
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).dpd;
  const field = (path: string) => schema.find((p) => p.path === path)!;

  it('builds a Rules panel in the spec’s groups, with negative payoffs on the sliders and the play rule only in space', () => {
    expect([...new Set(schema.map((p) => p.group))]).toEqual(['Game', 'Population', 'Evolution', 'Timing', 'Interaction', 'Run']);
    for (const path of ['t', 'r', 'p', 's']) expect([field(path).min, field(path).max]).toEqual([-20, 20]);
    expect(field('play').show_if).toEqual({ path: 'pairing', equals: 'space' });
    expect([paramShown(field('play'), preset('dpd-run-1').config), paramShown(field('play'), preset('dpd-soup').config)]).toEqual([true, false]);
    // Every preset's numbers sit on their sliders (the coordination game's −3, footnote 27's 16, max age 1,000).
    expect(dpdPresets).toHaveLength(13);
    for (const p of dpdPresets) {
      for (const f of schema.filter((f) => f.kind === 'number' || f.kind === 'integer')) {
        const v = (p.config as unknown as Record<string, number>)[f.path];
        expect(v, `${p.id}: ${f.path}`).toBeGreaterThanOrEqual(f.min!);
        expect(v, `${p.id}: ${f.path}`).toBeLessThanOrEqual(f.max!);
      }
    }
  });

  it('takes live edits of the payoffs and the pairing, and replays them through keyframes and a share link', async () => {
    const e = await create('dpd-rr-best');
    await e.advance(40);
    expect(await e.applyModelConfig((c) => void ((c as DpdConfig).s = -3))).toBeNull();
    await e.advance(40);
    expect(await e.applyModelConfig((c) => void ((c as DpdConfig).pairing = 'soup'))).toBeNull();
    await e.advance(20);
    const want = await e.fingerprint();
    expect(e.tick).toBe(100);
    await e.seek(30);
    await e.seek(100);
    expect(await e.fingerprint()).toBe(want);
    const { session, tick } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(tick);
    expect((opened.config as DpdConfig).pairing).toBe('soup');
    expect(await opened.fingerprint()).toBe(want);
    const errors = await e.applyModelConfig((c) => void ((c as DpdConfig).mutation = 2));
    expect(errors?.map((f) => f.field)).toEqual(['mutation']);
  });

  it('stops at its last cycle, once, and inspects agents, surrounded cooperators and empty sites', async () => {
    const e = await create('dpd-run-1', (c) => (c.end = 120));
    e.setDisplay({ colorMode: 'surrounded' });
    // At the start 100 agents hold 900 sites: an empty one is an empty demographic PD site.
    const empties: number[] = [];
    for (let x = 0; x < 30; x++) {
      await e.select(x, 0);
      const v = e.inspection!.view;
      expect(isDpdView(v, e.model)).toBe(true);
      if ((v as DpdInspection).agent === null) empties.push(x);
    }
    expect(empties.length).toBeGreaterThan(20);
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(200);
    expect([e.tick, e.finished, ends]).toEqual([120, true, 1]);
    const stats = e.latest as DpdStats;
    expect(stats.cooperators + stats.defectors).toBe(stats.population);
    const seen = { agent: 0, surrounded: 0, empty: 0 };
    for (let y = 0; y < 30; y++) {
      for (let x = 0; x < 30; x += 3) {
        await e.select(x, y);
        const v = e.inspection!.view;
        expect(isDpdView(v, e.model)).toBe(true);
        const a = (v as DpdInspection).agent;
        if (!a) {
          seen.empty++;
          continue;
        }
        seen.agent++;
        expect(e.inspection!.agentId).toBe(a.id);
        expect(a.neighbors.length).toBeLessThanOrEqual(4);
        if (a.surrounded) {
          seen.surrounded++;
          expect(a.strategy).toBe('C');
          expect(a.neighbors.map((n) => n.strategy)).toEqual(['C', 'C', 'C', 'C']);
        }
      }
    }
    expect(seen.agent).toBeGreaterThan(0);
    expect(seen.surrounded).toBeGreaterThan(0);
    // By then the lattice is all but full (Run 1 holds about 900 agents).
    expect(seen.agent).toBeGreaterThan(290);
  });

  it('opens its four Compare entries and sweeps its default form', () => {
    for (const id of ['dpd-wp-vs-published', 'dpd-negative-vs-metabolism', 'dpd-space-vs-soup', 'dpd-published-vs-closest']) {
      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
    }
    const { sweep, errors } = formToSweep(defaultForm('dpd'), { preset: 'dpd-run-1' });
    expect(errors).toEqual([]);
    // Five rewards × the form's three seeds.
    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(15);
  });
});

describe('image scoring through the engine', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const imagePresets = presets.filter((p) => modelOf(p.config) === 'image');
  const create = (id: string, edit: (c: ImageConfig) => void = () => {}) => {
    const config = structuredClone(preset(id).config) as ImageConfig;
    edit(config);
    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
  };
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).image;
  const field = (path: string) => schema.find((p) => p.path === path)!;

  it('builds a Rules panel in the spec’s groups, with observers only under observers and every preset on its sliders', () => {
    expect([...new Set(schema.map((p) => p.group))]).toEqual(['Game', 'Population', 'Rounds', 'Information', 'Errors', 'Evolution', 'Run']);
    // Presets, files and links set the strategies and the start (as ethnocentrism's allowed strategies).
    expect(schema.map((p) => p.path)).not.toContain('strategies');
    expect(schema.map((p) => p.path)).not.toContain('initial');
    expect(field('observers').show_if).toEqual({ path: 'information', equals: 'observers' });
    expect([paramShown(field('observers'), preset('ns-fig-1').config), paramShown(field('observers'), preset('ns-fig-3-n20').config)]).toEqual([false, true]);
    expect(imagePresets).toHaveLength(21);
    for (const p of imagePresets) {
      for (const f of schema.filter((f) => f.kind === 'number' || f.kind === 'integer')) {
        const v = (p.config as unknown as Record<string, number>)[f.path];
        expect(v, `${p.id}: ${f.path}`).toBeGreaterThanOrEqual(f.min!);
        expect(v, `${p.id}: ${f.path}`).toBeLessThanOrEqual(f.max!);
      }
    }
  });

  it('takes live edits of the cost, the records and the observers, and replays them through keyframes and a share link', async () => {
    const e = await create('ns-fig-3-n20');
    await e.advance(30);
    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).c = 0.2))).toBeNull();
    await e.advance(30);
    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).records = 'score'))).toBeNull();
    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).observers = 5))).toBeNull();
    await e.advance(20);
    const want = await e.fingerprint();
    expect(e.tick).toBe(80);
    await e.seek(20);
    await e.seek(80);
    expect(await e.fingerprint()).toBe(want);
    const { session, tick } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(tick);
    expect([(opened.config as ImageConfig).records, (opened.config as ImageConfig).observers]).toEqual(['score', 5]);
    expect(await opened.fingerprint()).toBe(want);
    const errors = await e.applyModelConfig((c) => void ((c as ImageConfig).mutation = 2));
    expect(errors?.map((f) => f.field)).toEqual(['mutation']);
  });

  it('stops at its last generation, once, and inspects agents, a tile’s unused cell and a gap', async () => {
    // Two groups of 20 with observers: tiles of 5 × 5 side by side, a one-cell gap at x = 5.
    const e = await create('ns-fig-3-n20', (c) => {
      c.groups = 2;
      c.end = 30;
    });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(100);
    expect([e.tick, e.finished, ends]).toEqual([30, true, 1]);
    const at = async (x: number, y: number): Promise<ImageInspection> => {
      await e.select(x, y);
      const v = e.inspection!.view;
      expect(isImageView(v)).toBe(true);
      return v as ImageInspection;
    };
    expect(await at(5, 0)).toEqual({ cell: { x: 5, y: 0 }, group: null, agent: null });
    expect(await at(0, 4)).toEqual({ cell: { x: 0, y: 4 }, group: 0, agent: null });
    let given = 0;
    let received = 0;
    for (const x0 of [0, 6]) {
      for (let i = 0; i < 20; i++) {
        const v = await at(x0 + (i % 5), Math.floor(i / 5));
        const a = v.agent!;
        expect(v.group).toBe(x0 === 0 ? 0 : 1);
        expect(e.inspection!.agentId).toBe(a.id);
        // With observers each agent's record among the others is known.
        expect(a.known).toBeGreaterThanOrEqual(0);
        expect(a.known).toBeLessThanOrEqual(19);
        given += a.given;
        received += a.received;
      }
    }
    const stats = e.latest as ImageStats;
    expect([given, received]).toEqual([stats.helps, stats.helps]);
  });

  it('opens its Compare entries and sweeps its default form', () => {
    for (const id of ['image-one-vs-island', 'image-scoring-vs-standing', 'image-offset', 'image-group-size']) {
      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
    }
    const { sweep, errors } = formToSweep(defaultForm('image'), { preset: 'ns-fig-2' });
    expect(errors).toEqual([]);
    // Ten values of m × the form's three seeds.
    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(30);
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

