import type { Engine } from '../engine';
import { isSugarView } from '../models';
import type { AgentView, LinkView } from '../types';
import { h } from './dom';

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

export class InspectPanel {
  readonly el = h('div', { class: 'inspect' });
  private visible = false;

  constructor(private engine: Engine) {
    for (const event of ['select', 'tick', 'reset', 'config', 'edit', 'follow'] as const) engine.on(event, () => this.render());
    this.render();
  }

  setVisible(visible: boolean): void {
    this.visible = visible;
    if (visible) this.render();
  }

  private goodName(i: number): string {
    return this.engine.sugar.goods[i]?.name ?? `good ${i}`;
  }

  private links(links: LinkView[]): HTMLElement {
    if (links.length === 0) return h('span', { class: 'hint' }, 'none');
    return h(
      'span',
      { class: 'links' },
      ...links.map((l) =>
        l.alive
          ? h('button', { class: 'link', onclick: () => this.engine.selectAgent(l.id) }, `#${l.id}`)
          : h('span', { class: 'hint', title: 'deceased' }, `#${l.id}†`),
      ),
    );
  }

  private followButton(id: number): HTMLElement {
    const following = this.engine.followed() === id;
    return h('button', {
      class: 'link',
      disabled: following,
      title: 'Draw this agent’s trail on the grid',
      onclick: () => this.engine.followAgent(id),
    }, following ? 'Following' : 'Follow');
  }

  private agentRows(a: AgentView): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    return [
      row('Agent', h('span', {}, `#${a.id} · ${a.sex} · ${this.engine.sugar.culture.groups[a.group]?.name ?? a.tribe} `, this.followButton(a.id))),
      ...a.holdings.map((held, i) =>
        row(this.goodName(i), `${fmt(held)} (born with ${fmt(a.initial[i])}) · metabolism ${a.metabolism[i]}`),
      ),
      ...(this.engine.sugar.foresight.enabled ? [row('Foresight φ', String(a.foresight))] : []),
      row('Vision', String(a.vision)),
      row('Age', `${a.age} / ${a.max_age}`),
      row('Fertile', `${a.fertile ? 'yes' : 'no'} (ages ${a.fertility_onset}–${a.fertility_end})`),
      row('Culture tags', h('code', {}, a.tags)),
      ...(this.engine.sugar.disease.enabled ? this.diseaseRows(a) : []),
      row('Born', `tick ${a.born}`),
      row('Parents', this.links(a.parents)),
      row('Children', this.links(a.children)),
      ...(a.loans.length
        ? [row('Loans', h('span', { class: 'links' }, ...a.loans.map((l) =>
            h('span', {}, `${l.role === 'lender' ? 'lent to' : 'owes'} `,
              l.counterparty.alive
                ? h('button', { class: 'link', onclick: () => this.engine.selectAgent(l.counterparty.id) }, `#${l.counterparty.id}`)
                : h('span', { class: 'hint' }, `#${l.counterparty.id}†`),
              ` ${fmt(l.due)} ${this.goodName(l.good)} by t=${l.due_tick}`))))]
        : []),
    ];
  }

  private diseaseRows(a: AgentView): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const learned = [...a.immune].filter((bit, i) => bit !== a.immune_genome[i]).length;
    return [
      row('Immune string', h('span', {}, h('code', {}, a.immune), h('span', { class: 'hint' }, ` ${learned} bits learned`))),
      row('Immune genome', h('code', {}, a.immune_genome)),
      row(
        'Diseases',
        a.diseases.length === 0
          ? h('span', { class: 'hint' }, 'none')
          : h(
              'span',
              { class: 'links' },
              ...a.diseases.map((d) =>
                h('span', { title: 'Hamming distance to the closest window of the immune string' }, `#${d.id} `, h('code', {}, d.bits), ` (${d.distance})`),
              ),
            ),
      ),
      row('Infected by', a.infected_by ? this.links([a.infected_by]) : h('span', { class: 'hint' }, 'nobody')),
    ];
  }

  private render(): void {
    if (!this.visible) return;
    const sel = this.engine.selection;
    const shown = this.engine.inspection;
    if (!sel || !shown) {
      this.el.replaceChildren(h('p', { class: 'hint' }, 'Choose the Inspect tool and click an agent or site.'));
      return;
    }
    // The host tracks a selected agent while it lives (Decision 3).
    const gone = shown.agentId !== null && !shown.alive;
    // Only a sugarscape's inspection has these rows; the other models' are added with their Inspect rows.
    if (!isSugarView(shown.view)) return;
    const { site, agent } = shown.view;
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    this.el.replaceChildren(
      ...(gone ? [h('p', { class: 'error' }, `Agent #${shown.agentId} has died.`)] : []),
      h(
        'table',
        {},
        row('Site', `(${site.x}, ${site.y})`),
        ...site.resources.map((r, i) => row(`${this.goodName(i)} here`, `${fmt(r)} / ${fmt(site.capacities[i])}`)),
        ...site.pollution.map((p, k) => row(this.engine.sugar.pollution.pollutants[k]?.name ?? `pollutant ${k}`, fmt(p))),
        ...(agent && !gone ? this.agentRows(agent) : []),
      ),
    );
  }
}
