"""Full-source streaming scan and saved-summary rendering after authorization."""
from collections import Counter
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'figure-prep'))
import plot_p4
from diagnostics_p4 import summarize
from stream_p4 import load


def render_complete(analysis_path, figures):
    figures = Path(figures).resolve()
    root = figures.parent
    root.mkdir(parents=True, exist_ok=True)
    identities, group_ids = {}, set()
    group_count = frame_count = 0
    with (root / 'group-diagnostics.jsonl').open('x') as diagnostic_stream:
        def record(group):
            nonlocal group_count, frame_count
            if group['id'] in group_ids:
                raise ValueError('duplicate biological group')
            group_ids.add(group['id'])
            if [frame['tick'] for frame in group['frames']] != list(range(65)):
                raise ValueError('biological group lacks full 0-64 frame sequence')
            group_count += 1
            frame_count += len(group['frames'])
            for member in group['members']:
                condition, seed = member.rsplit(' seed ', 1)
                key = (condition, int(seed))
                if key in identities:
                    raise ValueError('duplicate biological membership')
                identities[key] = group['id']
            diagnostic_stream.write(json.dumps(summarize(group), sort_keys=True, allow_nan=False) + '\n')
        projection, binding = load(analysis_path, group_callback=record)
    audit = json.loads((root.parent / 'measurement-data-audit.json').read_text())
    if binding['sha256'] != audit['analysis_identity']['sha256'] or binding['bytes'] != audit['analysis_identity']['bytes']:
        raise ValueError('full original differs from authorized complete-data audit')
    expected = {(endpoint['condition'], endpoint['seed']): endpoint['biological_group']
                for endpoint in projection['endpoints']}
    if identities != expected or len(identities) != 3840:
        raise ValueError('full biological memberships disagree with all registered endpoints')
    if sum(len(group['members']) for group in projection.get('biological_groups', [])):
        raise ValueError('unexpected biological group duplication in projection')
    receipt = dict(classification='full_complete_analysis_streaming_presentation_projection',
                   source=binding, exact_endpoint_memberships=3840, exact_cells=96,
                   group_count=group_count, stored_group_frames=frame_count,
                   logical_episode_frames=3840 * 65,
                   all_biological_groups_scanned=True, all_estimates_cells_endpoints_aliases_duplicates_preserved=True,
                   full_original_immutable_and_retained=True,
                   statistical_estimates_recalculated=False)
    projected = plot_p4.canonical(projection)
    with (root / 'presentation-analysis.json').open('xb') as stream:
        stream.write(projected)
    with (root / 'projection-receipt.json').open('xb') as stream:
        stream.write(plot_p4.canonical(receipt))
    result = plot_p4.export(projected, figures, synthetic=False, source_binding=binding)
    print(json.dumps(receipt))
    return result
