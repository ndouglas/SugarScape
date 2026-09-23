import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import { h } from '../ui/dom';
import { compactNumber } from '../ui/format';
import { describePoint, type ChartData } from './chart-data';

const HEIGHT = 360;
/** Line colors after --c1…--c4 (Decision 21). */
const EXTRA_COLORS = ['#b8860b', '#17a2b8', '#d63384', '#6c757d'];

/** One line per series (the mean over seeds) with a ±1 sd band; the legend shows mean, sd, range and n on hover. */
export class SweepChart {
  readonly el = h('figure', { class: 'chart sweep-chart' });
  private plot: uPlot | null = null;
  /** Labels and line names of the plot on screen; a change rebuilds it. */
  private signature = '';
  private data: ChartData | null = null;

  constructor() {
    new ResizeObserver(() => this.plot?.setSize({ width: this.width(), height: HEIGHT })).observe(this.el);
  }

  draw(data: ChartData): void {
    this.data = data;
    const aligned: uPlot.AlignedData = [data.x, ...data.lines.map((l) => l.mean)];
    const signature = JSON.stringify([data.xLabel, data.yLabel, data.lines.map((l) => l.name)]);
    if (this.plot && signature === this.signature) {
      this.plot.setData(aligned);
      return;
    }
    this.plot?.destroy();
    this.signature = signature;
    this.plot = new uPlot(this.options(data), aligned, this.el);
  }

  clear(): void {
    this.plot?.destroy();
    this.plot = null;
    this.signature = '';
    this.data = null;
  }

  canvas(): HTMLCanvasElement | null {
    return this.plot?.ctx.canvas ?? null;
  }

  private width(): number {
    return Math.max(320, this.el.clientWidth - 4);
  }

  private options(data: ChartData): uPlot.Options {
    const css = getComputedStyle(document.documentElement);
    const token = (name: string, fallback: string) => css.getPropertyValue(name).trim() || fallback;
    const colors = data.lines.map((_, k) => (k < 4 ? token(`--c${k + 1}`, '#888') : EXTRA_COLORS[(k - 4) % EXTRA_COLORS.length]));
    const axis = { stroke: token('--muted', '#888'), grid: { stroke: token('--grid', '#ddd') }, ticks: { stroke: token('--grid', '#ddd') } };
    return {
      width: this.width(),
      height: HEIGHT,
      scales: { x: { time: false }, y: { range: (_u, min, max) => this.yRange(min, max) } },
      axes: [
        { ...axis, label: data.xLabel },
        { ...axis, label: data.yLabel, size: 56, values: (_u, splits) => splits.map(compactNumber) },
      ],
      series: [
        { label: data.xLabel },
        ...data.lines.map((line, k) => ({
          label: line.name,
          stroke: colors[k],
          width: 2,
          points: { show: true, size: 5 },
          value: (_u: uPlot, _v: number | null, _s: number, i: number | null) =>
            i === null || !this.data ? '—' : describePoint(this.data.lines[k], i),
        })),
      ],
      hooks: { drawAxes: [(u: uPlot) => this.drawBands(u, colors)] },
    };
  }

  /** The data range widened to cover the bands, padded by 5%. */
  private yRange(min: number, max: number): [number, number] {
    let lo = Number.isFinite(min) ? min : Infinity;
    let hi = Number.isFinite(max) ? max : -Infinity;
    for (const line of this.data?.lines ?? []) {
      for (const v of line.lo) if (v !== null && v < lo) lo = v;
      for (const v of line.hi) if (v !== null && v > hi) hi = v;
    }
    if (!Number.isFinite(lo) || !Number.isFinite(hi)) return [0, 1];
    if (lo === hi) return [lo - 1, hi + 1];
    const pad = (hi - lo) * 0.05;
    return [lo - pad, hi + pad];
  }

  /** Paints each line's mean ± sd band under the lines (Decision 21); gaps split a band. */
  private drawBands(u: uPlot, colors: string[]): void {
    const data = this.data;
    if (!data) return;
    const ctx = u.ctx;
    ctx.save();
    ctx.beginPath();
    ctx.rect(u.bbox.left, u.bbox.top, u.bbox.width, u.bbox.height);
    ctx.clip();
    ctx.globalAlpha = 0.18;
    data.lines.forEach((line, k) => {
      ctx.fillStyle = colors[k];
      let run: [number, number, number][] = [];
      const flush = () => {
        if (run.length > 1) {
          ctx.beginPath();
          run.forEach(([x, hi], i) => (i === 0 ? ctx.moveTo(x, hi) : ctx.lineTo(x, hi)));
          for (let i = run.length - 1; i >= 0; i--) ctx.lineTo(run[i][0], run[i][2]);
          ctx.closePath();
          ctx.fill();
        }
        run = [];
      };
      data.x.forEach((xv, i) => {
        const hi = line.hi[i];
        const lo = line.lo[i];
        if (hi === null || lo === null) {
          flush();
          return;
        }
        run.push([u.valToPos(xv, 'x', true), u.valToPos(hi, 'y', true), u.valToPos(lo, 'y', true)]);
      });
      flush();
    });
    ctx.restore();
  }
}
