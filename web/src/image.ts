/**
 * Capacities from RGBA pixels (row-major, 4 bytes each): luminance
 * Y = 0.2126 R + 0.7152 G + 0.0722 B, v = Y / 255 (or 1 − v inverted),
 * capacity round(v · max). A pixel with alpha < 128 is always capacity 0,
 * whether or not Invert is on.
 */
export function capacitiesFromPixels(rgba: ArrayLike<number>, max: number, invert: boolean): Uint8Array {
  const n = Math.floor(rgba.length / 4);
  const out = new Uint8Array(n);
  for (let i = 0; i < n; i++) {
    const [r, g, b, a] = [rgba[4 * i], rgba[4 * i + 1], rgba[4 * i + 2], rgba[4 * i + 3]];
    const y = a < 128 ? 0 : 0.2126 * r + 0.7152 * g + 0.0722 * b;
    const v = a < 128 ? 0 : invert ? 1 - y / 255 : y / 255;
    out[i] = Math.round(v * max);
  }
  return out;
}
