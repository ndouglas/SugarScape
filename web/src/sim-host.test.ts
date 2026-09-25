import { describe, expect, it, vi } from 'vitest';
import { fakeModule, FakeSim } from './fake-sim.fixture';
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
import { AGE_BIN, BATCH_MS, channelDefer, KEYFRAME_EVERY, LOG_CAP, MAX_KEYFRAMES, MAX_TICKS, serve, SimHost } from './sim-host';
import type { Config, FieldError, ModelConfig } from './types';

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
    expect((s.config as Config).goods).toHaveLength(1);
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
    expect((t.snap(t.send({ type: 'step', n: 2 })).config as Config).schedule).toHaveLength(1); // the step from 5 started
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
    expect(t.send({ type: 'fingerprint' }).result).toEqual({ ok: true, value: '0x1a|1@3,1|10' });
  });
});

describe('SimHost with another model', () => {
  /** A host whose world is a fake of `model`. */
  function other(model: 'schelling' | 'ring' | 'anasazi', finish?: number) {
    const host = new SimHost(fakeModule());
    const config = { model, width: 6, height: 4, finish } as unknown as ModelConfig;
    let id = 0;
    const send = (cmd: Command, wants?: Wants): WorldSnapshot => {
      const reply = host.handle({ id: ++id, cmd, wants });
      if (!reply.result.ok || !reply.result.snapshot) throw new Error(JSON.stringify(reply.result));
      return reply.result.snapshot;
    };
    const init = send({ type: 'init', config, seed: 1, landscapes: [], display });
    return { init, send };
  }

  it('clamps the display to the model and sends no maps', () => {
    const { init } = other('schelling');
    expect(init.display).toEqual({ colorMode: 'color', layer: 'resource:0', overlays: noOverlays() });
    expect(init.editedLandscapes).toEqual([]);
  });

  it('answers only the wishes the model can: charts and the ring, never sugarscape extras', () => {
    const { send } = other('ring');
    const s = send({ type: 'step', n: 2 }, {
      ring: true,
      trail: true,
      networks: ['trade'],
      lorenz: true,
      diseaseList: true,
      charts: { groups: [['population']], max: 10 },
    });
    expect(s.ring?.sugar).toHaveLength(6);
    expect(Array.from(s.ring!.agents)).toEqual([3]);
    expect(s.charts).toBeDefined();
    for (const key of ['trail', 'networks', 'lorenz', 'diseaseList'] as const) expect(s[key]).toBeUndefined();
  });

  it('sends the ring only for Ring World', () => {
    const { send } = other('schelling');
    expect(send({ type: 'refresh' }, { ring: true }).ring).toBeUndefined();
  });

  it('sends the valley’s overlays only for the anasazi, and only when wanted', () => {
    const { send } = other('anasazi');
    const s = send({ type: 'step', n: 1 }, { valley: true });
    expect(Array.from(s.valley!.water)).toEqual([0, 0, 2, 1]);
    expect(Array.from(s.valley!.settlements)).toEqual([2, 0, 1]);
    expect(Array.from(s.valley!.links)).toEqual([2, 1, 2, 0]);
    expect(send({ type: 'refresh' }).valley).toBeUndefined();
    expect(other('ring').send({ type: 'refresh' }, { valley: true }).valley).toBeUndefined();
  });

  it('says when the world is finished, and a step there changes nothing', () => {
    const { init, send } = other('anasazi', 3);
    expect(init.finished).toBeUndefined();
    expect(send({ type: 'step', n: 2 }).finished).toBeUndefined();
    const end = send({ type: 'step', n: 5 });
    expect([end.tick, end.finished]).toEqual([3, true]);
    const again = send({ type: 'step', n: 1 });
    expect([again.tick, again.finished]).toEqual([3, true]);
  });

  it('ends Max when the world is finished, posting the world there', () => {
    let clock = 0;
    const host = new SimHost(fakeModule(), () => clock++);
    const config = { model: 'anasazi', width: 6, height: 4, finish: 5 } as unknown as ModelConfig;
    host.handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display } });
    host.handle({ id: 2, cmd: { type: 'run' }, frame: new ArrayBuffer(96) });
    let post: WorldSnapshot | null = null;
    for (let i = 0; i < 10 && !post; i++) post = host.batch();
    expect([post?.tick, post?.finished]).toEqual([5, true]);
    expect(host.running).toBe(false);
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
    expect((post.config as Config).schedule).toHaveLength(1);
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

describe('SimHost at MAX_TICKS', () => {
  const FULL = { ok: false, errors: [{ field: 'tick', message: 'this world has reached 1,000,000 ticks, the most its history holds here' }] };

  it('steps no further than the cap, and answers a step there with a field error', () => {
    const t = start();
    expect(t.snap(t.send({ type: 'step', n: MAX_TICKS - 2 })).tick).toBe(MAX_TICKS - 2);
    expect(t.snap(t.send({ type: 'step', n: 10 })).tick).toBe(MAX_TICKS);
    expect(t.send({ type: 'step', n: 1 }).result).toEqual(FULL);
    expect(t.snap(t.send({ type: 'refresh' })).tick).toBe(MAX_TICKS);
  });

  it('ends Max at the cap, posting the world there', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'step', n: MAX_TICKS - 3 });
    t.send({ type: 'run' }, { frame: new ArrayBuffer(48) });
    let post: WorldSnapshot | null = null;
    for (let i = 0; i < 10 && !post; i++) post = t.host.batch();
    expect(post?.tick).toBe(MAX_TICKS);
    expect(post?.frame).toBeDefined();
    expect(t.host.running).toBe(false);
    expect(t.host.batch()).toBeNull();
    expect(t.snap(t.send({ type: 'stop' })).tick).toBe(MAX_TICKS);
  });

  it('holds Max at the cap until a buffer is free to post the last frame in', () => {
    let clock = 0;
    const t = start(() => clock++);
    t.send({ type: 'step', n: MAX_TICKS - 3 });
    t.send({ type: 'run' });
    expect(t.host.batch()).toBeNull();
    expect(t.host.running).toBe(true);
    const b = new ArrayBuffer(48);
    t.send({ type: 'frame' }, { frame: b });
    const post = t.host.batch();
    expect(post?.frame).toBe(b);
    expect(post?.tick).toBe(MAX_TICKS);
    expect(t.host.running).toBe(false);
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
    expect((sent[1][0] as HostReply).result).toEqual({ ok: true, value: '0x0|1@1,1|10' });
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

describe('SimHost keyframes', () => {
  const init = (host: SimHost) =>
    host.handle({ id: 1, cmd: { type: 'init', config: { width: 4, height: 3 } as never, seed: 1, landscapes: [], display: { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } } });

  it('keeps one at t = 0 and one every KEYFRAME_EVERY ticks', () => {
    const host = new SimHost(fakeModule());
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: 175 } });
    expect(host.keyframeTicks()).toEqual([0, 50, 100, 150]);
  });

  it('thins to every other one and doubles the interval past MAX_KEYFRAMES', () => {
    const host = new SimHost(fakeModule());
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: KEYFRAME_EVERY * MAX_KEYFRAMES } });
    const ticks = host.keyframeTicks();
    expect(ticks.length).toBeLessThanOrEqual(MAX_KEYFRAMES);
    expect(ticks.every((t) => t % (2 * KEYFRAME_EVERY) === 0)).toBe(true);
    expect(ticks.at(-1)).toBe(KEYFRAME_EVERY * MAX_KEYFRAMES);
  });

  it('frees thinned keyframes, and all of them on reset', () => {
    const log: string[] = [];
    const host = new SimHost(fakeModule(log));
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: KEYFRAME_EVERY * MAX_KEYFRAMES } });
    const freed = log.filter((l) => l === 'free checkpoint').length;
    expect(freed).toBeGreaterThan(0);
    host.handle({ id: 3, cmd: { type: 'reset', config: { width: 4, height: 3 } as never, seed: 2, landscapes: [] } });
    expect(host.keyframeTicks()).toEqual([0]);
  });

  it('resets the doubled interval on a branch, so a fresh branch near t = 0 keeps 50-tick spacing', () => {
    const host = new SimHost(fakeModule());
    init(host);
    // Thins and doubles `every` at least once.
    host.handle({ id: 2, cmd: { type: 'step', n: KEYFRAME_EVERY * MAX_KEYFRAMES } });
    host.handle({ id: 3, cmd: { type: 'seek', tick: 0 } });
    // A page edit at (near) t = 0 branches: without the reset, `every` would stay doubled and only
    // every other 50-tick mark would get a keyframe.
    host.handle({ id: 4, cmd: { type: 'place', x: 0, y: 0, overrides: {} } });
    host.handle({ id: 5, cmd: { type: 'step', n: 175 } });
    expect(host.keyframeTicks()).toEqual([0, 50, 100, 150]);
  });

  it('keeps none for a model without keyframes', () => {
    const host = new SimHost(fakeModule([], { keyframes: false }));
    init(host);
    host.handle({ id: 2, cmd: { type: 'step', n: 120 } });
    expect(host.keyframeTicks()).toEqual([]);
  });
});

describe('SimHost seek', () => {
  const display = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } as const;
  let id = 0;
  const send = (host: SimHost, cmd: Command) => host.handle({ id: ++id, cmd }).result;
  const setup = (opts: { keyframes?: boolean } = {}) => {
    const module = fakeModule([], opts);
    const host = new SimHost(module);
    send(host, { type: 'init', config: { width: 8, height: 3 } as never, seed: 1, landscapes: [], display });
    return { host, module, sim: () => module.sims.at(-1)! };
  };
  const fp = (host: SimHost) => (send(host, { type: 'fingerprint' }) as { value: string }).value;
  const place = (x: number): Command => ({ type: 'place', x, y: 0, overrides: {} });

  /** A reference run: the same edits at the same ticks, straight through to `to`. */
  function straight(edits: [number, Command][], to: number): string {
    const { host } = setup();
    for (const [t, cmd] of edits) {
      send(host, { type: 'step', n: t - (host as unknown as { sim: FakeSim }).sim.ticks });
      send(host, cmd);
    }
    send(host, { type: 'step', n: to - (host as unknown as { sim: FakeSim }).sim.ticks });
    return fp(host);
  }

  it('seeks back through keyframes and edits (before, at and after the target) to the straight run', () => {
    const edits: [number, Command][] = [[30, place(2)], [50, place(3)], [120, place(4)]];
    const { host, sim } = setup();
    for (const [t, cmd] of edits) {
      send(host, { type: 'step', n: t - sim().ticks });
      send(host, cmd);
    }
    send(host, { type: 'step', n: 200 - sim().ticks });
    for (const target of [130, 120, 51, 50, 49, 0, 175]) {
      const r = send(host, { type: 'seek', tick: target });
      expect(r.ok).toBe(true);
      expect(sim().ticks).toBe(target);
      expect(fp(host)).toBe(straight(edits.filter(([t]) => t <= target), target));
    }
  });

  it('restores a keyframe rather than rebuilding when one is at or before the target', () => {
    const { host, module, sim } = setup();
    send(host, { type: 'step', n: 180 });
    send(host, { type: 'seek', tick: 120 });
    expect(module.sims.length).toBe(1);
    expect(sim().restores).toBe(1);
  });

  it('rebuilds from the setup when the model has no keyframes', () => {
    const { host, module } = setup({ keyframes: false });
    send(host, { type: 'place', x: 2, y: 0, overrides: {} });
    send(host, { type: 'step', n: 80 });
    const before = fp(host);
    send(host, { type: 'seek', tick: 10 });
    send(host, { type: 'seek', tick: 80 });
    expect(module.sims.length).toBe(2);
    expect(fp(host)).toBe(before);
  });

  it('replays the later edits after seeking back, and reports them pending', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 60 });
    send(host, place(5));
    send(host, { type: 'step', n: 40 });
    const end = fp(host);
    const r = send(host, { type: 'seek', tick: 20 }) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.replayLeft).toBe(1);
    expect(r.snapshot.reached).toBe(100);
    send(host, { type: 'step', n: 80 });
    expect(fp(host)).toBe(end);
  });

  it('branches on a page edit: pending entries and later keyframes go, reached drops', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 200 });
    send(host, { type: 'seek', tick: 70 });
    const r = send(host, place(6)) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.reached).toBe(70);
    expect(host.keyframeTicks().every((t) => t <= 70)).toBe(true);
    expect((send(host, { type: 'seek', tick: 71 }) as { ok: false; errors: FieldError[] }).errors[0].field).toBe('seek');
  });

  it('branches on ending a replay too: keyframes and reached past the current tick go, so a later seek cannot restore the old branch', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 60 });
    send(host, place(5));
    send(host, { type: 'step', n: 140 });
    send(host, { type: 'seek', tick: 20 });
    const ended = send(host, { type: 'endReplay' }) as { ok: true; snapshot: WorldSnapshot };
    expect(ended.snapshot.reached).toBe(20);
    expect(host.keyframeTicks().every((t) => t <= 20)).toBe(true);
    send(host, { type: 'step', n: 130 });
    send(host, { type: 'seek', tick: 120 });
    // The placed agent (at tick 60, now beyond the branch point) must not reappear: this matches a
    // fresh run to 120 with no edits at all, exactly what the kept session (an empty log) implies.
    expect(fp(host)).toBe(straight([], 120));
  });

  it('keeps a keyframe taken before a same-tick edit valid', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 50 }); // keyframe at 50 taken with no edits
    send(host, place(7)); // an edit at tick 50, after the keyframe
    send(host, { type: 'step', n: 30 });
    const end = fp(host);
    send(host, { type: 'seek', tick: 50 });
    send(host, { type: 'seek', tick: 80 });
    expect(fp(host)).toBe(end);
  });

  it('keeps the followed agent followed', () => {
    const { host, sim } = setup();
    send(host, { type: 'follow', id: 1 });
    send(host, { type: 'step', n: 120 });
    send(host, { type: 'seek', tick: 10 });
    expect(sim().followed()).toBe(1);
  });

  it('refuses a seek past reached, a negative or fractional tick, and any seek with a full log', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 10 });
    for (const tick of [11, -1, 2.5]) {
      const r = send(host, { type: 'seek', tick }) as { ok: false; errors: FieldError[] };
      expect(r.ok).toBe(false);
      expect(r.errors[0].field).toBe('seek');
    }
    (host as unknown as { logFull: boolean }).logFull = true;
    expect(send(host, { type: 'seek', tick: 5 }).ok).toBe(false);
  });

  it('sends reached and seekable', () => {
    const { host } = setup();
    const r = send(host, { type: 'step', n: 10 }) as { ok: true; snapshot: WorldSnapshot };
    expect(r.snapshot.reached).toBe(10);
    const init = send(host, { type: 'refresh' }) as { ok: true; snapshot: WorldSnapshot };
    expect(init.snapshot.reached).toBeUndefined(); // unchanged: not resent
  });
});

describe('SimHost stop rules', () => {
  const display = { colorMode: 'tribe', layer: 'resource:0', overlays: noOverlays() } as const;
  let id = 0;
  const send = (host: SimHost, cmd: Command) => host.handle({ id: ++id, cmd }).result as { ok: true; snapshot?: WorldSnapshot };
  const setup = (now?: () => number) => {
    const module = fakeModule();
    const host = new SimHost(module, now);
    send(host, { type: 'init', config: { width: 8, height: 3 } as never, seed: 1, landscapes: [], display });
    return { host, sim: () => module.sims.at(-1)! };
  };

  it('stops a step at tick N and says so', () => {
    const { host, sim } = setup();
    send(host, { type: 'setStops', stops: { tick: 37 } });
    const r = send(host, { type: 'step', n: 100 });
    expect(sim().ticks).toBe(37);
    expect(r.snapshot?.stopped).toBe('Stopped at tick 37');
    // Past N the rule is spent: the next step runs in full.
    expect(send(host, { type: 'step', n: 10 }).snapshot?.stopped).toBeUndefined();
    expect(sim().ticks).toBe(47);
  });

  it('fires a condition only when it becomes true', () => {
    const { host, sim } = setup();
    // One agent: population > 1 is false; place two more at tick 5 → true.
    send(host, { type: 'setStops', stops: { when: { series: 'population', op: '>', value: 1 } } });
    send(host, { type: 'step', n: 5 });
    send(host, { type: 'place', x: 3, y: 1, overrides: {} });
    // Already true after the edit: re-evaluated, so stepping does not fire.
    const r = send(host, { type: 'step', n: 20 });
    expect(r.snapshot?.stopped).toBeUndefined();
    expect(sim().ticks).toBe(25);
  });

  it('fires on a false → true transition at the exact tick', () => {
    const { host, sim } = setup();
    send(host, { type: 'setStops', stops: { when: { series: 'tick', op: '>', value: 41 } } });
    const r = send(host, { type: 'step', n: 100 });
    expect(sim().ticks).toBe(42);
    expect(r.snapshot?.stopped).toBe('Stopped at tick 42: tick > 41');
  });

  it('ends the Max loop on the tick a rule fires', () => {
    // Models the engine: exactly one buffer lent at `run`, and one handed back with `frame` after
    // every post (engine.ts's onPost), so the buffer is never the reason a firing rule can't post.
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 90 } });
    host.handle({ id: ++id, cmd: { type: 'run' }, frame: new ArrayBuffer(8 * 3 * 4) });
    let post: WorldSnapshot | null = null;
    for (let i = 0; i < 1000 && host.running; i++) {
      const p = host.batch();
      if (p) {
        post = p;
        host.handle({ id: ++id, cmd: { type: 'frame' }, frame: new ArrayBuffer(8 * 3 * 4) });
      }
    }
    expect(host.running).toBe(false);
    expect(sim().ticks).toBe(90);
    expect(post?.stopped).toBe('Stopped at tick 90');
  });

  it('still posts periodically at Max while a rule is pending, well before it fires', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 100_000 } });
    host.handle({ id: ++id, cmd: { type: 'run' }, frame: new ArrayBuffer(8 * 3 * 4) });
    let posts = 0;
    for (let i = 0; i < 20 && posts < 2; i++) {
      const p = host.batch();
      if (p) {
        posts++;
        host.handle({ id: ++id, cmd: { type: 'frame' }, frame: new ArrayBuffer(8 * 3 * 4) });
      }
    }
    expect(posts).toBeGreaterThanOrEqual(2);
    expect(sim().ticks).toBeLessThan(100_000);
    expect(host.running).toBe(true);
  });

  it('does not fire during a seek', () => {
    const { host } = setup();
    send(host, { type: 'step', n: 100 });
    send(host, { type: 'setStops', stops: { tick: 50 } });
    send(host, { type: 'seek', tick: 10 });
    const r = send(host, { type: 'seek', tick: 80 });
    expect(r.snapshot?.stopped).toBeUndefined();
  });

  it('ends Max on the stop tick even with no buffer free when the rule fires', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 20 } });
    host.handle({ id: ++id, cmd: { type: 'run' } }); // no buffer lent
    for (let i = 0; i < 100; i++) host.batch();
    expect(sim().ticks).toBe(20);
    host.handle({ id: ++id, cmd: { type: 'frame' }, frame: new ArrayBuffer(8 * 3 * 4) });
    const post = host.batch();
    expect(post?.stopped).toBe('Stopped at tick 20');
    expect(host.running).toBe(false);
  });

  it('clears a pending stop after Pause, so the next Play at Max steps again', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 20 } });
    host.handle({ id: ++id, cmd: { type: 'run' } }); // no buffer lent: the rule fires with none free
    for (let i = 0; i < 100; i++) host.batch();
    expect(sim().ticks).toBe(20);
    expect(host.running).toBe(true); // stopPending, waiting on a buffer
    send(host, { type: 'stop' });
    expect(host.running).toBe(false);
    host.handle({ id: ++id, cmd: { type: 'run' }, frame: new ArrayBuffer(8 * 3 * 4) });
    for (let i = 0; i < 5; i++) host.batch();
    expect(sim().ticks).toBeGreaterThan(20);
  });

  it('ends Max when setStops arrives while a fired rule is waiting on a buffer, so the world stays at the reported tick', () => {
    let t = 0;
    const { host, sim } = setup(() => (t += 1));
    send(host, { type: 'setStops', stops: { tick: 20 } });
    host.handle({ id: ++id, cmd: { type: 'run' } }); // no buffer lent: the rule fires with none free
    for (let i = 0; i < 100; i++) host.batch();
    expect(sim().ticks).toBe(20);
    expect(host.running).toBe(true); // stopPending, waiting on a buffer
    const r = host.handle({ id: ++id, cmd: { type: 'setStops', stops: {} } });
    expect(host.running).toBe(false);
    expect(sim().ticks).toBe(20); // not a tick further, even though Max was still nominally running
    expect((r.result as { ok: true; snapshot?: WorldSnapshot }).snapshot?.stopped).toBe('Stopped at tick 20');
    expect(host.batch()).toBeNull(); // Max has already ended: nothing left to step
  });

  it('re-evaluates a rule’s truth after a replayed edit, so it does not fire on the edit’s own effect', () => {
    const { host, sim } = setup();
    // One agent: population > 1 is false until the replayed place at tick 5 makes it true — the
    // live equivalent (setStops, step to 5, then place) does not fire either (see above).
    const log: LogEntry[] = [{ tick: 5, cmd: { type: 'place', x: 3, y: 1, overrides: {} } }];
    send(host, { type: 'reset', config: { width: 8, height: 3 } as never, seed: 1, landscapes: [], log });
    send(host, { type: 'setStops', stops: { when: { series: 'population', op: '>', value: 1 } } });
    const r = send(host, { type: 'step', n: 20 });
    expect(r.snapshot?.stopped).toBeUndefined();
    expect(sim().ticks).toBe(20);
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
