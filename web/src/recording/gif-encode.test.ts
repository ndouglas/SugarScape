import { describe, expect, it } from 'vitest';
import { GifBuilder } from './gif-encode';

/** Walks a GIF's blocks and counts its images (throws on anything malformed). */
function countFrames(b: Uint8Array): number {
  let i = 6; // "GIF89a"
  const packed = b[i + 4];
  i += 7; // logical screen descriptor
  if (packed & 0x80) i += 3 * 2 ** ((packed & 7) + 1); // global color table
  const skipSubBlocks = () => {
    while (b[i] !== 0) i += b[i] + 1;
    i++;
  };
  let frames = 0;
  for (;;) {
    const kind = b[i++];
    if (kind === 0x3b) return frames; // trailer
    if (kind === 0x21) {
      i++; // extension label
      skipSubBlocks();
    } else if (kind === 0x2c) {
      const flags = b[i + 8];
      i += 9; // image descriptor
      if (flags & 0x80) i += 3 * 2 ** ((flags & 7) + 1); // local color table
      i++; // LZW minimum code size
      skipSubBlocks();
      frames++;
    } else {
      throw new Error(`unexpected block 0x${kind.toString(16)} at ${i - 1}`);
    }
  }
}

describe('GifBuilder', () => {
  it('writes a GIF89a file with one image per frame', () => {
    const builder = new GifBuilder();
    for (let k = 0; k < 3; k++) {
      const rgba = new Uint8ClampedArray(8 * 6 * 4);
      for (let p = 0; p < 8 * 6; p++) rgba.set([k * 80, 255 - k * 80, (p % 8) * 30, 255], p * 4);
      builder.add(rgba, 8, 6, 70);
    }
    expect(builder.frames).toBe(3);
    const bytes = builder.finish();
    expect(new TextDecoder().decode(bytes.subarray(0, 6))).toBe('GIF89a');
    expect(countFrames(bytes)).toBe(3);
    expect(bytes.at(-1)).toBe(0x3b);
  });
});
