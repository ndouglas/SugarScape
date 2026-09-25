import { describe, expect, it } from 'vitest';
import {
  formatClock,
  frameLayout,
  GIF_BACKLOG,
  GIF_MAX_FRAMES,
  gifDelay,
  GifSampler,
  pickMime,
  recordingName,
  Stopwatch,
  worldViews,
} from './frames';

describe('frameLayout', () => {
  it('makes a cell 8 px when the frame fits in 1080 px', () => {
    expect(frameLayout([{ width: 50, height: 50 }])).toEqual({
      width: 400,
      height: 400,
      scale: 8,
      rects: [{ x: 0, y: 0, width: 400, height: 400 }],
    });
  });

  it('shrinks the cell so the long side stays within 1080 px', () => {
    expect(frameLayout([{ width: 200, height: 200 }])).toMatchObject({ width: 1000, height: 1000, scale: 5 });
    expect(frameLayout([{ width: 150, height: 40 }])).toMatchObject({ width: 1050, height: 280, scale: 7 });
  });

  it('puts two grids side by side, top-aligned, with a 4 px gap', () => {
    expect(frameLayout([{ width: 50, height: 50 }, { width: 50, height: 50 }])).toEqual({
      width: 804,
      height: 400,
      scale: 8,
      rects: [
        { x: 0, y: 0, width: 400, height: 400 },
        { x: 404, y: 0, width: 400, height: 400 },
      ],
    });
    expect(frameLayout([{ width: 200, height: 200 }, { width: 100, height: 150 }])).toEqual({
      width: 904,
      height: 600,
      scale: 3,
      rects: [
        { x: 0, y: 0, width: 600, height: 600 },
        { x: 604, y: 0, width: 300, height: 450 },
      ],
    });
  });

  it('never goes below one pixel per cell', () => {
    expect(frameLayout([{ width: 2000, height: 10 }]).scale).toBe(1);
  });
});

describe('pickMime', () => {
  it('takes the first supported type in the spec\'s order, with a matching extension', () => {
    expect(pickMime(() => true)).toEqual({ mime: 'video/webm;codecs=vp9', ext: 'webm' });
    expect(pickMime((t) => t === 'video/webm;codecs=vp8' || t === 'video/mp4')).toEqual({ mime: 'video/webm;codecs=vp8', ext: 'webm' });
    expect(pickMime((t) => t === 'video/webm')).toEqual({ mime: 'video/webm', ext: 'webm' });
    expect(pickMime((t) => t === 'video/mp4')).toEqual({ mime: 'video/mp4', ext: 'mp4' });
    expect(pickMime(() => false)).toBeNull();
  });
});

describe('names and clock', () => {
  it('names a recording by its first and last tick', () => {
    expect(recordingName('sugarscape-ii-2-unit-seed7', 10, 250, 'webm')).toBe('sugarscape-ii-2-unit-seed7-t10-t250.webm');
  });

  it('formats elapsed time as m:ss', () => {
    expect(formatClock(0)).toBe('0:00');
    expect(formatClock(61_500)).toBe('1:01');
    expect(formatClock(600_000)).toBe('10:00');
  });

  it('counts recorded time, not pauses', () => {
    const w = new Stopwatch(1000);
    expect(w.elapsed(1500)).toBe(500);
    w.pause(3000);
    w.pause(3500); // already paused
    expect(w.elapsed(5000)).toBe(2000);
    w.resume(6000);
    w.resume(6500); // already running
    expect(w.elapsed(7000)).toBe(3000);
  });
});

describe('GifSampler', () => {
  it('takes about 15 frames a second from 60 Hz snapshots; each delay is the time to the next frame', () => {
    const s = new GifSampler();
    const delays: number[] = [];
    for (let k = 0; k <= 60; k++) {
      const t = (k * 1000) / 60;
      if (!s.due(t, 0)) continue;
      const d = s.take(t);
      if (d !== null) delays.push(d);
    }
    expect(s.frames).toBe(16);
    expect(delays).toHaveLength(15);
    for (const d of delays) expect(gifDelay(d)).toBe(70);
  });

  it('holds back while the encoder is behind, and stops at the frame cap', () => {
    const s = new GifSampler();
    expect(s.due(0, GIF_BACKLOG)).toBe(false);
    expect(s.due(0, GIF_BACKLOG - 1)).toBe(true);
    for (let i = 0; i < GIF_MAX_FRAMES; i++) s.take(i * 100);
    expect(s.full).toBe(true);
    expect(s.due(1e9, 0)).toBe(false);
  });

  it('rounds delays to the 10 ms a GIF stores, at least 20 ms', () => {
    expect(gifDelay(66.7)).toBe(70);
    expect(gifDelay(134)).toBe(130);
    expect(gifDelay(5)).toBe(20);
  });
});

describe('worldViews', () => {
  const size = () => ({ width: 150, height: 150 });
  it("records a grid alone, or Ring World's ring beside its space–time diagram", () => {
    const sugar = worldViews({ model: 'sugarscape', grid: 'grid', ring: 'ring', size: () => ({ width: 50, height: 50 }) }, 'A');
    expect(sugar.map((v) => [v.canvas, v.cells(), v.label])).toEqual([['grid', { width: 50, height: 50 }, 'A']]);
    const ring = worldViews({ model: 'ring', grid: 'grid', ring: 'ring', size }, 'B');
    expect(ring.map((v) => [v.canvas, v.cells(), v.label])).toEqual([
      ['ring', { width: 150, height: 150 }, 'B'],
      ['grid', { width: 150, height: 150 }, undefined],
    ]);
    // Side by side at 3 px a cell, so the frame stays within 1080 px.
    expect(frameLayout(ring.map((v) => v.cells()))).toMatchObject({ width: 904, height: 450, scale: 3 });
  });
});
