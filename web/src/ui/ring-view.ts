import type { Engine } from '../engine';
import { RING_AGENT, sugarShade, siteAngle, siteAt } from '../ring';
import type { RingConfig } from '../types';

/** The canvas behind the ring: the frame's dark background (as the grid's), so light agents show. */
const BACKGROUND = sugarShade(0, 1);

/** The ring view's canvas size in pixels (CSS scales it). */
const SIZE = 600;
/** The sugar band's inner and outer radius, and the agents' dots' radius, as fractions of half the size. */
const INNER = 0.72;
const OUTER = 0.9;
const DOTS = 0.64;

/**
 * Ring World's ring (Decision 12): `sites` cells round a circle, shaded by sugar, site 0 at the top
 * and counterclockwise onwards, with a dot inside the band for each agent and the selected site
 * outlined. It draws the engine's `ring` state (sent with every snapshot of a Ring World).
 */
export class RingView {
  /** A pointerdown on the ring picks the site in that direction. */
  onSite: ((site: number) => void) | null = null;
  private readonly ctx: CanvasRenderingContext2D;

  constructor(
    readonly canvas: HTMLCanvasElement,
    private readonly engine: Engine,
  ) {
    canvas.width = SIZE;
    canvas.height = SIZE;
    this.ctx = canvas.getContext('2d')!;
    canvas.addEventListener('pointerdown', (e) => {
      const sites = this.engine.ring?.sugar.length;
      if (e.button !== 0 || !sites) return;
      const r = canvas.getBoundingClientRect();
      this.onSite?.(siteAt(e.clientX - r.left - r.width / 2, e.clientY - r.top - r.height / 2, sites));
    });
  }

  draw(): void {
    const ring = this.engine.ring;
    const config = this.engine.config as RingConfig;
    const ctx = this.ctx;
    ctx.fillStyle = BACKGROUND;
    ctx.fillRect(0, 0, SIZE, SIZE);
    if (this.engine.model !== 'ring' || !ring) return;
    const n = ring.sugar.length;
    const c = SIZE / 2;
    const half = Math.PI / n;
    for (let i = 0; i < n; i++) {
      const a = siteAngle(i, n);
      ctx.beginPath();
      ctx.arc(c, c, OUTER * c, a - half, a + half);
      ctx.arc(c, c, INNER * c, a + half, a - half, true);
      ctx.closePath();
      ctx.fillStyle = sugarShade(ring.sugar[i], config.capacity);
      ctx.fill();
    }
    ctx.fillStyle = RING_AGENT;
    const dot = Math.max(2, Math.min(6, (Math.PI * DOTS * c) / n));
    for (const site of ring.agents) {
      const a = siteAngle(site, n);
      ctx.beginPath();
      ctx.arc(c + Math.cos(a) * DOTS * c, c + Math.sin(a) * DOTS * c, dot, 0, 2 * Math.PI);
      ctx.fill();
    }
    const sel = this.engine.selection;
    if (sel && sel.x < n) {
      const a = siteAngle(sel.x, n);
      ctx.beginPath();
      ctx.arc(c, c, (OUTER + 0.02) * c, a - half, a + half);
      ctx.arc(c, c, (INNER - 0.02) * c, a + half, a - half, true);
      ctx.closePath();
      ctx.lineWidth = 3;
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#fff';
      ctx.stroke();
    }
  }
}
