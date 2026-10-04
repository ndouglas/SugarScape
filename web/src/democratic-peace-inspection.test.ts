import { describe, expect, it } from 'vitest';
import * as presentation from './democratic-peace';
import type { DemocraticPeaceConfig, DemocraticPeaceMetrics } from './types';
const id = { capital_cell: 4, sovereignty_generation: 2 };
const opponent = { capital_cell: 9, sovereignty_generation: 0 };
const allocation = { fixed: 1, mobile_pool: 2, eligible_fronts: 1, old_opposing: 0, enemy_total: 0, inactive_term: 0, active: false };
const metrics: DemocraticPeaceMetrics = { democratic_cells: 0, total_cells: 16, democratic_share: 0, sovereign_count: 16, democratic_states: 0, predatory_states: 16, democratic_mean_size: null, predatory_mean_size: 1, democratic_max_size: null, predatory_max_size: 1, democratic_size_reason: 'no_surviving_states', predatory_size_reason: null, democratic_exposure: null, clustering_ratio: null, clustering_reason: 'undefined_extinction', conflict_fronts: 1, alliance_count: 0, pariah_count: 1, democratic_extinction: true, all_democratic: false, first_extinction_period: 0, first_all_democratic_period: null };
const fixture = {
  model: 'democratic_peace', config: { horizon_periods: 14, periods_per_tick: 3, probability_direction: 'printed_decreasing' } as DemocraticPeaceConfig,
  cell: { id: 4, owner: id, initial_regime: 'predatory', latent_regime: 'democratic', next_generation: 3 },
  state: { id, regime: 'predatory', resources: 0, members: [4] }, members: [4], distance: 0, extraction: null,
  fronts: [{ states: [id, opponent], previous: [false, true], actions: [true, true], old_commitments: [0, 0], commitments: [5, 0], allocations: [allocation, allocation], initiations: [true, false], previous_initiations: [false, false], obligations: [['attacked_ally'], []], path: [4, 9], path_proposer: id, attack_probabilities: [0, null], attack_ratios: [{ numerator: 5, denominator: 0, tag: 'positive_over_zero', value: null }, null], victory_probabilities: [0, 1], victory_ratios: [null, null], claims: [false, true] }],
  alliances: [], pariah_sources: [opponent], metrics, setup: { initial_democratic_cells: 1, initial_resourced_cells: 0, total_cells: 16 },
  periods: 2, completed_periods: 2, attempted_period: 3, last_tick_periods: 2, horizon_periods: 14,
  finish_reason: 'invalid', invalidity: 'zero denominator', invalid_phase: 'allocation', outcome: null,
  census: { initiated_fronts: 1, mutual_d_front_periods: 1, completed_victory_battles: 0, completed_stalemate_battles: 0, opposing_claims: 0, successful_claims: 0, stale_claims: 0, locked_claims: 0, retired_states: 0, released_states: 0 },
  last_structural_event: null, events: [], events_dropped: 5, path_priority: 'lower_id_path_priority', agent: null,
};
const rows = () => {
  const fn = (presentation as unknown as { democraticPeaceRows?: (view: unknown) => [string, string][] }).democraticPeaceRows;
  expect(fn).toBeTypeOf('function');
  return fn!(fixture);
};
describe('Democratic peace Inspect explanations', () => {
  it('distinguishes governing, latent and initial regimes while retaining valid zero resources', () => {
    const table = Object.fromEntries(rows());
    expect(table.Regimes).toContain('governing predatory');
    expect(table.Regimes).toContain('latent democratic');
    expect(table.Resources).toContain('0.000');
    expect(table.Government).toContain('generation 2');
  });
  it('shows undefined availability reasons and zero first-passage separately from absent first-passage', () => {
    const table = Object.fromEntries(rows());
    expect(table.Clustering).toContain('undefined_extinction');
    expect(table['State sizes']).toContain('no_surviving_states');
    expect(table['First passages']).toContain('extinction 0');
    expect(table['First passages']).toContain('all-democratic not observed');
  });
  it('shows commitments, probability ratio limits, obligations and failed atomic clocks', () => {
    const table = Object.fromEntries(rows());
    expect(table.Fronts).toContain('positive_over_zero');
    expect(table.Fronts).toContain('attacked_ally');
    expect(table['Invalid reconstruction']).toContain('allocation');
    expect(table['Source clock']).toContain('2 completed');
    expect(table['Source clock']).toContain('attempted 3');
    expect(JSON.stringify(rows())).not.toContain('Infinity');
  });
});
