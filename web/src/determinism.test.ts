import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { Engine, type Speed } from './engine';
import type { Overlay } from './protocol';
import { SimHost } from './sim-host';
import { wasmSimModule } from './sim-module';
import { InlineTransport } from './transport';
import { decodeShare, encodeShare } from './share';
import type { Preset } from './types';
import { initSync, presets_json } from './wasm-pkg/sugarscape.js';

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
    const networks: Overlay[] = ['trade', 'credit'];
    e.want(() => ({ charts: { groups: [['population', 'gini']], max: 50 }, lorenz: true, wealthHist: true, networks }));
    for (let i = 0; i < 20; i++) await e.advance(10);
    expect(await e.fingerprint()).toBe(GOLDEN);
  });
});

describe('sessions replay exactly', () => {
  const endemic = presets.find((p) => p.id === 'v-2-endemic')!;
  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
  const create = (seed: number) => Engine.create({ config: structuredClone(endemic.config), seed }, { presets, transport: inline() });

  /**
   * A Max run's pace depends on the machine, so waits here poll for progress instead of sleeping a
   * fixed time (PF1): a generous cap still fails the test with a clear message if `e` ever gets
   * stuck rather than hanging until Vitest's own timeout.
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

  it(
    'replays a session recorded at mixed speeds, through a share link, to the same world at the same tick',
    async () => {
      const live = await create(5);
      expect(await live.paint(10, 10, 2, 0, 0)).toBeNull(); // tick 0
      await run(live, 1);
      await live.place(3, 3, {}); // may be occupied: then there is an agent to infect anyway
      expect(await live.infect(3, 3, -1)).toBeNull();
      await run(live, 25);
      await live.erase(3, 3);
      await live.place(0, 0, { sex: 'female' });
      expect(await live.applyConfig((c) => void (c.growback.rate = 2))).toBeNull();
      await run(live, 'max');
      await live.vaccinate(20, 20, 3, 0);
      // Edits while Max runs land between batches: wait for real progress on each side (not a
      // fixed sleep) so they land regardless of how fast Max runs on this machine (PF1).
      live.setSpeed('max');
      live.setRunning(true);
      await waitForTickPast(live, live.tick);
      await live.place(1, 1, {});
      await live.paint(30, 30, 1, 4, 0);
      await waitForTickPast(live, live.tick);
      live.setRunning(false);
      await run(live, 100, 2);
      const { session, full, tick } = await live.session();
      expect(full).toBe(false);
      // Places and erases may meet an occupied or empty site (then they are not logged); these always land:
      // the paint at 0, the infection, the live change and the paint during Max, at four different ticks.
      expect(session.log.length).toBeGreaterThanOrEqual(4);
      expect(new Set(session.log.map((e) => e.tick)).size).toBeGreaterThanOrEqual(4);
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
      await live.place(4, 4, {});
      await live.infect(4, 4, -1);
      await live.advance(3);
      await live.paint(12, 30, 3, 1, 0);
      await live.applyConfig((c) => void (c.growback.rate = 3));
      await live.advance(4);
      const { session } = await live.session();
      const lastLoggedTick = session.log[session.log.length - 1].tick;
      const replayed = await Engine.create(session, { presets, transport: inline() });
      replayed.setSpeed('max');
      replayed.setRunning(true);
      // Run until replay is past the last logged tick, not for a fixed time (PF1): only then can
      // every edit in the log have replayed.
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
});
