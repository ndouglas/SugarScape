// The legend under the map: the active color mode's colors and the overlay symbols on screen.
import { theftOn } from '../models';
import type { ColorMode, Config, LabView } from '../types';
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
} as const;

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
      return [swatch('hoarder', c.HOARDER), swatch('cheater', c.CHEATER)];
    case 'caching_rule':
      return [
        swatch(theftOn(config) ? 'none (and cheaters)' : 'none', c.NEUTRAL),
        swatch('even', c.RULE_EVEN),
        swatch('compensate', c.RULE_COMPENSATE),
        swatch('plan', c.RULE_PLAN),
      ];
    default:
      return [];
  }
}

/** What is on the map besides agents, as the grid view draws it (`GridView.marks`). */
export interface MapMarks {
  /** The all-caches overlay is on, and whether any agent is a cheater (their caches tint apart). */
  allCaches: boolean;
  cheaters: boolean;
  /** The selected agent's own caches (highlighted). */
  ownCaches: boolean;
  /** A central world's homes and larders. */
  homes: boolean;
  /** The selected agent's remembered sites and known truffle spots. */
  memory: boolean;
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
    out.push(symbol(m.cheaters ? "a hoarder's cache (size: sugar)" : 'a cache (size: sugar)', 'cache'));
    if (m.cheaters) out.push(symbol("a cheater's cache", 'cheater-cache'));
  }
  if (m.ownCaches) out.push(symbol("the selected agent's caches", 'own-cache'));
  if (m.homes) out.push(symbol('home', 'home'), symbol('larder (size: sugar)', 'larder'));
  if (m.memory) out.push(symbol('remembered site (fades with age)', 'memory'), symbol('known truffle spot (filled: ripe)', 'spot'));
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
