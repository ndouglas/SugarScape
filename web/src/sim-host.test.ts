import { describe, expect, it, vi } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import {
  noOverlays,
  type Command,
  type DisplayState,
  type EditCommand,
  type HostMessage,
  type HostReply,
  type LogEntry,
  type SessionLog,
  type Wants,
  type WorldSnapshot,
} from './protocol';
import { AGE_BIN, BATCH_MS, channelDefer, LOG_CAP, serve, SimHost } from './sim-host';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() };

/** A host with a world (`init` already answered); `send` numbers requests. */
function start(clock?: () => number) {
  const log: string[] = [];
  const module = fakeModule(log);
  const host = new SimHost(module, clock);
  let id = 0;
  const send = (cmd: Command, extra: { wants?: Wants; frame?: ArrayBuffer } = {}): HostReply =>
    host.handle({ id: ++id, cmd, ...extra });
  const snap = (reply: HostReply): WorldSnapshot => {
    if (!reply.result.ok || !reply.result.snapshot) throw new Error(JSON.stringify(reply.result));
    return reply.result.snapshot;
  };
  send({ type: 'init', config, seed: 1, landscapes: [], display });
  return { host, log, module, send, snap };
}

describe('SimHost', () => {
  it('needs init before anything but ready', () => {
    const host = new SimHost(fakeModule());
    expect(host.handle({ id: 1, cmd: { type: 'ready' } }).result).toEqual({ ok: true });
    expect(host.handle({ id: 2, cmd: { type: 'step', n: 1 } }).result).toEqual({
      ok: false,
      errors: [{ field: 'world', message: 'no world yet' }],
    });
  });

  it('answers init with the world, its config and landscapes, rendered into the lent buffer', () => {
    const host = new SimHost(fakeModule());
    const frame = new ArrayBuffer(48);
    const reply = host.handle({ id: 7, cmd: { type: 'init', config, seed: 3, landscapes: [], display }, frame });
    expect(reply.id).toBe(7);
    expect(reply.spare).toBeUndefined();
    const s = (reply.result as { snapshot: WorldSnapshot }).snapshot;
    expect(s).toMatchObject({ width: 4, height: 3, tick: 0, population: 1, followed: null, followedAlive: false, editedLandscapes: [null] });
    expect(s.latest).toEqual({ tick: 0, population: 1 });
    expect(s.config?.goods).toHaveLength(1);
    expect(s.frame).toBe(frame);
  });

  it('renders into the lent buffer, or a new one when the size differs, and returns what it did not use', () => {
    const t = start();
    t.send({ type: 'step', n: 3 });
    const right = new ArrayBuffer(48);
    const r1 = t.send({ type: 'refresh' }, { frame: right });
    expect(t.snap(r1).frame).toBe(right);
    expect(new Uint8Array(right)[0]).toBe(3);
    const small = new ArrayBuffer(8);
    const r2 = t.send({ type: 'refresh' }, { frame: small });
    expect(t.snap(r2).frame?.byteLength).toBe(48);
    expect(r2.spare?.[0]).toBe(small);
    expect(t.snap(t.send({ type: 'refresh' })).frame).toBeUndefined();
  });

  it('adds extras only when wanted', () => {
    const t = start();
    const bare = t.snap(t.send({ type: 'step', n: 1 }));
    expect(bare.tick).toBe(1);
    const keys = [
      'inspection', 'trail', 'networks', 'charts', 'lorenz', 'wealthHist',
      'ageHist', 'tagHist', 'lorenzTotal', 'goodWealthHists', 'supplyDemand',
      'creditGraph', 'diseaseList', 'config', 'editedLandscapes', 'display', 'frame',
    ] as const;
    for (const key of keys) expect(bare[key], key).toBeUndefined();
    const all: Wants = {
      select: { x: 0, y: 0, agentId: null },
      trail: true,
      networks: ['trade', 'neighbors', 'friends', 'family'],
      charts: { groups: [['population']], max: 10 },
      lorenz: true,
      wealthHist: true,
      ageHist: true,
      tagHist: true,
      lorenzTotal: true,
      goodWealthHists: true,
      supplyDemand: true,
      creditGraph: true,
      diseaseList: true,
    };
    const full = t.snap(t.send({ type: 'step', n: 1 }, { wants: all }));
    expect(full.inspection?.view.site).toMatchObject({ x: 0, y: 0 });
    expect(full.trail).toBeDefined();
    expect(full.networks?.trade).toEqual(Uint32Array.of(0, 0, 1, 1));
    expect(Object.keys(full.charts ?? {})).toEqual(['population']);
    expect(full.lorenz).toHaveLength(101);
    expect(full.wealthHist).toHaveLength(21);
    expect(Object.keys(full.networks ?? {})).toEqual(['trade', 'neighbors', 'friends', 'family']);
    expect(full.ageHist).toEqual(Float64Array.of(AGE_BIN, 1, 0));
    expect(full.tagHist).toEqual(Float64Array.of(100, 0));
    expect(full.lorenzTotal).toHaveLength(101);
    expect(full.goodWealthHists?.map((h) => [h.length, h[0]])).toEqual([[21, 1]]); // one good, 20 bins
    expect(full.supplyDemand).toHaveLength(5);
    expect(full.creditGraph).toEqual({ agents: [], loans: [] });
    expect(full.diseaseList).toEqual([{ id: 0, bits: '01', carriers: 2 }]);
  });

  it('sends a chart group only with news: 250 ms later or 1 % more history, and afresh after a config change', () => {
    let clock = 0;
    const t = start(() => clock);
    const charts = { groups: [['population', 'gini']], max: 50 };
    const key = 'population|gini';
    const step = (n: number) => t.snap(t.send({ type: 'step', n }, { wants: { charts } })).charts?.[key];
    const first = step(999); // history: 1000 ticks
    expect(first?.ticks).toHaveLength(50);
    expect(first?.columns).toHaveLength(2);
    clock = 10;
    expect(step(1)).toBeUndefined(); // 0.1 % more, 10 ms later
    expect(step(10)).toBeDefined(); // 1011 > 1010: more than 1 % more
    clock = 300;
    expect(step(1)).toBeDefined(); // 290 ms later
    clock = 600;
    expect(t.snap(t.send({ type: 'refresh' }, { wants: { charts } })).charts).toBeUndefined(); // no new ticks
    expect(t.snap(t.send({ type: 'setConfig', config }, { wants: { charts } })).charts?.[key]).toBeDefined();
    const reset = t.snap(t.send({ type: 'reset', config, seed: 2, landscapes: [] }, { wants: { charts } }));
    expect(reset.charts?.[key]?.ticks).toEqual(Float64Array.of(0));
  });

  it('only a refresh escapes the charts throttle; a step stays throttled like everything else', () => {
    let clock = 0;
    const t = start(() => clock);
    const charts = { groups: [['population']], max: 50 };
    const key = 'population';
    t.send({ type: 'step', n: 999 }, { wants: { charts } }); // history: 1000 ticks, sent now
    clock = 10;
    // A further step this soon, with this little growth, is throttled (same numbers as above).
    expect(t.snap(t.send({ type: 'step', n: 1 }, { wants: { charts } })).charts?.[key]).toBeUndefined();
    // But a refresh is not: it sees the one extra tick and sends it, even though it is just as soon
    // and just as small a change — it is the only path the paused catch-up needs, and it is already
    // paced client-side by REFRESH_MS (Engine.pump), so the panel must not stall waiting for it.
    expect(t.snap(t.send({ type: 'refresh' }, { wants: { charts } })).charts?.[key]).toBeDefined();
  });

  it('throttles a paint command exactly like a step, so a drag cannot resend on every pointermove', () => {
    let clock = 0;
    const t = start(() => clock);
    const charts = { groups: [['population']], max: 50 };
    const key = 'population';
    t.send({ type: 'step', n: 999 }, { wants: { charts } }); // history: 1000 ticks, sent now
    clock = 10;
    t.send({ type: 'step', n: 1 }); // one more tick, but this step does not ask for charts
    // Paint tools send a command on every pointermove with no debounce: this one arrives 10 ms
    // after the last send, with 0.1 % more history — it must not resend the group.
    const paint = { type: 'paint' as const, x: 0, y: 0, radius: 1, value: 3, good: 0 };
    expect(t.snap(t.send(paint, { wants: { charts } })).charts?.[key]).toBeUndefined();
    // A refresh in the same window still gets it.
    expect(t.snap(t.send({ type: 'refresh' }, { wants: { charts } })).charts?.[key]).toBeDefined();
  });

  it('leaves out extras the world cannot give instead of failing the command', () => {
    const t = start();
    const s = t.snap(
      t.send({ type: 'setConfig', config }, { wants: { charts: { groups: [['nope']], max: 10 }, select: { x: 99, y: 99, agentId: null } } }),
    );
    expect(s.config).toBeDefined();
    expect(s.charts).toBeUndefined();
    expect(s.inspection).toBeUndefined();
  });

  it('answers field errors in the core shape, keeps the world and returns the buffer', () => {
    const t = start();
    t.send({ type: 'step', n: 2 });
    const frame = new ArrayBuffer(48);
    const bad = { ...config, population: 5000 };
    const r = t.send({ type: 'setConfig', config: bad }, { frame });
    expect(r.result).toEqual({ ok: false, errors: [{ field: 'population', message: 'too many' }] });
    expect(r.spare?.[0]).toBe(frame);
    expect(t.send({ type: 'reset', config: bad, seed: 1, landscapes: [] }).result.ok).toBe(false);
    expect(t.module.sims).toHaveLength(1);
    expect(t.snap(t.send({ type: 'refresh' })).tick).toBe(2);
    expect(t.send({ type: 'erase', x: 3, y: 2 }).result).toEqual({
      ok: false,
      errors: [{ field: 'edit', message: 'no agent at (3, 2)' }],
    });
  });

  it('turns a panic into a fatal reply and refuses every later command', () => {
    const t = start();
    const r = t.send({ type: 'paint', x: 0, y: 0, radius: 1, value: -1, good: 0 });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
    expect(t.send({ type: 'step', n: 1 }).result).toEqual(r.result);
  });

  it('inspects by site or agent id and keeps a selected agent tracked', () => {
    const t = start();
    expect(t.snap(t.send({ type: 'inspect', target: { x: 1, y: 1 } })).inspection).toMatchObject({ x: 1, y: 1, agentId: 1, alive: true });
    const moved = t.snap(t.send({ type: 'step', n: 2 }, { wants: { select: { x: 1, y: 1, agentId: 1 } } })).inspection;
    expect(moved).toMatchObject({ x: 3, y: 1, agentId: 1, alive: true });
    expect(t.snap(t.send({ type: 'inspect', target: { agentId: 1 } })).inspection).toMatchObject({ x: 3, y: 1 });
    t.send({ type: 'erase', x: 3, y: 1 });
    const gone = t.snap(t.send({ type: 'refresh' }, { wants: { select: { x: 3, y: 1, agentId: 1 } } })).inspection;
    expect(gone).toMatchObject({ x: 3, y: 1, agentId: 1, alive: false });
    expect(t.snap(t.send({ type: 'inspect', target: { agentId: 1 } })).inspection).toBeNull();
  });

  it('follows with the trail included, and clamps the display to the config', () => {
    const t = start();
    const followed = t.snap(t.send({ type: 'follow', id: 1 }));
    expect(followed).toMatchObject({ followed: 1, followedAlive: true });
    expect(Array.from(followed.trail ?? [])).toEqual([1, 1]);
    const clamped = t.snap(t.send({ type: 'setDisplay', display: { ...display, colorMode: 'disease', layer: 'capacity:3' } }));
    expect(clamped.display).toEqual(display);
  });

  it('reports a scheduled change as a config change, and edited landscapes after edits', () => {
    const t = start();
    t.send({ type: 'setConfig', config: { ...config, schedule: [{ tick: 5, set: {} }] } });
    expect(t.snap(t.send({ type: 'step', n: 4 })).config).toBeUndefined(); // ticks 0–3 started
    expect(t.snap(t.send({ type: 'step', n: 2 })).config?.schedule).toHaveLength(1); // the step from 5 started
    expect(t.snap(t.send({ type: 'paint', x: 0, y: 0, radius: 1, value: 3, good: 0 })).editedLandscapes).toEqual([
      new Uint8Array(12).fill(7),
    ]);
    expect(t.snap(t.send({ type: 'place', x: 0, y: 2, overrides: {} })).editedLandscapes).toBeUndefined();
  });

  it('answers exports and the fingerprint with values', () => {
    const t = start();
    t.send({ type: 'step', n: 26 });
    expect(t.send({ type: 'seriesCsv' }).result).toEqual({ ok: true, value: 'tick,population\n26,1\n' });
    expect(t.send({ type: 'agentsCsv' }).result).toEqual({ ok: true, value: 'id\n1\n' });
    expect(t.send({ type: 'fingerprint' }).result).toEqual({ ok: true, value: '0x1a' });
  });
});

describe('SimHost at Max speed', () => {
  it('steps in batches and posts about every 33 ms while it holds a free buffer', () => {
    let clock = 0;
    const t = start(() => clock++); // each reading of the clock is 1 ms later
    const a = new ArrayBuffer(48);
    const b = new ArrayBuffer(48);
    const run = t.send({ type: 'run' }, { frame: a });
    expect(run.result).toEqual({ ok: true });
    expect(run.spare).toBeUndefined();
    expect(t.host.running).toBe(true);
    const posts: WorldSnapshot[] = [];
    for (let i = 0; i < 6; i++) {
      const post = t.host.batch();
      if (post) posts.push(post);
    }
    expect(posts).toHaveLength(1);
    expect(posts[0].frame).toBe(a);
    expect(posts[0].tick).toBeGreaterThan(16);
    t.send({ type: 'frame' }, { frame: b });
    expect(t.host.batch()?.frame).toBe(b);
  });

  it('handles commands between batches and answers stop with the last frame and the unused buffers', () => {
    let clock = 0;
    const t = start(() => clock++);
    const a = new ArrayBuffer(48);
    const b = new ArrayBuffer(48);
    t.send({ type: 'run' }, { frame: a });
    t.send({ type: 'frame' }, { frame: b });
    expect(t.host.batch()).toBeNull();
    expect(t.snap(t.send({ type: 'place', x: 0, y: 2, overrides: {} })).population).toBe(2);
    const stop = t.send({ type: 'stop' });
    expect(t.host.running).toBe(false);
    expect(t.snap(stop).frame).toBe(b);
    expect(stop.spare).toHaveLength(1);
    expect(stop.spare?.[0]).toBe(a);
    expect(t.host.batch()).toBeNull();
    const late = new ArrayBuffer(48);
    expect(t.send({ type: 'frame' }, { frame: late }).spare?.[0]).toBe(late);
  });

  it('reports a scheduled change in the next post', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'setConfig', config: { ...config, schedule: [{ tick: 3, set: {} }] } });
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.config?.schedule).toHaveLength(1);
  });

  it('updates the loop selection on inspect while running, so the next post carries it', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    // No `wants` on this request: at send time the engine's own wants still describe whatever
    // selection existed before the inspect (none, here) — only the reply says what was selected.
    t.send({ type: 'inspect', target: { x: 1, y: 1 } });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.inspection?.agentId).toBe(1);
    expect(post.inspection?.alive).toBe(true);
  });

  it('caps a batch at the next post deadline: the posting batch is shorter than a full one', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    const tickNow = (): number => t.snap(t.send({ type: 'refresh' })).tick;
    let prevTick = tickNow();
    const deltas: number[] = [];
    let post: WorldSnapshot | null = null;
    for (let i = 0; i < 4 && !post; i++) {
      post = t.host.batch();
      const tick = tickNow();
      deltas.push(tick - prevTick);
      prevTick = tick;
    }
    expect(post).not.toBeNull();
    // Every batch before the last ran the full BATCH_MS; the one that posts is capped short by
    // min(start + BATCH_MS, posted + POST_MS), landing the post near POST_MS instead of a whole
    // extra BATCH_MS late.
    expect(deltas[deltas.length - 1]).toBeLessThan(deltas[0]);
  });

  it('keeps stepping full batches while no buffer is free, even long past the post deadline', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch(); // uses the only buffer
    const tickNow = (): number => t.snap(t.send({ type: 'refresh' })).tick;
    // No buffer is pooled now; a few more batches push the clock well past `posted + POST_MS`
    // without ever returning one. If the deadline cap applied regardless of a free buffer, each of
    // these would step only a single tick once past the (unreachable) deadline.
    for (let i = 0; i < 3; i++) t.host.batch();
    const before = tickNow();
    t.host.batch();
    const after = tickNow();
    expect(after - before).toBeGreaterThan(BATCH_MS / 2);
  });

  it('keeps the pooled buffers when run is sent again while already running', () => {
    let clock = 0;
    const t = start(() => clock++);
    const a = new ArrayBuffer(48);
    const b = new ArrayBuffer(48);
    t.send({ type: 'run' }, { frame: a });
    const run2 = t.send({ type: 'run' }, { frame: b });
    expect(run2.spare).toBeUndefined(); // neither buffer bounced back
    const stop = t.send({ type: 'stop' });
    const kept = new Set([t.snap(stop).frame, ...(stop.spare ?? [])]);
    expect(kept).toEqual(new Set([a, b]));
  });

  it('answers stop with the buffer it was just lent when nothing was pooled', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch(); // empties the pool
    const b = new ArrayBuffer(48);
    const stop = t.send({ type: 'stop' }, { frame: b });
    expect(t.snap(stop).frame).toBe(b);
    expect(stop.spare).toBeUndefined();
  });

  it('updates the loop selection on follow while running, so the next post carries the trail', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    // No `wants` on this request either, same reasoning as the inspect case above.
    t.send({ type: 'follow', id: 1 });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.followed).toBe(1);
    expect(post.trail).toBeDefined();
  });

  it('serve keeps posting between requests until stop', () => {
    let clock = 0;
    const messages: HostMessage[] = [];
    // Deferred batches run only when the test says so.
    const queue: (() => void)[] = [];
    const flush = (batches: number) => {
      for (let i = 0; i < batches; i++) queue.shift()?.();
    };
    const handle = serve(new SimHost(fakeModule(), () => clock++), (m) => messages.push(m), (fn) => queue.push(fn));
    const posts = () => messages.filter((m) => m.id === null).length;
    handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display } });
    handle({ id: 2, cmd: { type: 'run' }, frame: new ArrayBuffer(48) });
    flush(20);
    expect(posts()).toBe(1); // one buffer: one post until it comes back
    handle({ id: 3, cmd: { type: 'frame' }, frame: new ArrayBuffer(48) });
    flush(20);
    expect(posts()).toBe(2);
    handle({ id: 4, cmd: { type: 'stop' } });
    const count = messages.length;
    flush(20);
    expect(messages.length).toBe(count);
    expect(queue).toHaveLength(0); // the loop ended
  });
});

describe('serve', () => {
  it('answers each request in order and lists its buffers for transfer', () => {
    const sent: [HostMessage, Transferable[]][] = [];
    const handle = serve(new SimHost(fakeModule()), (m, t) => sent.push([m, t]), () => {});
    const frame = new ArrayBuffer(48);
    handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display }, frame });
    handle({ id: 2, cmd: { type: 'fingerprint' } });
    expect(sent.map(([m]) => m.id)).toEqual([1, 2]);
    expect(sent[0][1]).toHaveLength(1);
    expect(sent[0][1][0]).toBe(frame);
    expect((sent[1][0] as HostReply).result).toEqual({ ok: true, value: '0x0' });
  });
});

describe('SimHost edit log and replay', () => {
  const place = (x: number, y: number): EditCommand => ({ type: 'place', x, y, overrides: {} });
  const paint: EditCommand = { type: 'paint', x: 0, y: 0, radius: 1, value: 3, good: 0 };
  const session = (t: ReturnType<typeof start>): SessionLog => {
    const r = t.send({ type: 'session' }).result;
    if (!r.ok || !r.session) throw new Error(JSON.stringify(r));
    return r.session;
  };

  it('logs world-changing commands that succeed, with the tick they were applied at', () => {
    const t = start();
    t.send({ type: 'step', n: 2 });
    t.send(place(0, 2));
    expect(t.send({ type: 'erase', x: 3, y: 2 }).result.ok).toBe(false); // nobody there: not logged
    t.send({ type: 'follow', id: 1 });
    t.send({ type: 'inspect', target: { x: 1, y: 1 } });
    t.send({ type: 'setDisplay', display });
    t.send({ type: 'step', n: 1 });
    t.send(paint);
    t.send({ type: 'importLandscape', good: 0, capacities: new Uint8Array(12) });
    t.send({ type: 'infect', x: 0, y: 2, disease: -1 });
    t.send({ type: 'vaccinate', x: 0, y: 2, radius: 1, disease: 0 });
    t.send({ type: 'setConfig', config });
    t.send({ type: 'refresh' });
    t.send({ type: 'seriesCsv' });
    const s = session(t);
    expect(s.full).toBe(false);
    expect(s.tick).toBe(3);
    expect(s.log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [2, 'place'],
      [3, 'paint'],
      [3, 'importLandscape'],
      [3, 'infect'],
      [3, 'vaccinate'],
      [3, 'setConfig'],
    ]);
    expect(s.log[5].cmd).toEqual({ type: 'setConfig', config });
  });

  it('starts an empty log with every new world', () => {
    const t = start();
    t.send(place(0, 2));
    t.send({ type: 'reset', config, seed: 2, landscapes: [] });
    expect(session(t).log).toEqual([]);
  });

  it(
    `stops recording past ${LOG_CAP} entries and says so`,
    () => {
      const t = start();
      const infect: Command = { type: 'infect', x: 0, y: 0, disease: -1 };
      for (let i = 0; i < LOG_CAP; i++) t.send(infect);
      expect(session(t).full).toBe(false);
      t.send(infect);
      const s = session(t);
      expect(s.full).toBe(true);
      expect(s.log).toHaveLength(LOG_CAP);
    },
    20_000,
  );

  it('replays tick-0 entries at once and later ones inside a step, stopping at each entry’s tick', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 0, cmd: paint },
      { tick: 3, cmd: place(0, 2) },
      { tick: 3, cmd: { type: 'erase', x: 0, y: 2 } },
      { tick: 5, cmd: place(0, 0) },
    ];
    const built = t.snap(t.send({ type: 'reset', config, seed: 1, landscapes: [], log }));
    expect(built.replayLeft).toBe(3);
    expect(built.editedLandscapes).toEqual([new Uint8Array(12).fill(7)]); // painted at tick 0
    const sim = t.module.sims.at(-1)!;
    expect(t.snap(t.send({ type: 'step', n: 2 })).replayLeft).toBeUndefined(); // unchanged
    expect(sim.stepCalls).toBe(1);
    expect(t.snap(t.send({ type: 'step', n: 2 }))).toMatchObject({ tick: 4, population: 1, replayLeft: 1 });
    expect(sim.stepCalls).toBe(3); // 2 → 3, apply both entries at 3, 3 → 4
    expect(t.log.filter((line) => line.startsWith('place'))).toHaveLength(1);
    expect(t.snap(t.send({ type: 'step', n: 10 }))).toMatchObject({ tick: 14, population: 2, replayLeft: 0 });
    expect(sim.stepCalls).toBe(5); // 4 → 5, apply, 5 → 14
    expect(session(t).log).toEqual(log); // re-logged identically
  });

  it('replays inside Max batches', () => {
    let clock = 0;
    const t = start(() => clock++);
    const log: LogEntry[] = [{ tick: 4, cmd: place(0, 2) }];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    while (!post) post = t.host.batch();
    expect(post.tick).toBeGreaterThan(4);
    expect(post).toMatchObject({ population: 2, replayLeft: 0 });
    t.send({ type: 'stop' });
    expect(session(t).log).toEqual(log);
  });

  it('forks: a page edit that succeeds while entries are pending drops them, then is applied and logged', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 1, cmd: place(0, 2) },
      { tick: 5, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    expect(t.snap(t.send({ type: 'step', n: 2 })).replayLeft).toBe(1);
    expect(t.send({ type: 'erase', x: 3, y: 0 }).result.ok).toBe(false); // failed: no fork
    expect(session(t).log.map((e) => e.tick)).toEqual([1, 5]);
    expect(t.snap(t.send(paint))).toMatchObject({ forked: true, replayLeft: 0 });
    const later = t.snap(t.send({ type: 'step', n: 5 }));
    expect(later.forked).toBeUndefined();
    expect(later.population).toBe(2); // the erase at 5 was dropped
    expect(session(t).log.map((e) => [e.tick, e.cmd.type])).toEqual([
      [1, 'place'],
      [2, 'paint'],
    ]);
  });

  it('endReplay drops the pending entries and keeps the world', () => {
    const t = start();
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log: [{ tick: 2, cmd: place(0, 2) }] });
    const s = t.snap(t.send({ type: 'endReplay' }));
    expect(s).toMatchObject({ tick: 0, replayLeft: 0 });
    expect(s.forked).toBeUndefined();
    expect(t.snap(t.send({ type: 'step', n: 3 })).population).toBe(1);
    expect(session(t).log).toEqual([]);
  });

  it('includes pending entries in the session, after the applied ones', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 1, cmd: place(0, 2) },
      { tick: 9, cmd: { type: 'erase', x: 0, y: 2 } },
    ];
    t.send({ type: 'reset', config, seed: 1, landscapes: [], log });
    t.send({ type: 'step', n: 3 });
    expect(session(t)).toEqual({ log, full: false, tick: 3 });
  });

  it('skips a replayed entry the world rejects, but a panic is still fatal', () => {
    const t = start();
    const log: LogEntry[] = [
      { tick: 0, cmd: { type: 'erase', x: 3, y: 2 } },
      { tick: 0, cmd: place(0, 2) },
    ];
    expect(t.snap(t.send({ type: 'reset', config, seed: 1, landscapes: [], log }))).toMatchObject({ population: 2, replayLeft: 0 });
    expect(session(t).log.map((e) => e.cmd.type)).toEqual(['place']);
    const panic: LogEntry[] = [{ tick: 0, cmd: { ...paint, value: -1 } as EditCommand }];
    const r = t.send({ type: 'reset', config, seed: 1, landscapes: [], log: panic });
    expect(r.result).toEqual({ ok: false, fatal: 'The simulation stopped: unreachable executed' });
  });
});

describe('channelDefer', () => {
  it('runs deferred callbacks in FIFO order', async () => {
    // Captures the MessageChannel channelDefer creates internally so its ports can be closed
    // afterwards (Node keeps the process alive while a MessagePort is open).
    const RealMessageChannel = globalThis.MessageChannel;
    const channels: MessageChannel[] = [];
    const spy = vi.spyOn(globalThis, 'MessageChannel').mockImplementation(function (this: unknown) {
      const channel = new RealMessageChannel();
      channels.push(channel);
      return channel;
    } as unknown as typeof MessageChannel);
    try {
      const defer = channelDefer();
      const order: number[] = [];
      defer(() => order.push(1));
      defer(() => order.push(2));
      defer(() => order.push(3));
      // A fourth deferred callback: when it runs, every earlier one has (a MessagePort message
      // is not ordered against a timer, so waiting on setTimeout would race).
      await new Promise<void>((resolve) => defer(resolve));
      expect(order).toEqual([1, 2, 3]);
    } finally {
      spy.mockRestore();
      for (const channel of channels) {
        channel.port1.close();
        channel.port2.close();
      }
    }
  });
});
