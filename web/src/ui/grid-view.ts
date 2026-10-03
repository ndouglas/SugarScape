import { geosimLegend } from '../geosim';
import { polarityLegend } from '../polarity';
import { auctionLegend } from '../auctions';
import type { Engine } from '../engine';
import { isSugarView } from '../models';
import { NETWORKS, type NetworkOverlay } from '../protocol';
import type { AgentView, GeosimConfig, GeosimStats, AuctionsConfig, AuctionsStats, PolarityConfig, PolarityStats, HoardConfig } from '../types';
import { linkSegments, SETTLEMENT_COLOR, settlementRadius, settlements, WATER_COLOR } from '../valley';
import { cacheSize, cacheSummary, compartmentName, isCheaterOnly, isLarder, labStatus } from '../minds';
import { h } from './dom';
import { hoardColumn } from '../hoard';
import { colorLegend, HOARD_STATUS, hoardLegend, legendElement, overlayLegend, type MapMarks } from './legend';
import { cacheMarks, memoryMarks } from './memory-overlay';
import { arrowHead, wrappedSegments } from './overlay';
import { planSegments, routeSegments } from './plan-path';
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
  /** Under the map: a lab's status line, then the legend (the color mode's colors, the overlays shown). */
  readonly legend = h('div', { class: 'map-legend' });
  private legendKey = '';
  /** The fullest cache site this draw (`marks` finds it in its one pass over the flat array). */
  private cacheMax = 0;

  constructor(readonly canvas: HTMLCanvasElement, private engine: Engine) {
    this.ctx = canvas.getContext('2d')!;
    this.bctx = this.buffer.getContext('2d')!;
    canvas.after(this.legend);
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
    // The backing store matches the canvas's size on screen (k device pixels per cell-scale pixel),
    // so overlays and labels are drawn at device resolution rather than upscaled; everything below
    // draws in cell-scale units (CELL a cell) through the transform.
    const k = backingScale(this.canvas.clientWidth, width);
    if (this.canvas.width !== width * CELL * k || this.canvas.height !== height * CELL * k) {
      this.canvas.width = width * CELL * k;
      this.canvas.height = height * CELL * k;
    }
    this.canvas.style.aspectRatio = `${width} / ${height}`;
    const ctx = this.ctx;
    ctx.setTransform(k, 0, 0, k, 0, 0);
    // The agents' bitmap stays crisp: one sharp block per cell.
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

    const inspection = this.engine.inspection;
    const agent = inspection && isSugarView(inspection.view) ? inspection.view.agent : null;
    const marks = this.marks(agent);
    this.drawLab(marks);
    this.drawAllCaches(marks);
    this.drawHomes(marks);
    this.drawMemory(agent, marks);
    this.drawCaches(agent);
    const accent = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#fff';
    if (agent?.plan && agent.plan.path.length) {
      ctx.save();
      ctx.setLineDash([4, 4]);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (const [x1, y1, x2, y2] of planSegments([agent.x, agent.y], agent.plan.path, width, height)) {
        ctx.moveTo((x1 + 0.5) * CELL, (y1 + 0.5) * CELL);
        ctx.lineTo((x2 + 0.5) * CELL, (y2 + 0.5) * CELL);
      }
      ctx.stroke();
      ctx.restore();
    }
    // Minds 4: a GOAP agent's plan, a sparser dashed route from the agent through its targets
    // (straight lines between targets, the short way around the torus; the walk to the next one is
    // the A* path above), each target ringed.
    if (agent?.goap && agent.goap.steps.length) {
      ctx.save();
      ctx.setLineDash([2, 5]);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (const [x1, y1, x2, y2] of routeSegments([agent.x, agent.y], agent.goap.steps, width, height)) {
        ctx.moveTo((x1 + 0.5) * CELL, (y1 + 0.5) * CELL);
        ctx.lineTo((x2 + 0.5) * CELL, (y2 + 0.5) * CELL);
      }
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.lineWidth = 1.5;
      for (const [x, y] of agent.goap.steps) {
        ctx.beginPath();
        ctx.arc((x + 0.5) * CELL, (y + 0.5) * CELL, CELL * 0.45, 0, Math.PI * 2);
        ctx.stroke();
      }
      ctx.restore();
    }

    const sel = this.engine.selection;
    if (sel) {
      // Minds 7: a click selects an agent's whole column, so the whole column is outlined.
      const [x, y, w, hgt] =
        this.engine.model === 'hoard'
          ? (() => {
              const [from, to] = hoardColumn(sel.x, (this.engine.config as HoardConfig).n, width);
              return [from, 0, to - from, height];
            })()
          : [sel.x, sel.y, 1, 1];
      // The hoard frame is drawn small (200 cells across), so its outline is drawn thicker.
      const thick = this.engine.model === 'hoard' ? 3 : 1;
      ctx.strokeStyle = '#000';
      ctx.lineWidth = 4 * thick;
      ctx.strokeRect(x * CELL - 2, y * CELL - 2, w * CELL + 4, hgt * CELL + 4);
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2 * thick;
      ctx.strokeRect(x * CELL - 2, y * CELL - 2, w * CELL + 4, hgt * CELL + 4);
    }
    this.drawLegend(marks);
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
   * What the map shows besides agents (the legend lists the same): the Minds overlays from the
   * engine's `MindsView`, and the selected agent's memory, caches and routes.
   */
  marks(agent: AgentView | null): MapMarks {
    const sugar = this.engine.model === 'sugarscape';
    const minds = sugar ? this.engine.minds : null;
    const config = this.engine.sugar;
    const overlay = sugar && this.engine.overlays.caches;
    const sites = overlay ? this.engine.cacheSites : null;
    const summary = sites ? cacheSummary(sites) : null;
    this.cacheMax = summary?.max ?? 0;
    const homes = overlay ? (minds?.homes ?? []) : [];
    const span = config.memory?.span ?? 0;
    const remembered =
      span > 0 && config.memory?.prior !== 'map' && agent !== null ? memoryMarks(this.engine.inspectMemory(), span) : [];
    return {
      allCaches: (sites?.length ?? 0) > 0,
      cheaterCaches: summary?.cheaterOnly ?? false,
      ownCaches: (agent?.caching?.caches.length ?? 0) > 0,
      homes: homes.length > 0,
      guarding: homes.some((home) => home.guarding),
      larders: homes.some((home) => home.larder > 0),
      memory: remembered.length > 0,
      spots: remembered.some((m) => m.shape === 'circle'),
      path: (agent?.plan?.path.length ?? 0) > 0,
      route: (agent?.goap?.steps.length ?? 0) > 0,
      lab: minds?.lab ?? null,
    };
  }

  /** The lab's status line and the legend, rebuilt only when their contents change. */
  private drawLegend(marks: MapMarks): void {
    const sugar = this.engine.model === 'sugarscape';
    // Minds 7: the frame is a population panel, one column per agent.
    const hoard = this.engine.model === 'hoard';
    const auction = this.engine.model === 'auctions' ? auctionLegend(this.engine.config as AuctionsConfig, this.engine.colorMode, (this.engine.latest as AuctionsStats | null)?.periods ?? 0) : null;
    const polarity = this.engine.model === 'polarity' ? polarityLegend(this.engine.config as PolarityConfig, this.engine.colorMode, (this.engine.latest as PolarityStats | null)?.periods ?? 0) : null;
    const geosim = this.engine.model === 'geosim' ? geosimLegend(this.engine.config as GeosimConfig, this.engine.colorMode, (this.engine.latest as GeosimStats | null)?.periods ?? 0) : null;
    const status = geosim ?? polarity ?? (auction ? auction.status : sugar && marks.lab ? labStatus(marks.lab) : hoard ? HOARD_STATUS : '');
    const items = auction ? auction.items : sugar
      ? [...colorLegend(this.engine.colorMode, this.engine.sugar), ...overlayLegend(marks, this.engine.sugar)]
      : hoard
        ? hoardLegend(((this.engine.config as HoardConfig).cheaters ?? 0) > 0)
        : [];
    const key = JSON.stringify([status, items]);
    if (key === this.legendKey) return;
    this.legendKey = key;
    this.legend.replaceChildren(
      ...(status ? [h('div', { class: 'map-status' }, status)] : []),
      ...(items.length ? [h('div', { class: 'legend-items' }, ...legendElement(items))] : []),
    );
    this.legend.hidden = !status && items.length === 0;
  }

  /**
   * Minds 5, a lab: each compartment's name (K1–K3) in its corner, the caching trays as dashed
   * squares (`--c3`), and on the test evening a ring round the agent whose turn it is.
   */
  private drawLab(marks: MapMarks): void {
    const lab = marks.lab;
    if (!lab) return;
    const ctx = this.ctx;
    const style = getComputedStyle(this.canvas);
    const tray = style.getPropertyValue('--c3').trim() || '#0c0';
    const accent = style.getPropertyValue('--accent').trim() || '#fa0';
    ctx.save();
    ctx.font = `bold ${Math.round(CELL * 0.62)}px ui-sans-serif, system-ui, sans-serif`;
    ctx.textBaseline = 'top';
    // The grid is dark in either theme: light labels.
    ctx.fillStyle = '#ece8dd';
    ctx.globalAlpha = 0.9;
    lab.compartments.forEach(([x, y], k) => ctx.fillText(compartmentName(k), x * CELL + 1, y * CELL + 1));
    ctx.globalAlpha = 1;
    ctx.strokeStyle = tray;
    ctx.lineWidth = 1.5;
    ctx.setLineDash([2, 1.5]);
    for (const [, x, y] of lab.trays) ctx.strokeRect((x + 0.12) * CELL, (y + 0.12) * CELL, CELL * 0.76, CELL * 0.76);
    ctx.setLineDash([]);
    if (lab.phase === 'test' && lab.turn_at) {
      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc((lab.turn_at[0] + 0.5) * CELL, (lab.turn_at[1] + 0.5) * CELL, CELL * 0.62, 0, Math.PI * 2);
      ctx.stroke();
    }
    ctx.restore();
  }

  /**
   * Minds 5–6: every cache in the world (the Caches overlay), a diamond per site sized by its sugar,
   * `--c4` or, where only cheaters own caches, `--red`; a central world's larders draw with the homes.
   */
  private drawAllCaches(marks: MapMarks): void {
    if (!marks.allCaches) return;
    const flat = this.engine.cacheSites;
    const ctx = this.ctx;
    const style = getComputedStyle(this.canvas);
    const hoarder = style.getPropertyValue('--c4').trim() || '#a0f';
    const cheater = style.getPropertyValue('--red').trim() || '#f44';
    ctx.save();
    ctx.globalAlpha = 0.75;
    ctx.lineWidth = 0.75;
    ctx.strokeStyle = '#000';
    // Straight from the flat `[x, y, total, flags, …]`: no per-site objects.
    for (let i = 0; i + 3 < flat.length; i += 4) {
      const flags = flat[i + 3];
      if (marks.homes && isLarder(flags)) continue;
      ctx.fillStyle = isCheaterOnly(flags) ? cheater : hoarder;
      diamond(ctx, (flat[i] + 0.5) * CELL, (flat[i + 1] + 0.5) * CELL, cacheSize(flat[i + 2], this.cacheMax) * CELL);
    }
    ctx.restore();
  }

  /**
   * Minds 5, central worlds, with the caches overlay: every agent's home as a faint square (`--c3`)
   * and its larder as a diamond sized by what it holds against the fullest.
   */
  private drawHomes(marks: MapMarks): void {
    const homes = this.engine.minds?.homes;
    if (!marks.homes || !homes) return;
    const ctx = this.ctx;
    const color = getComputedStyle(this.canvas).getPropertyValue('--c3').trim() || '#0c0';
    const most = homes.reduce((m, home) => Math.max(m, home.larder), 0);
    ctx.save();
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.globalAlpha = 0.6;
    for (const home of homes) ctx.strokeRect((home.x + 0.1) * CELL, (home.y + 0.1) * CELL, CELL * 0.8, CELL * 0.8);
    ctx.globalAlpha = 0.85;
    ctx.fillStyle = color;
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 0.75;
    for (const home of homes) {
      if (home.larder <= 0) continue;
      const r = (0.12 + 0.2 * Math.sqrt(home.larder / most)) * CELL;
      diamond(ctx, (home.x + 0.5) * CELL, (home.y + 0.5) * CELL, r);
    }
    // A solid outer frame marks an owner who guarded this tick.
    ctx.globalAlpha = 1;
    ctx.strokeStyle = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#ffb000';
    ctx.lineWidth = 2.5;
    for (const home of homes) {
      if (home.guarding) ctx.strokeRect((home.x + 0.03) * CELL, (home.y + 0.03) * CELL, CELL * 0.94, CELL * 0.94);
    }
    ctx.restore();
  }

  /**
   * Minds 3: the inspected agent's remembered sites — small squares (`--c2`), fading with age —
   * and, among them, known truffle spots as small circles (`--accent`, filled when believed ripe).
   * Nothing while memory is off, nothing is selected, or the selection has no agent; nor when
   * founders know the map (`memory.prior: 'map'`), where it would mark every site.
   */
  private drawMemory(agent: AgentView | null, shown: MapMarks): void {
    const span = this.engine.sugar.memory?.span ?? 0;
    if (!agent || span <= 0 || !shown.memory) return;
    const marks = memoryMarks(this.engine.inspectMemory(), span);
    if (marks.length === 0) return;
    const ctx = this.ctx;
    const siteColor = getComputedStyle(this.canvas).getPropertyValue('--c2').trim() || '#0f0';
    const spotColor = getComputedStyle(this.canvas).getPropertyValue('--accent').trim() || '#f80';
    ctx.save();
    for (const m of marks) {
      ctx.globalAlpha = m.alpha;
      if (m.shape === 'square') {
        ctx.fillStyle = siteColor;
        ctx.fillRect((m.x + 0.3) * CELL, (m.y + 0.3) * CELL, CELL * 0.4, CELL * 0.4);
        continue;
      }
      ctx.strokeStyle = spotColor;
      ctx.fillStyle = spotColor;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.arc((m.x + 0.5) * CELL, (m.y + 0.5) * CELL, CELL * 0.28, 0, Math.PI * 2);
      if (m.filled) ctx.fill();
      ctx.stroke();
    }
    ctx.restore();
  }

  /**
   * Minds 5: the inspected agent's caches as small diamonds (`--c4`, outlined in `--accent`, so they
   * stand out among everyone's), sized by how much each holds, and, in a central-place world, its
   * home as an outlined square.
   */
  private drawCaches(agent: AgentView | null): void {
    if (!agent) return;
    const marks = cacheMarks(agent.caching?.caches ?? []);
    const home = agent.central?.home;
    if (marks.length === 0 && !home) return;
    const ctx = this.ctx;
    const style = getComputedStyle(this.canvas);
    const color = style.getPropertyValue('--c4').trim() || '#a0f';
    ctx.save();
    ctx.lineWidth = 2;
    ctx.strokeStyle = style.getPropertyValue('--accent').trim() || '#fa0';
    ctx.fillStyle = color;
    for (const m of marks) diamond(ctx, (m.x + 0.5) * CELL, (m.y + 0.5) * CELL, m.size * CELL);
    if (home) {
      ctx.strokeStyle = style.getPropertyValue('--c3').trim() || '#0c0';
      ctx.lineWidth = 2;
      ctx.strokeRect((home[0] + 0.1) * CELL, (home[1] + 0.1) * CELL, CELL * 0.8, CELL * 0.8);
    }
    ctx.restore();
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

/** A filled, stroked diamond of half-width `r` centered on (cx, cy). */
function diamond(ctx: CanvasRenderingContext2D, cx: number, cy: number, r: number): void {
  ctx.beginPath();
  ctx.moveTo(cx, cy - r);
  ctx.lineTo(cx + r, cy);
  ctx.lineTo(cx, cy + r);
  ctx.lineTo(cx - r, cy);
  ctx.closePath();
  ctx.fill();
  ctx.stroke();
}

/**
 * Device pixels per cell-scale pixel for a grid `width` cells wide shown `clientWidth` CSS pixels
 * wide: at least 1, at most 16 (a small lab rig on a large screen).
 */
export function backingScale(clientWidth: number, width: number): number {
  const dpr = typeof devicePixelRatio === 'number' ? devicePixelRatio : 1;
  if (!(clientWidth > 0) || !(width > 0)) return 1;
  return Math.min(16, Math.max(1, Math.round((clientWidth * dpr) / (width * CELL))));
}
