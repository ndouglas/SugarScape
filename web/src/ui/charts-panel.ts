import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type { Engine } from '../engine';
import { MAX_GOODS } from '../goods';
import { CHART_POINTS, type ChartGroup, type Wants } from '../protocol';
import type { Config } from '../types';
import { h } from './dom';
import { compactNumber } from './format';
import {
  bandData,
  barsData,
  chartsBehind,
  distributionsDue,
  distributionWants,
  emptyTable,
  histTable,
  lineData,
  overlayData,
  positionBars,
  positionSteps,
  showsAgeHist,
  showsGoodWealth,
  showsTagHist,
  showsTotalWealth,
  supplyDemandTable,
  type DistState,
  type LineData,
} from './series-data';

interface Line { key: string; label: string; color: string }
type Section = 'top' | 'goods' | 'pollution' | 'economy' | 'disease';
type Kind = 'time' | 'band' | 'lorenz' | 'lorenzTotal' | 'wealth' | 'goodWealth' | 'age' | 'tags' | 'supplyDemand';

/** One chart: its lines follow a world's config; it shows when its section and `shown` hold for either world. */
interface ChartDef {
  title: string;
  kind: Kind;
  section: Section;
  lines?: (c: Config) => Line[];
  range?: [number, number];
  shown?: (c: Config) => boolean;
  /** The caption names the traded pair (goods 0 and 1). */
  pair?: boolean;
  /** A `goodWealth` chart's good: the caption names it. */
  good?: number;
}

const HEIGHT = 150;
/** The distributions (Lorenz curves, wealth, age and tag histograms, supply & demand) are fetched at most this often (per world). */
const REFRESH_MS = 250;
const POLLUTANT_COLORS = ['--c1', '--c2', '--c3', '--c4'];
/** The Trade price chart's series: the mean and its ± SD band share one x axis. */
const PRICE_GROUP = ['mean_log_price', 'sd_log_price'];
const LABELS = ['A', 'B'];
/** B's lines are dashed; A's are solid (Decision 11). */
const B_DASH = [6, 4];
const XS = Array.from({ length: 101 }, (_, i) => i / 100);
const X_LABEL: Record<Kind, string> = {
  time: 'Tick',
  band: 'Tick',
  lorenz: 'Population share',
  lorenzTotal: 'Population share',
  wealth: 'Sugar',
  goodWealth: 'Holding',
  age: 'Age',
  tags: 'Tag position',
  supplyDemand: 'Price',
};

const fixed = (lines: Line[]) => () => lines;
const perGood = (prefix: string) => (c: Config): Line[] =>
  c.goods.map((g, i) => ({ key: `${prefix}${i}`, label: g.name, color: g.color }));

const SECTIONS: { id: Section; title?: string; shown: (c: Config) => boolean }[] = [
  { id: 'top', shown: () => true },
  { id: 'goods', title: 'Goods', shown: () => true },
  { id: 'pollution', title: 'Pollution', shown: (c) => c.pollution.enabled },
  // Market charts need two goods; loan charts need credit; the section needs either.
  { id: 'economy', title: 'Economy', shown: (c) => showsTotalWealth(c) || c.credit.enabled },
  { id: 'disease', title: 'Disease', shown: (c) => c.disease.enabled },
];

const CHARTS: ChartDef[] = [
  { title: 'Population', kind: 'time', section: 'top', lines: fixed([{ key: 'population', label: 'Agents', color: '--c1' }]) },
  { title: 'Gini coefficient', kind: 'time', section: 'top', lines: fixed([{ key: 'gini', label: 'Gini', color: '--c2' }]), range: [0, 1] },
  {
    title: 'Mean traits',
    kind: 'time',
    section: 'top',
    lines: fixed([
      { key: 'mean_vision', label: 'Vision', color: '--c1' },
      { key: 'mean_metabolism', label: 'Metabolism', color: '--c3' },
    ]),
  },
  // Group shares sit where the Blue share chart was.
  {
    title: 'Group shares',
    kind: 'time',
    section: 'top',
    lines: (c) => c.culture.groups.map((g, k) => ({ key: `group_share_${k}`, label: g.name, color: g.color })),
    range: [0, 1],
  },
  {
    title: 'Births and deaths',
    kind: 'time',
    section: 'top',
    lines: fixed([
      { key: 'births', label: 'Births', color: '--c3' },
      { key: 'deaths', label: 'Deaths', color: '--c2' },
    ]),
  },
  { title: 'Lorenz curve', kind: 'lorenz', section: 'top' },
  { title: 'Wealth distribution', kind: 'wealth', section: 'top' },
  { title: 'Age histogram', kind: 'age', section: 'top', shown: showsAgeHist },
  { title: 'Cultural tags (% zeros by position)', kind: 'tags', section: 'top', shown: showsTagHist, range: [0, 100] },
  { title: 'Mean holdings', kind: 'time', section: 'goods', lines: perGood('mean_holding_') },
  { title: 'Mean metabolism', kind: 'time', section: 'goods', lines: perGood('mean_metabolism_') },
  { title: 'Units traded', kind: 'time', section: 'goods', lines: perGood('traded_'), shown: (c) => c.trade.enabled },
  {
    title: 'Gini coefficient (total wealth)',
    kind: 'time',
    section: 'goods',
    lines: fixed([{ key: 'gini_total', label: 'Gini', color: '--c2' }]),
    range: [0, 1],
    shown: showsTotalWealth,
  },
  { title: 'Lorenz curve (total wealth)', kind: 'lorenzTotal', section: 'goods', shown: showsTotalWealth },
  // One per possible good; each shows while some world has that good (and two or more goods).
  ...Array.from(
    { length: MAX_GOODS },
    (_, good): ChartDef => ({ title: 'Wealth distribution', kind: 'goodWealth', section: 'goods', good, shown: showsGoodWealth(good) }),
  ),
  {
    title: 'Mean pollution',
    kind: 'time',
    section: 'pollution',
    lines: (c) => c.pollution.pollutants.map((p, k) => ({ key: `mean_pollution_${k}`, label: p.name, color: POLLUTANT_COLORS[k] })),
    shown: (c) => c.pollution.enabled,
  },
  { title: 'Trade price (ln)', kind: 'band', section: 'economy', shown: showsTotalWealth, pair: true },
  { title: 'Trade volume', kind: 'time', section: 'economy', lines: fixed([{ key: 'trade_volume', label: 'Volume', color: '--c1' }]), shown: showsTotalWealth },
  { title: 'Supply & demand', kind: 'supplyDemand', section: 'economy', shown: showsTotalWealth, pair: true },
  {
    title: 'Loans',
    kind: 'time',
    section: 'economy',
    lines: fixed([
      { key: 'loans_made', label: 'Loans made', color: '--c1' },
      { key: 'defaults', label: 'Defaults', color: '--c2' },
    ]),
    shown: (c) => c.credit.enabled,
  },
  {
    title: 'Debt outstanding',
    kind: 'time',
    section: 'economy',
    lines: fixed([{ key: 'debt_outstanding', label: 'Debt', color: '--c3' }]),
    shown: (c) => c.credit.enabled,
  },
  {
    title: 'Foresight',
    kind: 'time',
    section: 'economy',
    lines: fixed([{ key: 'mean_foresight', label: 'Foresight φ', color: '--c3' }]),
    shown: (c) => c.foresight.enabled,
  },
  {
    title: 'Infected',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'infected_fraction', label: 'Infected share', color: '--red' }]),
    range: [0, 1],
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'Diseases per agent',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'mean_diseases', label: 'Mean', color: '--c2' }]),
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'Diseases in circulation',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'diseases_in_circulation', label: 'Distinct diseases', color: '--c4' }]),
    shown: (c) => c.disease.enabled,
  },
  {
    title: 'New infections',
    kind: 'time',
    section: 'disease',
    lines: fixed([{ key: 'new_infections', label: 'Infections', color: '--c1' }]),
    shown: (c) => c.disease.enabled,
  },
];

/** The host chart group a chart draws for a world (7a Decision 4); none for the distributions. */
function groupOf(def: ChartDef, c: Config): string[] {
  if (def.kind === 'band') return PRICE_GROUP;
  return def.kind === 'time' ? def.lines!(c).map((l) => l.key) : [];
}

interface Plot {
  def: ChartDef;
  plot: uPlot;
  figure: HTMLElement;
  caption: HTMLElement;
  /** Each world's group (time and band charts), by world index. */
  groups: string[][];
  /** Each world's line count, for an empty table until its group arrives. */
  counts: number[];
  /** Each world's group as last drawn (a new copy from the engine means redraw). */
  drawn: (ChartGroup | undefined)[];
  /** The distribution versions last drawn. */
  drawnDist: string;
}

/** A world's latest distributions, and when (and at which tick) they arrived. */
interface Dist extends DistState {
  lorenz: Float64Array | null;
  lorenzTotal: Float64Array | null;
  wealthHist: Float64Array | null;
  goodWealthHists: Float64Array[] | null;
  ageHist: Float64Array | null;
  tagHist: Float64Array | null;
  supplyDemand: Float64Array | null;
  version: number;
}

const freshDist = (): Dist => ({
  lorenz: null,
  lorenzTotal: null,
  wealthHist: null,
  goodWealthHists: null,
  ageHist: null,
  tagHist: null,
  supplyDemand: null,
  version: 0,
  at: -Infinity,
  tick: -1,
  stale: true,
});

/**
 * The Charts tab (Decision 11): one list of charts drawn for `worlds` — the playground's engine, and
 * Compare's B while comparing — on shared axes, A solid and B dashed.
 */
export class ChartsPanel {
  readonly el = h('div', { class: 'charts' });
  private worlds: Engine[];
  private dist: Dist[];
  private plots: Plot[] = [];
  private visible = false;
  /** The lines signature the plots were built for. */
  private built = '';
  private readonly sections = new Map<Section, HTMLElement>();
  private offB: (() => void)[] = [];
  private readonly color: (v: string) => string;
  private readonly axes: uPlot.Axis[];

  constructor(private readonly engine: Engine) {
    this.worlds = [engine];
    this.dist = [freshDist()];
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
    for (const s of SECTIONS) if (s.id !== 'top') this.sections.set(s.id, h('section', { class: s.id }));
    this.attach(engine, 0);
    this.sync();
    new ResizeObserver(() => this.resize()).observe(this.el);
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    // Nothing is fetched here: the charts keep what they drew while hidden, `wants` asks for
    // whatever has fallen behind since, and the frame loop sends that (and only that) while paused.
    if (visible) {
      this.resize();
      this.redraw();
    }
  }

  /** Compare on (B's lines join A's) or off. */
  setCompare(b: Engine | null): void {
    for (const off of this.offB) off();
    this.offB = [];
    this.worlds = b ? [this.engine, b] : [this.engine];
    this.dist = this.worlds.map(() => freshDist());
    if (b) this.offB = this.attach(b, 1);
    this.built = '';
    this.sync();
    this.redraw();
  }

  canvases(): { name: string; canvas: HTMLCanvasElement }[] {
    return this.plots.map((p) => ({ name: p.def.title, canvas: p.plot.ctx.canvas }));
  }

  /** World `i`'s provider and listeners; returns their removal. */
  private attach(w: Engine, i: number): (() => void)[] {
    return [
      w.want((now) => this.wants(i, now)),
      w.on('snapshot', () => this.receive(i)),
      // Edits, resets and config changes move the distributions without a tick.
      ...(['edit', 'reset', 'config'] as const).map((event) => w.on(event, () => (this.dist[i].stale = true))),
      ...(['reset', 'config'] as const).map((event) => w.on(event, () => this.sync())),
    ];
  }

  private shown(def: ChartDef): boolean {
    const section = SECTIONS.find((s) => s.id === def.section)!;
    return this.worlds.some((w) => section.shown(w.config) && (def.shown?.(w.config) ?? true));
  }

  /**
   * World `i`'s groups for the charts on show (only while the engine's copy of some group is missing
   * or behind the tick, so a paused, caught-up panel asks for nothing), and its distributions when
   * due; nothing while hidden.
   */
  private wants(i: number, now: number): Wants {
    const w = this.worlds[i];
    if (!this.visible || !w) return {};
    const groups = this.plots.filter((p) => !p.figure.hidden && p.groups[i].length > 0).map((p) => p.groups[i]);
    const out: Wants = {};
    if (groups.length > 0 && chartsBehind(groups, w.tick, (g) => w.chartGroup(g))) out.charts = { groups, max: CHART_POINTS };
    if (distributionsDue(this.dist[i], w.tick, now, REFRESH_MS)) Object.assign(out, distributionWants(w.config));
    return out;
  }

  /** Takes world `i`'s snapshot's distributions and redraws what changed. */
  private receive(i: number): void {
    const s = this.worlds[i]?.last;
    if (!s) return;
    // The host answers in order and computes these fresh, so no reply after an edit's own reply
    // can carry pre-edit data: clearing `stale` on any snapshot that has them is safe.
    if (s.lorenz) {
      const d = this.dist[i];
      d.lorenz = s.lorenz;
      d.wealthHist = s.wealthHist ?? d.wealthHist;
      // A world that drops to one good (or turns lifetimes or culture off) stops sending these:
      // clear them, not keep the last one.
      d.supplyDemand = s.supplyDemand ?? null;
      d.ageHist = s.ageHist ?? null;
      d.tagHist = s.tagHist ?? null;
      d.lorenzTotal = s.lorenzTotal ?? null;
      d.goodWealthHists = s.goodWealthHists ?? null;
      d.version++;
      d.tick = s.tick;
      d.at = performance.now();
      d.stale = false;
    }
    this.redraw();
  }

  /**
   * Rebuilds the plots when any world's chart lines changed (or Compare started or ended), then
   * shows the charts and sections either world would show and names the traded pair.
   */
  private sync(): void {
    const signature = JSON.stringify(this.worlds.map((w) => CHARTS.map((d) => d.lines?.(w.config) ?? null)));
    if (signature !== this.built) {
      this.built = signature;
      this.build();
    }
    const configs = this.worlds.map((w) => w.config);
    for (const s of SECTIONS) {
      const el = this.sections.get(s.id);
      if (el) el.hidden = !configs.some((c) => s.shown(c));
    }
    const goods = configs.find(showsTotalWealth)?.goods;
    for (const p of this.plots) {
      p.figure.hidden = !this.shown(p.def);
      const good = p.def.good;
      const named = good === undefined ? undefined : configs.find((c) => good < c.goods.length)?.goods[good];
      p.caption.textContent =
        p.def.pair && goods
          ? `${p.def.title} · ${goods[0].name}/${goods[1].name}`
          : named
            ? `${p.def.title} · ${named.name}`
            : p.def.title;
    }
    for (const d of this.dist) d.stale = true;
  }

  private build(): void {
    for (const p of this.plots) p.plot.destroy();
    this.plots = [];
    for (const [id, el] of this.sections) el.replaceChildren(h('h3', {}, SECTIONS.find((s) => s.id === id)!.title!));
    const top: HTMLElement[] = [];
    for (const def of CHARTS) {
      const figure = this.plotFor(def);
      if (def.section === 'top') top.push(figure);
      else this.sections.get(def.section)!.append(figure);
    }
    this.el.replaceChildren(...top, ...this.sections.values());
    this.resize();
  }

  private plotFor(def: ChartDef): HTMLElement {
    const caption = h('figcaption', {}, def.title);
    const figure = h('figure', { class: 'chart' }, caption);
    const groups = this.worlds.map((w) => groupOf(def, w.config));
    const counts = groups.map((g) => (def.kind === 'band' ? 3 : g.length));
    const data = def.kind === 'time' || def.kind === 'band' ? this.merge(counts.map(emptyTable)) : this.distData(def);
    const plot = new uPlot({ ...this.options(def), width: this.width(), height: HEIGHT }, data, figure);
    this.plots.push({ def, plot, figure, caption, groups, counts, drawn: this.worlds.map(() => undefined), drawnDist: '' });
    return figure;
  }

  private options(def: ChartDef): Omit<uPlot.Options, 'width' | 'height'> {
    const multi = this.worlds.length > 1;
    const series: uPlot.Series[] = [{ label: X_LABEL[def.kind] }];
    const lorenz = def.kind === 'lorenz' || def.kind === 'lorenzTotal';
    if (lorenz) series.push({ label: 'Equality', stroke: this.color('--muted'), dash: [4, 4], width: 1 });
    this.worlds.forEach((w, i) => series.push(...this.seriesFor(def, w.config, multi ? `${LABELS[i]} · ` : '', i === 1)));
    const x: uPlot.Scale = { time: false };
    if (lorenz) x.range = [0, 1];
    if (def.kind === 'supplyDemand') x.distr = 3;
    const y: uPlot.Scale = {};
    if (lorenz) y.range = [0, 1];
    else if (def.range) y.range = def.range;
    const lines = def.kind === 'time' ? def.lines!(this.engine.config).length : 0;
    const legend = multi || def.kind === 'band' || def.kind === 'supplyDemand' || lines > 1;
    return { scales: { x, y }, axes: this.axes, legend: { show: legend }, series };
  }

  /** One world's series: labelled "A · …"/"B · …" in Compare, B dashed and its points hollow. */
  private seriesFor(def: ChartDef, c: Config, tag: string, b: boolean): uPlot.Series[] {
    const dash = b ? B_DASH : undefined;
    switch (def.kind) {
      case 'time':
        return def.lines!(c).map((l) => ({ label: tag + l.label, stroke: this.color(l.color), width: 1.5, dash }));
      case 'band': {
        const sd = b ? [2, 3] : [4, 4];
        return [
          { label: `${tag}Mean`, stroke: this.color('--c2'), width: 1.5, dash },
          { label: `${tag}+SD`, stroke: this.color('--muted'), width: 1, dash: sd },
          { label: `${tag}-SD`, stroke: this.color('--muted'), width: 1, dash: sd },
        ];
      }
      case 'lorenz':
      case 'lorenzTotal':
        return [{ label: `${tag}Wealth share`, stroke: this.color('--c2'), width: 2, dash }];
      case 'goodWealth':
        return this.histSeries(`${tag}Agents`, c.goods[def.good!]?.color ?? '--c1', dash);
      case 'wealth':
      case 'age':
        return this.histSeries(`${tag}Agents`, '--c1', dash);
      case 'tags':
        return this.histSeries(`${tag}% zeros`, '--c4', dash);
      case 'supplyDemand': {
        const fill = b ? this.color('--surface') : undefined;
        return [
          { label: `${tag}Demand`, stroke: this.color('--c1'), width: 1.5, dash },
          { label: `${tag}Supply`, stroke: this.color('--c2'), width: 1.5, dash },
          { label: `${tag}Equilibrium`, stroke: this.color('--c3'), points: { show: true, size: 9, fill }, paths: () => null },
          { label: `${tag}Actual`, stroke: this.color('--text'), points: { show: true, size: 9, fill }, paths: () => null },
        ];
      }
    }
  }

  /** A histogram's series: bars for one world; in Compare a step outline per world (B dashed). */
  private histSeries(label: string, color: string, dash: number[] | undefined): uPlot.Series[] {
    const c = this.color(color);
    return this.worlds.length > 1
      ? [{ label, stroke: c, width: 1.5, dash, paths: uPlot.paths.stepped!({ align: 1 }), points: { show: false } }]
      : [{ label, fill: c, stroke: c, paths: uPlot.paths.bars!({ size: [0.9, 64] }), points: { show: false } }];
  }

  /** One table as is; several on the union of their x values (Decision 11). */
  private merge(tables: LineData[]): uPlot.AlignedData {
    return tables.length === 1 ? tables[0] : overlayData(tables);
  }

  private distData(def: ChartDef): uPlot.AlignedData {
    const curve = (l: Float64Array | null) => (l ? Array.from(l) : XS.map(() => null));
    const good = (d: Dist) => d.goodWealthHists?.[def.good!] ?? null;
    switch (def.kind) {
      case 'lorenz':
        return [XS, XS, ...this.dist.map((d) => curve(d.lorenz))] as uPlot.AlignedData;
      case 'lorenzTotal':
        return [XS, XS, ...this.dist.map((d) => curve(d.lorenzTotal))] as uPlot.AlignedData;
      case 'goodWealth':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => histTable(good(d)))) : barsData(good(this.dist[0]));
      case 'wealth':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => histTable(d.wealthHist))) : barsData(this.dist[0].wealthHist);
      case 'age':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => histTable(d.ageHist))) : barsData(this.dist[0].ageHist);
      case 'tags':
        return this.worlds.length > 1 ? overlayData(this.dist.map((d) => positionSteps(d.tagHist))) : positionBars(this.dist[0].tagHist);
      default:
        return this.merge(this.dist.map((d) => supplyDemandTable(d.supplyDemand)));
    }
  }

  /** Redraws each chart on show whose data changed: a new group copy, or new distributions. */
  private redraw(): void {
    if (!this.visible) return;
    for (const p of this.plots) if (!p.figure.hidden) this.draw(p);
  }

  private draw(p: Plot): void {
    if (p.def.kind === 'time' || p.def.kind === 'band') {
      const groups = this.worlds.map((w, i) => w.chartGroup(p.groups[i]));
      if (groups.every((g, i) => g === p.drawn[i])) return;
      p.drawn = groups;
      const band = p.def.kind === 'band';
      p.plot.setData(this.merge(groups.map((g, i) => (g ? (band ? bandData(g) : lineData(g)) : emptyTable(p.counts[i])))));
      return;
    }
    const version = this.dist.map((d) => d.version).join();
    if (version === p.drawnDist) return;
    p.drawnDist = version;
    p.plot.setData(this.distData(p.def));
  }

  private width(): number {
    return Math.max(240, this.el.clientWidth - 4);
  }

  private resize(): void {
    if (!this.visible) return;
    for (const p of this.plots) p.plot.setSize({ width: this.width(), height: HEIGHT });
  }
}
