import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createEpisodeClient } from './client';
import type { EpisodeRecord, StudyDescriptor, WorkerLike, WorkerReply, WorkerRequest } from './types';
class FakeWorker implements WorkerLike {
  onmessage: WorkerLike['onmessage'] = null;
  onerror: WorkerLike['onerror'] = null;
  onmessageerror: WorkerLike['onmessageerror'] = null;
  sent: WorkerRequest[] = []; terminated = false;
  postMessage(message: WorkerRequest) { this.sent.push(message); }
  terminate() { this.terminated = true; }
  deliver(reply: WorkerReply) { this.onmessage?.({ data: reply } as MessageEvent<WorkerReply>); }
}
function clientFixture() {
  const workers: FakeWorker[] = [];
  const client = createEpisodeClient(() => { const worker = new FakeWorker(); workers.push(worker); return worker; });
  return { client, workers };
}
const input = '{"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}';
afterEach(() => vi.useRealTimers());
describe('episode worker ownership', () => {
  it('cancel rejects the pending request and terminates its engine', async () => {
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    client.cancel();
    await expect(pending).rejects.toThrow(/canceled/);
    expect(workers[0].terminated).toBe(true);
  });
  it('times out an incomplete request after sixty seconds and releases its worker', async () => {
    vi.useFakeTimers();
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    const rejected = expect(pending).rejects.toThrow(/timed out/);
    await vi.advanceTimersByTimeAsync(60_000);
    await rejected;
    expect(workers[0].terminated).toBe(true);
    expect(vi.getTimerCount()).toBe(0);
  });
  it('initialization and runtime failures reject with context and terminate', async () => {
    const { client, workers } = clientFixture();
    const pending = client.catalog();
    workers[0].onerror?.({ message: 'WASM load failed' } as ErrorEvent);
    await expect(pending).rejects.toThrow(/WASM load failed/);
    expect(workers[0].terminated).toBe(true);
  });
  it('worker message decoding failures reject and release all state', async () => {
    const { client, workers } = clientFixture();
    const pending = client.recorded();
    workers[0].onmessageerror?.({} as MessageEvent);
    await expect(pending).rejects.toThrow(/decode/);
    expect(workers[0].terminated).toBe(true);
  });
  it('concurrent requests fail without replacing the live engine', async () => {
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    await expect(client.recorded()).rejects.toThrow(/already running/);
    expect(workers).toHaveLength(1);
    expect(workers[0].terminated).toBe(false);
    client.cancel();
    await expect(pending).rejects.toThrow(/canceled/);
  });
  it('dispose cancels and prevents subsequent requests from owning an engine', async () => {
    const { client, workers } = clientFixture();
    const pending = client.request('validate', '{}');
    client.dispose();
    await expect(pending).rejects.toThrow(/disposed/);
    await expect(client.catalog()).rejects.toThrow(/disposed/);
    expect(workers).toHaveLength(1);
    expect(workers[0].terminated).toBe(true);
  });
  it('raw UTF-8 limits reject before posting oversized input or imported records', async () => {
    const { client, workers } = clientFixture();
    await expect(client.request('run', 'é'.repeat(32_769))).rejects.toThrow(/64 KiB/);
    await expect(client.request('validate', ' '.repeat(16 * 1024 * 1024 + 1))).rejects.toThrow(/16 MiB/);
    expect(workers).toHaveLength(0);
  });
  it('creation failures settle without stranding the client', async () => {
    let attempts = 0;
    const worker = new FakeWorker();
    const client = createEpisodeClient(() => { if (++attempts === 1) throw new Error('creation failed'); return worker; });
    await expect(client.catalog()).rejects.toThrow(/creation failed/);
    const next = client.catalog();
    worker.deliver({ id: 2, kind: 'catalog', catalog: [] });
    await expect(next).resolves.toEqual([]);
    expect(worker.terminated).toBe(true);
  });
});

// Complete real records and defaults from the native checked boundary, never a simulator double.
function nativeFixture(studyId: StudyDescriptor['id'] = 'wink') {
  const cwd = fileURLToPath(new URL('../../../', import.meta.url));
  const binary = `${cwd}/target/release/sugarscape`;
  const catalog = JSON.parse(execFileSync(binary, ['experiment-view', 'catalog'], { cwd, encoding: 'utf8' })) as StudyDescriptor[];
  const scratch = mkdtempSync(`${tmpdir()}/episode-client-`);
  try {
    const inputPath = `${scratch}/input.json`;
    writeFileSync(inputPath, JSON.stringify(catalog.find(study => study.id === studyId)!.default_input));
    const options = { cwd, encoding: 'utf8' as const, maxBuffer: 16 * 1024 * 1024 + 1 };
    return { catalog, record: JSON.parse(execFileSync(binary, ['experiment-view', 'run', '--input', inputPath], options)) as EpisodeRecord };
  } finally { rmSync(scratch, { recursive: true, force: true }); }
}
describe('episode reply isolation and termination', () => {
  it('passes the complete episode and cancels its timer on success', async () => {
    const { record } = nativeFixture();
    vi.useFakeTimers();
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    expect(workers[0].sent).toEqual([{ id: 1, op: 'run', text: input }]);
    workers[0].deliver({ id: 1, kind: 'episode', record });
    await expect(pending).resolves.toEqual(record);
    expect(workers[0].terminated).toBe(true);
    expect(vi.getTimerCount()).toBe(0);
  });
  it('late replies cannot replace canceled requests or affect the next engine', async () => {
    const { record, catalog } = nativeFixture();
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    const lateCallback = workers[0].onmessage!;
    client.cancel();
    await expect(pending).rejects.toThrow(/canceled/);
    const next = client.catalog();
    lateCallback({ data: { id: 1, kind: 'episode', record } } as MessageEvent<WorkerReply>);
    workers[1].deliver({ id: 1, kind: 'episode', record });
    expect(workers[1].terminated).toBe(false);
    expect(workers[1].sent).toEqual([{ id: 2, op: 'catalog', text: '' }]);
    workers[1].deliver({ id: 2, kind: 'catalog', catalog });
    await expect(next).resolves.toEqual(catalog);
    expect(workers.every(worker => worker.terminated)).toBe(true);
  });
  it('validation and recorded exports use their own discriminated reply types', async () => {
    const { record } = nativeFixture();
    const { client, workers } = clientFixture();
    const imported = client.request('validate', JSON.stringify(record));
    workers[0].deliver({ id: 1, kind: 'episode', record });
    await expect(imported).resolves.toEqual(record);
    const recorded = client.recorded();
    const value = { kind: 'recorded_results', version: 1, studies: {} };
    expect(workers[1].sent).toEqual([{ id: 2, op: 'recorded', text: '' }]);
    workers[1].deliver({ id: 2, kind: 'recorded', recorded: value });
    await expect(recorded).resolves.toEqual(value);
    expect(workers.every(worker => worker.terminated)).toBe(true);
  });
  it('contextual engine and init errors reject without successful partial records', async () => {
    const { client, workers } = clientFixture();
    const pending = client.request('run', '{}');
    workers[0].deliver({ id: 1, errors: [{ field: 'input', message: 'unknown study' }] });
    await expect(pending).rejects.toMatchObject({ errors: [{ field: 'input', message: 'unknown study' }] });
    expect(workers[0].terminated).toBe(true);
    const next = client.catalog();
    workers[1].deliver({ id: 2, errors: [{ field: 'worker', message: 'WASM initialization failed' }] });
    await expect(next).rejects.toThrow(/initialization failed/);
    expect(workers[1].terminated).toBe(true);
  });
  it('a reply of the wrong kind rejects and terminates', async () => {
    const { client, workers } = clientFixture();
    const pending = client.request('run', input);
    workers[0].deliver({ id: 1, kind: 'catalog', catalog: [] });
    await expect(pending).rejects.toThrow(/unexpected/);
    expect(workers[0].terminated).toBe(true);
  });
  it('post failures terminate the created worker and permit a new request', async () => {
    const worker = new FakeWorker();
    worker.postMessage = () => { throw new Error('post failed'); };
    const client = createEpisodeClient(() => worker);
    await expect(client.catalog()).rejects.toThrow(/post failed/);
    expect(worker.terminated).toBe(true);
  });
  it('a timeout cannot terminate a later successful request', async () => {
    vi.useFakeTimers();
    const { client, workers } = clientFixture();
    const first = client.catalog();
    workers[0].deliver({ id: 1, kind: 'catalog', catalog: [] });
    await first;
    await vi.advanceTimersByTimeAsync(59_000);
    const next = client.catalog();
    await vi.advanceTimersByTimeAsync(1_000);
    expect(workers[1].terminated).toBe(false);
    workers[1].deliver({ id: 2, kind: 'catalog', catalog: [] });
    await expect(next).resolves.toEqual([]);
    expect(vi.getTimerCount()).toBe(0);
  });
});


describe('spatial records keep the shared single-worker client contract', () => {
  for (const studyId of ['burrow_excavation', 'burrow_access', 'foraging_fixed', 'foraging_passage', 'foraging_construction'] as const) {
    it(`${studyId} complete checked reply passes unchanged and releases its only worker`, async () => {
      const { record } = nativeFixture(studyId);
      const { client, workers } = clientFixture();
      const text = JSON.stringify(record.input);
      const pending = client.request('run', text);
      workers[0].deliver({ id: 1, kind: 'episode', record });
      expect(await pending).toEqual(record);
      expect(workers).toHaveLength(1);
      expect(workers[0].terminated).toBe(true);
      const imported = client.request('validate', JSON.stringify(record));
      workers[1].deliver({ id: 2, kind: 'episode', record });
      expect(await imported).toEqual(record);
      expect(workers.every(worker => worker.terminated)).toBe(true);
    });
  }
});
