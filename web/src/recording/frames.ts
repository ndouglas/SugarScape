// The pure parts of recording (Decisions 13 and 14): frame layout, codec choice, names and timing.

/** A recorded cell is this many pixels… */
export const MIN_CELL_PX = 8;
/** …unless that would make the frame's long side longer than this. */
export const MAX_SIDE_PX = 1080;
/** Pixels between the two grids in Compare. */
export const GAP_PX = 4;
/** GIF frames per second of recorded time (about). */
export const GIF_FPS = 15;
/** A GIF stops recording at this many frames. */
export const GIF_MAX_FRAMES = 900;
/** Frames the GIF worker may hold unacknowledged before capture skips frames. */
export const GIF_BACKLOG = 3;
/** Video formats in order of preference (the spec's). */
export const MIMES = ['video/webm;codecs=vp9', 'video/webm;codecs=vp8', 'video/webm', 'video/mp4'] as const;

export interface Rect { x: number; y: number; width: number; height: number }
export interface FrameLayout { width: number; height: number; scale: number; rects: Rect[] }

/**
 * Where each grid goes in a recorded frame: side by side, top-aligned, `scale` pixels per cell —
 * 8, or less so the long side stays within 1080 px, but at least 1.
 */
export function frameLayout(grids: { width: number; height: number }[]): FrameLayout {
  const gaps = GAP_PX * Math.max(0, grids.length - 1);
  const wide = Math.max(1, grids.reduce((sum, g) => sum + g.width, 0));
  const high = Math.max(1, ...grids.map((g) => g.height));
  const fit = Math.min(Math.floor((MAX_SIDE_PX - gaps) / wide), Math.floor(MAX_SIDE_PX / high));
  const scale = Math.max(1, Math.min(MIN_CELL_PX, fit));
  const rects: Rect[] = [];
  let x = 0;
  for (const g of grids) {
    rects.push({ x, y: 0, width: g.width * scale, height: g.height * scale });
    x += g.width * scale + GAP_PX;
  }
  return { width: Math.max(1, x - GAP_PX), height: high * scale, scale, rects };
}

/** The first video type the browser can record, and its file extension. */
export function pickMime(supported: (type: string) => boolean): { mime: string; ext: 'webm' | 'mp4' } | null {
  const mime = MIMES.find((t) => supported(t));
  if (!mime) return null;
  return { mime, ext: mime.startsWith('video/mp4') ? 'mp4' : 'webm' };
}

/** `<base>-t<from>-t<to>.<ext>`. */
export function recordingName(base: string, from: number, to: number, ext: string): string {
  return `${base}-t${from}-t${to}.${ext}`;
}

/** `m:ss`. */
export function formatClock(ms: number): string {
  const s = Math.floor(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

/** Recorded time: wall-clock time since the start, not counting pauses. */
export class Stopwatch {
  private pausedAt: number | null = null;
  private pausedFor = 0;

  constructor(private readonly started: number) {}

  pause(now: number): void {
    if (this.pausedAt === null) this.pausedAt = now;
  }

  resume(now: number): void {
    if (this.pausedAt === null) return;
    this.pausedFor += now - this.pausedAt;
    this.pausedAt = null;
  }

  elapsed(now: number): number {
    return (this.pausedAt ?? now) - this.started - this.pausedFor;
  }
}

/**
 * Picks GIF frames at about GIF_FPS on recorded time (the 2 ms slack absorbs 60 Hz jitter), holds
 * back while the encoder is GIF_BACKLOG frames behind, and stops at GIF_MAX_FRAMES (Decision 14).
 */
export class GifSampler {
  frames = 0;
  private last: number | null = null;

  get full(): boolean {
    return this.frames >= GIF_MAX_FRAMES;
  }

  /** Whether to take a frame at recorded time `t` with `backlog` frames still being encoded. */
  due(t: number, backlog: number): boolean {
    if (this.full || backlog >= GIF_BACKLOG) return false;
    return this.last === null || t - this.last >= 1000 / GIF_FPS - 2;
  }

  /** Takes a frame at `t`; returns the frame before it's delay (the time between them), or null for the first. */
  take(t: number): number | null {
    this.frames++;
    const prev = this.last;
    this.last = t;
    return prev === null ? null : t - prev;
  }
}

/** A GIF stores delays in hundredths of a second, and browsers slow anything under 20 ms. */
export function gifDelay(ms: number): number {
  return Math.max(20, Math.round(ms / 10) * 10);
}
