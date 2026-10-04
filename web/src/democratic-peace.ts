import type { ColorMode, DemocraticPeaceConfig, DemocraticPeaceInspection, DemocraticPeaceRatio, DemocraticPeaceStateId, Param } from './types';

export const DEMOCRATIC_PEACE_RULES = 'Cederman 2001 reconstruction: tagging, defensive alliances and collective security. Democratic nonattack is partly stipulated; it does not establish learned cooperation or a historical causal result. The primary probability decreases with own advantage as printed on pp.496–497 and 499; the separately named prose alternative increases. Reconstruction choices are explicit and unmeasured unless an external scientific artifact says otherwise. Battles count territorial contests, not casualties. UI events are bounded; full counters are separate.';

const HELP: Record<string, string> = {
  width: 'Bounded lattice width in cells; 15 in the 2001 experiments. Every cell starts independently sovereign.',
  height: 'Bounded lattice height in cells; 15 in the 2001 experiments. There is no growth or warmup phase.',
  initial_democratic_share: 'Configured democratic assignment probability (default .1). Clustering divides by this configured share, not realized setup density; zero makes clustering undefined.',
  initial_resourced_share: 'Assignment probability for initial resources 10 (default .05); all other cells start with 0. Regimes are assigned before resources.',
  mobile_share: 'Contingent mobile commitment share, default .5. Front plans are conditional, not an additive spending budget.',
  superiority_threshold: 'Own/opponent commitment ratio with attack probability .5; default 2.2. Printed probability decreases above this ratio.',
  superiority_exponent: 'Attack probability steepness, default 30 in Cederman 2001.',
  victory_threshold: 'Both sides use the same victory threshold, default 2.2. No 2003 reciprocal defender threshold or geographic combat projection.',
  victory_exponent: 'Victory probability steepness, default 10 in Cederman 2001.',
  stalemate_probability: 'Probability of terminating mutual attack when neither victory succeeds; default .2.',
  tax_rate: 'Province extraction multiplier, default 1. Resources update to 10 plus distance-discounted provincial yield; no battle damage.',
  distance_gradient: 'Distance discount base, default .95. Capital contributes 10 once; each province contributes 10 × tax rate × gradient^distance.',
  min_threat: 'Incoming/outgoing commitment ratio must be strictly greater than this threshold, default 2, to qualify as a threat.',
  horizon_periods: 'Complete atomic source periods, default 1000. Extinction, all-democratic occupation and quiet periods do not stop the clock early.',
  periods_per_tick: 'Complete source periods grouped into a display tick. The final display tick may contain fewer periods.',
  event_recording: 'Retain a bounded optional UI trace. Exact battle and structural counters remain available without recording.',
  event_limit: 'Maximum UI trace entries; additional entries are counted as dropped. This limit does not bound scientific counters.',
  mechanism: 'Tagging forbids democratic initiation against democracies. Alliances adds threat-specific defense; collective_security also adds obligations against pariahs (Cederman 2001 pp.484,499).',
  probability_direction: 'Printed pp.496–497 and 499: 1/(1+(own/opponent ÷ threshold)^exponent), decreasing with own advantage. prose_increasing uses the opposite direction; neither is silently substituted.',
  zero_ratio: 'equal_zero_neutral defines 0/0 probability as .5 and threat balance as 1; positive/zero uses an analytic limit. reject_zero_denominator invalidates the attempted period when a denominator is zero. These are explicit reconstruction choices.',
  assignment: 'independent_bernoulli draws per cell for regimes then resources. rounded_quota independently shuffles both assignments and rounds the desired counts. Stream semantics are reconstructed, not recovered RePast execution.',
  distance_metric: 'Euclidean is the primary extraction reading. Manhattan and shortest within-territory path are named reconstruction alternatives.',
  enemy_total: 'active_fronts sums previous incoming commitments on fronts with previous D; all_fronts includes every eligible external front. Democratic–democratic fronts are ineligible.',
  inactive_commitment: 'per_front_opponent uses each inactive front’s own opposing commitment. first_inactive_opponent repeats the lowest-ID inactive term, exposing the anomalous worked example on p.495.',
  latent_regime: 'persistent_cell_tags keeps a province’s regime tag under occupation and on release. overwrite_on_conquest changes absorbed tags to the winner’s regime. Governing regime always follows the current sovereign capital.',
  capital_capture: 'collapse_only releases compound capital and provinces independently. capture_and_fragment absorbs the capital and releases other cells; this is a separately named reconstruction reading.',
  claim_locking: 'affected_cells locks every changed or released cell plus the path agent. affected_states instead locks affected and newly created sovereign identities.',
  opposing_victories: 'independent_claims tests both sides separately and resolves successful claims structurally. single_draw normalizes the two probabilities into one exclusive test.',
  alliance_maintenance: 'rebuild_each_period forms threat groups anew. persist_while_threatened retains still-adjacent, still-threatened members and admits unattached actors; both are reconstructed algorithms.',
  threat_ties: 'lowest_state_id resolves exactly equal ratios deterministically. random_tie uniformly selects among exact ties, including matching positive-over-zero tags.',
  obligation_observation: 'prior_actions observes last period’s attacks. current_plans_once observes a fixed baseline plan and adds obligations once, without recursive cascades.',
  security_scope: 'all_democracies follows p.484 and p.499 pseudocode. same_alliance follows p.499’s paragraph, restricting security to shared defensive-alliance membership.',
  clustering_exposure: 'unique_state_neighbors divides democratic neighboring sovereigns by all sovereign neighbors. border_edges instead weights territorial border edges. A sole democratic state has exposure 1.',
  clustering_weights: 'territory_weighted weights democratic exposure by state cell counts. equal_states averages states equally. Extinction and initial density zero retain explicit undefined reasons.',
};

/** Preserve core bounds, choices and reset semantics while explaining each source reading. */
export function democraticPeaceParams(params: Param[]): Param[] {
  return params.map((param) => ({ ...param, help: [param.help, HELP[param.path]].filter(Boolean).join(' ') }));
}

export function democraticPeaceLegend(config: DemocraticPeaceConfig, mode: ColorMode, periods: number): string {
  const meaning: Partial<Record<ColorMode, string>> = {
    territory: 'Ownership: varied hues = sovereign ownership (hues can repeat; Inspect identifies the state); white marks = capitals',
    governing_regime: 'Regime of the current sovereign capital: teal = democratic; coral = predatory',
    latent_regime: 'Cell regime tag used on independence: teal = democratic; coral = predatory; occupation can have a different governing regime',
    resources: 'Current state resources: dark blue = 0; bright green = 100 or more; intermediate shades show values between 0 and 100',
    alliances: 'Alliances: varied hues = defensive alliance membership by shared named threat (hues can repeat; Inspect identifies the alliance); gray = unaligned; pooled deterrence is distinct from combat commitments',
    pariahs: 'Pariahs: red = pariah; muted gray-teal = unmarked; identified from democratic–predatory conflict under the selected observation reading',
  };
  return `${periods} of ${config.horizon_periods} source periods · ${config.periods_per_tick} periods/tick · Cederman 2001 · ${config.mechanism} · ${config.probability_direction} · ${meaning[mode] ?? meaning.territory}`;
}

export function democraticPeaceChartCaption(title: string, configs: DemocraticPeaceConfig[]): string {
  return `${title} · ${configs.map((config, index) => `${configs.length > 1 ? `${index === 0 ? 'A' : 'B'}: ` : ''}${config.periods_per_tick} source periods/tick`).join(' · ')} · exploratory`;
}

const finite = (value: number | null, reason?: string | null): string => value !== null && Number.isFinite(value) ? value.toFixed(3) : `unavailable${reason ? ` (${reason})` : ''}`;
const identity = (id: DemocraticPeaceStateId): string => `${id.capital_cell} (generation ${id.sovereignty_generation})`;
const actions = (pair: boolean[]): string => pair.map((action) => action ? 'D' : 'C').join('/');
const ratio = (value: DemocraticPeaceRatio | null): string => value ? `${value.tag} (${finite(value.numerator)} / ${finite(value.denominator)})${value.value === null ? '' : ` = ${finite(value.value)}`}` : 'not tested';

/** Explain local decisions first; retain complete front diagnostics and exact counters below. */
export function democraticPeaceRows(view: DemocraticPeaceInspection): [string, string][] {
  const metrics = view.metrics;
  const rows: [string, string][] = [
    ['Source clock', `${view.periods} completed of ${view.horizon_periods} source periods · attempted ${view.attempted_period} · ${view.last_tick_periods} in last display tick`],
    ['Government', `Cell ${view.cell.id} · capital ${identity(view.state.id)} · ${view.members.length} territory cells`],
    ['Regimes', `governing ${view.state.regime} · latent ${view.cell.latent_regime} · initial ${view.cell.initial_regime}`],
    ['Territory', view.members.join(', ')],
    ['Resources', `${finite(view.state.resources)} · own-capital distance ${finite(view.distance)} · each update replaces resources with 10 + 10 × tax rate × sum(distance gradient^province distance)`],
    ['Last extraction', view.extraction ? `${finite(view.extraction.resources_before)} → ${finite(view.extraction.resources_after)}; province terms ${view.extraction.province_terms.map(([cell, distance, term]) => `${cell}: distance ${finite(distance)}, discount ${finite(term)}`).join('; ') || 'none'}` : 'No extraction yet'],
    ['Probability reading', `${view.config.probability_direction} · ratio is own/opponent commitment · simultaneous paths use ${view.path_priority}`],
    ['Democratic territory', `${metrics.democratic_cells}/${metrics.total_cells} cells · share ${finite(metrics.democratic_share)}`],
    ['State sizes', `Democratic mean ${finite(metrics.democratic_mean_size, metrics.democratic_size_reason)}, maximum ${finite(metrics.democratic_max_size, metrics.democratic_size_reason)} · predatory mean ${finite(metrics.predatory_mean_size, metrics.predatory_size_reason)}, maximum ${finite(metrics.predatory_max_size, metrics.predatory_size_reason)}`],
    ['Clustering', `Exposure ${finite(metrics.democratic_exposure, metrics.clustering_reason)} · ratio ${finite(metrics.clustering_ratio, metrics.clustering_reason)}`],
    ['First passages', `extinction ${metrics.first_extinction_period ?? 'not observed'} · all-democratic ${metrics.first_all_democratic_period ?? 'not observed'}`],
    ['Fronts', view.fronts.map((front) => `${front.states.map(identity).join(' — ')}: current ${actions(front.actions)}, previous ${actions(front.previous)} · commitments ${front.commitments.map((value) => finite(value)).join('/')} (old ${front.old_commitments.map((value) => finite(value)).join('/')}) · initiation ${front.initiations.join('/')} (previous ${front.previous_initiations.join('/')}) · obligations ${front.obligations.map((reasons) => reasons.join(', ') || 'none').join('/')} · attack ratios ${front.attack_ratios.map(ratio).join(' | ')} · attack probabilities ${front.attack_probabilities.map((value) => finite(value)).join('/')} · victory ratios ${front.victory_ratios.map(ratio).join(' | ')} · victory probabilities ${front.victory_probabilities.map((value) => finite(value)).join('/')} · path ${front.path?.join(' → ') ?? 'none'} (proposer ${front.path_proposer ? identity(front.path_proposer) : 'none'}) · claims ${front.claims.join('/')}`).join('; ') || 'None'],
    ['Conditional allocation', view.fronts.flatMap((front) => front.allocations.map((allocation, side) => `${identity(front.states[side])}: ${allocation.eligible_fronts} eligible fronts · fixed ${finite(allocation.fixed)} · mobile pool ${finite(allocation.mobile_pool)} · old opponent ${finite(allocation.old_opposing)} · enemy total ${finite(allocation.enemy_total)} · inactive term ${finite(allocation.inactive_term)} · ${allocation.active ? 'active' : 'inactive'}`)).join('; ') || 'No eligible front plans'],
    ['Defensive alliances', view.alliances.map((alliance) => `Threat ${identity(alliance.threat_id)} · created ${alliance.creation_period} / serial ${alliance.serial} · members ${alliance.members.map(identity).join(', ')} · pooled deterrence ${finite(alliance.pooled_resources)}`).join('; ') || 'None'],
    ['Pariah sources', view.pariah_sources.map(identity).join(', ') || 'None'],
    ['Battle and structure census', Object.entries(view.census).map(([name, count]) => `${name.replaceAll('_', ' ')} ${count}`).join(' · ')],
    ['Last structural event', view.last_structural_event ? `Period ${view.last_structural_event.period} · ${view.last_structural_event.kind} · cells ${view.last_structural_event.cells.join(', ')}` : 'None'],
    ['UI event trace', `${view.events.length} retained · ${view.events_dropped} dropped · bounded trace; exact counters remain separate`],
  ];
  if (view.invalidity) rows.push(['Invalid reconstruction', `${view.invalid_phase ?? 'unknown phase'}: ${view.invalidity}; last committed state retained, final metrics unavailable`]);
  else if (view.finish_reason) rows.push(['Completion', `${view.finish_reason} · all ${view.periods} source periods executed`]);
  return rows;
}
