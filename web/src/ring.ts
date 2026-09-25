// Ring World's page-side geometry and colors (the ring view, Decision 12).

/** Rows of the host's space–time diagram (the core's `ring::HISTORY`). */
export const RING_HISTORY = 150;

/** The frame's colors (crates/sugarscape-core/src/render.rs `BACKGROUND`, `SUGAR`; ring.rs `AGENT`). */
export const RING_BACKGROUND: [number, number, number] = [0x16, 0x15, 0x12];
export const RING_SUGAR: [number, number, number] = [0xf2, 0xc1, 0x4e];
export const RING_AGENT = 'rgb(79, 157, 255)';

/**
 * Site `i`'s angle on the canvas (radians; canvas y points down): site 0 at the top and increasing
 * index counterclockwise, the direction agents look and move.
 */
export function siteAngle(i: number, sites: number): number {
  return -Math.PI / 2 - (2 * Math.PI * i) / sites;
}

/** The site whose angle is nearest the direction (dx, dy) from the centre (canvas coordinates). */
export function siteAt(dx: number, dy: number, sites: number): number {
  const turns = (-Math.PI / 2 - Math.atan2(dy, dx)) / (2 * Math.PI);
  const i = Math.round((((turns % 1) + 1) % 1) * sites);
  return i % sites;
}

/** A site's color: dark to sugar yellow by `sugar / capacity` (as the diagram shades it). */
export function sugarShade(sugar: number, capacity: number): string {
  const t = capacity > 0 ? Math.min(1, Math.max(0, sugar / capacity)) : 0;
  const [r, g, b] = RING_BACKGROUND.map((c, k) => Math.round(c + (RING_SUGAR[k] - c) * t));
  return `rgb(${r}, ${g}, ${b})`;
}
