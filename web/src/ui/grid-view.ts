import type { Engine } from '../engine';
import { NETWORKS, type NetworkOverlay } from '../protocol';
import { linkSegments, SETTLEMENT_COLOR, settlementRadius, settlements, WATER_COLOR } from '../valley';
import { arrowHead, wrappedSegments } from './overlay';
import { trailSegments } from './trail';

const CELL = 12;

/** How each network's edges are drawn: a color token, and a direction marker for directed networks. */
const OVERLAY_STYLE: Record<NetworkOverlay, { color: string; directed?: true; width: number; alpha: number }> = {
  trade: { color: '--c3', width: 1.5, alpha: 0.8 },
  credit: { color: '--c2', width: 1.5, alpha: 0.8 },
  disease: { color: '--c4', width: 1.5, alpha: 0.8 },
  neighbors: { color: '--muted', directed: true, width: 1, alpha: 0.7 },
  friends: { color: '--c1', width: 1.5, alpha: 0.8 },
  family: { color: '--accent', width: 1.5, alpha: 0.8 },
};

export type CellEvent = 'down' | 'drag';

/** Draws the Rust-rendered frame scaled up with crisp pixels, plus overlays. */
export class GridView {
  onCell: ((x: number, y: number, kind: CellEvent) => void) | null = null;
  /** When set, a brush outline of this radius follows the pointer. */
  brushRadius: number | null = null;
  private hover: { x: number; y: number } | null = null;
  private ctx: CanvasRenderingContext2D;
  private buffer = document.createElement('canvas');
  private bctx: CanvasRenderingContext2D;

  constructor(readonly canvas: HTMLCanvasElement, private engine: Engine) {
    this.ctx = canvas.getContext('2d')!;
    this.bctx = this.buffer.getContext('2d')!;
    canvas.addEventListener('pointerdown', (e) => {
      if (e.button !== 0) return; // only the primary button uses tools
      canvas.setPointerCapture(e.pointerId);
      const c = this.cellAt(e);
      this.hover = c;
      this.onCell?.(c.x, c.y, 'down');
    });
    canvas.addEventListener('pointermove', (e) => {
      const c = this.cellAt(e);
      const moved = !this.hover || c.x !== this.hover.x || c.y !== this.hover.y;
      this.hover = c;
      if (moved && e.buttons & 1) this.onCell?.(c.x, c.y, 'drag');
      if (moved) this.draw();
    });
    canvas.addEventListener('pointerleave', () => {
      this.hover = null;
      this.draw();
    });
  }

  private cellAt(e: PointerEvent): { x: number; y: number } {
    const r = this.canvas.getBoundingClientRect();
    const { width, height } = this.engine.size();
    const clamp = (v: number, n: number) => Math.min(n - 1, Math.max(0, Math.floor(v)));
    return {
      x: clamp(((e.clientX - r.left) / r.width) * width, width),
      y: clamp(((e.clientY - r.top) / r.height) * height, height),
    };
  }

  private blit(): { width: number; height: number } {
    const { width, height } = this.engine.size();
    if (this.buffer.width !== width || this.buffer.height !== height) {
      this.buffer.width = width;
      this.buffer.height = height;
    }
    const frame = this.engine.frame();
    // Until the reply that resized the grid brings a frame of the new size, the old one is not drawn.
    if (frame && frame.length === width * height * 4) this.bctx.putImageData(new ImageData(frame, width, height), 0, 0);
    return { width, height };
  }

  draw(): void {
    const { width, height } = this.blit();
    if (this.canvas.width !== width * CELL || this.canvas.height !== height * CELL) {
      this.canvas.width = width * CELL;
      this.canvas.height = height * CELL;
      this.canvas.style.aspectRatio = `${width} / ${height}`;
    }
    const ctx = this.ctx;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.buffer, 0, 0, width * CELL, height * CELL);

    for (const kind of NETWORKS) {
      const style = OVERLAY_STYLE[kind];
      if (!this.engine.overlays[kind]) continue;
      const e = this.engine.networks(kind);
      const color = getComputedStyle(this.canvas).getPropertyValue(style.color).trim() || '#fff';
      ctx.save();
      ctx.strokeStyle = color;
      ctx.fillStyle = color;
      ctx.globalAlpha = style.alpha;
      ctx.lineWidth = style.width;
      ctx.beginPath();
      const heads = new Path2D();
      for (let i = 0; i < e.length; i += 4) {
        const segments = wrappedSegments(e[i], e[i + 1], e[i + 2], e[i + 3], width, height);
        for (const [ax, ay, bx, by] of segments) {
          ctx.moveTo((ax + 0.5) * CELL, (ay + 0.5) * CELL);
          ctx.lineTo((bx + 0.5) * CELL, (by + 0.5) * CELL);
        }
        if (!style.directed) continue;
        // The last segment ends at the target: the marker sits just outside its cell.
        const [ax, ay, bx, by] = segments[segments.length - 1];
        const head = arrowHead((ax + 0.5) * CELL, (ay + 0.5) * CELL, (bx + 0.5) * CELL, (by + 0.5) * CELL, 4, CELL / 2);
        if (!head) continue;
        heads.moveTo(head[0], head[1]);
        heads.lineTo(head[2], head[3]);
        heads.lineTo(head[4], head[5]);
        heads.closePath();
      }
      ctx.stroke();
      if (style.directed) ctx.fill(heads);
      ctx.restore();
    }

    this.drawValley(width, height);

    if (this.engine.followed() !== null) {
      ctx.save();
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--text').trim() || '#000';
      ctx.lineWidth = 2;
      ctx.lineCap = 'round';
      for (const s of trailSegments(this.engine.trail(), width, height)) {
        ctx.globalAlpha = s.alpha;
        ctx.beginPath();
        ctx.moveTo((s.x1 + 0.5) * CELL, (s.y1 + 0.5) * CELL);
        ctx.lineTo((s.x2 + 0.5) * CELL, (s.y2 + 0.5) * CELL);
        ctx.stroke();
      }
      ctx.restore();
    }

    const accent = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#fff';
    const sel = this.engine.selection;
    if (sel) {
      ctx.strokeStyle = '#000';
      ctx.lineWidth = 4;
      ctx.strokeRect(sel.x * CELL - 2, sel.y * CELL - 2, CELL + 4, CELL + 4);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.strokeRect(sel.x * CELL - 2, sel.y * CELL - 2, CELL + 4, CELL + 4);
    }
    if (this.hover && this.brushRadius !== null) {
      ctx.save();
      ctx.setLineDash([4, 3]);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc((this.hover.x + 0.5) * CELL, (this.hover.y + 0.5) * CELL, (this.brushRadius + 0.5) * CELL, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
    }
  }

  /**
   * The anasazi's overlays (milestone 10), those that are on: water sources as small blue squares,
   * each household's farm–home link as a faint line, and settlements as yellow dots sized by their
   * households.
   */
  private drawValley(width: number, height: number): void {
    const valley = this.engine.valley;
    if (this.engine.model !== 'anasazi' || !valley) return;
    const on = this.engine.overlays;
    const ctx = this.ctx;
    ctx.save();
    if (on.water) {
      ctx.fillStyle = WATER_COLOR;
      ctx.globalAlpha = 0.8;
      const w = valley.water;
      for (let i = 0; i + 1 < w.length; i += 2) ctx.fillRect((w[i] + 0.25) * CELL, (w[i + 1] + 0.25) * CELL, CELL / 2, CELL / 2);
    }
    if (on.links) {
      ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--text').trim() || '#fff';
      ctx.globalAlpha = 0.45;
      ctx.lineWidth = 1;
      ctx.beginPath();
      const l = valley.links;
      const config = this.engine.config;
      for (let i = 0; i + 3 < l.length; i += 4) {
        for (const [ax, ay, bx, by] of linkSegments(config, l[i], l[i + 1], l[i + 2], l[i + 3], width, height)) {
          ctx.moveTo((ax + 0.5) * CELL, (ay + 0.5) * CELL);
          ctx.lineTo((bx + 0.5) * CELL, (by + 0.5) * CELL);
        }
      }
      ctx.stroke();
    }
    if (on.settlements) {
      ctx.globalAlpha = 0.9;
      ctx.fillStyle = SETTLEMENT_COLOR;
      ctx.strokeStyle = '#000';
      ctx.lineWidth = 1;
      for (const s of settlements(valley)) {
        ctx.beginPath();
        ctx.arc((s.x + 0.5) * CELL, (s.y + 0.5) * CELL, settlementRadius(s.households, CELL), 0, 2 * Math.PI);
        ctx.fill();
        ctx.stroke();
      }
    }
    ctx.restore();
  }

  /** The grid without overlays, scaled up, as a PNG. */
  toPngBlob(scale = 10): Promise<Blob> {
    const { width, height } = this.blit();
    const out = document.createElement('canvas');
    out.width = width * scale;
    out.height = height * scale;
    const ctx = out.getContext('2d')!;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(this.buffer, 0, 0, out.width, out.height);
    return new Promise((resolve, reject) =>
      out.toBlob((b) => (b ? resolve(b) : reject(new Error('PNG encoding failed'))), 'image/png'),
    );
  }
}
