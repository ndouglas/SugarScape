"""All-cell descriptive diagnostic distributions; no paired re-estimation."""
from collections import Counter, defaultdict
import json
from pathlib import Path

root = Path('.')
projection = json.loads((root / 'presentation-analysis.json').read_text())
endpoint = {(item['condition'], item['seed']): item for item in projection['endpoints']}
groups = {}
for line in (root / 'group-diagnostics.jsonl').open():
    item = json.loads(line)
    if item['id'] in groups:
        raise ValueError('duplicate diagnostics group')
    groups[item['id']] = item
cells = []
for cell in projection['cells']:
    ident = cell['condition']['id']
    records = [groups[endpoint[ident, seed]['biological_group']] for seed in cell['seeds']]
    assert len(records) == 40
    def distribution(getter):
        counter = Counter(json.dumps(getter(item), sort_keys=True) for item in records)
        return [dict(value=json.loads(value), count=count) for value, count in sorted(counter.items())]
    observation_patterns = distribution(lambda item: item['observation_records'])
    result = dict(condition=cell['condition'], registered_seed_count=40,
        biological_groups=len(set(cell['biological_groups'])),
        lifetime_distribution=[dict(value=value,count=count) for value,count in sorted(Counter(endpoint[ident,seed]['owner_ticks_alive'] for seed in cell['seeds']).items())],
        original_food_transfer_distribution=[dict(value=value,count=count) for value,count in sorted(Counter(endpoint[ident,seed]['thief_transferred'] for seed in cell['seeds']).items())],
        owner_alive_at_horizon=sum(endpoint[ident,seed]['owner_alive'] for seed in cell['seeds']),
        episodes_with_false_display_choice=sum(item['receiver_false_display_choices'] > 0 for item in records),
        episodes_with_display_target=sum(item['receiver_display_choices'] > 0 for item in records),
        episodes_with_display_arrival=sum(item['receiver_display_arrivals'] > 0 for item in records),
        episodes_with_occupied_display_target=sum(item['receiver_occupied_display_choices'] > 0 for item in records),
        episodes_with_wasted_display_inspection=sum(item['receiver_wasted_display_inspections'] > 0 for item in records),
        episodes_with_owner_source_recovery=sum(item['owner_source_recovery_tick'] is not None for item in records),
        owner_total_distributions={key:distribution(lambda item,key=key: item['owner_totals'][key]) for key in
            ('harvest','dug','buried','effort','metabolic_demand','metabolic_consumed')},
        receiver_total_distributions={key:distribution(lambda item,key=key: item['receiver_totals'][key]) for key in
            ('harvest','dug','buried','effort','metabolic_demand','metabolic_consumed')},
        source_recovery_ticks=distribution(lambda item:item['owner_source_recovery_tick']),
        post_recovery_departure_ticks=distribution(lambda item:item['owner_post_recovery_departure_tick']),
        source_recovery_flag_first_ticks=distribution(lambda item:item['source_recovery_flag_first_tick']),
        source_owner_occupancy_after_release_frames=distribution(lambda item:item['owner_source_occupancy_after_release_frames']),
        physical_recovery_frame_records=distribution(lambda item:item['physical_recovery_frames']),
        receiver_first_source_arrival_ticks=distribution(lambda item:item['receiver_first_source_arrival_tick']),
        restrictions=distribution(lambda item:item['restrictions']),
        walk_outcomes=distribution(lambda item:item['walk_outcome_counts']),
        cancellations=distribution(lambda item:item['cancellation_counts']),
        action_counts=distribution(lambda item:item['action_counts']),
        display_actions=distribution(lambda item:item['display_actions']),
        observation_patterns=observation_patterns,
        lineage_cohort_distributions=[dict(value=json.loads(value),count=count) for value,count in sorted(Counter(json.dumps(endpoint[ident,seed]['cohorts'],sort_keys=True) for seed in cell['seeds']).items())],
        episode_wall_seconds=sum(cell['wall_seconds']))
    cells.append(result)
assert len(cells) == 96
result = dict(classification='actual_complete_per_cell_descriptive_diagnostics_no_paired_reestimation',
    source=projection['presentation_projection'], biological_groups=len(groups),
    endpoint_coverage=len(endpoint), cell_coverage=len(cells),
    actual_repeated_endpoints=projection['repeated_endpoints'], actual_effective_aliases=projection['effective_aliases'],
    every_cell_all40_seeds_retained=True,cells=cells)
Path('cell-diagnostics.json').write_text(json.dumps(result,indent=2,sort_keys=True,allow_nan=False)+'\n')
print(json.dumps(dict(cells=len(cells),endpoints=len(endpoint),biological_groups=len(groups),
    repeated_endpoint_groups=len(projection['repeated_endpoints']),effective_alias_groups=len(projection['effective_aliases']))))
for item in cells:
    ident=item['condition']['id']
    if ident.endswith('m0') and (ident.startswith('sham-') or ident.startswith('matched-neutral-ambiguous-seen-off-route') or ident=='ordinary-ambiguous-seen-off-route-cost0-m0'):
        print(json.dumps({key:item[key] for key in ('condition','lifetime_distribution','original_food_transfer_distribution',
            'episodes_with_false_display_choice','episodes_with_display_arrival','episodes_with_occupied_display_target',
            'episodes_with_wasted_display_inspection','episodes_with_owner_source_recovery','owner_total_distributions',
            'source_recovery_ticks','receiver_first_source_arrival_ticks','restrictions','cancellations')}))
