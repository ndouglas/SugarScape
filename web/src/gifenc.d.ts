// gifenc 1.0.3 ships no types; these are the parts the GIF worker uses (Decision 15).
declare module 'gifenc' {
  export type Palette = number[][];
  export type Format = 'rgb565' | 'rgb444' | 'rgba4444';
  export interface GifStream {
    writeFrame(
      index: Uint8Array,
      width: number,
      height: number,
      opts?: { palette?: Palette; delay?: number; repeat?: number; transparent?: boolean; transparentIndex?: number; dispose?: number },
    ): void;
    finish(): void;
    bytes(): Uint8Array;
  }
  export function GIFEncoder(opts?: { auto?: boolean; initialCapacity?: number }): GifStream;
  export function quantize(rgba: Uint8Array | Uint8ClampedArray, maxColors: number, options?: { format?: Format }): Palette;
  export function applyPalette(rgba: Uint8Array | Uint8ClampedArray, palette: Palette, format?: Format): Uint8Array;
}
