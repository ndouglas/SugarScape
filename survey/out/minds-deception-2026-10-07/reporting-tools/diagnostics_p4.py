"""Descriptive trace reduction, with full frames retained in the source analysis.

Counts distinguish choice, false remembered display amount, physical arrival,
empty inspection, body occupancy, gross raid, original-food lineage and life.
This module computes no paired effect estimate or inferential interval.
"""
from collections import Counter

TOTALS = ('harvest', 'dug', 'buried', 'effort', 'metabolic_demand', 'metabolic_consumed')


def summarize(group):
    frames = group['frames']
    condition = group['members'][0].rsplit(' seed ', 1)[0]
    mirrored = condition.endswith('-m1')
    source = dict(x=5 if mirrored else 3, y=3)
    display = dict(x=3 if mirrored else 5, y=6) if '-off-route-' in condition else dict(x=5 if mirrored else 3, y=5)
    actions = [dict(tick=frame['tick'], **item) for frame in frames for item in frame['actions']]
    choices = [dict(tick=frame['tick'], **item) for frame in frames for item in frame['choices']]
    observations = [dict(tick=frame['tick'], **item) for frame in frames for item in frame['observations']]
    receiver = [item for item in choices if item['actor'] == 2]
    display_choices = [item for item in receiver if item['target'] == display]
    source_choices = [item for item in receiver if item['target'] == source]
    owner = [item for item in actions if item['actor'] == 1]
    recovery_actions = [item for item in owner if item['dug'] > 0 and item['pos'] == source]
    recovery = [item['tick'] for item in recovery_actions]
    departure_ticks = [item['tick'] for item in owner if item['phase'] == 'departure']
    flag_ticks = [item['tick'] for item in owner if item['source_recovered']]
    recovery_frames = []
    for frame in frames:
        if frame['tick'] not in recovery:
            continue
        ordering = [action['actor'] for action in frame['actions']]
        recovery_frames.append(dict(frame,
            owner_occupies_source_at_frame_end=any(role['id'] == 1 and role['pos'] == source for role in frame['roles']),
            receiver_choices=[choice for choice in frame['choices'] if choice['actor'] == 2],
            owner_action_precedes_receiver_action=1 in ordering and 2 in ordering and ordering.index(1) < ordering.index(2)))
    return dict(id=group['id'], members=group['members'], frame_count=len(frames),
        ticks=[frame['tick'] for frame in frames], source=source, display=display,
        owner_totals={key: sum(item[key] for item in owner) for key in TOTALS},
        receiver_totals={key: sum(item[key] for item in actions if item['actor'] == 2) for key in TOTALS},
        action_counts=dict(sorted(Counter(f"{item['actor']}:{item['phase']}:{item['action']}" for item in actions).items())),
        walk_outcome_counts=dict(sorted(Counter(f"{item['actor']}:{item['walk_outcome']}" for item in actions if item['walk_outcome'] is not None).items())),
        cancellation_counts=dict(sorted(Counter(str(item['cancellation']) for item in actions if item['cancellation'] is not None).items())),
        restrictions=frames[-1]['restrictions'], owner_source_recovery_tick=min(recovery) if recovery else None,
        owner_post_recovery_departure_tick=min(departure_ticks) if departure_ticks else None,
        source_recovery_flag_first_tick=min(flag_ticks) if flag_ticks else None,
        physical_recovery_frames=recovery_frames,
        owner_source_occupancy_after_release_frames=[frame['tick'] for frame in frames if frame['tick'] >= 33
            and any(role['id'] == 1 and role['pos'] == source for role in frame['roles'])],
        display_actions=[item for item in owner if item['phase'] == 'display'],
        recovery_actions=recovery_actions,
        receiver_display_choices=len(display_choices),
        receiver_false_display_choices=sum(item['remembered_value'] > item['actual_value'] for item in display_choices),
        receiver_display_arrivals=sum(item['arrived'] for item in display_choices),
        receiver_occupied_display_choices=sum(item['target_occupant'] is not None for item in display_choices),
        receiver_wasted_display_inspections=sum(item['wasted'] for item in display_choices),
        receiver_display_gross_raid=sum(item['raid_amount'] for item in display_choices),
        receiver_source_gross_raid=sum(item['raid_amount'] for item in source_choices),
        receiver_first_source_arrival_tick=min((item['tick'] for item in source_choices if item['arrived']), default=None),
        receiver_display_choice_records=display_choices,
        receiver_source_choice_records=[item for item in source_choices if item['remembered_value'] > 0 or item['raid_amount'] > 0],
        observation_records=observations,
        death_records=[dict(tick=frame['tick'], **item) for frame in frames for item in frame['deaths']],
        full_roles_stocks_positions_action_choices='unfiltered originals retained in full analysis.json; this is an explicitly declared descriptive projection')
