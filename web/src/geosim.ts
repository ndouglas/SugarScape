import type { ColorMode, GeosimConfig, GeosimEvent, GeosimInspection, GeosimStateId } from './types';
const number = (value: number | null): string => typeof value === 'number' && Number.isFinite(value) ? value.toFixed(3) : 'unavailable';
const state = (value: GeosimStateId): string => `${value.capital_cell} (generation ${value.sovereignty_generation})`;
const actions = (value: boolean[]) => value.map(v => v ? 'D' : 'C').join('/');
const event = (value: GeosimEvent | null): string => value ? `#${value.id} · period ${value.period} · ${value.kind}${value.cells.length ? ` · cells ${value.cells.join(', ')}` : ''}` : 'None';
/** Local resources and bounded UI events remain distinct from the full terminal war census. */
export function geosimRows(view: GeosimInspection): [string,string][] {
  const s = view.state;
  const rows: [string,string][] = [
    ['Source clock', `${view.periods} completed periods · attempted ${view.attempted_period} · counting from ${view.counting_start}`],
    ['Government', `Cell ${view.cell.id} · capital ${s.id.capital_cell} · generation ${s.id.sovereignty_generation}`],
    ['Territory', `${view.members.length} cells: ${view.members.join(', ')}`],
    ['Capacity', `${number(s.capacity)} · extracted yield ${number(s.extracted_yield)} · previous damage ${number(s.previous_damage)}`],
    ['Technology and distance', `Threshold ${number(s.threshold)} · own-capital distance ${number(view.distance)} · projection ${number(view.projection)}`],
    ['Alert and campaign', `${s.alert ? 'Alert' : 'Inactive'} · target ${s.campaign ? state(s.campaign) : 'None'}`],
    ['Resource recurrence', view.resource_recurrence ? `Period ${view.resource_recurrence.period} · old ${number(view.resource_recurrence.old_capacity)} · yield ${number(view.resource_recurrence.extracted_yield)} · applied damage ${number(view.resource_recurrence.applied_damage)} · target ${number(view.resource_recurrence.target_capacity)} · new ${number(view.resource_recurrence.new_capacity)} · residual ${number(view.resource_recurrence.residual)} · ${view.resource_recurrence.reset ? 'independence reset' : 'smoothed'}` : 'None'],
    ['Fronts', view.fronts.length ? view.fronts.map(f => `${f.states.map(state).join(' — ')}: ${actions(f.actions)} · previous ${actions(f.previous)} · commitments ${f.commitments.map(number).join('/')} · prior ${f.old_commitments.map(number).join('/')} · damage ${f.last_damage.map(number).join('/')} · path ${f.path?.join(' → ') ?? 'None'}`).join('; ') : 'None'],
    ['Active wars', view.wars.length ? view.wars.map(w => `#${w.id} · abstract damage ${number(w.raw_severity)} · ${w.participants.map(p => `${state(p.state)}, last fight ${p.last_fighting_period}`).join('; ')}`).join(' | ') : 'None'],
    ['Last structural event', event(view.last_structural_event)],
    ['UI event trace', `${view.events.length} retained · ${view.events_dropped} dropped · bounded trace, not the full war census`],
  ];
  if (view.invalidity) rows.push(['Invalid reconstruction', view.invalidity]);
  else if (view.finish_reason) rows.push(['Completion', view.finish_reason]);
  if (view.outcome) {
    const o = view.outcome;
    rows.push(['Terminal war census', `${o.completed_wars.length} completed · ${o.censored_wars.length} censored · ${o.legacy_visible_wars.length} exported · ${o.exporter_backlog.length} completed awaiting export`]);
    rows.push(['Terminal damage ledger', `Total ${number(o.ledger?.damage ?? null)} · measured ${number(o.ledger?.measured_damage ?? null)} · recurrence residual ${number(o.ledger?.recurrence_residual ?? null)}`]);
    rows.push(['Severity units', 'Abstract resource damage; not battle deaths']);
    const exported = new Set(o.legacy_visible_wars.map(w=>w.id));
    const queued = new Set(o.exporter_backlog.map(w=>w.id));
    for (const [label,wars] of [['Completed',o.completed_wars],['Censored',o.censored_wars]] as const) {
      for (const w of wars) rows.push([`${label} war #${w.id}`, `Periods ${w.start_period}–${w.end_period ?? `open at ${o.finish_reason}`} · raw damage ${number(w.raw_severity)} · exported ${number(w.exported_severity)} · ${w.active_periods} fighting periods · ${w.elapsed_periods} elapsed · end cause ${w.end_cause ?? 'censored'}${w.parents.length ? ` · merged episodes ${w.parents.join(', ')}` : ''} · participants ${w.participants.map(p=>state(p.state)).join(', ')}${exported.has(w.id) ? ' · exported by collector' : ''}${queued.has(w.id) ? ' · awaiting collector export' : ''}${w.java_saturated ? ' · Java integer saturation' : ''}${w.java_subunit_zero ? ' · positive raw damage serialized as zero' : ''}`]);
    }
  }
  return rows;
}
export function geosimChartCaption(title: string, configs: GeosimConfig[]): string {
  return `${title} · ${configs.map((c,i) => `${configs.length > 1 ? `${i === 0 ? 'A' : 'B'}: ` : ''}${c.periods_per_tick} source periods/tick`).join(' · ')}`;
}
export function geosimLegend(config: GeosimConfig, mode: ColorMode, periods: number): string {
  const meanings: Partial<Record<ColorMode,string>> = {territory:'Colors identify sovereign ownership; white center is capital',capacity:'Distance-extracted resource capacity; Inspect marks unavailable values',technology:'Technology threshold controlling extraction and projection',alert:'Alert status; campaign target is shown in Inspect',wars:'Membership in active abstract conflict clusters'};
  const attribution = config.initial_capacity === 'artifact_random_100_1' ? 'Later-port readings; full artifact identity Unresolved' : 'Cederman2003 paper reconstruction';
  return `${periods} of ${config.initialization_periods + config.observation_periods} source periods · ${config.periods_per_tick} periods/tick · ${attribution} · ${meanings[mode] ?? meanings.territory} · Severity is abstract resource damage, not deaths`;
}
/** Source ambiguities stay visible beside the schema-driven controls. */
export const GEOSIM_RULES = 'Cederman2003 APSR reconstruction. Mobile commitments are contingent plans on fronts, not an additive budget over passive attacks. Severity is abstract resource damage, not battle deaths. Reciprocal defender threshold is the printed p.148 default; same_threshold is unmeasured. Attacked-party damage follows Table A1/resource accounting; acting_party with own_commitment exposes p.148 cost wording. Other named readings include explicitly attributed reconstruction alternatives, including unmeasured founder ordering, counting boundaries, inheritance, retirement, numerical handling and collector draining. GeoSim2 archived-port readings are not certified identical archived execution or exact APSR source. UI event retention is bounded; terminal Inspect retains completed, censored and queued events separately.';

const FIELD_HELP: Record<string,string> = {
  width:'Bounded lattice width in primitive cells; 50 in the source baseline.',
  height:'Bounded lattice height in primitive cells; 50 in the source baseline.',
  initial_states:'Founder governments grown before competition; 200 on the baseline lattice.',
  initialization_periods:'Initial competition before measurement and technological change; 500 source periods by default.',
  observation_periods:'Periods after initialization; source horizon is initialization plus observation (10500 by default).',
  periods_per_tick:'Whole source periods per display tick; the final tick can contain fewer periods.',
  resource_adjustment:'Fraction of the next extraction/damage target applied each period; source default .01.',
  mobile_share:'Share allocated as contingent mobile commitments. Passive-front plans are not an additive budget.',
  campaign_drop_probability:'Chance of dropping a retained campaign at the selected timing; source default .2.',
  attack_probability:'Chance of contemplating an unprovoked attack while inactive; contextual activation is separate.',
  deactivation_probability:'Chance of leaving alert status after neighborhood fighting disappears.',
  superiority_threshold:'Local projected resource advantage giving attack probability .5.',
  victory_threshold:'Attacker resource advantage giving victory probability .5; defender reading is separate.',
  superiority_exponent:'Steepness of the probabilistic attack threshold; source default20.',
  victory_exponent:'Steepness of the probabilistic victory threshold; source default20.',
  damage_fraction:'Abstract resource damage fraction, independent of its incidence and amount basis. Not battle deaths.',
  distance_offset:'Long-distance floor for extraction/projection; source default .1.',
  distance_threshold:'Initial distance threshold in primitive-cell units; source default2.',
  distance_exponent:'Steepness of the distance curve; source default3.',
  shock_probability:'Independent per-state chance of catching up to the current technology frontier.',
  shock_shift:'Total frontier shift across observation periods. Zero disables technological change.',
  war_shadow:'Persistence after fighting, measured in source periods; source default20.',
  context_activation:'Alert states contemplate attacks after own or neighboring fighting; disabling is a source treatment.',
  event_log:'Retain a bounded UI trace. Full terminal war census and the last structural event remain separately available.',
  event_log_limit:'Maximum retained UI events; events beyond the limit are counted as dropped. This is not the scientific census size.',
  topology:'Bounded is the source reading; torus is an unmeasured alternative.',
  founder_growth:'Ordered round-robin is the attributed formation reading; shuffled ordering is unmeasured.',
  initial_capacity:'Extracted capacity is the paper reconstruction; random100/1 is a later-port initialization reading.',
  distance_metric:'Euclidean is the source reading; Manhattan is a registered reading control.',
  distance_formula:'Decreasing is the implemented/source interpretation; printed-increasing exposes the printed-sign ambiguity.',
  enemy_total:'Active-front opposing commitments form the source denominator; all fronts is a reading control.',
  initiation_guard:'Literal precedence follows p.148 pseudocode; global no-action is the later-port guard and a reading control.',
  campaign_drop_timing:'Each decision is the later-port-supported timing; after battle is an unmeasured reading of Table A1 wording.',
  path_sampling:'Target first follows APSR p.148; attacker first is attributed to the predecessor and a reading control.',
  attack_projection:'Respective state curves are the paper reading; initiator curve uses the later-port threshold on both own-government distances.',
  damage_basis:'Opponent projected resources are the attributed default amount; own commitment exposes literal local-cost wording.',
  damage_feedback:'Subtract losses follows the paper resource equation; add losses exposes the later-port sign convention.',
  damage_incidence:'Attacked party is the Table A1/resource/port reading; acting party exposes conflicting p.148 wording and is unmeasured.',
  severity_damage:'All damaged fronts is the paper aggregation reading; mutual-only exposes the later-port severity branch.',
  victory_draws:'Independent draws with defender priority are supported by the later port; exclusive resolution is a reading control.',
  defender_threshold:'Reciprocal follows explicit APSR p.148 defense advantage. Same threshold is an unmeasured reconstruction.',
  capital_capture:'Capture and fragment is supported by the predecessor/later port; collapse-only exposes the APSR capital wording.',
  locking:'Affected-cell locking is the source reconstruction; affected-state locking is a reading control.',
  technology_inheritance:'Reset on reemergence is the paper reconstruction; retain own-cell threshold is the later-port reading, unmeasured separately.',
  cluster_linkage:'Conflict edges implement the source aggregation; adjacent active states is a reading control.',
  retired_participants:'Retain shadow is the paper reconstruction; drop immediately is the later-port reading, unmeasured separately.',
  count_boundary:'After initialization starts measurement at501; at initialization starts at500 as in the later port. Boundary choice is unmeasured separately.',
  severity_export:'Raw damage is the source reconstruction; Java integer100 applies truncation/saturation and reports zero/saturated events.',
  completed_export:'All completed preserves the full census; one per period exposes the later-port FIFO backlog. No horizon flush.',
  numerical_policy:'Reject nonpositive preserves explicit invalidity; floor-zero is a partial port reading. Neutral zero/zero handling is not certified artifact-equivalent.',
};
/** Add model-specific guidance without changing core bounds, choices or reset semantics. */
export function geosimParams(params: import('./types').Param[]): import('./types').Param[] {
  return params.map(p=>({...p,help:FIELD_HELP[p.path] ?? p.help}));
}
