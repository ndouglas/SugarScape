import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { downloadBlob } from '../downloads';
import type { GifReply, GifRequest } from './gif-encode';
import { GIF_LIMIT_NOTICE, startRecording, type RecordSource } from './recorder';

vi.mock('../downloads', () => ({ downloadBlob: vi.fn() }));

/** Stands in for the GIF worker: records what the page posts; the test answers. */
class StubWorker {
  static last: StubWorker;
  onmessage: ((e: { data: GifReply }) => void) | null = null;
  onerror: ((e: { message: string }) => void) | null = null;
  posted: GifRequest[] = [];
  terminated = false;

  constructor() {
    StubWorker.last = this;
  }

  postMessage(m: GifRequest): void {
    this.posted.push(m);
  }

  terminate(): void {
    this.terminated = true;
  }

  frames(): Extract<GifRequest, { type: 'frame' }>[] {
    return this.posted.filter((m) => m.type === 'frame');
  }

  reply(data: GifReply): void {
    this.onmessage!({ data });
  }
}

/** Stands in for the capture track a recorded canvas's `captureStream()` hands out. */
class StubTrack {
  stopped = false;
  requestFrame = vi.fn();
  stop(): void {
    this.stopped = true;
  }
}

/** Stands in for `MediaRecorder`: the test drives `state`, `onstop` and `onerror` by hand. */
class StubMediaRecorder {
  static isTypeSupported = (): boolean => true;
  static last: StubMediaRecorder;
  state: 'inactive' | 'recording' | 'paused' = 'inactive';
  ondataavailable: ((e: { data: Blob }) => void) | null = null;
  onstop: (() => void) | null = null;
  onerror: ((e: { error?: { message: string } }) => void) | null = null;
  stopCalls = 0;
  pauseCalls = 0;
  resumeCalls = 0;

  constructor(public stream: unknown, public opts: { mimeType: string }) {
    StubMediaRecorder.last = this;
  }

  start(): void {
    this.state = 'recording';
  }

  stop(): void {
    this.stopCalls++;
  }

  pause(): void {
    this.pauseCalls++;
    this.state = 'paused';
  }

  resume(): void {
    this.resumeCalls++;
    this.state = 'recording';
  }
}

/** Only what the recording's canvas and 2D context are asked for. */
function stubCanvas() {
  const track = new StubTrack();
  const canvas = {
    width: 0,
    height: 0,
    getContext: () => ({
      setTransform() {},
      fillRect() {},
      drawImage() {},
      fillText() {},
      measureText: () => ({ width: 10 }),
      getImageData: (_x: number, _y: number, w: number, h: number) => ({ data: new Uint8ClampedArray(w * h * 4) }),
    }),
    captureStream: () => ({ getVideoTracks: () => [track] }),
    track,
  };
  return canvas;
}

let now = 0;
let tick = 0;
let canvases: ReturnType<typeof stubCanvas>[] = [];

const source: RecordSource = {
  // 1001 × 3 cells fit the 1080 px long side at 1 px each: an odd 1001 × 3 layout.
  grids: () => [{ canvas: {} as HTMLCanvasElement, cells: () => ({ width: 1001, height: 3 }) }],
  tick: () => tick,
  running: () => true,
  base: () => 'sugarscape-test-seed1',
};

beforeEach(() => {
  now = 0;
  tick = 10;
  canvases = [];
  vi.spyOn(performance, 'now').mockImplementation(() => now);
  vi.stubGlobal('Worker', StubWorker);
  vi.stubGlobal('MediaRecorder', StubMediaRecorder);
  vi.stubGlobal('document', {
    documentElement: {},
    createElement: () => {
      const c = stubCanvas();
      canvases.push(c);
      return c;
    },
  });
  vi.stubGlobal('getComputedStyle', () => ({ getPropertyValue: () => '#123' }));
  vi.mocked(downloadBlob).mockClear();
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

/** Advances recorded time to `t` and captures. */
const captureAt = (r: { capture(): void }, t: number) => {
  now = t;
  r.capture();
};

describe('GIF recording', () => {
  it('uses even frame sizes, sends each frame with the time until the next, and holds back 3 frames behind', async () => {
    const r = startRecording('gif', source, true, () => {});
    const worker = StubWorker.last;
    expect([canvases[0].width, canvases[0].height]).toEqual([1002, 4]);

    captureAt(r, 30); // not due yet
    expect(worker.frames()).toHaveLength(0);
    captureAt(r, 70); // frame 0 goes, with the 70 ms until frame 1
    captureAt(r, 140);
    captureAt(r, 230);
    expect(worker.frames().map((f) => f.delay)).toEqual([70, 70, 90]);
    expect(worker.frames()[0]).toMatchObject({ width: 1002, height: 4 });

    captureAt(r, 400); // 3 unacknowledged: skipped
    expect(worker.frames()).toHaveLength(3);
    worker.reply({ type: 'ack', frames: 3 });
    captureAt(r, 410); // frame 3 was held from 230 until now
    expect(worker.frames().map((f) => f.delay)).toEqual([70, 70, 90, 180]);

    tick = 42;
    const progress: [number, number][] = [];
    const stopped = r.stop((done, total) => progress.push([done, total]));
    // The held last frame goes with a 1/15 s delay, then "finish".
    expect(worker.frames().map((f) => f.delay)).toEqual([70, 70, 90, 180, 70]);
    expect(worker.posted.at(-1)).toEqual({ type: 'finish' });
    worker.reply({ type: 'ack', frames: 5 });
    worker.reply({ type: 'done', bytes: new ArrayBuffer(6) });
    await stopped;
    expect(progress).toEqual([[5, 5]]);
    expect(worker.terminated).toBe(true);
    expect(vi.mocked(downloadBlob)).toHaveBeenCalledWith('sugarscape-test-seed1-t10-t42.gif', expect.any(Blob));
  });

  it('stops at 900 frames', async () => {
    const onLimit = vi.fn();
    const r = startRecording('gif', source, false, onLimit);
    const worker = StubWorker.last;
    for (let i = 1; onLimit.mock.calls.length === 0 && i < 2000; i++) {
      captureAt(r, i * 70);
      worker.reply({ type: 'ack', frames: worker.frames().length });
    }
    expect(onLimit).toHaveBeenCalledTimes(1);
    expect(worker.frames()).toHaveLength(899); // the 900th is held
    captureAt(r, 1_000_000);
    expect(worker.frames()).toHaveLength(899);
    const stopped = r.stop();
    worker.reply({ type: 'done', bytes: new ArrayBuffer(6) });
    await stopped;
    expect(worker.frames()).toHaveLength(900);
    expect(worker.frames().at(-1)!.delay).toBe(70);
    expect(GIF_LIMIT_NOTICE).toBe('The GIF reached 900 frames, so recording stopped.');
  });

  it('stops on an encoder error reply and never waits for it', async () => {
    const onFail = vi.fn();
    const r = startRecording('gif', source, false, () => {}, onFail);
    const worker = StubWorker.last;
    captureAt(r, 70);
    worker.reply({ type: 'error', message: 'out of memory' });
    expect(onFail).toHaveBeenCalledWith('out of memory');
    captureAt(r, 140);
    captureAt(r, 210);
    expect(worker.frames()).toHaveLength(1);
    await expect(r.stop()).rejects.toThrow('out of memory');
    expect(worker.posted.at(-1)).toMatchObject({ type: 'frame' }); // no "finish" sent
    expect(worker.terminated).toBe(true);
    expect(vi.mocked(downloadBlob)).not.toHaveBeenCalled();
  });

  it('fails the stop in progress when the worker crashes', async () => {
    const onFail = vi.fn();
    const r = startRecording('gif', source, false, () => {}, onFail);
    const worker = StubWorker.last;
    const stopped = r.stop();
    worker.onerror!({ message: 'worker crashed' });
    await expect(stopped).rejects.toThrow('worker crashed');
    expect(onFail).not.toHaveBeenCalled();
    expect(worker.terminated).toBe(true);
    expect(vi.mocked(downloadBlob)).not.toHaveBeenCalled();
  });
});

describe('WebM recording', () => {
  it('downloads the file once the recorder stops, and always stops the capture track', async () => {
    const r = startRecording('webm', source, true, () => {});
    const rec = StubMediaRecorder.last;
    const track = canvases[0].track;
    rec.ondataavailable!({ data: new Blob(['abc']) });
    tick = 42;
    const stopped = r.stop();
    expect(rec.stopCalls).toBe(1);
    expect(track.stopped).toBe(false); // still waiting on the recorder's own onstop
    rec.onstop!();
    await stopped;
    expect(track.stopped).toBe(true);
    expect(vi.mocked(downloadBlob)).toHaveBeenCalledWith('sugarscape-test-seed1-t10-t42.webm', expect.any(Blob));
  });

  it('resolves the stop at once, and still stops the track, when the recorder already went inactive', async () => {
    const onFail = vi.fn();
    const r = startRecording('webm', source, false, () => {}, onFail);
    const rec = StubMediaRecorder.last;
    const track = canvases[0].track;
    // An encoder error takes the browser's recorder inactive on its own: 'error' then 'stop', with
    // nothing left for a later recorder.stop() to do — finish() must not wait for an onstop that
    // will never come.
    rec.onerror!({ error: { message: 'encoder crashed' } });
    rec.state = 'inactive';
    expect(onFail).toHaveBeenCalledWith('encoder crashed');
    await expect(r.stop()).rejects.toThrow('encoder crashed');
    expect(rec.stopCalls).toBe(0);
    expect(track.stopped).toBe(true);
    expect(vi.mocked(downloadBlob)).not.toHaveBeenCalled();
  });

  it('reports one onerror (with a default message if none is given) and never again', () => {
    const onFail = vi.fn();
    startRecording('webm', source, false, () => {}, onFail);
    const rec: StubMediaRecorder = StubMediaRecorder.last;
    rec.onerror!({});
    expect(onFail).toHaveBeenCalledWith('the recorder failed');
    rec.onerror!({ error: { message: 'a second failure' } });
    expect(onFail).toHaveBeenCalledTimes(1);
  });
});
