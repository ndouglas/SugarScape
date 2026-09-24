import { applyPalette, GIFEncoder, quantize } from 'gifenc';

/** Page → GIF worker: a frame's RGBA pixels (transferred) and its delay in ms, or "finish the file". */
export type GifRequest = { type: 'frame'; rgba: ArrayBuffer; width: number; height: number; delay: number } | { type: 'finish' };

/** GIF worker → page: a frame was encoded (`frames` so far), or the finished file (transferred). */
export type GifReply = { type: 'ack'; frames: number } | { type: 'done'; bytes: ArrayBuffer };

/** An animated GIF built a frame at a time, each with its own 256-colour palette (Decision 14). */
export class GifBuilder {
  frames = 0;
  private readonly gif = GIFEncoder();

  add(rgba: Uint8Array | Uint8ClampedArray, width: number, height: number, delay: number): void {
    const palette = quantize(rgba, 256);
    this.gif.writeFrame(applyPalette(rgba, palette), width, height, { palette, delay });
    this.frames++;
  }

  /** The finished file. */
  finish(): Uint8Array {
    this.gif.finish();
    return this.gif.bytes();
  }
}
