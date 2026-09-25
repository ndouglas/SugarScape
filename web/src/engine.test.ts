import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Engine, finishedNotice, FULL_NOTICE, type EngineEvent } from './engine';
import { fakeModule } from './fake-sim.fixture';
import type { Command, HostReply, LogEntry, Wants } from './protocol';
import { MAX_TICKS, SimHost } from './sim-host';
import { InlineTransport } from './transport';
import { isSugar } from './models';
import type { Config, ModelConfig, Preset, RingConfig } from './types';
import { DiseaseListPoll } from './ui/disease-picker';
import { chartsBehind, distributionsDue, distributionWants, type DistState } from './ui/series-data';

const config = { width: 4, height: 3 } as unknown as Config;
const presets: Preset[] = [{ id: 'ii-2-unit', name: 'Unit', source: 'II-2', description: '', config }];
/** Lets queued microtasks and zero-delay timers run. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
/** A config the test knows is a sugarscape's. */
const sugar = (c: ModelConfig): Config => {
  if (!isSugar(c)) throw new Error('not a sugarscape config');
  return c;
};

/** An inline transport that runs `after(cmd, wants)` right after each request is sent, and can rewrite replies. */
class HookedTransport extends InlineTransport {
  after: ((cmd: Command, wants?: Wants) => void) | null = null;
  rewrite: ((cmd: Command, reply: HostReply) => HostReply) | null = null;
  override async request(cmd: Command, extra?: { wants?: Wants; frame?: ArrayBuffer }): Promise<HostReply> {
    const pending = super.request(cmd, extra);
    this.after?.(cmd, extra?.wants);
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
    expect(sugar(engine.config).width).toBe(4);
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

  it('steps one tick at a time by the clock below 1×', async () => {
    const { engine, module } = await setup();
    engine.setSpeed(2 / 60); // two ticks a second
    engine.setRunning(true);
    for (const now of [0, 100, 400, 499, 500, 520, 1000]) {
      engine.pump(now);
      await settle();
    }
    expect(module.sims[0].stepCalls).toBe(3);
    expect(engine.tick).toBe(3);
    // A long gap (a hidden tab) steps once, not the ticks it missed.
    engine.pump(10_000);
    await settle();
    engine.pump(10_010);
    await settle();
    expect(engine.tick).toBe(4);
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
    expect(sugar(engine.baseConfig).population).toBe(10);
    expect(await engine.reset({ ...engine.baseConfig, population: 5000 })).toEqual([{ field: 'population', message: 'too many' }]);
    expect(await engine.erase(3, 2)).toEqual([{ field: 'edit', message: 'no agent at (3, 2)' }]);
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    expect(sugar(engine.config).population).toBe(20);
    expect(sugar(engine.baseConfig).population).toBe(20);
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
    expect(sugar(engine.config).population).toBe(21);
    expect(sugar(engine.baseConfig).population).toBe(21);
  });

  it('builds a reset-requiring change on a config write still in flight', async () => {
    const { engine, module } = await setup();
    const live = engine.applyConfig((c) => void (c.population = 20));
    const rebuilt = engine.resetWith((c) => void (c.height = 5));
    expect(await live).toBeNull();
    expect(await rebuilt).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(sugar(engine.baseConfig).population).toBe(20);
    expect(sugar(engine.baseConfig).height).toBe(5);
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
    expect(await engine.fingerprint()).toBe('0x3|1@0,1|10');
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

  it('fires config once for a config write and not at all for a reset', async () => {
    const { engine } = await setup();
    const seen: EngineEvent[] = [];
    for (const event of ['config', 'reset'] as const) engine.on(event, () => seen.push(event));
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    await settle();
    expect(seen).toEqual(['config']);
    expect(await engine.reset()).toBeNull();
    await settle();
    expect(seen).toEqual(['config', 'reset']);
  });

  it('keeps a display chosen while a reset is outstanding over the reset reply\'s clamp of the old one', async () => {
    const withDisease = { ...config, disease: { enabled: true } } as unknown as Config;
    const log: string[] = [];
    const transport = new HookedTransport(new SimHost(fakeModule(log)));
    const engine = await Engine.create({ config: withDisease, seed: 7 }, { presets, transport });
    engine.setDisplay({ colorMode: 'disease' });
    await settle();
    // The reset turns disease off, so the host clamps the Disease color mode to Tribe in its reply;
    // the user picks Age before that reply arrives.
    transport.after = (cmd) => {
      if (cmd.type === 'reset') engine.setDisplay({ colorMode: 'age' });
    };
    const base = sugar(engine.baseConfig);
    expect(await engine.reset({ ...base, disease: { ...base.disease, enabled: false } })).toBeNull();
    transport.after = null;
    await settle();
    expect(engine.colorMode).toBe('age');
    // The host draws what the selectors show.
    log.length = 0;
    await engine.advance(1);
    expect(log).toEqual(['render age resource:0']);
  });

  it('fetches the disease list while paused only when the world changed or the tool opened', async () => {
    const { engine, transport } = await setup();
    const sent: string[] = [];
    transport.after = (cmd, wants) => {
      if (wants?.diseaseList) sent.push(cmd.type);
    };
    // The disease tools' provider (tools.ts), without its rate limit: only what changed matters here.
    const poll = new DiseaseListPoll(0);
    engine.want((now) => (poll.due(now, engine.tick) ? { diseaseList: true } : {}));
    engine.on('edit', () => poll.invalidate());
    engine.on('snapshot', () => {
      if (engine.last?.diseaseList) poll.received(performance.now(), engine.tick);
    });
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        engine.pump((now += 1000));
        await settle();
      }
    };
    await pumps(5);
    expect(sent).toEqual(['refresh']);
    await engine.advance(1); // the Step's own request was sent before the tick moved
    await pumps(5);
    expect(sent).toEqual(['refresh', 'refresh']);
    await engine.place(0, 2, {});
    await pumps(5);
    expect(sent).toEqual(['refresh', 'refresh', 'refresh']);
  });

  it('asks for a chart group only while its cached copy is behind the tick; pump sends nothing while caught up', async () => {
    const { engine, transport } = await setup();
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);
    const groups = [['population']];
    // The Charts panel's provider: its visible groups, only while the engine's copy is behind.
    let visible = true;
    engine.want(() => (visible && chartsBehind(groups, engine.tick, (g) => engine.chartGroup(g)) ? { charts: { groups, max: 2000 } } : {}));
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        engine.pump((now += 1000));
        await settle();
      }
    };

    // Paused, never seen: the first pump asks, and the reply catches the group up.
    await pumps(1);
    expect(sent).toEqual(['refresh']);
    // Caught up, still paused: later pumps ask nothing, even though the 250 ms gate would allow one.
    await pumps(3);
    expect(sent).toEqual(['refresh']);

    // The tick moves without this provider's own request (another Step): exactly one refresh catches up.
    await engine.advance(1);
    sent.length = 0;
    await pumps(3);
    expect(sent).toEqual(['refresh']);
    expect(Array.from(engine.chartGroup(['population'])!.ticks).at(-1)).toBe(1);
  });

  it('catches the charts up once after the tab is shown again while paused, then goes quiet', async () => {
    const { engine, transport } = await setup();
    const groups = [['population']];
    let visible = true;
    engine.want(() => (visible && chartsBehind(groups, engine.tick, (g) => engine.chartGroup(g)) ? { charts: { groups, max: 2000 } } : {}));
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        engine.pump((now += 1000));
        await settle();
      }
    };
    await pumps(1);
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);

    // Hidden and shown again with nothing changed: the engine still holds every group, so nothing is sent.
    visible = false;
    await pumps(2);
    visible = true;
    await pumps(5);
    expect(sent).toEqual([]);

    // Hidden while the world moves on, then shown while paused: one refresh, then nothing.
    visible = false;
    await engine.advance(2);
    await pumps(2);
    sent.length = 0;
    visible = true;
    await pumps(5);
    expect(sent).toEqual(['refresh']);
    expect(Array.from(engine.chartGroup(['population'])!.ticks).at(-1)).toBe(2);

    // Export → Charts (PNG) shows the tab and refreshes once explicitly; the pumps after it stay quiet.
    sent.length = 0;
    await engine.refresh();
    await pumps(5);
    expect(sent).toEqual(['refresh']);
  });

  it("keeps wants().select current for a request sent before a new selectAgent's own reply lands", async () => {
    const { engine, transport } = await setup();
    expect(await engine.place(0, 2, {})).toBeNull(); // a second agent, id 2, at (0, 2)
    await engine.selectAgent(1); // agent 1 starts at (1, 1)
    let capturedWants: Wants | undefined;
    transport.after = (cmd, wants) => {
      if (cmd.type === 'inspect') void engine.refresh();
      else if (cmd.type === 'refresh') capturedWants = wants;
    };
    await engine.selectAgent(2);
    transport.after = null;
    // agentId is exact immediately; x/y are only the previous selection's as a fallback.
    expect(capturedWants?.select).toEqual({ x: 1, y: 1, agentId: 2 });
  });

  it("omits a pending site-only select from wants rather than guessing wrong", async () => {
    const { engine, transport } = await setup();
    await engine.select(1, 1); // agent 1's site
    let capturedWants: Wants | undefined;
    transport.after = (cmd, wants) => {
      if (cmd.type === 'inspect') void engine.refresh();
      else if (cmd.type === 'refresh') capturedWants = wants;
    };
    await engine.select(2, 1); // an empty site; still pending when the refresh goes out
    transport.after = null;
    expect(capturedWants?.select).toBeUndefined();
  });

  it("keeps wants().trail current for a request sent before follow's own reply lands", async () => {
    const { engine, transport } = await setup();
    let capturedWants: Wants | undefined;
    transport.after = (cmd, wants) => {
      if (cmd.type === 'follow') void engine.refresh();
      else if (cmd.type === 'refresh') capturedWants = wants;
    };
    await engine.followAgent(1);
    transport.after = null;
    expect(capturedWants?.trail).toBe(true);
  });

  it('seeks back and forward, keeping the selection and resending charts', async () => {
    const { engine } = await setup();
    await engine.select(1, 1);
    await engine.advance(120);
    expect(engine.reached).toBe(120);
    expect(engine.seekable).toBe(true);
    let configs = 0;
    engine.on('config', () => configs++);
    expect(await engine.seek(30)).toBeNull();
    expect(engine.tick).toBe(30);
    expect(engine.reached).toBe(120);
    expect(engine.selection).not.toBeNull();
    expect(configs).toBe(1);
    await engine.seek(120);
    expect(engine.tick).toBe(120);
  });

  it('returns the host’s errors for a refused seek', async () => {
    const { engine } = await setup();
    await engine.advance(5);
    const errors = await engine.seek(6);
    expect(errors?.[0].field).toBe('seek');
  });

  it('pauses and fires stopped when a stop rule fires', async () => {
    const { engine } = await setup();
    engine.setStops({ tick: 3 });
    engine.setSpeed(2);
    engine.setRunning(true);
    let stopped = 0;
    engine.on('stopped', () => stopped++);
    for (let i = 0; i < 5; i++) {
      engine.pump(i);
      await settle();
    }
    expect(engine.tick).toBe(3);
    expect(engine.running).toBe(false);
    expect(stopped).toBe(1);
    expect(engine.lastStop).toBe('Stopped at tick 3');
  });
});

describe('Engine at Max speed', () => {
  // The host's batches are scheduled with zero-delay timers: fake timers make how many run
  // (one per fake millisecond) independent of how busy the machine is.
  beforeEach(() => void vi.useFakeTimers());
  afterEach(() => void vi.useRealTimers());
  const wait = (ms: number) => vi.advanceTimersByTimeAsync(ms);

  /** A host on a fake clock (1 ms per reading) and a log of the commands the engine sends. */
  async function maxSetup() {
    let clock = 0;
    const transport = new InlineTransport(new SimHost(fakeModule(), () => clock++));
    const sent: string[] = [];
    const request = transport.request.bind(transport);
    transport.request = (cmd, extra) => {
      sent.push(cmd.type);
      return request(cmd, extra);
    };
    const engine = await Engine.create({ config, seed: 7 }, { presets, transport });
    engine.setSpeed('max');
    return { engine, sent };
  }

  it('runs until paused, handing each posted buffer back', async () => {
    const { engine, sent } = await maxSetup();
    let ticks = 0;
    engine.on('tick', () => ticks++);
    engine.setRunning(true);
    await wait(30);
    for (let i = 0; i < 3; i++) engine.pump();
    expect(ticks).toBeGreaterThan(1);
    expect(sent.filter((c) => c === 'frame').length).toBeGreaterThanOrEqual(ticks - 1);
    engine.setRunning(false);
    await wait(5);
    const at = engine.tick;
    await wait(20);
    expect(engine.tick).toBe(at);
    expect(sent.filter((c) => c === 'run')).toHaveLength(1);
    expect(sent).toContain('stop');
    expect(sent).not.toContain('step');
  });

  it('stops before a reset and starts again after it', async () => {
    const { engine, sent } = await maxSetup();
    engine.setRunning(true);
    await wait(10);
    expect(await engine.reset()).toBeNull();
    const at = sent.lastIndexOf('reset');
    expect(sent.lastIndexOf('stop', at)).toBeGreaterThan(sent.indexOf('run'));
    expect(sent.indexOf('run', at)).toBeGreaterThan(at);
    const t = engine.tick;
    await wait(20);
    expect(engine.tick).toBeGreaterThan(t);
    engine.setRunning(false);
  });

  it('applies edits while running', async () => {
    const { engine } = await maxSetup();
    engine.setRunning(true);
    await wait(5);
    const edits: number[] = [];
    engine.on('edit', () => edits.push(engine.population));
    expect(await engine.place(0, 2, {})).toBeNull();
    expect(edits).toEqual([2]);
    const t = engine.tick;
    await wait(20);
    expect(engine.tick).toBeGreaterThan(t);
    engine.setRunning(false);
  });

  it('leaves Max for a step speed', async () => {
    const { engine, sent } = await maxSetup();
    engine.setRunning(true);
    await wait(5);
    engine.setSpeed(5);
    await wait(5);
    expect(sent).toContain('stop');
    const at = engine.tick;
    engine.pump();
    await wait(5);
    expect(engine.tick).toBe(at + 5);
    engine.setRunning(false);
  });

  it("updates the Inspect panel's selection from posts, before stopping", async () => {
    const { engine } = await maxSetup();
    await engine.selectAgent(1); // agent 1 starts at (1, 1) and walks +1 x per tick
    const before = engine.inspection;
    expect(before?.x).toBe(1);
    // A post (not just the final `stop`, which goes through send() and always updates it) must be
    // what moves this while Max is still running.
    let changedWhileRunning = false;
    engine.on('snapshot', () => {
      if (engine.running && engine.inspection !== before) changedWhileRunning = true;
    });
    engine.setRunning(true);
    await wait(30);
    expect(changedWhileRunning).toBe(true);
    expect(engine.selection).toEqual({ x: (1 + engine.tick) % 4, y: 1, agentId: 1 });
    engine.setRunning(false);
    await wait(5);
  });

  it('keeps a followed trail updating from posts too, before stopping (no regression: not gated the same way)', async () => {
    const { engine } = await maxSetup();
    await engine.followAgent(1);
    const before = engine.trail();
    let changedWhileRunning = false;
    engine.on('snapshot', () => {
      if (engine.running && engine.trail() !== before) changedWhileRunning = true;
    });
    engine.setRunning(true);
    await wait(30);
    expect(changedWhileRunning).toBe(true);
    expect(Array.from(engine.trail())).toEqual([(1 + engine.tick) % 4, 1]);
    engine.setRunning(false);
    await wait(5);
  });

  it('gives exactly one stop and one run for two writes queued together', async () => {
    const { engine, sent } = await maxSetup();
    engine.setRunning(true);
    sent.length = 0;
    const first = engine.applyConfig((c) => void (c.population = 2));
    const second = engine.applyConfig((c) => void (c.population = 3));
    expect(await first).toBeNull();
    expect(await second).toBeNull();
    expect(sent.filter((c) => c === 'stop')).toHaveLength(1);
    expect(sent.filter((c) => c === 'run')).toHaveLength(1);
    engine.setRunning(false);
  });

  it('fires config once when a scheduled change reaches the page in an edit reply rather than a post', async () => {
    let clock = 0;
    const transport = new InlineTransport(new SimHost(fakeModule(), () => clock++));
    // Never hands a buffer back: after the first post the host has nothing to post into, so it
    // keeps stepping unseen, and the next command's reply is the first to carry the config.
    const request = transport.request.bind(transport);
    transport.request = (cmd, extra) => request(cmd, cmd.type === 'frame' ? { ...extra, frame: undefined } : extra);
    const scheduled = { ...config, schedule: [{ tick: 100, set: {} }] } as unknown as Config;
    const engine = await Engine.create({ config: scheduled, seed: 7 }, { presets, transport });
    let configs = 0;
    engine.on('config', () => configs++);
    engine.setSpeed('max');
    engine.setRunning(true);
    await wait(50);
    expect(engine.tick).toBeLessThan(100); // only the first post arrived, before the change fired
    expect(configs).toBe(0);
    expect(await engine.paint(0, 0, 1, 3)).toBeNull();
    expect(engine.tick).toBeGreaterThan(100);
    expect(configs).toBe(1);
    engine.setRunning(false);
    await wait(5);
    expect(configs).toBe(1);
  });

  it('pauses at the tick cap when Max reaches it, and says so', async () => {
    const { engine, sent } = await maxSetup();
    await engine.advance(MAX_TICKS - 20);
    let fulls = 0;
    engine.on('full', () => fulls++);
    engine.setRunning(true);
    await wait(60);
    expect(engine.tick).toBe(MAX_TICKS);
    expect(engine.running).toBe(false);
    expect(fulls).toBe(1);
    expect(sent.filter((c) => c === 'run')).toHaveLength(1);
    const at = sent.length;
    await wait(20);
    expect(sent.slice(at).filter((c) => c === 'run' || c === 'frame')).toEqual([]);
  });

  it('ends Max and emits crash on a fatal post', async () => {
    const module = fakeModule();
    let clock = 0;
    const transport = new InlineTransport(new SimHost(module, () => clock++));
    const engine = await Engine.create({ config, seed: 7 }, { presets, transport });
    const sim = module.sims[0];
    const realStep = sim.step.bind(sim);
    let calls = 0;
    // Panics inside a batch, not a request: exercises the `{ id: null, fatal }` post path (as
    // opposed to a request-reply fatal, already covered by 'stops for good after a panic').
    sim.step = (n: number) => {
      calls++;
      if (calls > 2) throw new Error('boom');
      realStep(n);
    };
    const crashes: string[] = [];
    engine.on('crash', () => crashes.push(engine.crashed ?? ''));
    engine.setSpeed('max');
    engine.setRunning(true);
    await wait(10);
    expect(crashes).toHaveLength(1);
    expect(engine.crashed).toContain('boom');
    expect(engine.running).toBe(false);
  });

  it('pauses on a stop rule at Max', async () => {
    const { engine } = await maxSetup();
    engine.setStops({ tick: 40 });
    engine.setSpeed('max');
    engine.setRunning(true);
    while (engine.running) await wait(5);
    expect(engine.tick).toBe(40);
    expect(engine.lastStop).toBe('Stopped at tick 40');
  });
});

describe('Engine at the tick cap', () => {
  it('pauses when a step reaches the cap, and says so again when a step is refused there', async () => {
    const { engine } = await setup();
    expect(FULL_NOTICE).toBe('This world has reached 1,000,000 ticks, the most its history holds here — export its data, or Reset to start again');
    await engine.advance(MAX_TICKS - 7);
    let fulls = 0;
    engine.on('full', () => fulls++);
    engine.setSpeed(5);
    engine.setRunning(true);
    for (let i = 0; i < 4; i++) {
      engine.pump(i);
      await settle();
    }
    expect(engine.tick).toBe(MAX_TICKS);
    expect(engine.running).toBe(false);
    expect(fulls).toBe(1);
    await engine.advance(1);
    expect(engine.tick).toBe(MAX_TICKS);
    expect(engine.crashed).toBeNull();
    expect(fulls).toBe(2);
  });
});

describe('Engine sessions', () => {
  const deps = () => ({ presets, transport: new InlineTransport(new SimHost(fakeModule())) });

  it('logs edits with their ticks and returns the session it was built from', async () => {
    const { engine } = await setup();
    await engine.advance(2);
    expect(await engine.place(0, 2, {})).toBeNull();
    expect(await engine.erase(3, 2)).not.toBeNull(); // failed: not logged
    await engine.advance(1);
    expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
    const { session, full, tick } = await engine.session();
    expect(full).toBe(false);
    expect(tick).toBe(3);
    expect(session.config).toEqual(config);
    expect(session.seed).toBe(7);
    expect(session.landscapes).toEqual([]);
    expect(session.log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [2, 'place'],
      [3, 'setConfig'],
    ]);
  });

  it('replays a session on a new engine: counts down and shares back the same log', async () => {
    const first = await setup();
    await first.engine.advance(2);
    await first.engine.place(0, 2, {});
    await first.engine.advance(3);
    await first.engine.paint(0, 0, 1, 3);
    const { session } = await first.engine.session();
    const engine = await Engine.create(session, deps());
    expect(engine.replayLeft).toBe(2);
    const counts: number[] = [];
    engine.on('replay', () => counts.push(engine.replayLeft));
    await engine.advance(2);
    expect(engine.population).toBe(2);
    await engine.advance(3);
    expect(counts).toEqual([1, 0]);
    expect((await engine.session()).session.log).toEqual(session.log);
    expect([engine.tick, engine.population]).toEqual([first.engine.tick, first.engine.population]);
  });

  it('forks on an edit during a replay; endReplay keeps the world', async () => {
    const log: LogEntry[] = [
      { tick: 3, cmd: { type: 'place', x: 0, y: 2, overrides: {} } },
      { tick: 5, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    const a = await Engine.create({ config, seed: 7, log }, deps());
    const events: EngineEvent[] = [];
    a.on('fork', () => events.push('fork'));
    await a.advance(1);
    expect(await a.paint(0, 0, 1, 3)).toBeNull();
    expect(events).toEqual(['fork']);
    expect(a.replayLeft).toBe(0);
    expect((await a.session()).session.log.map((e) => e.cmd.type)).toEqual(['paint']);

    const b = await Engine.create({ config, seed: 7, log }, deps());
    await b.advance(3);
    expect(b.replayLeft).toBe(1);
    await b.endReplay();
    expect(b.replayLeft).toBe(0);
    expect(b.population).toBe(2);
    await b.advance(3);
    expect(b.population).toBe(2);
    expect((await b.session()).session.log).toHaveLength(1);
  });

  it('Reset (replay) rebuilds the session and keeps the setup; a new seed starts an empty log', async () => {
    const { engine, module } = await setup();
    const preset = engine.presetId;
    const originalPopulation = sugar(engine.config).population;
    await engine.advance(2);
    await engine.place(0, 2, {});
    await engine.applyConfig((c) => void (c.population = 20));
    await engine.advance(3);
    expect(await engine.replay()).toBeNull();
    expect(module.sims).toHaveLength(2);
    expect(engine.tick).toBe(0);
    expect(engine.replayLeft).toBe(2);
    // Rebuilt from the session's own config (population 10), not the folded baseConfig (20): the
    // setConfig entry is still pending, replayed only once the world reaches its tick.
    expect(sugar(engine.config).population).toBe(originalPopulation);
    expect(sugar(engine.baseConfig).population).toBe(20);
    expect(engine.presetId).toBe(preset);
    await engine.advance(2);
    expect(engine.population).toBe(2);
    expect(sugar(engine.config).population).toBe(20);
    expect(await engine.reset(undefined, 99)).toBeNull();
    expect(engine.replayLeft).toBe(0);
    const fresh = await engine.session();
    expect(fresh.session.log).toEqual([]);
    expect(fresh.session.seed).toBe(99);
  });

  it('opens a session on the running page', async () => {
    const { engine } = await setup();
    await engine.advance(4);
    const log: LogEntry[] = [{ tick: 0, cmd: { type: 'place', x: 0, y: 2, overrides: {} } }];
    expect(await engine.open({ config, seed: 3, log })).toBeNull();
    expect([engine.tick, engine.seed, engine.population, engine.replayLeft]).toEqual([0, 3, 2, 0]);
    expect((await engine.session()).session.log).toEqual(log);
  });

  it("endReplay queued behind a still-pending replay() cannot be raced into re-arming it (Reset then ✕ quickly)", async () => {
    const log: LogEntry[] = [
      { tick: 1, cmd: { type: 'place', x: 0, y: 2, overrides: {} } },
      { tick: 2, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    const engine = await Engine.create({ config, seed: 7, log }, deps());
    await engine.advance(1);
    expect(engine.replayLeft).toBe(1);
    const counts: number[] = [];
    engine.on('replay', () => counts.push(engine.replayLeft));
    // Reset (same seed) re-arms the full log; the chip's ✕ fires right after, before replay()
    // has resolved. Without queuing endReplay behind it, ✕'s endReplay could reach the host
    // first and then replay()'s own reset would re-arm the replay underneath it.
    const replaying = engine.replay();
    const ending = engine.endReplay();
    await Promise.all([replaying, ending]);
    expect(engine.replayLeft).toBe(0);
    expect(engine.tick).toBe(0);
    expect(engine.population).toBe(1);
    await engine.advance(5);
    expect(engine.replayLeft).toBe(0);
    expect(engine.population).toBe(1);
    expect(counts.every((n) => n === 0 || n === 2)).toBe(true);
    expect(counts.at(-1)).toBe(0);
  });
});

describe('Engine handing over a world', () => {
  it('takes over another engine’s world: its transport, session and state', async () => {
    const a = await setup();
    const b = await setup();
    await b.engine.advance(4);
    expect(await b.engine.place(0, 2, {})).toBeNull();
    await b.engine.select(0, 2);
    const seen: EngineEvent[] = [];
    for (const e of ['reset', 'select', 'snapshot'] as const) a.engine.on(e, () => seen.push(e));
    await a.engine.takeWorld(b.engine);
    expect(seen).toEqual(['reset', 'select', 'snapshot']);
    expect([a.engine.tick, a.engine.population]).toEqual([4, 2]);
    expect(a.engine.selection).toEqual({ x: 0, y: 2, agentId: 2 });
    await a.engine.advance(1);
    expect(b.module.sims[0].ticks).toBe(5);
    expect(a.module.sims[0].ticks).toBe(0);
    expect((await a.engine.session()).session.log.map((e) => e.cmd.type)).toEqual(['place']);
    expect(b.engine.crashed).not.toBeNull();
    expect(a.engine.crashed).toBeNull();
  });

  it('leaves the other engine holding nothing: closing or using it cannot touch the world it gave up', async () => {
    const a = await setup();
    const b = await setup();
    await b.engine.advance(4);
    await a.engine.takeWorld(b.engine);
    b.engine.close();
    await b.engine.advance(1);
    expect(await b.engine.place(0, 2, {})).toEqual([{ field: 'simulation', message: 'This world now runs in another engine.' }]);
    await a.engine.advance(1);
    expect([a.engine.tick, b.module.sims[0].ticks]).toEqual([5, 5]);
    expect(a.engine.crashed).toBeNull();
  });

  it('refuses to take over its own world or a dead one', async () => {
    const a = await setup();
    const b = await setup();
    await expect(a.engine.takeWorld(a.engine)).rejects.toThrow('its own world');
    b.engine.close();
    await expect(a.engine.takeWorld(b.engine)).rejects.toThrow('no world to take over');
    await a.engine.advance(1);
    expect([a.engine.tick, a.module.sims[0].ticks]).toEqual([1, 1]);
    expect(a.engine.crashed).toBeNull();
  });

  it('close stops an engine for good, without a crash event', async () => {
    const { engine } = await setup();
    let crashes = 0;
    engine.on('crash', () => crashes++);
    engine.close();
    expect(engine.crashed).not.toBeNull();
    await engine.advance(1);
    expect(engine.tick).toBe(0);
    expect(crashes).toBe(0);
  });
});

describe('Chapter VI views while paused', () => {
  it('fetch the networks and histograms once, then send nothing while caught up', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const chapterVi = { width: 4, height: 3, sex: { enabled: true }, lifespan: { enabled: true }, culture: { enabled: true } } as unknown as Config;
    const engine = await Engine.create({ config: chapterVi, seed: 7 }, { presets, transport });
    // The Charts panel's distributions provider (ui/charts-panel.ts), with its receive step.
    const dist: DistState = { at: -Infinity, tick: -1, stale: true };
    engine.want((now) => (distributionsDue(dist, engine.tick, now, 250) ? distributionWants(engine.sugar) : {}));
    engine.on('snapshot', () => {
      const s = engine.last;
      if (s?.lorenz) Object.assign(dist, { at: performance.now(), tick: s.tick, stale: false });
    });
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        engine.pump((now += 1000));
        await settle();
      }
    };

    await pumps(1);
    expect(sent).toEqual(['refresh']);
    expect(engine.last?.ageHist).toEqual(Float64Array.of(5, 1, 0));
    expect(engine.last?.tagHist).toEqual(Float64Array.of(100, 0));

    engine.setDisplay({ colorMode: 'lineage', overlays: { neighbors: true, friends: true, family: true } });
    await settle();
    expect(Object.keys(engine.last?.networks ?? {})).toEqual(['neighbors', 'friends', 'family']);
    expect(engine.colorMode).toBe('lineage');

    // Paused and caught up, with three overlays and both histograms shown: nothing more is sent.
    await pumps(5);
    expect(sent).toEqual(['refresh', 'setDisplay']);
  });
});

describe('Engine.loadPreset', () => {
  it('loads a preset with a given seed', async () => {
    const { engine, module } = await setup();
    await engine.loadPreset('ii-2-unit', 42);
    expect(engine.seed).toBe(42);
    expect(engine.presetId).toBe('ii-2-unit');
    expect(module.sims.at(-1)!.seed).toBe(42);
  });
});

describe('Engine with other models', () => {
  const schelling = { model: 'schelling', width: 6, height: 4 } as unknown as ModelConfig;
  const ring = { model: 'ring', width: 5, height: 3 } as unknown as ModelConfig;
  /** A valley of 6 × 4 cells finishing at tick 10 (AD 800 to 810). */
  const valley = { model: 'anasazi', width: 6, height: 4, start_year: 800, end_year: 810, finish: 10 } as unknown as ModelConfig;
  const models: Preset[] = [
    ...presets,
    { id: 'vi-4', name: 'Schelling', source: 'VI-4', description: '', config: schelling },
    { id: 'vi-8', name: 'Ring', source: 'VI-8', description: '', config: ring },
  ];
  const make = (config: ModelConfig, transport = new HookedTransport(new SimHost(fakeModule()))) =>
    Engine.create({ config, seed: 1 }, { presets: models, transport });

  it('knows its model and keeps the last sugarscape config for the sugarscape panels', async () => {
    const e = await make(config);
    expect(e.model).toBe('sugarscape');
    expect(e.sugar).toBe(e.config);
    const before = e.sugar;
    expect(await e.loadPreset('vi-4')).toBeNull();
    expect(e.presetId).toBe('vi-4');
    expect(e.model).toBe('schelling');
    expect(e.sugar).toBe(before);
  });

  it('refuses sugarscape rule edits in another model and applies its own', async () => {
    const e = await make(ring);
    const refused = [{ field: 'config', message: 'this world is a ring world, not a sugarscape' }];
    expect(await e.applyConfig((c) => void (c.population = 5))).toEqual(refused);
    expect(await e.resetWith((c) => void (c.population = 5))).toEqual(refused);
    expect(await e.applyModelConfig((c) => void ((c as RingConfig).growback = 2))).toBeNull();
    expect((e.config as RingConfig).growback).toBe(2);
    expect((e.baseConfig as RingConfig).growback).toBe(2);
    expect(await e.resetModelWith((c) => void ((c as RingConfig).sites = 7))).toBeNull();
    expect(e.size().width).toBe(5);
  });

  it('asks for the ring with every request in Ring World and never for overlays; other models drop it', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const sent: Wants[] = [];
    transport.after = (_cmd, wants) => sent.push(wants ?? {});
    const e = await make(ring, transport);
    e.setDisplay({ overlays: { trade: true } });
    await e.advance(1);
    expect(sent.at(-1)).toMatchObject({ ring: true });
    expect(sent.at(-1)?.networks).toBeUndefined();
    expect(e.ring?.agents).toHaveLength(1);
    expect(await e.loadPreset('ii-2-unit')).toBeNull();
    expect(e.ring).toBeNull();
  });

  it('drops painted maps when a reset changes the model', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const resets: Command[] = [];
    transport.after = (cmd) => void (cmd.type === 'reset' && resets.push(cmd));
    const e = await make(config, transport);
    await e.paint(0, 0, 1, 3);
    expect(e.editedLandscapes()).toBeDefined();
    expect(await e.reset(ring)).toBeNull();
    expect(resets.at(-1)).toMatchObject({ type: 'reset', landscapes: [] });
  });

  it('asks only its own model’s chart groups, never sugarscape distributions; a paused, caught-up Charts page sends nothing', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const e = await make(schelling, transport);
    expect(e.model).toBe('schelling');
    // The Charts panel's provider (ui/charts-panel.ts): its own model's groups, and distributions
    // only in a sugarscape (the host silently drops them otherwise, so a model-blind provider would
    // never catch up and would refresh every 250 ms even while paused).
    const groups = [['population']];
    const dist: DistState = { at: -Infinity, tick: -1, stale: true };
    e.on('snapshot', () => {
      const s = e.last;
      if (s?.lorenz) Object.assign(dist, { at: performance.now(), tick: s.tick, stale: false });
    });
    e.want((now) => {
      const out: Wants = {};
      if (chartsBehind(groups, e.tick, (g) => e.chartGroup(g))) out.charts = { groups, max: 2000 };
      if (e.model === 'sugarscape' && distributionsDue(dist, e.tick, now, 250)) Object.assign(out, distributionWants(e.sugar));
      return out;
    });
    const sent: string[] = [];
    transport.after = (cmd) => sent.push(cmd.type);
    let now = performance.now();
    const pumps = async (n: number) => {
      for (let i = 0; i < n; i++) {
        e.pump((now += 1000));
        await settle();
      }
    };
    // One refresh catches the chart group up.
    await pumps(1);
    expect(sent).toEqual(['refresh']);
    // Paused and caught up: no repeated refresh, and no sugarscape distributions ever asked for.
    sent.length = 0;
    await pumps(5);
    expect(sent).toEqual([]);
  });

  it('asks for the valley with every request in the anasazi, and drops it in another model', async () => {
    const transport = new HookedTransport(new SimHost(fakeModule()));
    const sent: Wants[] = [];
    transport.after = (_cmd, wants) => sent.push(wants ?? {});
    const e = await make(valley, transport);
    e.setDisplay({ overlays: { trade: true, water: true } });
    await e.advance(1);
    expect(sent.at(-1)).toMatchObject({ valley: true });
    expect(sent.at(-1)?.networks).toBeUndefined();
    expect(e.overlays.water).toBe(true);
    expect(e.overlays.trade).toBe(false);
    expect(Array.from(e.valley!.water)).toEqual([0, 0, 2, 1]);
    expect(e.ticksLeft).toBe(9);
    expect(await e.loadPreset('ii-2-unit')).toBeNull();
    expect(e.valley).toBeNull();
    expect(e.ticksLeft).toBe(Infinity);
  });

  it('pauses at the end year and says so, again on a step there', async () => {
    const e = await make(valley);
    expect(finishedNotice(e.config, 10)).toBe('This run has reached its end year (AD 810) — Reset to run it again');
    expect(finishedNotice(ring, 10)).toBe('This run has reached its end year — Reset to run it again');
    let ends = 0;
    e.on('finished', () => ends++);
    e.setSpeed(4);
    e.setRunning(true);
    for (let i = 0; i < 4; i++) {
      e.pump(i);
      await settle();
    }
    expect([e.tick, e.finished, e.running, ends]).toEqual([10, true, false, 1]);
    await e.advance(1);
    expect([e.tick, ends]).toEqual([10, 2]);
    expect(await e.reset()).toBeNull();
    expect([e.tick, e.finished]).toEqual([0, false]);
  });
});

describe('Engine at the end year at Max', () => {
  beforeEach(() => void vi.useFakeTimers());
  afterEach(() => void vi.useRealTimers());

  it('pauses when the host’s Max loop ends at the end year', async () => {
    let clock = 0;
    const transport = new InlineTransport(new SimHost(fakeModule(), () => clock++));
    const config = { model: 'anasazi', width: 6, height: 4, start_year: 800, end_year: 850, finish: 50 } as unknown as ModelConfig;
    const e = await Engine.create({ config, seed: 1 }, { presets, transport });
    let ends = 0;
    e.on('finished', () => ends++);
    e.setSpeed('max');
    e.setRunning(true);
    await vi.advanceTimersByTimeAsync(60);
    expect([e.tick, e.running, ends]).toEqual([50, false, 1]);
  });
});
