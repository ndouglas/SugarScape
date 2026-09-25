import { citizenRows, shownCitizen } from '../civil';
import type { Engine } from '../engine';
import { isCivilView, isRingView, isSpatialView, isSugarView, isValleyView } from '../models';
import type { AgentView, AnasaziInspection, CivilInspection, LinkView, RingInspection, SchellingInspection, SpatialInspection } from '../types';
import { PDSI_CLASSES, waterText } from '../valley';
import { h } from './dom';
import { percent } from './format';

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

  /** Schelling's site and agent: color, preference, satisfaction and residence (Decision 13). */
  private schellingRows(view: SchellingInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const a = view.agent;
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (!a || gone) return rows;
    const alike = a.neighbors === 0 ? 'no neighbors' : `${a.like} of ${a.neighbors} neighbors alike`;
    return [
      ...rows,
      row('Agent', `#${a.id} · ${a.color === 'red' ? 'Red' : 'Blue'}`),
      row('Preference', `at least ${percent(a.preference)} alike`),
      row('Satisfied', `${a.satisfied ? 'yes' : 'no'} (${alike})`),
      row('Residence', a.residence === null ? `age ${a.age} (no maximum)` : `age ${a.age} of ${a.residence}`),
    ];
  }

  /** A spatial cell (Task 5 fills in the player's rows). */
  private spatialRows(view: SpatialInspection, _gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    return [row('Cell', `(${view.site.x}, ${view.site.y}, ${view.site.z})`)];
  }

  /** A civil site: its cop, the agent shown there (followed into jail), and others jailed after arrest here. */
  private civilRows(view: CivilInspection, followed: number | null, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (view.cop) rows.push(row('Cop', `#${view.cop.id}`));
    const a = gone ? null : shownCitizen(view, followed);
    if (a) rows.push(...citizenRows(a).map(([k, v]) => row(k, v)));
    const others = view.jailed.filter((j) => j.id !== a?.id).length;
    if (others > 0) rows.push(row('Jailed here', `${others} arrested on this site`));
    return rows;
  }

  /** Ring World's site and its agent (Decision 13). */
  private ringRows(view: RingInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const a = view.agent;
    return [
      row('Site', `#${view.site.x}`),
      row('Sugar', `${fmt(view.site.sugar)} / ${view.site.capacity}`),
      ...(a && !gone ? [row('Agent', `#${a.id}`), row('Vision', `${a.vision} sites`)] : []),
    ];
  }

  /**
   * A Long House Valley cell — its zone, this year's PDSI class and yields, water, and who farms
   * and lives there — and the household shown there (its farmer, else its first resident).
   */
  private valleyRows(view: AnasaziInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: HTMLElement | string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const kg = (n: number) => `${Math.round(n)} kg`;
    const alive = (ids: number[]) => this.links(ids.map((id) => ({ id, alive: true })));
    const cell = (xy: [number, number]) =>
      h('button', { class: 'link', onclick: () => this.engine.select(xy[0], xy[1]) }, `(${xy[0]}, ${xy[1]})`);
    const s = view.site;
    const rows = [
      row('Cell', `(${s.x}, ${s.y}) · ${s.zone_name}`),
      row('PDSI', `${fmt(s.pdsi)} · class ${PDSI_CLASSES[s.pdsi_class]}`),
      row('Yield', `${kg(s.base_yield)} this year (zone ${kg(s.zone_yield)} × soil ${s.quality.toFixed(2)} × adjustment)`),
      row('Water', waterText(s)),
      row('Homes', s.habitable ? 'allowed this year' : 'not allowed this year (hydrology)'),
      row('Farmed by', s.farmed_by === null ? h('span', { class: 'hint' }, 'nobody') : alive([s.farmed_by])),
      row('Residents', alive(s.residents)),
    ];
    const a = view.agent;
    if (!a || gone) return rows;
    return [
      ...rows,
      row('Household', `#${a.id} · age ${a.age}`),
      row('Corn', `${kg(a.stock)} (${a.corn.map((c) => Math.round(c)).join(' / ')}, newest first)`),
      row('Harvest', `${kg(a.harvest)} this year · expects ${kg(a.expected)} next`),
      row('Farm', cell(a.farm)),
      row('Home', cell(a.home)),
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
    const view = shown.view;
    if (!isSugarView(view)) {
      // A Schelling agent that reached its maximum residence has left the landscape; a household
      // dies or leaves the valley; a civil agent dies, is released, or (Model II) is killed.
      const left = isValleyView(view)
        ? `Household #${shown.agentId} is gone: it died or left the valley.`
        : isCivilView(view)
          ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
          : `Agent #${shown.agentId} has left.`;
      const note = gone ? [h('p', { class: 'error' }, left)] : [];
      const rows = isRingView(view)
        ? this.ringRows(view, gone)
        : isValleyView(view)
          ? this.valleyRows(view, gone)
          : isCivilView(view)
            ? this.civilRows(view, shown.agentId, gone)
            : isSpatialView(view)
              ? this.spatialRows(view, gone)
              : this.schellingRows(view, gone);
      this.el.replaceChildren(...note, h('table', {}, ...rows));
      return;
    }
    const { site, agent } = view;
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
