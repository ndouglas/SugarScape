import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { h } from './dom';

interface Line { key: string; label: string; color: string }
interface TimeChart { title: string; lines: Line[]; range?: [number, number] }

const TIME_CHARTS: TimeChart[] = [
  { title: 'Population', lines: [{ key: 'population', label: 'Agents', color: '--c1' }] },
  { title: 'Gini coefficient', lines: [{ key: 'gini', label: 'Gini', color: '--c2' }], range: [0, 1] },
  {
    title: 'Mean traits',
    lines: [
      { key: 'mean_vision', label: 'Vision', color: '--c1' },
      { key: 'mean_metabolism', label: 'Metabolism', color: '--c3' },
    ],
  },
  { title: 'Blue share', lines: [{ key: 'blue_fraction', label: 'Blue', color: '--blue' }], range: [0, 1] },
  {
    title: 'Births and deaths',
    lines: [
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ],
  },
];

const HEIGHT = 150;
const REFRESH_MS = 250;

/** Fetches `Sim.series(name)` at most once per refresh; shared across all charts' update closures. */
type SeriesCache = (name: string) => number[];

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: { name: string; plot: uPlot; update: (series: SeriesCache) => void }[] = [];
  private visible = false;
  private last = 0;

  constructor(private engine: Engine) {
    this.build();
    engine.on('reset', () => this.refresh());
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.resize();
      this.refresh();
    }
  }

  /** Called every animation frame; redraws at most every REFRESH_MS while visible. */
  maybeRefresh(now: number): void {
    if (this.visible && now - this.last > REFRESH_MS) this.refresh();
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.name, canvas: p.plot.ctx.canvas }));
  }

  private refresh(): void {
    this.last = performance.now();
    if (!this.visible) return;
    const cache = new Map<string, number[]>();
    const series: SeriesCache = (name) => {
      let arr = cache.get(name);
      if (!arr) {
        arr = Array.from(this.engine.sim.series(name));
        cache.set(name, arr);
      }
      return arr;
    };
    this.plots.forEach((p) => p.update(series));
  }

  private width(): number {
    return Math.max(240, this.el.clientWidth - 4);
  }

  private resize(): void {
    if (!this.visible) return;
    this.plots.forEach((p) => p.plot.setSize({ width: this.width(), height: HEIGHT }));
  }

  private add(
    title: string,
    opts: Omit<uPlot.Options, 'width' | 'height'>,
    data: uPlot.AlignedData,
    update: (plot: uPlot, series: SeriesCache) => void,
  ): void {
    const figure = h('figure', { class: 'chart' }, h('figcaption', {}, title));
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, update: (series) => update(plot, series) });
    this.el.append(figure);
  }

  private build(): void {
    const css = getComputedStyle(document.documentElement);
    const color = (v: string) => css.getPropertyValue(v).trim() || '#888';
    const axes: uPlot.Axis[] = [
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') } },
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') }, size: 44 },
    ];

    for (const chart of TIME_CHARTS) {
      this.add(
        chart.title,
        {
          scales: { x: { time: false }, y: chart.range ? { range: chart.range } : {} },
          axes,
          legend: { show: chart.lines.length > 1 },
          series: [{ label: 'Tick' }, ...chart.lines.map((l) => ({ label: l.label, stroke: color(l.color), width: 1.5 }))],
        },
        [[], ...chart.lines.map(() => [])],
        (plot, series) => plot.setData([series('tick'), ...chart.lines.map((l) => series(l.key))]),
      );
    }

    const xs = Array.from({ length: 101 }, (_, i) => i / 100);
    this.add(
      'Lorenz curve',
      {
        scales: { x: { time: false, range: [0, 1] }, y: { range: [0, 1] } },
        axes,
        legend: { show: false },
        series: [
          { label: 'Population share' },
          { label: 'Equality', stroke: color('--muted'), dash: [4, 4], width: 1 },
          { label: 'Wealth share', stroke: color('--c2'), width: 2 },
        ],
      },
      [xs, xs, xs],
      (plot) => plot.setData([xs, xs, Array.from(this.engine.sim.lorenz(101))]),
    );

    const bars = uPlot.paths.bars!({ size: [0.9, 64] });
    this.add(
      'Wealth distribution',
      {
        scales: { x: { time: false } },
        axes,
        legend: { show: false },
        series: [{ label: 'Sugar' }, { label: 'Agents', fill: color('--c1'), stroke: color('--c1'), paths: bars, points: { show: false } }],
      },
      [[], []],
      (plot) => {
        const hist = Array.from(this.engine.sim.wealth_hist(20));
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );
  }
}
