import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { chartsSignature } from '../goods';
import { groupSharesSignature } from '../groups';
import { CHART_POINTS, type ChartGroup, type Wants, type WorldSnapshot } from '../protocol';
import { h } from './dom';
import { compactNumber } from './format';
import { bandData, chartsBehind, lineData, type LineData } from './series-data';

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
  {
    title: 'Births and deaths',
    lines: [
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ],
  },
];

const HEIGHT = 150;
/** The Lorenz curve, wealth histogram and supply & demand are fetched at most this often. */
const REFRESH_MS = 250;
const POLLUTANT_COLORS = ['--c1', '--c2', '--c3', '--c4'];
/** The Trade price chart's series: the mean and its ± SD band share one x axis. */
const PRICE_GROUP = ['mean_log_price', 'sd_log_price'];

export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private plots: {
    name: string;
    plot: uPlot;
    figure: HTMLElement;
    /** The host chart group a time chart draws (Decision 4). */
    group?: string[];
    update: (s: WorldSnapshot) => void;
    /** Hidden (and not redrawn) while false. */
    visible: () => boolean;
  }[] = [];
  private visible = false;
  /** When, and at which tick, the distributions last arrived; `distStale` asks for them regardless of the tick. */
  private distAt = -Infinity;
  private distTick = -1;
  private distStale = true;
  private color!: (v: string) => string;
  private axes!: uPlot.Axis[];
  private addTimeChart!: (chart: TimeChart, container?: HTMLElement, visible?: () => boolean) => void;
  private goodsSection = h('section', { class: 'goods' });
  private pollutionSection = h('section', { class: 'pollution' });
  /** Plots whose lines follow the goods and pollutants; rebuilt when `chartsSignature` changes. */
  private dynamic = new Set<uPlot>();
  /** Holds the Group shares chart, whose lines follow `culture.groups`. */
  private groupsSection = h('section', { class: 'group-shares' });
  private groupPlots = new Set<uPlot>();
  /** Captions that name the traded pair (0, 1). */
  private pairCaptions: { el: HTMLElement; title: string }[] = [];

  constructor(private engine: Engine) {
    this.build();
    engine.want((now) => this.wants(now));
    engine.on('snapshot', () => this.receive());
    // Edits, resets and config changes move the distributions without a tick.
    for (const event of ['edit', 'reset', 'config'] as const) engine.on(event, () => (this.distStale = true));
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    // Nothing is fetched here: the charts keep what they drew while hidden, `wants` asks for
    // whatever has fallen behind since, and the frame loop sends that (and only that) while paused.
    if (visible) {
      this.resize();
      this.receive();
    }
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.name, canvas: p.plot.ctx.canvas }));
  }

  private twoGoods(): boolean {
    return this.engine.config.goods.length >= 2;
  }

  /**
   * The visible time charts' groups (only while the engine's copy of some group is missing or
   * behind the tick, so a paused, caught-up panel asks for nothing), and the distributions when
   * due; nothing while hidden.
   */
  private wants(now: number): Wants {
    if (!this.visible) return {};
    const groups = this.plots.flatMap((p) => (p.group && p.visible() ? [p.group] : []));
    const w: Wants = {};
    if (chartsBehind(groups, this.engine.tick, (g) => this.engine.chartGroup(g))) w.charts = { groups, max: CHART_POINTS };
    if ((this.distStale || this.engine.tick !== this.distTick) && now - this.distAt >= REFRESH_MS) {
      w.lorenz = true;
      w.wealthHist = true;
      if (this.twoGoods()) w.supplyDemand = true;
    }
    return w;
  }

  /** Draws whatever the latest snapshot brought. */
  private receive(): void {
    const s = this.engine.last;
    if (!s) return;
    // Safe even though this clears distStale for whichever snapshot happens to carry lorenz, not
    // necessarily the one requested right after the edit that set it: the host answers requests
    // strictly in the order they were sent and computes lorenz fresh (no caching) from whatever
    // world state exists at that moment, so no reply that arrives after an edit's own reply can
    // carry pre-edit data — the edit is always applied to the host's world before anything sent
    // after it is even processed.
    if (s.lorenz) {
      this.distTick = s.tick;
      this.distAt = performance.now();
      this.distStale = false;
    }
    for (const p of this.plots) if (p.visible()) p.update(s);
  }

  /**
   * Draws a time chart from the engine's latest copy of its group (Decision 4), redrawing only
   * when a new copy has arrived: the snapshot that brought it may be long gone (the chart was
   * hidden, or rebuilt after a config change).
   */
  private groupDrawer(names: string[], data: (g: ChartGroup) => LineData): (plot: uPlot) => void {
    let drawn: ChartGroup | undefined;
    return (plot) => {
      const g = this.engine.chartGroup(names);
      if (!g || g === drawn) return;
      drawn = g;
      plot.setData(data(g));
    };
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

  private rebuildGroupChart(): void {
    this.plots = this.plots.filter((p) => {
      if (!this.groupPlots.has(p.plot)) return true;
      p.plot.destroy();
      return false;
    });
    this.groupPlots.clear();
    this.groupsSection.replaceChildren();
    const before = this.plots.length;
    const lines = this.engine.config.culture.groups.map((g, k) => ({ key: `group_share_${k}`, label: g.name, color: g.color }));
    this.addTimeChart({ title: 'Group shares', lines, range: [0, 1] }, this.groupsSection);
    for (const p of this.plots.slice(before)) this.groupPlots.add(p.plot);
    this.resize();
  }

  private add(
    title: string,
    opts: Omit<uPlot.Options, 'width' | 'height'>,
    data: uPlot.AlignedData,
    update: (plot: uPlot, s: WorldSnapshot) => void,
    container: HTMLElement = this.el,
    visible: () => boolean = () => true,
    group?: string[],
  ): HTMLElement {
    const figcaption = h('figcaption', {}, title);
    const figure = h('figure', { class: 'chart' }, figcaption);
    const plot = new uPlot({ ...opts, width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ name: title, plot, figure, group, update: (s) => update(plot, s), visible });
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
      const group = chart.lines.map((l) => l.key);
      this.add(
        chart.title,
        {
          scales: { x: { time: false }, y: chart.range ? { range: chart.range } : {} },
          axes: this.axes,
          legend: { show: chart.lines.length > 1 },
          series: [{ label: 'Tick' }, ...chart.lines.map((l) => ({ label: l.label, stroke: this.color(l.color), width: 1.5 }))],
        },
        [[], ...chart.lines.map(() => [])],
        this.groupDrawer(group, lineData),
        container,
        visible,
        group,
      );
    };

    for (const chart of TIME_CHARTS) {
      this.addTimeChart(chart);
      // Group shares sit where the Blue share chart was.
      if (chart.title === 'Mean traits') this.el.append(this.groupsSection);
    }
    let groupLines = '';
    const syncGroupChart = () => {
      const next = groupSharesSignature(this.engine.config);
      if (next === groupLines) return;
      groupLines = next;
      this.rebuildGroupChart();
    };
    this.engine.on('reset', syncGroupChart);
    this.engine.on('config', syncGroupChart);
    syncGroupChart();

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
      (plot, s) => {
        if (s.lorenz) plot.setData([xs, xs, s.lorenz]);
      },
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
      (plot, s) => {
        const hist = s.wealthHist;
        if (!hist) return;
        const width = hist[0];
        const counts = hist.slice(1);
        plot.setData([counts.map((_, i) => (i + 0.5) * width), counts]);
      },
    );

    // Market charts need two goods; loan charts need credit; the section needs
    // either. The disease section needs disease.
    const economy = h('section', { class: 'economy' }, h('h3', {}, 'Economy'));
    const disease = h('section', { class: 'disease' }, h('h3', {}, 'Disease'));
    this.el.append(this.goodsSection, this.pollutionSection, economy, disease);
    const twoGoods = () => this.twoGoods();
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
      this.distStale = true;
    };
    // Rebuilt only when the goods' or pollutants' lines change, not on every config event.
    let lines = '';
    const syncGoodsCharts = () => {
      const next = chartsSignature(this.engine.config);
      if (next === lines) return;
      lines = next;
      this.rebuildGoodsCharts();
    };
    this.engine.on('reset', syncGoodsCharts);
    this.engine.on('config', syncGoodsCharts);
    syncGoodsCharts();
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
      this.groupDrawer(PRICE_GROUP, bandData),
      economy,
      twoGoods,
      PRICE_GROUP,
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
      (plot, s) => {
        const sd = s.supplyDemand;
        if (!sd) return;
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
