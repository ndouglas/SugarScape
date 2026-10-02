// The legend under the map: the active color mode's colors and the overlay symbols on screen.
import type { CachingRule, ColorMode, Config, LabView } from '../types';
import { h } from './dom';

/**
 * The core renderer's colors (crates/sugarscape-core/src/render.rs), as `#rrggbb`. legend.test.ts
 * reads render.rs and checks each one, so the two can't drift apart.
 */
export const RENDER_COLORS = {
  BLUE: '#3d7eff',
  RED: '#ff4d4d',
  FEMALE: '#ff7ac6',
  MALE: '#36d6c3',
  COOL: '#4f9dff',
  HOT: '#ff3d8b',
  LENDER: '#3dd66b',
  BORROWER: '#ff4d4d',
  BOTH: '#ffe04d',
  NEUTRAL: '#8a867a',
  SICK: '#ff4d4d',
  HEALTHY: '#3d7eff',
  FOUNDER: '#5a5a5a',
  FOUNDER_PARENT: '#ff4d4d',
  BORN: '#3dd66b',
  BORN_PARENT: '#ffe04d',
  WALL: '#5a554c',
  FENCE: '#8a6d3b',
  HOARDER: '#3d7eff',
  CHEATER: '#ff4d4d',
  RULE_EVEN: '#3d7eff',
  RULE_COMPENSATE: '#36d6c3',
  RULE_PLAN: '#ff3d8b',
  REMEMBERS: '#36d6c3',
} as const;

/**
 * Minds 7's frame colors (crates/sugarscape-core/src/hoard/view.rs), as `#rrggbb`; legend.test.ts
 * checks each against view.rs.
 */
export const HOARD_COLORS = {
  DEAD: '#3a3630',
  CHEATER: '#b06ad8',
  DEFENDING: '#4a7cd8',
  RAIDING: '#e03c31',
  FED: '#3ca85a',
  HUNGRY: '#f29a3a',
  LARDER: '#f2c14e',
  SCATTER: '#7ab05a',
  LOW: '#2a2620',
  HIGH: '#6ad8f6',
} as const;

/** The hoard frame's one-line key: what a column is. */
export const HOARD_STATUS =
  'One column per agent, #1 at the left. Top: its state. Then its L and D strips. From the midline: its larder items up and its scattered items down, one cell an item (up to 40).';

/** Minds 7's legend: the state strip's colors (in the renderer's order of precedence), the trait strips and the stores. */
export function hoardLegend(cheaters: boolean): LegendItem[] {
  const c = HOARD_COLORS;
  return [
    swatch('hungry', c.HUNGRY),
    swatch('fed', c.FED),
    swatch('defending its larder', c.DEFENDING),
    swatch('raiding a larder', c.RAIDING),
    ...(cheaters ? [swatch('cheater', c.CHEATER)] : []),
    swatch('dead', c.DEAD),
    { label: 'L and D strips, 0 → 1', mark: { kind: 'ramp', from: c.LOW, to: c.HIGH } },
    swatch('larder items (up)', c.LARDER),
    swatch('scattered items (down)', c.SCATTER),
  ];
}
/** The overlay symbols the legend can show; the grid view draws each the same way. */
export type LegendSymbol = 'cache' | 'cheater-cache' | 'own-cache' | 'home' | 'larder' | 'memory' | 'spot' | 'path' | 'route' | 'tray' | 'turn';

export type LegendMark =
  | { kind: 'swatch'; color: string }
  | { kind: 'ramp'; from: string; to: string }
  | { kind: 'symbol'; symbol: LegendSymbol };

export interface LegendItem {
  label: string;
  mark: LegendMark;
}

const swatch = (label: string, color: string): LegendItem => ({ label, mark: { kind: 'swatch', color } });
const ramp = (label: string): LegendItem => ({ label, mark: { kind: 'ramp', from: RENDER_COLORS.COOL, to: RENDER_COLORS.HOT } });
const symbol = (label: string, s: LegendSymbol): LegendItem => ({ label, mark: { kind: 'symbol', symbol: s } });

/** A sugarscape color mode's legend entries, in the renderer's terms (render.rs `agent_color`). */
export function colorLegend(mode: ColorMode, config: Config): LegendItem[] {
  const c = RENDER_COLORS;
  switch (mode) {
    case 'tribe':
      return config.culture.groups.map((g) => swatch(g.name, g.color));
    case 'wealth':
      return [ramp('sugar held, low → high (log)')];
    case 'age':
      return [ramp('age, young → old (of its maximum)')];
    case 'vision':
      return [ramp(`vision ${config.vision.min} → ${config.vision.max}`)];
    case 'sex':
      return [swatch('female', c.FEMALE), swatch('male', c.MALE)];
    case 'credit':
      return [swatch('lender', c.LENDER), swatch('borrower', c.BORROWER), swatch('both', c.BOTH), swatch('neither', c.NEUTRAL)];
    case 'disease':
      return [swatch('sick', c.SICK), swatch('healthy', c.HEALTHY)];
    case 'lineage':
      return [swatch('founder', c.FOUNDER), swatch('founder, a parent', c.FOUNDER_PARENT), swatch('born', c.BORN), swatch('born, a parent', c.BORN_PARENT)];
    case 'culture':
      return [{ label: 'a color per culture', mark: { kind: 'swatch', color: c.NEUTRAL } }];
    case 'strategy':
      return (config.theft?.cheaters ?? 0) > 0 ? [swatch('hoarder', c.HOARDER), swatch('cheater', c.CHEATER)] : [swatch('hoarder', c.HOARDER)];
    case 'caching_rule':
      return cachingRuleLegend(config);
    case 'watching':
      return [swatch('watcher who buries', c.HOARDER), swatch('scrounger (watches, never buries)', c.CHEATER), swatch('does not watch', c.NEUTRAL)];
    case 'memory':
      return [swatch('remembers', c.REMEMBERS), swatch("doesn't remember", c.NEUTRAL)];
    default:
      return [];
  }
}

const RULE_COLORS: Record<CachingRule, string> = {
  none: RENDER_COLORS.NEUTRAL,
  even: RENDER_COLORS.RULE_EVEN,
  compensate: RENDER_COLORS.RULE_COMPENSATE,
  plan: RENDER_COLORS.RULE_PLAN,
};

/**
 * The rules agents actually follow: all four under mixed rules, else the config's one; a cheater
 * follows `none`, so it joins the list where there are cheaters.
 */
function cachingRuleLegend(config: Config): LegendItem[] {
  const rules: CachingRule[] = config.caching?.mixed ? ['none', 'even', 'compensate', 'plan'] : [config.caching?.rule ?? 'none'];
  const cheaters = (config.theft?.cheaters ?? 0) > 0;
  if (cheaters && !rules.includes('none')) rules.unshift('none');
  return rules.map((r) => swatch(r === 'none' && cheaters ? (config.caching?.mixed ? 'none (and cheaters)' : 'none (cheaters)') : r, RULE_COLORS[r]));
}

/** What is on the map besides agents, as the grid view draws it (`GridView.marks`). */
export interface MapMarks {
  /** The caches overlay shows some cache, and some site's caches are only cheaters' (tinted apart). */
  allCaches: boolean;
  cheaterCaches: boolean;
  /** The selected agent's own caches (highlighted). */
  ownCaches: boolean;
  /** A central world's homes (with the caches overlay), and whether any larder holds sugar. */
  homes: boolean;
  larders: boolean;
  /** The selected agent's remembered sites, and whether any is a known truffle spot. */
  memory: boolean;
  spots: boolean;
  /** The selected agent's walk (A*) and GOAP route. */
  path: boolean;
  route: boolean;
  /** A lab: the trays, and whose turn it is on the test evening. */
  lab: LabView | null;
}

/** The overlay symbols on screen, and the walls (and fences) where there are any. */
export function overlayLegend(m: MapMarks, config: Config): LegendItem[] {
  const out: LegendItem[] = [];
  const walls = config.walls ?? [];
  if (walls.some((w) => w.opaque)) out.push(swatch('wall', RENDER_COLORS.WALL));
  if (walls.some((w) => !w.opaque)) out.push(swatch('fence', RENDER_COLORS.FENCE));
  if (m.allCaches) {
    out.push(symbol(m.cheaterCaches ? "a hoarder's cache (size: sugar)" : 'a cache (size: sugar)', 'cache'));
    if (m.cheaterCaches) out.push(symbol("a cheater's cache", 'cheater-cache'));
  }
  if (m.ownCaches) out.push(symbol("the selected agent's caches", 'own-cache'));
  if (m.homes) out.push(symbol('home', 'home'));
  if (m.homes && m.larders) out.push(symbol('larder (size: sugar)', 'larder'));
  if (m.memory) out.push(symbol('remembered site (fades with age)', 'memory'));
  if (m.memory && m.spots) out.push(symbol('known truffle spot (filled: ripe)', 'spot'));
  if (m.path) out.push(symbol('planned walk', 'path'));
  if (m.route) out.push(symbol('GOAP route and targets', 'route'));
  if (m.lab) {
    out.push(symbol('caching tray', 'tray'));
    if (m.lab.phase === 'test') out.push(symbol("whose turn it is", 'turn'));
  }
  return out;
}

const SVG = 'http://www.w3.org/2000/svg';

/** A 14 × 14 SVG of `mark`, colored as the grid view draws it (CSS variables for overlay colors). */
function markSvg(mark: LegendMark): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('width', '14');
  svg.setAttribute('height', '14');
  svg.setAttribute('viewBox', '0 0 14 14');
  svg.setAttribute('aria-hidden', 'true');
  const add = (tag: string, attrs: Record<string, string>) => {
    const el = document.createElementNS(SVG, tag);
    for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v);
    svg.append(el);
  };
  const diamond = '7,2 12,7 7,12 2,7';
  if (mark.kind === 'swatch') add('rect', { x: '1', y: '1', width: '12', height: '12', rx: '2', fill: mark.color });
  else if (mark.kind === 'ramp') {
    const id = `ramp-${mark.from.slice(1)}-${mark.to.slice(1)}`;
    const defs = document.createElementNS(SVG, 'defs');
    const grad = document.createElementNS(SVG, 'linearGradient');
    grad.setAttribute('id', id);
    for (const [offset, color] of [['0', mark.from], ['1', mark.to]]) {
      const stop = document.createElementNS(SVG, 'stop');
      stop.setAttribute('offset', offset);
      stop.setAttribute('stop-color', color);
      grad.append(stop);
    }
    defs.append(grad);
    svg.append(defs);
    svg.setAttribute('width', '28');
    svg.setAttribute('viewBox', '0 0 28 14');
    add('rect', { x: '1', y: '1', width: '26', height: '12', rx: '2', fill: `url(#${id})` });
  } else {
    switch (mark.symbol) {
      case 'cache':
        add('polygon', { points: diamond, style: 'fill: var(--c4)', stroke: '#000', 'stroke-width': '1', opacity: '0.75' });
        break;
      case 'cheater-cache':
        add('polygon', { points: diamond, style: 'fill: var(--red)', stroke: '#000', 'stroke-width': '1', opacity: '0.75' });
        break;
      case 'own-cache':
        add('polygon', { points: diamond, style: 'fill: var(--c4); stroke: var(--accent)', 'stroke-width': '2' });
        break;
      case 'home':
        add('rect', { x: '2', y: '2', width: '10', height: '10', fill: 'none', style: 'stroke: var(--c3)', 'stroke-width': '1.5' });
        break;
      case 'larder':
        add('polygon', { points: diamond, style: 'fill: var(--c3)', stroke: '#000', 'stroke-width': '1' });
        break;
      case 'memory':
        add('rect', { x: '4', y: '4', width: '6', height: '6', style: 'fill: var(--c2)', opacity: '0.6' });
        break;
      case 'spot':
        add('circle', { cx: '7', cy: '7', r: '3.5', style: 'fill: var(--accent); stroke: var(--accent)', 'stroke-width': '1.5' });
        break;
      case 'path':
        add('line', { x1: '1', y1: '7', x2: '13', y2: '7', style: 'stroke: var(--accent)', 'stroke-width': '2', 'stroke-dasharray': '3 2' });
        break;
      case 'route':
        add('line', { x1: '1', y1: '7', x2: '9', y2: '7', style: 'stroke: var(--accent)', 'stroke-width': '2', 'stroke-dasharray': '1.5 3' });
        add('circle', { cx: '10', cy: '7', r: '3', fill: 'none', style: 'stroke: var(--accent)', 'stroke-width': '1.5' });
        break;
      case 'tray':
        add('rect', { x: '2', y: '2', width: '10', height: '10', fill: 'none', style: 'stroke: var(--c3)', 'stroke-width': '1.5', 'stroke-dasharray': '2 1.5' });
        break;
      case 'turn':
        add('circle', { cx: '7', cy: '7', r: '5', fill: 'none', style: 'stroke: var(--accent)', 'stroke-width': '2' });
        break;
    }
  }
  return svg;
}

/** The legend's element: one line of entries, each a mark and its label. Empty items hide it. */
export function legendElement(items: LegendItem[]): HTMLElement[] {
  return items.map((item) => {
    const el = h('span', { class: 'legend-item' }, h('span', {}, item.label));
    el.prepend(markSvg(item.mark));
    return el;
  });
}
