"""Strict JSON and readable presentation; never computes scientific judges."""
import json
import math


def serialize_report(report):
    return json.dumps(report, indent=2, allow_nan=False) + '\n'


def markdown_report(report):
    def cell(value):
        return 'Unavailable' if value is None else str(value).replace('|', '\\|').replace('\n', ' ')
    lines = ['# Democratic peace offline findings', '',
        f"Classification: {report['classification']}. Received {report['received_histories']}/{report['attempted_slots']} attempted history slots.", '',
        'Exact source equivalence remains Unresolved. Source curve readings and conditional reconstruction compatibility are distinct.', '',
        f"Source readings: {report['source']['readable_count']}/105 potential slots. All105 remain in multiplicity; density-zero clustering exclusions are outside that family.", '',
        '| Scope | Conditional verdict | Readable/potential | Reason |', '|---|---|---:|---|']
    for name, scope in report['source']['scopes'].items():
        lines.append(f"| {name} | {scope['conditional_verdict']} | {scope['readable_count']}/{scope['potential_count']} | {cell(scope['reason'])} |")
    lines += ['', '| Source target | Read status | Source envelope | Original mean | Original complete/registered | Original clustering defined/extinct | Predictive interval | Defined/attempted draws | Raw p | Holm p | Availability |',
        '|---|---|---|---:|---:|---:|---|---:|---:|---:|---|']
    for target in report['source']['targets']:
        original = target['original_census']
        prediction = target['result'] or {}
        values = [target['id'], target['read_status'], target['interval'], target['original'],
            f"{original['complete_count']}/{original['registered_count']}",
            f"{original['clustering_defined_count']}/{original['extinction_count']}",
            prediction.get('predictive_interval'),
            f"{prediction.get('defined_draws', 0)}/{prediction.get('attempted_draws', 0)}",
            target['p'], target['holm_p'], target['unavailable_reason'] or 'Available']
        lines.append('| ' + ' | '.join(cell(v) for v in values) + ' |')
    for family in ('primary_contrasts', 'secondary_contrasts'):
        group = report[family]
        lines += ['', f"{family}: separate six-test family; aggregate {group['aggregate_verdict']}.", '',
            '| Contrast | Verdict | Estimate | 95% whole-history interval | Raw p | Holm p | Reason |',
            '|---|---|---:|---|---:|---:|---|']
        for component in group['components']:
            values = [component['id'], component['verdict'], component['estimate'], component['interval'],
                component['p'], component['holm_p'], component['unavailable_reason'] or 'Available']
            lines.append('| ' + ' | '.join(cell(v) for v in values) + ' |')
    lines += ['', 'Missing, invalid and legitimately undefined histories remain in their censuses. Clustering predictions condition on defined resamples and preserve extinction draws. Original and precision histories are never pooled or seed-paired. Bootstrap support does not establish impossible model events; these lattice results do not establish historical causal democratic peace.', '']
    return '\n'.join(lines)
