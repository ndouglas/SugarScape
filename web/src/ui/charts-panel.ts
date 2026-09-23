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
type SeriesCache = (name: string) => Float64Array;

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: { name: string; plot: uPlot; update: (series: SeriesCache) => void; visible: () => boolean }[] = [];
  private visible = false;
  private last = 0;
  /** Tick drawn by the last refresh; null forces the next one. */
  private drawnTick: number | null = null;

  constructor(private engine: Engine) {
    this.build();
    engine.on('reset', () => this.refresh(true));
    // Edits change wealth without a tick; redraw on the next throttled refresh.
    engine.on('edit', () => (this.drawnTick = null));
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.resize();
      this.refresh(true);
    }
  }

  /** Called every animation frame; redraws at most every REFRESH_MS while visible. */
  maybeRefresh(now: number): void {
    if (this.visible && now - this.last > REFRESH_MS) this.refresh();
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.name, canvas: p.plot.ctx.canvas }));
  }

  /** Redraws unless the tick is unchanged since the last draw (`force` overrides). */
  private refresh(force = false): void {
    this.last = performance.now();
    if (!this.visible) {
      this.drawnTick = null;
      return;
    }
    const tick = this.engine.sim.tick();
    if (!force && tick === this.drawnTick) return;
    this.drawnTick = tick;
    const cache = new Map<string, Float64Array>();
    const series: SeriesCache = (name) => {
      let arr = cache.get(name);
      if (!arr) {
        arr = this.engine.sim.series(name);
        cache.set(name, arr);
      }
      return arr;
    };
    this.plots.forEach((p) => {
      if (p.visible()) p.update(series);
    });
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
    container: HTMLElement = this.el,
    visible: () => boolean = () => true,
  ): void {
    const figure = h('figure', { class: 'chart' }, h('figcaption', {}, title));
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, update: (series) => update(plot, series), visible });
    container.append(figure);
  }

  private build(): void {
    const css = getComputedStyle(document.documentElement);
    const color = (v: string) => css.getPropertyValue(v).trim() || '#888';
    const axes: uPlot.Axis[] = [
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') } },
      { stroke: color('--muted'), grid: { stroke: color('--grid') }, ticks: { stroke: color('--grid') }, size: 44 },
    ];

    const addTimeChart = (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => {
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
        container,
        visible,
      );
    };

    for (const chart of TIME_CHARTS) addTimeChart(chart);

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
      (plot) => plot.setData([xs, xs, this.engine.sim.lorenz(101)]),
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
        const hist = this.engine.sim.wealth_hist(20);
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );

    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    this.el.append(economy);
    const econOn = () => this.engine.config.spice.enabled || this.engine.config.credit.enabled;
    const syncSection = () => {
      economy.hidden = !econOn();
      this.drawnTick = null;
    };
    this.engine.on('reset', syncSection);
    this.engine.on('config', syncSection);
    syncSection();

    this.add(
      'Trade price (ln)',
      {
        scales: { x: { time: false }, y: {} },
        axes,
        legend: { show: true },
        series: [
          { label: 'Tick' },
          { label: 'Mean', stroke: color('--c2'), width: 1.5 },
          { label: '+SD', stroke: color('--muted'), width: 1, dash: [4, 4] },
          { label: '-SD', stroke: color('--muted'), width: 1, dash: [4, 4] },
        ],
      },
      [[], [], [], []],
      (plot, series) => {
        const m = series('mean_log_price');
        const sd = series('sd_log_price');
        plot.setData([series('tick'), m, m.map((v, i) => v + sd[i]), m.map((v, i) => v - sd[i])]);
      },
      economy,
      econOn,
    );

    addTimeChart({ title: 'Trade volume', lines: [{ key: 'trade_volume', label: 'Volume', color: '--c1' }] }, economy, econOn);

    this.add(
      'Supply & demand',
      {
        scales: { x: { time: false, distr: 3 }, y: {} },
        axes,
        legend: { show: true },
        series: [
          { label: 'Price' },
          { label: 'Demand', stroke: color('--c1'), width: 1.5 },
          { label: 'Supply', stroke: color('--c2'), width: 1.5 },
          { label: 'Equilibrium', stroke: color('--c3'), points: { show: true, size: 9 }, paths: () => null },
          { label: 'Actual', stroke: color('--text'), points: { show: true, size: 9 }, paths: () => null },
        ],
      },
      [[], [], [], [], []],
      (plot) => {
        const sd = this.engine.sim.supply_demand();
        const n = sd[0];
        const prices = Array.from(sd.subarray(1, 1 + n));
        const demand = Array.from(sd.subarray(1 + n, 1 + 2 * n));
        const supply = Array.from(sd.subarray(1 + 2 * n, 1 + 3 * n));
        const [eqP, eqQ, actP, actQ] = Array.from(sd.subarray(1 + 3 * n));
        const nearest = (p: number) =>
          prices.reduce((best, q, i) => (Math.abs(Math.log(q / p)) < Math.abs(Math.log(prices[best] / p)) ? i : best), 0);
        const point = (p: number, q: number) => {
          const col: (number | null)[] = prices.map(() => null);
          if (Number.isFinite(p) && Number.isFinite(q)) col[nearest(p)] = q;
          return col;
        };
        plot.setData([prices, demand, supply, point(eqP, eqQ), point(actP, actQ)]);
      },
      economy,
      econOn,
    );

    addTimeChart(
      {
        title: 'Loans',
        lines: [
          { key: 'loans_made', label: 'Loans made', color: '--c1' },
          { key: 'defaults', label: 'Defaults', color: '--c2' },
        ],
      },
      economy,
      econOn,
    );

    addTimeChart({ title: 'Debt outstanding', lines: [{ key: 'debt_outstanding', label: 'Debt', color: '--c3' }] }, economy, econOn);

    addTimeChart(
      {
        title: 'Spice & foresight',
        lines: [
          { key: 'mean_spice_metabolism', label: 'Spice metabolism', color: '--c1' },
          { key: 'mean_foresight', label: 'Foresight', color: '--c3' },
        ],
      },
      economy,
      econOn,
    );
  }
}
