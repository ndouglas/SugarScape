import { describe, expect, it } from 'vitest';
import { Engine, type EngineEvent } from './engine';
import { fakeModule } from './fake-sim.fixture';
import type { Command, HostReply, Wants } from './protocol';
import { SimHost } from './sim-host';
import { InlineTransport } from './transport';
import type { Config, Preset } from './types';
import { ChartFreshness } from './ui/series-data';

const config = { width: 4, height: 3 } as unknown as Config;
const presets: Preset[] = [{ id: 'ii-2-unit', name: 'Unit', source: 'II-2', description: '', config }];
/** Lets queued microtasks and zero-delay timers run. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

/** An inline transport that runs `after(cmd)` right after each request is sent, and can rewrite replies. */
class HookedTransport extends InlineTransport {
  after: ((cmd: Command) => void) | null = null;
  rewrite: ((cmd: Command, reply: HostReply) => HostReply) | null = null;
  override async request(cmd: Command, extra?: { wants?: Wants; frame?: ArrayBuffer }): Promise<HostReply> {
    const pending = super.request(cmd, extra);
    this.after?.(cmd);
    const reply = await pending;
    return this.rewrite ? this.rewrite(cmd, reply) : reply;
  }
}

async function setup() {
  const log: string[] = [];
  const module = fakeModule(log);
  const transport = new HookedTransport(new SimHost(module));
  const engine = await Engine.create({ config, seed: 7 }, { presets, transport });
  return { engine, log, module, transport };
}

describe('Engine', () => {
  it('starts from the init snapshot', async () => {
    const { engine } = await setup();
    expect(engine.tick).toBe(0);
    expect(engine.population).toBe(1);
    expect(engine.size()).toEqual({ width: 4, height: 3 });
    expect(engine.frame()).toHaveLength(48);
    expect(engine.config.width).toBe(4);
    expect(engine.baseConfig).toEqual(engine.config);
    expect(engine.seed).toBe(7);
  });

  it('keeps at most one step outstanding', async () => {
    const { engine, module } = await setup();
    engine.setRunning(true);
    engine.pump(0);
    engine.pump(1);
    engine.pump(2);
    await settle();
    expect(module.sims[0].stepCalls).toBe(1);
    engine.pump(3);
    await settle();
    expect(module.sims[0].stepCalls).toBe(2);
    expect(engine.tick).toBe(2);
  });

  it('holds the frame loop while a reset waits for the step in flight', async () => {
    const { engine, module } = await setup();
    engine.setRunning(true);
    engine.pump(0);
    const done = engine.reset();
    engine.pump(1);
    expect(await done).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(module.sims[0].stepCalls).toBe(1);
    expect(module.sims[1].stepCalls).toBe(0);
    expect(engine.tick).toBe(0);
    engine.pump(2);
    await settle();
    expect(module.sims[1].stepCalls).toBe(1);
  });

  it('fires events only when replies arrive', async () => {
    const { engine } = await setup();
    const seen: EngineEvent[] = [];
    for (const event of ['edit', 'snapshot'] as const) engine.on(event, () => seen.push(event));
    const pending = engine.place(0, 2, {});
    expect(seen).toEqual([]);
    expect(await pending).toBeNull();
    expect(seen).toEqual(['edit', 'snapshot']);
    expect(engine.population).toBe(2);
  });

  it("resolves writes with the core's errors", async () => {
    const { engine } = await setup();
    expect(await engine.applyConfig((c) => void (c.population = 5000))).toEqual([{ field: 'population', message: 'too many' }]);
    expect(engine.baseConfig.population).toBe(10);
    expect(await engine.reset({ ...engine.baseConfig, population: 5000 })).toEqual([{ field: 'population', message: 'too many' }]);
    expect(await engine.erase(3, 2)).toEqual([{ field: 'edit', message: 'no agent at (3, 2)' }]);
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    expect(engine.config.population).toBe(20);
    expect(engine.baseConfig.population).toBe(20);
  });

  it('ping-pongs two frame buffers', async () => {
    const { engine } = await setup();
    // Buffers are transferred, so each reply's frame is a new ArrayBuffer object; a buffer the
    // engine lends again is detached on its side, which is how reuse shows here.
    const shown: ArrayBuffer[] = [];
    for (let i = 0; i < 4; i++) {
      await engine.advance(1);
      shown.push(engine.frame()!.buffer);
    }
    expect(shown.map((b) => b.byteLength)).toEqual([0, 0, 48, 48]);
    expect(engine.frame()![0]).toBe(4);
  });

  it('keeps the selection on a selected agent as it moves', async () => {
    const { engine } = await setup();
    await engine.select(1, 1);
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
    await engine.advance(1);
    expect(engine.selection).toEqual({ x: 2, y: 1, agentId: 1 });
    expect(engine.inspection?.alive).toBe(true);
    await engine.reset();
    expect(engine.selection).toBeNull();
    expect(engine.inspection).toBeNull();
  });

  it('ignores a selection sent before a reset whose reply arrives after it', async () => {
    const { engine, transport } = await setup();
    await engine.select(1, 1);
    let late: Promise<void> | null = null;
    // Queued behind the reset, but carrying the old selection in its wants.
    transport.after = (cmd) => {
      if (cmd.type === 'reset') late = engine.refresh();
    };
    await engine.reset();
    transport.after = null;
    await late;
    expect(late).not.toBeNull();
    expect(engine.selection).toBeNull();
    expect(engine.inspection).toBeNull();
    await engine.advance(1);
    expect(engine.selection).toBeNull();
  });

  it('treats all-null edited landscapes as none', async () => {
    const { engine } = await setup();
    expect(engine.editedLandscapes()).toBeUndefined();
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    expect(engine.editedLandscapes()).toBeUndefined();
    expect(await engine.paint(0, 0, 1, 3)).toBeNull();
    expect(engine.editedLandscapes()?.[0]).toHaveLength(12);
  });

  it('keeps the last chart group the host sent', async () => {
    const { engine } = await setup();
    engine.want(() => ({ charts: { groups: [['population']], max: 2000 } }));
    await engine.advance(1);
    const group = engine.chartGroup(['population']);
    expect(Array.from(group!.ticks)).toEqual([0, 1]);
    // The history has not grown, so the host does not send the group again.
    await engine.refresh();
    expect(engine.last?.charts).toBeUndefined();
    expect(engine.chartGroup(['population'])).toBe(group);
  });

  it('builds each queued config write on the one before it', async () => {
    const { engine } = await setup();
    const first = engine.applyConfig((c) => void (c.population = 20));
    const second = engine.applyConfig((c) => void (c.population += 1));
    expect(await first).toBeNull();
    expect(await second).toBeNull();
    expect(engine.config.population).toBe(21);
    expect(engine.baseConfig.population).toBe(21);
  });

  it('builds a reset-requiring change on a config write still in flight', async () => {
    const { engine, module } = await setup();
    const live = engine.applyConfig((c) => void (c.population = 20));
    const rebuilt = engine.resetWith((c) => void (c.height = 5));
    expect(await live).toBeNull();
    expect(await rebuilt).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(engine.baseConfig.population).toBe(20);
    expect(engine.baseConfig.height).toBe(5);
    expect(engine.size()).toEqual({ width: 4, height: 5 });
  });

  it('sends a write issued during a Step only after its reply', async () => {
    const { engine, transport } = await setup();
    const sentAt: number[] = [];
    transport.after = (cmd) => {
      if (cmd.type === 'setConfig') sentAt.push(engine.tick);
    };
    const step = engine.advance(1);
    const write = engine.applyConfig((c) => void (c.population = 20));
    await step;
    expect(await write).toBeNull();
    expect(sentAt).toEqual([1]);
  });

  it('keeps the selection and still takes the snapshot when a selected agent is gone', async () => {
    const { engine } = await setup();
    await engine.select(1, 1);
    await engine.place(0, 2, {});
    const seen: EngineEvent[] = [];
    for (const event of ['select', 'snapshot'] as const) engine.on(event, () => seen.push(event));
    await engine.selectAgent(99);
    expect(seen).toEqual(['snapshot']);
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
    await engine.selectAgent(2);
    expect(seen).toEqual(['snapshot', 'select', 'snapshot']);
    expect(engine.selection).toEqual({ x: 0, y: 2, agentId: 2 });
  });

  it('keeps a selection made while a reset is outstanding', async () => {
    const { engine, transport } = await setup();
    await engine.select(1, 1);
    let click: Promise<void> | null = null;
    transport.after = (cmd) => {
      if (cmd.type === 'reset') click = engine.select(0, 2);
    };
    expect(await engine.reset()).toBeNull();
    transport.after = null;
    await click;
    expect(click).not.toBeNull();
    expect(engine.selection).toEqual({ x: 0, y: 2, agentId: null });
  });

  it('fails a config write or reset whose reply has no snapshot', async () => {
    const { engine, transport } = await setup();
    transport.rewrite = (cmd, reply) => (cmd.type === 'setConfig' || cmd.type === 'reset' ? { ...reply, result: { ok: true } } : reply);
    const missing = [{ field: 'simulation', message: 'the simulation sent no snapshot' }];
    expect(await engine.applyConfig((c) => void (c.population = 20))).toEqual(missing);
    expect(await engine.reset()).toEqual(missing);
  });

  it('stops for good after a panic', async () => {
    const { engine } = await setup();
    const crashes: string[] = [];
    engine.on('crash', () => crashes.push(engine.crashed ?? ''));
    engine.setRunning(true);
    const message = 'The simulation stopped: unreachable executed';
    expect(await engine.paint(0, 0, 1, -1)).toEqual([{ field: 'simulation', message }]);
    expect(crashes).toEqual([message]);
    expect(engine.running).toBe(false);
    expect(await engine.place(0, 2, {})).toEqual([{ field: 'simulation', message }]);
  });

  it('follows an agent and keeps its trail', async () => {
    const { engine } = await setup();
    await engine.followAgent(1);
    expect(engine.followed()).toBe(1);
    expect(engine.followedAlive()).toBe(true);
    expect(Array.from(engine.trail())).toEqual([1, 1]);
    await engine.advance(1);
    expect(Array.from(engine.trail())).toEqual([2, 1]);
    await engine.erase(2, 1);
    expect(engine.followedAlive()).toBe(false);
    await engine.unfollow();
    expect(engine.followed()).toBeNull();
    expect(engine.trail()).toHaveLength(0);
  });

  it('fetches the networks of the overlays that are on', async () => {
    const { engine } = await setup();
    expect(engine.networks('trade')).toHaveLength(0);
    engine.setDisplay({ overlays: { trade: true } });
    await settle();
    expect(Array.from(engine.networks('trade'))).toEqual([0, 0, 1, 1]);
  });

  it('keeps a selected agent that died selected, as gone', async () => {
    const { engine } = await setup();
    await engine.selectAgent(1);
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
    await engine.erase(1, 1);
    expect(engine.inspection).toMatchObject({ agentId: 1, alive: false });
    await engine.selectAgent(1); // no longer alive: nothing changes
    expect(engine.selection).toEqual({ x: 1, y: 1, agentId: 1 });
  });

  it('returns exports as values', async () => {
    const { engine } = await setup();
    await engine.advance(3);
    expect(await engine.seriesCsv()).toBe('tick,population\n3,1\n');
    expect(await engine.agentsCsv()).toBe('id\n1\n');
    expect(await engine.fingerprint()).toBe('0x3');
  });

  it('carries what providers want, and refreshes while paused only for them', async () => {
    const { engine } = await setup();
    const before = engine.last;
    engine.pump(1000);
    await settle();
    expect(engine.last).toBe(before);
    const stop = engine.want(() => ({ diseaseList: true, creditGraph: true }));
    engine.pump(2000);
    await settle();
    expect(engine.last?.diseaseList).toEqual([{ id: 0, bits: '01', carriers: 2 }]);
    expect(engine.last?.creditGraph).toEqual({ agents: [], loans: [] });
    stop();
    await engine.refresh();
    expect(engine.last?.diseaseList).toBeUndefined();
  });

  it('a chart-group provider (PF6) asks again only once it is behind the tick; pump sends nothing while caught up', async () => {
    const { engine, transport } = await setup();
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);
    const fresh = new ChartFreshness();
    const groups = [['population']];
    engine.want(() => (fresh.behind(groups, engine.tick) ? { charts: { groups, max: 2000 } } : {}));
    engine.on('snapshot', () => fresh.receive(engine.last?.charts));

    // Paused, never seen: the first pump asks, and the reply catches the group up.
    const base = performance.now();
    engine.pump(base);
    await settle();
    expect(sent).toEqual(['refresh']);
    expect(fresh.behind(groups, engine.tick)).toBe(false);

    // Caught up, still paused: later pumps ask nothing, even once the 250 ms gate would allow one.
    engine.pump(base + 1000);
    await settle();
    expect(sent).toEqual(['refresh']); // no second request went out

    // The tick moves without going through this provider's own request (e.g. another Step): behind again.
    await engine.advance(1);
    sent.length = 0;
    engine.pump(base + 2000);
    await settle();
    expect(sent).toEqual(['refresh']); // exactly one refresh brings it current
    expect(fresh.behind(groups, engine.tick)).toBe(false);
  });
});
