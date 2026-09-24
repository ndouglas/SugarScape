import { downloadBlob } from '../downloads';
import type { GifReply, GifRequest } from './gif-encode';
import { frameLayout, GIF_FPS, GIF_MAX_FRAMES, gifDelay, GifSampler, pickMime, recordingName, Stopwatch } from './frames';

export type RecordFormat = 'webm' | 'gif';

/** Shown when a GIF stops itself at its frame cap. */
export const GIF_LIMIT_NOTICE = `The GIF reached ${GIF_MAX_FRAMES} frames, so recording stopped.`;

/** A grid to record: its canvas as drawn (overlays, trail, selection) and its size in cells. */
export interface RecordedGrid { canvas: HTMLCanvasElement; cells: () => { width: number; height: number }; label?: string }

/** What a recording shows and follows: the grids, the tick for the stamp, the run state, the file-name base. */
export interface RecordSource {
  grids: () => RecordedGrid[];
  tick: () => number;
  running: () => boolean;
  /** The file name before `-t<from>-t<to>.<ext>`. */
  base: () => string;
}

export interface Recording {
  /** A displayed snapshot has been drawn: record a frame (if running, and for a GIF if one is due). */
  capture(): void;
  /** Pauses or resumes with the world. */
  sync(): void;
  /** Recorded time so far, in ms (pauses excluded). */
  elapsed(): number;
  /** Stops and downloads the file; `progress` reports the GIF worker finishing its queue. */
  stop(progress?: (done: number, total: number) => void): Promise<void>;
}

/** The first video type this browser records, or null. */
export function webmSupport(): { mime: string; ext: 'webm' | 'mp4' } | null {
  if (typeof MediaRecorder === 'undefined') return null;
  return pickMime((type) => MediaRecorder.isTypeSupported(type));
}

/**
 * Starts recording; throws if this browser cannot. `onLimit` is called when a GIF reaches its frame
 * cap; `onFail` when the GIF encoder fails mid-recording (stopping then rejects with its message).
 */
export function startRecording(
  format: RecordFormat,
  source: RecordSource,
  stamp: boolean,
  onLimit: () => void,
  onFail?: (message: string) => void,
): Recording {
  if (format === 'gif') return new GifRecording(source, stamp, onLimit, onFail);
  const mime = webmSupport();
  if (!mime) throw new Error('this browser cannot record video');
  return new WebmRecording(source, stamp, mime);
}

/** Rounds up to an even number: H.264/MP4 and some encoders reject odd frame sizes. */
const even = (n: number): number => n + (n % 2);

/**
 * Draws the grids, as shown on screen, into a recording frame whose size is fixed when recording
 * starts; a later layout of another size (a grid resized by a reset) is scaled to fit (Decision 13).
 * The canvas is the layout rounded up to even sizes; the content sits at 0,0 and any extra pixel row
 * or column stays background.
 */
class Composer {
  readonly canvas = document.createElement('canvas');
  readonly ctx: CanvasRenderingContext2D;
  /** The layout's size when recording started: the area the content is fitted into. */
  private readonly width: number;
  private readonly height: number;

  constructor(private readonly source: RecordSource, private readonly stamp: boolean, willReadFrequently: boolean) {
    const layout = frameLayout(source.grids().map((g) => g.cells()));
    this.width = layout.width;
    this.height = layout.height;
    this.canvas.width = even(layout.width);
    this.canvas.height = even(layout.height);
    this.ctx = this.canvas.getContext('2d', { willReadFrequently })!;
  }

  draw(): void {
    const { canvas, ctx } = this;
    const grids = this.source.grids();
    const layout = frameLayout(grids.map((g) => g.cells()));
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--bg').trim() || '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const f = Math.min(this.width / layout.width, this.height / layout.height);
    ctx.setTransform(f, 0, 0, f, (this.width - layout.width * f) / 2, (this.height - layout.height * f) / 2);
    ctx.imageSmoothingEnabled = false;
    grids.forEach((g, i) => {
      const r = layout.rects[i];
      ctx.drawImage(g.canvas, r.x, r.y, r.width, r.height);
      if (g.label) this.tag(g.label, r.x + 4, r.y + 4);
    });
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    if (this.stamp) this.tag(`t = ${this.source.tick()}`, 4, this.height - this.fontSize() - 10);
  }

  private fontSize(): number {
    return Math.max(12, Math.round(this.height / 40));
  }

  /** White text on a dark box, readable on any landscape. */
  private tag(text: string, x: number, y: number): void {
    const { ctx } = this;
    const size = this.fontSize();
    ctx.font = `600 ${size}px ui-sans-serif, system-ui, sans-serif`;
    ctx.textBaseline = 'top';
    const width = ctx.measureText(text).width;
    ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
    ctx.fillRect(x, y, width + 8, size + 6);
    ctx.fillStyle = '#fff';
    ctx.fillText(text, x + 4, y + 3);
  }
}

/**
 * What both formats share: recorded time that pauses with the world, the starting tick, capture
 * only while running, and stop-once-then-download. Subclasses draw frames and build the file.
 */
abstract class BaseRecording implements Recording {
  private readonly watch = new Stopwatch(performance.now());
  private readonly from: number;
  private stopped = false;

  constructor(protected readonly source: RecordSource) {
    this.from = source.tick();
  }

  capture(): void {
    if (this.stopped) return;
    this.sync();
    if (this.source.running()) this.frame();
  }

  sync(): void {
    if (this.stopped) return;
    const now = performance.now();
    const running = this.source.running();
    if (running) this.watch.resume(now);
    else this.watch.pause(now);
    this.onRun(running);
  }

  elapsed(): number {
    return this.watch.elapsed(performance.now());
  }

  async stop(progress?: (done: number, total: number) => void): Promise<void> {
    if (this.stopped) return;
    this.stopped = true;
    const to = this.source.tick();
    const file = await this.finish(progress);
    downloadBlob(recordingName(this.source.base(), this.from, to, file.ext), file.blob);
  }

  /** The world is running and a snapshot was drawn: record it (a GIF only when a frame is due). */
  protected abstract frame(): void;

  /** The world is running or paused (called on every sync). */
  protected onRun(_running: boolean): void {}

  /** Ends the recording and returns the file. */
  protected abstract finish(progress?: (done: number, total: number) => void): Promise<{ blob: Blob; ext: string }>;
}

/** WebM (or MP4): one video frame per captured snapshot, paused with the world (Decision 14). */
class WebmRecording extends BaseRecording {
  private readonly composer: Composer;
  private readonly recorder: MediaRecorder;
  private readonly track: CanvasCaptureMediaStreamTrack;
  private readonly chunks: Blob[] = [];

  constructor(source: RecordSource, stamp: boolean, private readonly type: { mime: string; ext: string }) {
    super(source);
    this.composer = new Composer(source, stamp, false);
    const stream = this.composer.canvas.captureStream(0);
    this.track = stream.getVideoTracks()[0] as CanvasCaptureMediaStreamTrack;
    this.recorder = new MediaRecorder(stream, { mimeType: type.mime });
    this.recorder.ondataavailable = (e) => {
      if (e.data.size > 0) this.chunks.push(e.data);
    };
    this.recorder.start(1000);
    // The first frame, even while paused, so the file is never empty; the next sync pauses it.
    this.frame();
  }

  protected frame(): void {
    this.composer.draw();
    this.track.requestFrame();
  }

  protected onRun(running: boolean): void {
    if (running && this.recorder.state === 'paused') this.recorder.resume();
    else if (!running && this.recorder.state === 'recording') this.recorder.pause();
  }

  protected async finish(): Promise<{ blob: Blob; ext: string }> {
    await new Promise<void>((resolve) => {
      this.recorder.onstop = () => resolve();
      this.recorder.stop();
    });
    this.track.stop();
    return { blob: new Blob(this.chunks, { type: this.type.mime }), ext: this.type.ext };
  }
}

interface GifFrame { rgba: ArrayBuffer; width: number; height: number }

/** A GIF: frames sampled at about 15 fps of recorded time, encoded in a worker (Decision 14). */
class GifRecording extends BaseRecording {
  private readonly composer: Composer;
  private readonly worker = new Worker(new URL('./gif-worker.ts', import.meta.url), { type: 'module' });
  private readonly sampler = new GifSampler();
  /** The latest frame, sent once the next one is taken (its delay is the time until then). */
  private held: GifFrame | null = null;
  private sent = 0;
  private acked = 0;
  /** The encoder's failure, once it has failed: nothing more is sent. */
  private failed: string | null = null;
  /** Set while stopping: settles when the worker answers `finish` (or fails). */
  private finishing: { resolve: (bytes: ArrayBuffer) => void; reject: (e: Error) => void; progress?: (done: number) => void } | null = null;

  constructor(source: RecordSource, stamp: boolean, private readonly onLimit: () => void, private readonly onFail?: (message: string) => void) {
    super(source);
    this.composer = new Composer(source, stamp, true);
    this.worker.onmessage = (e: MessageEvent<GifReply>) => this.reply(e.data);
    this.worker.onerror = (e) => this.fail(e.message || 'the GIF encoder failed');
    this.grab(0);
  }

  protected frame(): void {
    if (this.failed !== null) return;
    const t = this.elapsed();
    if (this.sampler.due(t, this.sent - this.acked)) this.grab(t);
  }

  protected async finish(progress?: (done: number, total: number) => void): Promise<{ blob: Blob; ext: string }> {
    if (this.held && this.failed === null) this.send(this.held, 1000 / GIF_FPS);
    this.held = null;
    const total = this.sent;
    try {
      const bytes = await new Promise<ArrayBuffer>((resolve, reject) => {
        if (this.failed !== null) {
          reject(new Error(this.failed));
          return;
        }
        this.finishing = { resolve, reject, progress: progress && ((done) => progress(done, total)) };
        const finish: GifRequest = { type: 'finish' };
        this.worker.postMessage(finish);
      });
      return { blob: new Blob([bytes], { type: 'image/gif' }), ext: 'gif' };
    } finally {
      this.finishing = null;
      this.worker.terminate();
    }
  }

  private reply(r: GifReply): void {
    switch (r.type) {
      case 'ack':
        this.acked = r.frames;
        this.finishing?.progress?.(r.frames);
        break;
      case 'done':
        this.finishing?.resolve(r.bytes);
        break;
      case 'error':
        this.fail(r.message);
        break;
    }
  }

  /** The encoder failed: stop sending, and fail the stop in progress or ask to be stopped. */
  private fail(message: string): void {
    if (this.failed !== null) return;
    this.failed = message;
    if (this.finishing) this.finishing.reject(new Error(message));
    else this.onFail?.(message);
  }

  private grab(t: number): void {
    this.composer.draw();
    const { width, height } = this.composer.canvas;
    const rgba = this.composer.ctx.getImageData(0, 0, width, height).data.buffer as ArrayBuffer;
    const delay = this.sampler.take(t);
    if (this.held && delay !== null) this.send(this.held, delay);
    this.held = { rgba, width, height };
    if (this.sampler.full) this.onLimit();
  }

  private send(frame: GifFrame, delay: number): void {
    const message: GifRequest = { type: 'frame', rgba: frame.rgba, width: frame.width, height: frame.height, delay: gifDelay(delay) };
    this.worker.postMessage(message, [frame.rgba]);
    this.sent++;
  }
}
