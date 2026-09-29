import { citizenRows, shownCitizen } from '../civil';
import { dpdRows } from '../dpd';
import type { Engine } from '../engine';
import { ethnoRows } from '../ethno';
import { imageRows } from '../image-scoring';
import { isAgreementView, isAntsView, isPunishmentView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
import { playerRows } from '../spatial';
import type {
  AgentView,
  AntsInspection,
  PunishmentInspection,
  RetirementInspection,
  ThresholdsInspection,
  FarolInspection,
  AgreementInspection,
  AnasaziInspection,
  CivilInspection,
  ClassesInspection,
  OpinionsInspection,
  StructureInspection,
  CultureInspection,
  CultureSiteView,
  DpdInspection,
  NormsInspection,
  EthnoConfig,
  EthnoInspection,
  GoapView,
  ImageConfig,
  ImageInspection,
  LinkView,
  RingInspection,
  SchellingInspection,
  SpatialConfig,
  SpatialInspection,
  TagsInspection,
} from '../types';
import { PDSI_CLASSES, waterText } from '../valley';
import { h } from './dom';
import { percent } from './format';

const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/**
 * The Heading row's text for a walked plan: the target and steps left (singular/plural), "Can't
 * reach" when no path was found to a target elsewhere, or "Staying" when the agent is at its target.
 */
export function headingText(plan: NonNullable<AgentView['plan']>, at: [number, number]): string {
  const n = plan.path.length;
  const target = `(${plan.target_x}, ${plan.target_y})`;
  if (n) return `${target}, ${n} ${n === 1 ? 'step' : 'steps'} left`;
  return plan.target_x === at[0] && plan.target_y === at[1] ? 'Staying' : `Can't reach ${target}`;
}

/**
 * The Remembers row's text (Minds 3): "n site(s) (m truffle spot(s))", singular/plural for each
 * noun, or "Doesn't remember" for a non-rememberer (or `null`, memory off).
 */
export function memoryText(m: AgentView['memory']): string {
  if (!m?.remembers) return "Doesn't remember";
  const sites = m.sites === 1 ? 'site' : 'sites';
  const spots = m.spots === 1 ? 'spot' : 'spots';
  return `${m.sites} ${sites} (${m.spots} truffle ${spots})`;
}

/**
 * The Plan row's text (Minds 4, GOAP): "n step(s), gathers ~x of G", singular at one step, or
 * "Done (gathers ~x of G)" once no targets are left. `x` is rounded to one decimal.
 */
export function planText(goap: GoapView): string {
  const n = goap.steps.length;
  const gathers = `gathers ~${Number(goap.gathers.toFixed(1))} of ${fmt(goap.goal)}`;
  return n ? `${n} ${n === 1 ? 'step' : 'steps'}, ${gathers}` : `Done (${gathers})`;
}

/** The Average rate row's text (Minds 4, marginal value): ρ in sugar a tick, to two decimals. */
export function rateText(rate: number): string {
  return `${rate.toFixed(2)} sugar a tick`;
}

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
      ...((this.engine.sugar.memory?.span ?? 0) > 0 ? [row('Remembers', memoryText(a.memory))] : []),
      ...(a.plan
        ? [
            a.plan.walked
              ? row('Heading', headingText(a.plan, [a.x, a.y]))
              : row('Moved to', `(${a.plan.target_x}, ${a.plan.target_y})`),
          ]
        : []),
      ...(a.goap
        ? [
            row('Plan', planText(a.goap)),
            ...(a.goap.steps.length ? [row('Next target', `(${a.goap.steps[0][0]}, ${a.goap.steps[0][1]})`)] : []),
          ]
        : []),
      ...(a.rate != null ? [row('Average rate ρ', rateText(a.rate))] : []),
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

  /** A spatial cell and its player: strategy, score, its neighborhood's best of each kind and what it becomes. */
  private spatialRows(view: SpatialInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const { x, y, z } = view.site;
    const config = this.engine.config as SpatialConfig;
    const rows = [row('Cell', config.lattice === 'cube' ? `(${x}, ${y}, z = ${z})` : `(${x}, ${y})`)];
    if (!view.agent) return [...rows, row('Player', 'none (an empty cell)')];
    return [...rows, ...playerRows(view.agent, config.update === 'asynchronous').map(([k, v]) => row(k, v))];
  }

  /**
   * An ethnocentrism site and its agent: tag, strategy (and basis), PTR and helps, lineage, kin
   * marker and age, and its neighbors. An agent that died leaves the site's rows alone.
   */
  private ethnoSiteRows(view: EthnoInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (gone) return rows;
    if (!view.agent) return [...rows, row('Agent', 'none (an empty site)')];
    const kin = (this.engine.config as EthnoConfig).kin_strategies;
    return [...rows, ...ethnoRows(view.agent, kin).map(([k, v]) => row(k, v))];
  }

  /**
   * A demographic PD site and its agent: strategy, wealth, age, whether surrounded, this cycle's
   * payoffs, and its neighbors with what a game between them pays. An agent that died leaves the
   * site's rows alone.
   */
  /** A plane cell (its strategy and the agents holding it) or a strip row (one agent's generation). */
  private normsRows(view: NormsInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const agent = (a: NormsInspection['agents'][number]) => {
      const group = a.group ? ` · ${a.group}` : '';
      return `B ${a.boldness}/7 · V ${a.vengefulness}/7${group} · payoff ${fmt(a.payoff)}`;
    };
    if (view.agent) {
      const a = view.agent;
      return [
        row('Agent', `#${a.id}${a.group ? ` (${a.group})` : ''}`),
        row('Strategy', `${a.bits} — boldness ${a.boldness}/7, vengefulness ${a.vengefulness}/7`),
        row('Payoff', fmt(a.payoff)),
        row('Defected', `${a.defections}× · punished ${a.punished}×`),
        row('Punished others', `${a.punishments}× · metapunished ${a.metapunishments}× · was metapunished ${a.metapunished}×`),
        row('Copies', a.parent === null ? 'the starting population' : `#${a.parent} of the generation before`),
      ];
    }
    if (view.level) {
      const rows = [row('Strategy', `boldness ${view.level[0]}/7, vengefulness ${view.level[1]}/7`)];
      if (view.agents.length === 0) return [...rows, row('Agents', 'none')];
      for (const a of view.agents.slice(0, 12)) rows.push(row(`#${a.id}`, agent(a)));
      if (view.agents.length > 12) rows.push(row('', `and ${view.agents.length - 12} more`));
      return rows;
    }
    return [row('Point', 'between the plane and the agents')];
  }

  private dpdSiteRows(view: DpdInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (gone) return rows;
    if (!view.agent) return [...rows, row('Agent', 'none (an empty site)')];
    return [...rows, ...dpdRows(view.agent).map(([k, v]) => row(k, v))];
  }

  /**
   * An image-scoring cell and the agent of the generation that last played there. A followed agent
   * lives one generation (its offspring have new ids), so once it is gone only the cell shows.
   */
  private imageCellRows(view: ImageInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    if (gone) return [row('Cell', `(${view.cell.x}, ${view.cell.y})`)];
    return imageRows(view, this.engine.config as ImageConfig).map(([k, v]) => row(k, v));
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

  /** A cell of the agents' block (its agent) or of the p–q plane (its point and the agents there). */
  private structureRows(view: StructureInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const names = { tft: 'near Tit-for-Tat', alld: 'near Always Defect', allc: 'near Always Cooperate', other: 'mixed' };
    const strategy = (a: { y: number; p: number; q: number; class: keyof typeof names }) =>
      `y ${fmt(a.y)} · p ${fmt(a.p)} · q ${fmt(a.q)} (${names[a.class]})`;
    if (view.agent) {
      const a = view.agent;
      const rows = [row('Agent', `#${a.id}`), row('Strategy', strategy(a)), row('Payoff per move', fmt(a.score)), row('Copied', a.copied === null ? 'no one' : `#${a.copied}`)];
      for (const p of a.partners) rows.push(row(`Played #${p.id}`, `${p.games}× · p ${fmt(p.p)} · payoff ${fmt(p.score)}`));
      return rows;
    }
    if (view.plane) {
      const rows = [row('Point', `p ${fmt(view.plane[0])} · q ${fmt(view.plane[1])}`)];
      if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
      for (const a of view.agents.slice(0, 12)) rows.push(row(`#${a.id}`, `${strategy(a)} · payoff ${fmt(a.score)}`));
      if (view.agents.length > 12) rows.push(row('', `and ${view.agents.length - 12} more`));
      return rows;
    }
    if (view.block) return [row('Block cell', 'no agent here')];
    return [row('Point', 'between the agents and the plane')];
  }

  /** A round of the El Farol time panel, a row of its histogram, or an agent and its strategies. */
  private farolRows(view: FarolInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    if (view.panel === 'time') return [row('Round', String(view.round)), row('Attendance', String(view.attendance))];
    if (view.panel === 'histogram') return [row('Attendance', `about ${view.attendance}`), row('Rounds', String(view.count))];
    const a = view.member;
    if (!a) return [row('Point', 'between the panels')];
    const rows = [
      row('Agent', `#${a.id}${a.memory > 0 ? ` · memory ${a.memory}` : ''}`),
      row('This round', a.went ? 'went (side A)' : 'stayed (side B)'),
      row('Winnings', `${fmt(a.gain)} · switched ${a.switches}×`),
    ];
    for (const s of a.strategies) {
      const says = s.forecast !== null ? `forecasts ${s.forecast}` : s.attend ? 'would go' : 'would stay';
      rows.push(row(s.active ? `▶ ${s.label}` : s.label, `${says} · score ${fmt(s.score)}`));
    }
    return rows;
  }

  /** An agent and its group, or a period of the time strip. */
  private punishmentRows(view: PunishmentInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const pct = (x: number) => `${fmt(100 * x)} %`;
    if (view.panel === 'time')
      return [row('Period', String(view.period)), row('Cooperation', pct(view.cooperation ?? 0)), row('Punishment', pct(view.punishment ?? 0))];
    const a = view.agent;
    const g = view.group;
    if (!a || !g) return [row('Point', 'between groups')];
    const traits = a.kind ?? `cooperates ${pct(a.cooperate)} · punishes ${pct(a.punish)}`;
    const did = `${a.cooperated ? 'cooperated' : 'defected'}${a.punished ? ' and punished' : ''}`;
    return [
      row('Agent', `#${a.id} · ${traits}`),
      row('This period', `${did} · payoff ${fmt(a.payoff)}`),
      row('Group', `#${g.index + 1} · ${pct(g.contributors)} contributors · ${pct(g.punishers)} punishers · ${pct(g.defectors)} defectors`),
      row('Group this period', `${pct(g.acts)} cooperated · mean payoff ${fmt(g.payoff)}`),
      row('Last conflict', g.last_conflict === null ? 'none yet' : `period ${g.last_conflict} (${g.lost ? 'lost: replaced' : 'won'})`),
    ];
  }

  /** An agent of the retirement population, an age's retirements, or a period. */
  private retirementRows(view: RetirementInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const pct = (x: number) => `${fmt(100 * x)} %`;
    if (view.panel === 'time') return [row('Period', String(view.period)), row('Eligible retired', pct(view.retired ?? 0))];
    if (view.panel === 'ages') {
      const e = view.exposed ?? 0;
      return [row('Age', String(view.age)), row('Retired, last 10 periods', `${view.retirements} of ${e}${e > 0 ? ` (${pct((view.retirements ?? 0) / e)})` : ''}`)];
    }
    const a = view.member;
    if (!a) return [row('Age', view.age === null ? '—' : String(view.age)), row('Point', 'an empty place')];
    const rows = [
      row('Agent', `#${a.id} · ${a.kind}${a.group > 0 ? ' · second group' : ''}`),
      row('Age', `${a.age} (dies at ${fmt(a.death_age)})`),
      row('Status', a.retired ? `retired${a.retired_at !== null ? ` at ${a.retired_at}` : ''}` : 'working'),
      row('Network', `${a.network} · ${a.eligible} eligible · ${a.retired_members} retired`),
    ];
    if (a.kind === 'imitator') rows.push(row('Threshold', pct(a.threshold)));
    return rows;
  }

  /** A step of the thresholds' time panel, a point of Figure 1, a histogram row, or an actor. */
  private thresholdsRows(view: ThresholdsInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const pct = (x: number) => `${fmt(100 * x)} %`;
    if (view.panel === 'time') return [row('Step', String(view.step)), row('Acting', (view.crowds ?? []).map(pct).join(' · '))];
    if (view.panel === 'figure') return [row('Share acting', pct(view.share ?? 0)), row('Thresholds at or below', pct(view.cdf ?? 0))];
    if (view.panel === 'histogram') return [row('Final share', `about ${pct(view.share ?? 0)}`), row('Episodes', String(view.count))];
    const a = view.member;
    if (!a) return [row('Point', 'between the panels')];
    const rows = [
      row('Actor', `#${a.id}${a.seed ? ' · the spark' : ''}${a.crowd > 1 ? ` · crowd ${a.crowd}` : ''}`),
      row('Threshold', a.threshold === null ? 'never acts' : pct(a.threshold)),
      row('Sees acting', `${a.sees} of ${a.of}${a.degree !== null ? ` · watches ${a.degree}` : ''}`),
      row('Now', a.acting ? 'acting' : 'not acting'),
    ];
    if (a.ceiling !== null) rows.push(row('Leaves above', pct(a.ceiling)));
    return rows;
  }

  /** A step of the ants' time panel, a row of their histogram, or an ant. */
  private antsRows(view: AntsInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const pct = (x: number) => `${Math.round(100 * x)} %`;
    if (view.panel === 'time') return [row('Step', String(view.step)), row('Shares', (view.shares ?? []).map(pct).join(' · '))];
    if (view.panel === 'histogram') {
      const rows = [row('At the first source', `about ${pct(view.share ?? 0)}`), row('Steps', String(view.count))];
      if (view.theory !== null) rows.push(row('Theory', `${fmt(100 * view.theory)} % of steps`));
      return rows;
    }
    const a = view.member;
    if (!a) return [row('Point', 'between the panels')];
    return [
      row('Ant', `#${a.id} · source ${a.source}${a.independent ? ' · never herds' : ''}`),
      row('Can meet', `${a.degree} · ${a.elsewhere} at another source`),
    ];
  }

  /** A cell of relative agreement's diagram, start-against-now panel or torus, and the agents there. */
  private agreementRows(view: AgreementInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    if (view.panel === null) return [row('Point', 'between the panels')];
    const panel = { diagram: 'Opinion × time', scatter: 'Start against now', torus: 'Lattice' }[view.panel];
    const rows = [row('Panel', panel)];
    if (view.period !== null) rows.push(row('Period', String(view.period)));
    if (view.opinion !== null) rows.push(row('Opinion', fmt(view.opinion)));
    if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
    const role = { plus: 'extremist (+1)', minus: 'extremist (−1)', moderate: 'moderate' };
    const shown = view.agents.slice(0, 12);
    for (const a of shown) {
      rows.push(
        row(
          `#${a.id}`,
          `${fmt(a.opinion)} ± ${fmt(a.uncertainty)} · ${role[a.role]} · started ${fmt(a.start)} · ${a.degree} neighbors · moved in ${a.moves} of ${a.meetings} meetings`,
        ),
      );
    }
    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
    return rows;
  }

  /** A cell of the opinion × time diagram (its period, opinion and the agents passing) or of the lattice. */
  private opinionsRows(view: OpinionsInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows: HTMLElement[] = [];
    if (view.period !== null && view.opinion !== null) rows.push(row('Period', String(view.period)), row('Opinion', fmt(view.opinion)));
    else if (view.lattice_site) rows.push(row('Site', `(${view.lattice_site.x}, ${view.lattice_site.y})`));
    else return [row('Point', 'between the diagram and the lattice')];
    if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
    const shown = view.agents.slice(0, 12);
    for (const a of shown) {
      rows.push(row(`#${a.id}`, `${fmt(a.opinion)} (started ${fmt(a.start)}) · reach −${fmt(a.epsilon_left)} +${fmt(a.epsilon_right)} · hears ${a.reaches}`));
    }
    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
    return rows;
  }

  /** A point of a memory simplex: its mix, the best reply there, and the agents whose memory plots there. */
  private classesRows(view: ClassesInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    if (!view.simplex || !view.mix) return [row('Point', 'outside the simplexes')];
    const [l, m, hi] = view.mix.map((p) => `${Math.round(p * 100)}%`);
    const name = { one: 'Memories', intra: 'Memories of their own tag', inter: 'Memories of the other tag' }[view.simplex];
    const rows = [row('Simplex', name), row('Remembered', `L ${l} · M ${m} · H ${hi}`), row('Best reply', view.best_reply ?? '')];
    if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
    const shown = view.agents.slice(0, 12);
    for (const a of shown) {
      const tag = a.tag ? ` · ${a.tag}` : '';
      rows.push(row(`#${a.id}`, `L ${a.memory[0]} · M ${a.memory[1]} · H ${a.memory[2]}${tag} · last ${a.last_demand ?? '–'} · payoff ${fmt(a.mean_payoff)}`));
    }
    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
    return rows;
  }

  /** A culture site (its traits, region, zone and what each neighbor shares) or a lane between two sites. */
  private cultureRows(view: CultureInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const site = (s: CultureSiteView) => `(${s.x}, ${s.y}) · ${s.traits.join(' ')}`;
    const where = (s: CultureSiteView) => `region of ${s.region_size} · zone of ${s.zone_size}`;
    if (view.kind === 'lane' && view.b) {
      const f = view.a.traits.length;
      return [
        row('Between', `${site(view.a)} and ${site(view.b)}`),
        row('Shared', `${view.shared} of ${f} features${view.shared === f ? ' (identical)' : view.shared === 0 ? ' (cannot interact)' : ''}`),
      ];
    }
    return [
      row('Site', site(view.a)),
      row('Belongs to', where(view.a)),
      ...view.neighbors.map((n) => row(`(${n.x}, ${n.y})`, `shares ${n.shared} of ${view.a.traits.length}`)),
    ];
  }

  /**
   * A cell of the tags diagram: its generation and tag bin, what the bin held, and in the current
   * generation its agents (the first 12).
   */
  private tagsRows(view: TagsInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const tags = `${view.from.toFixed(2)}–${view.to.toFixed(2)}`;
    if (view.generation === null) return [row('Tags', tags), row('Generation', 'before the first')];
    const rows = [row('Generation', String(view.generation)), row('Tags', tags)];
    const t = view.tolerance;
    if (view.count === 0 || !t) return [...rows, row('Agents', 'none')];
    rows.push(
      row('Agents', `${view.count} with ${view.distinct} distinct ${view.distinct === 1 ? 'tag' : 'tags'}`),
      row('Tolerance', `${t.min.toFixed(4)} – ${t.max.toFixed(4)} (mean ${t.mean.toFixed(4)})`),
      row('Donations', `${view.given} made · ${view.received} received`),
    );
    const shown = view.agents.slice(0, 12);
    for (const a of shown) {
      rows.push(row(`#${a.id}`, `tag ${a.tag.toFixed(4)} · tolerance ${a.tolerance.toFixed(4)} · score ${fmt(a.score)} · from #${a.parent}`));
    }
    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
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
    // First: an image-scoring cell has no `site` for the guards below to read.
    if (isImageView(view)) {
      const note = gone ? [h('p', { class: 'error' }, `Agent #${shown.agentId}’s generation has passed: each agent lives one generation.`)] : [];
      this.el.replaceChildren(...note, h('table', {}, ...this.imageCellRows(view, gone)));
      return;
    }
    if (!isSugarView(view)) {
      // A Schelling agent that reached its maximum residence has left the landscape; a household
      // dies or leaves the valley; a civil agent dies, is released, or (Model II) is killed.
      const left = isValleyView(view)
        ? `Household #${shown.agentId} is gone: it died or left the valley.`
        : isCivilView(view)
          ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
          : isEthnoView(view, this.engine.model) || isDpdView(view, this.engine.model)
            ? `Agent #${shown.agentId} has died.`
            : `Agent #${shown.agentId} has left.`;
      const note = gone ? [h('p', { class: 'error' }, left)] : [];
      // First: an empty ethnocentrism or demographic PD site is shaped like an empty Schelling site.
      const rows = isEthnoView(view, this.engine.model)
        ? this.ethnoSiteRows(view, gone)
        : isDpdView(view, this.engine.model)
          ? this.dpdSiteRows(view, gone)
          : isNormsView(view)
            ? this.normsRows(view)
          : isAgreementView(view)
            ? this.agreementRows(view)
          : isPunishmentView(view)
            ? this.punishmentRows(view)
          : isRetirementView(view)
            ? this.retirementRows(view)
          : isThresholdsView(view)
            ? this.thresholdsRows(view)
          : isAntsView(view)
            ? this.antsRows(view)
          : isFarolView(view)
            ? this.farolRows(view)
          : isStructureView(view)
            ? this.structureRows(view)
            : isOpinionsView(view)
              ? this.opinionsRows(view)
              : isClassesView(view)
                ? this.classesRows(view)
                : isCultureView(view)
                  ? this.cultureRows(view)
                  : isTagsView(view)
                    ? this.tagsRows(view)
                    : isRingView(view)
                      ? this.ringRows(view, gone)
                      : isValleyView(view)
                        ? this.valleyRows(view, gone)
                        : isCivilView(view)
                          ? this.civilRows(view, shown.agentId, gone)
                          : isSpatialView(view)
                            ? this.spatialRows(view)
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
