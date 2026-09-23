import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { h } from './dom';
import { compactNumber } from './format';

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
const POLLUTANT_COLORS = ['--c1', '--c2', '--c3', '--c4'];

/** Fetches `Sim.series(name)` at most once per refresh; shared across all charts' update closures. */
type SeriesCache = (name: string) => Float64Array;

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: {
    name: string;
    plot: uPlot;
    figure: HTMLElement;
    update: (series: SeriesCache) => void;
    /** Hidden (and not redrawn) while false. */
    visible: () => boolean;
  }[] = [];
  private visible = false;
  private last = 0;
  /** Tick drawn by the last refresh; null forces the next one. */
  private drawnTick: number | null = null;
  private color!: (v: string) => string;
  private axes!: uPlot.Axis[];
  private addTimeChart!: (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => void;
  private goodsSection = h('section', { class: 'goods' });
  private pollutionSection = h('section', { class: 'pollution' });
  /** Plots whose lines follow the goods and pollutants; rebuilt on reset and config. */
  private dynamic = new Set<uPlot>();
  /** Captions that name the traded pair (0, 1). */
  private pairCaptions: { el: HTMLElement; title: string }[] = [];

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

  private rebuildGoodsCharts(): void {
    this.plots = this.plots.filter((p) => {
      if (!this.dynamic.has(p.plot)) return true;
      p.plot.destroy();
      return false;
    });
    this.dynamic.clear();
    this.goodsSection.replaceChildren(h('h3', {}, 'Goods'));
    this.pollutionSection.replaceChildren(h('h3', {}, 'Pollution'));
    const c = this.engine.config;
    const perGood = (prefix: string) => c.goods.map((g, i) => ({ key: `${prefix}${i}`, label: g.name, color: g.color }));
    const before = this.plots.length;
    this.addTimeChart({ title: 'Mean holdings', lines: perGood('mean_holding_') }, this.goodsSection);
    this.addTimeChart({ title: 'Mean metabolism', lines: perGood('mean_metabolism_') }, this.goodsSection);
    this.addTimeChart({ title: 'Units traded', lines: perGood('traded_') }, this.goodsSection, () => this.engine.config.trade.enabled);
    this.addTimeChart(
      {
        title: 'Mean pollution',
        lines: c.pollution.pollutants.map((p, k) => ({ key: `mean_pollution_${k}`, label: p.name, color: POLLUTANT_COLORS[k] })),
      },
      this.pollutionSection,
      () => this.engine.config.pollution.enabled,
    );
    for (const p of this.plots.slice(before)) this.dynamic.add(p.plot);
    this.resize();
  }

  private add(
    title: string,
    opts: Omit<uPlot.Options, 'width' | 'height'>,
    data: uPlot.AlignedData,
    update: (plot: uPlot, series: SeriesCache) => void,
    container: HTMLElement = this.el,
    visible: () => boolean = () => true,
  ): HTMLElement {
    const figcaption = h('figcaption', {}, title);
    const figure = h('figure', { class: 'chart' }, figcaption);
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, figure, update: (series) => update(plot, series), visible });
    container.append(figure);
    return figcaption;
  }

  private build(): void {
    const css = getComputedStyle(document.documentElement);
    this.color = (v: string) => (v.startsWith('#') ? v : css.getPropertyValue(v).trim() || '#888');
    this.axes = [
      { stroke: this.color('--muted'), grid: { stroke: this.color('--grid') }, ticks: { stroke: this.color('--grid') } },
      {
        stroke: this.color('--muted'),
        grid: { stroke: this.color('--grid') },
        ticks: { stroke: this.color('--grid') },
        size: 44,
        values: (_self, splits) => splits.map(compactNumber),
      },
    ];

    this.addTimeChart = (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => {
      this.add(
        chart.title,
        {
          scales: { x: { time: false }, y: chart.range ? { range: chart.range } : {} },
          axes: this.axes,
          legend: { show: chart.lines.length > 1 },
          series: [{ label: 'Tick' }, ...chart.lines.map((l) => ({ label: l.label, stroke: this.color(l.color), width: 1.5 }))],
        },
        [[], ...chart.lines.map(() => [])],
        (plot, series) => plot.setData([series('tick'), ...chart.lines.map((l) => series(l.key))]),
        container,
        visible,
      );
    };

    for (const chart of TIME_CHARTS) this.addTimeChart(chart);

    const xs = Array.from({ length: 101 }, (_, i) => i / 100);
    this.add(
      'Lorenz curve',
      {
        scales: { x: { time: false, range: [0, 1] }, y: { range: [0, 1] } },
        axes: this.axes,
        legend: { show: false },
        series: [
          { label: 'Population share' },
          { label: 'Equality', stroke: this.color('--muted'), dash: [4, 4], width: 1 },
          { label: 'Wealth share', stroke: this.color('--c2'), width: 2 },
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
        axes: this.axes,
        legend: { show: false },
        series: [{ label: 'Sugar' }, { label: 'Agents', fill: this.color('--c1'), stroke: this.color('--c1'), paths: bars, points: { show: false } }],
      },
      [[], []],
      (plot) => {
        const hist = this.engine.sim.wealth_hist(20);
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );

    // Market charts need spice; loan charts need credit; the section needs
    // either. The disease section needs disease.
    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    const disease = h('section', { class: 'disease' }, h('h3', {}, 'Disease'));
    this.el.append(this.goodsSection, this.pollutionSection, economy, disease);
    const twoGoods = () => this.engine.config.goods.length >= 2;
    const creditOn = () => this.engine.config.credit.enabled;
    const diseaseOn = () => this.engine.config.disease.enabled;
    const syncSection = () => {
      economy.hidden = !(twoGoods() || creditOn());
      disease.hidden = !diseaseOn();
      this.pollutionSection.hidden = !this.engine.config.pollution.enabled;
      const g = this.engine.config.goods;
      const pair = g.length >= 2 ? ` · ${g[0].name}/${g[1].name}` : '';
      for (const p of this.pairCaptions) p.el.textContent = p.title + pair;
      for (const p of this.plots) p.figure.hidden = !p.visible();
      this.drawnTick = null;
    };
    this.engine.on('reset', () => this.rebuildGoodsCharts());
    this.engine.on('config', () => this.rebuildGoodsCharts());
    this.rebuildGoodsCharts();
    this.engine.on('reset', syncSection);
    this.engine.on('config', syncSection);

    const priceCaption = this.add(
      'Trade price (ln)',
      {
        scales: { x: { time: false }, y: {} },
        axes: this.axes,
        legend: { show: true },
        series: [
          { label: 'Tick' },
          { label: 'Mean', stroke: this.color('--c2'), width: 1.5 },
          { label: '+SD', stroke: this.color('--muted'), width: 1, dash: [4, 4] },
          { label: '-SD', stroke: this.color('--muted'), width: 1, dash: [4, 4] },
        ],
      },
      [[], [], [], []],
      (plot, series) => {
        const m = series('mean_log_price');
        const sd = series('sd_log_price');
        plot.setData([series('tick'), m, m.map((v, i) => v + sd[i]), m.map((v, i) => v - sd[i])]);
      },
      economy,
      twoGoods,
    );
    this.pairCaptions.push({ el: priceCaption, title: 'Trade price (ln)' });

    this.addTimeChart({ title: 'Trade volume', lines: [{ key: 'trade_volume', label: 'Volume', color: '--c1' }] }, economy, twoGoods);

    const sdCaption = this.add(
      'Supply & demand',
      {
        scales: { x: { time: false, distr: 3 }, y: {} },
        axes: this.axes,
        legend: { show: true },
        series: [
          { label: 'Price' },
          { label: 'Demand', stroke: this.color('--c1'), width: 1.5 },
          { label: 'Supply', stroke: this.color('--c2'), width: 1.5 },
          { label: 'Equilibrium', stroke: this.color('--c3'), points: { show: true, size: 9 }, paths: () => null },
          { label: 'Actual', stroke: this.color('--text'), points: { show: true, size: 9 }, paths: () => null },
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
      twoGoods,
    );
    this.pairCaptions.push({ el: sdCaption, title: 'Supply & demand' });

    this.addTimeChart(
      {
        title: 'Loans',
        lines: [
          { key: 'loans_made', label: 'Loans made', color: '--c1' },
          { key: 'defaults', label: 'Defaults', color: '--c2' },
        ],
      },
      economy,
      creditOn,
    );

    this.addTimeChart({ title: 'Debt outstanding', lines: [{ key: 'debt_outstanding', label: 'Debt', color: '--c3' }] }, economy, creditOn);

    this.addTimeChart({ title: 'Foresight', lines: [{ key: 'mean_foresight', label: 'Foresight φ', color: '--c3' }] }, economy, () => this.engine.config.foresight.enabled);

    this.addTimeChart(
      { title: 'Infected', lines: [{ key: 'infected_fraction', label: 'Infected share', color: '--red' }], range: [0, 1] },
      disease,
      diseaseOn,
    );
    this.addTimeChart({ title: 'Diseases per agent', lines: [{ key: 'mean_diseases', label: 'Mean', color: '--c2' }] }, disease, diseaseOn);
    this.addTimeChart(
      { title: 'Diseases in circulation', lines: [{ key: 'diseases_in_circulation', label: 'Distinct diseases', color: '--c4' }] },
      disease,
      diseaseOn,
    );
    this.addTimeChart({ title: 'New infections', lines: [{ key: 'new_infections', label: 'Infections', color: '--c1' }] }, disease, diseaseOn);

    syncSection();
  }
}
