import { creditHeader, creditLayout } from '../credit';
import type { Engine } from '../engine';
import { h } from './dom';

const SVG_NS = 'http://www.w3.org/2000/svg';
const ROW = 56;
const PAD = 14;
const RADIUS = 5;
const REFRESH_MS = 500;

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
  private last = 0;

  constructor(private engine: Engine, private onSelect: () => void) {
    this.el.append(this.header, this.graph);
    engine.on('reset', () => this.refresh());
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) this.refresh();
  }

  /** Called every animation frame; redraws at most every REFRESH_MS while visible. */
  maybeRefresh(now: number): void {
    if (this.visible && now - this.last >= REFRESH_MS) this.refresh();
  }

  private refresh(): void {
    this.last = performance.now();
    if (!this.visible) return;
    const layout = creditLayout(this.engine.creditGraph());
    const summary = creditHeader(layout);
    this.header.textContent = summary;
    if (layout.rows.length === 0) {
      this.graph.replaceChildren();
      return;
    }
    const width = Math.max(240, this.graph.clientWidth || 360);
    const height = 2 * PAD + layout.rows.length * ROW;
    const at = new Map<number, [number, number]>();
    layout.rows.forEach((ids, level) =>
      ids.forEach((id, i) => at.set(id, [((i + 1) * width) / (ids.length + 1), PAD + level * ROW + ROW / 2])),
    );
    const edges = svg('g', { class: 'credit-edges' });
    for (const l of layout.loans) {
      const a = at.get(l.lender);
      const b = at.get(l.borrower);
      if (a && b) edges.append(svg('line', { x1: a[0], y1: a[1], x2: b[0], y2: b[1] }));
    }
    const nodes = svg('g', { class: 'credit-nodes' });
    for (const [id, [x, y]] of at) {
      const role = layout.roles.get(id) ?? 'both';
      const node = svg('circle', { cx: x, cy: y, r: RADIUS, class: `credit-node ${role}` });
      const title = svg('title');
      title.textContent = `#${id} · ${role}`;
      node.append(title);
      node.addEventListener('click', () => {
        this.engine.selectAgent(id);
        this.onSelect();
      });
      nodes.append(node);
    }
    const root = svg('svg', { viewBox: `0 0 ${width} ${height}`, width, height, role: 'img', 'aria-label': `Credit hierarchy: ${summary}` });
    root.append(edges, nodes);
    this.graph.replaceChildren(root);
  }
}
