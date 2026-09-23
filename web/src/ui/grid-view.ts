import type { Engine } from '../engine';

const CELL = 12;

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
    this.bctx.putImageData(new ImageData(this.engine.frame(), width, height), 0, 0);
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
