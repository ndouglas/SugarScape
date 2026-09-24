import { describe, expect, it } from 'vitest';
import { fakeModule } from './fake-sim.fixture';
import type { Command, DisplayState, HostMessage, HostReply, Wants, WorldSnapshot } from './protocol';
import { serve, SimHost } from './sim-host';
import type { Config } from './types';

const config = { width: 4, height: 3 } as unknown as Config;
const display: DisplayState = { colorMode: 'tribe', layer: 'resource:0', overlays: { trade: false, credit: false, disease: false } };

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
      'inspection', 'trail', 'networks', 'charts', 'lorenz', 'wealthHist', 'supplyDemand',
      'creditGraph', 'diseaseList', 'config', 'editedLandscapes', 'display', 'frame',
    ] as const;
    for (const key of keys) expect(bare[key], key).toBeUndefined();
    const all: Wants = {
      select: { x: 0, y: 0, agentId: null },
      trail: true,
      networks: ['trade'],
      charts: { groups: [['population']], max: 10 },
      lorenz: true,
      wealthHist: true,
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

  it('only throttles the running loop\'s own steps: any other command sends a group as soon as it has grown at all (PF6)', () => {
    let clock = 0;
    const t = start(() => clock);
    const charts = { groups: [['population']], max: 50 };
    const key = 'population';
    t.send({ type: 'step', n: 999 }, { wants: { charts } }); // history: 1000 ticks, sent now
    clock = 10;
    // A further step this soon, with this little growth, is throttled (same numbers as above).
    expect(t.snap(t.send({ type: 'step', n: 1 }, { wants: { charts } })).charts?.[key]).toBeUndefined();
    // But a refresh — not part of the step loop — is not: it sees the one extra tick and sends it,
    // even though it is just as soon and just as small a change (the paused panel must not stall).
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

describe('serve', () => {
  it('answers each request in order and lists its buffers for transfer', () => {
    const sent: [HostMessage, Transferable[]][] = [];
    const handle = serve(new SimHost(fakeModule()), (m, t) => sent.push([m, t]));
    const frame = new ArrayBuffer(48);
    handle({ id: 1, cmd: { type: 'init', config, seed: 1, landscapes: [], display }, frame });
    handle({ id: 2, cmd: { type: 'fingerprint' } });
    expect(sent.map(([m]) => m.id)).toEqual([1, 2]);
    expect(sent[0][1]).toHaveLength(1);
    expect(sent[0][1][0]).toBe(frame);
    expect((sent[1][0] as HostReply).result).toEqual({ ok: true, value: '0x0' });
  });
});
