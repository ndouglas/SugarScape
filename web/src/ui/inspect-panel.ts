import type { Engine } from '../engine';
import type { AgentView, LinkView } from '../types';
import { h } from './dom';

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

export class InspectPanel {
  readonly el = h('div', { class: 'inspect' });
  private visible = false;

  constructor(private engine: Engine) {
    for (const event of ['select', 'tick', 'reset', 'edit'] as const) engine.on(event, () => this.render());
    this.render();
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) this.render();
  }

  private links(links: LinkView[]): HTMLElement {
    if (links.length === 0) return h('span', { class: 'hint' }, 'none');
    return h(
      'span',
      { class: 'links' },
      ...links.map((l) =>
        l.alive
          ? h('button', { class: 'link', onclick: () => this.engine.follow(l.id) }, `#${l.id}`)
          : h('span', { class: 'hint', title: 'deceased' }, `#${l.id}†`),
      ),
    );
  }

  private agentRows(a: AgentView): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    return [
      row('Agent', `#${a.id} · ${a.sex} · ${a.tribe}`),
      row('Sugar', `${fmt(a.sugar)} (born with ${fmt(a.initial_sugar)})`),
      row('Vision', String(a.vision)),
      row('Metabolism', String(a.metabolism)),
      row('Age', `${a.age} / ${a.max_age}`),
      row('Fertile', `${a.fertile ? 'yes' : 'no'} (ages ${a.fertility_onset}–${a.fertility_end})`),
      row('Culture tags', h('code', {}, a.tags)),
      row('Born', `tick ${a.born}`),
      row('Parents', this.links(a.parents)),
      row('Children', this.links(a.children)),
    ];
  }

  private render(): void {
    if (!this.visible) return;
    const sel = this.engine.selection;
    if (!sel) {
      this.el.replaceChildren(h('p', { class: 'hint' }, 'Choose the Inspect tool and click an agent or site.'));
      return;
    }
    const gone = sel.agentId !== null && !this.engine.sim.locate(sel.agentId);
    const { site, agent } = this.engine.inspect(sel.x, sel.y);
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    this.el.replaceChildren(
      ...(gone ? [h('p', { class: 'error' }, `Agent #${sel.agentId} has died.`)] : []),
      h(
        'table',
        {},
        row('Site', `(${site.x}, ${site.y})`),
        row('Sugar', `${fmt(site.sugar)} / ${fmt(site.capacity)}`),
        row('Pollution', fmt(site.pollution)),
        ...(agent && !gone ? this.agentRows(agent) : []),
      ),
    );
  }
}
