import { creditHeader, creditLayout, rowPositions, type CreditGraph } from '../credit';
import type { Engine } from '../engine';
import { h } from './dom';

const SVG_NS = 'http://www.w3.org/2000/svg';
const ROW = 56;
const PAD = 14;
const RADIUS = 5;
const REFRESH_MS = 500;
/** Minimum gap between neighbouring nodes; wider rows scroll horizontally. */
const GAP = 10;

function svg<K extends keyof SVGElementTagNameMap>(tag: K, attrs: Record<string, string | number> = {}): SVGElementTagNameMap[K] {
  const el = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, String(value));
  return el;
}

/** Animation IV-5's lender → borrower hierarchy: one row per level, lenders green, borrowers red, both yellow. */
export class CreditPanel {
  readonly el = h('div', { class: 'credit' });
  private header = h('p', { class: 'hint' });
  private graph = h('div', { class: 'credit-graph' });
  private visible = false;
  private last = -Infinity;
  /** Tick and panel width of the last drawing; null forces the next refresh to redraw. */
  private drawn: { tick: number; width: number } | null = null;

  constructor(private engine: Engine, private onSelect: () => void) {
    this.el.append(this.header, this.graph);
    engine.want((now) => (this.visible && now - this.last >= REFRESH_MS && this.changed() ? { creditGraph: true } : {}));
    engine.on('snapshot', () => {
      const graph = engine.last?.creditGraph;
      if (graph) this.draw(graph);
    });
    // Resets, edits and config changes can change loans without a tick.
    for (const event of ['reset', 'edit', 'config'] as const) engine.on(event, () => (this.drawn = null));
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) {
      this.drawn = null;
      this.last = -Infinity;
      void this.engine.refresh();
    }
  }

  private panelWidth(): number {
    return Math.max(240, this.graph.clientWidth || 360);
  }

  /** Whether the tick or the panel width moved since the last drawing (or nothing is drawn). */
  private changed(): boolean {
    return !this.drawn || this.drawn.tick !== this.engine.tick || this.drawn.width !== this.panelWidth();
  }

  private draw(graph: CreditGraph): void {
    this.last = performance.now();
    if (!this.visible) {
      this.drawn = null;
      return;
    }
    // Redrawing replaces every node, so skip it while nothing changed:
    // otherwise a click's mousedown and mouseup land on different elements.
    if (!this.changed()) return;
    const panelWidth = this.panelWidth();
    this.drawn = { tick: this.engine.tick, width: panelWidth };
    const layout = creditLayout(graph);
    const summary = creditHeader(layout);
    this.header.textContent = summary;
    if (layout.rows.length === 0) {
      this.graph.replaceChildren();
      return;
    }
    const step = 2 * RADIUS + GAP;
    const widest = Math.max(...layout.rows.map((ids) => ids.length));
    const { width } = rowPositions(widest, panelWidth, step);
    const height = 2 * PAD + layout.rows.length * ROW;
    const at = new Map<number, [number, number]>();
    layout.rows.forEach((ids, level) => {
      const { xs } = rowPositions(ids.length, width, step);
      ids.forEach((id, i) => at.set(id, [xs[i], PAD + level * ROW + ROW / 2]));
    });
    const edges = svg('g', { class: 'credit-edges', 'aria-hidden': 'true' });
    for (const l of layout.loans) {
      const a = at.get(l.lender);
      const b = at.get(l.borrower);
      if (a && b) edges.append(svg('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1] }));
    }
    const focused = document.activeElement instanceof SVGElement ? document.activeElement.dataset.id : undefined;
    let refocus: SVGCircleElement | null = null;
    const nodes = svg('g', { class: 'credit-nodes', role: 'group', 'aria-label': `Credit hierarchy: ${summary}` });
    for (const [id, [x, y]] of at) {
      const role = layout.roles.get(id) ?? 'both';
      const label = `#${id} · ${role}`;
      const node = svg('circle', {
        cx: x,
        cy: y,
        r: RADIUS,
        class: `credit-node ${role}`,
        tabindex: 0,
        role: 'button',
        'aria-label': `Select agent ${label}`,
        'data-id': id,
      });
      const title = svg('title');
      title.textContent = label;
      node.append(title);
      const select = () => {
        this.engine.selectAgent(id);
        this.onSelect();
      };
      node.addEventListener('click', select);
      node.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          select();
        }
      });
      if (focused === String(id)) refocus = node;
      nodes.append(node);
    }
    const root = svg('svg', { viewBox: `0 0 ${width} ${height}`, width, height });
    root.append(edges, nodes);
    this.graph.replaceChildren(root);
    refocus?.focus();
  }
}
