"""Fixed figure slots, honest source availability and conditional comparisons."""
import math
from .manifest import MECHANISMS, DENSITIES, arm_suffix, number_id
from . import numerics

PAPER_SHA256 = '069ad22da938727b2020c5d2c3a52aa383687c74085f5bc0af8cea74817b5215'
READ_STATUSES = {'readable', 'unreadable_overlap', 'absent_source_point'}
SLOT_FIELDS = {'id', 'figure', 'printed_page', 'pdf_page_1based', 'mobile_share',
    'mechanism', 'initial_democratic_share', 'metric', 'read_status', 'interval',
    'attribution', 'pixel_reading', 'calibration_id', 'note'}


def slot_roster():
    result = []
    for figure, page, mobile, metric, densities in (
        (9, 486, .5, 'democratic_share', DENSITIES),
        (10, 488, .5, 'clustering_ratio', DENSITIES[1:]),
        (11, 489, .85, 'democratic_share', DENSITIES)):
        for mechanism in MECHANISMS:
            for density in densities:
                result.append({'id': f'fig{figure}.{mechanism}.density{number_id(density)}',
                    'figure': figure, 'printed_page': page, 'pdf_page_1based': page - 469,
                    'mobile_share': mobile, 'mechanism': mechanism,
                    'initial_democratic_share': density, 'metric': metric})
    return result


def structural_exclusions():
    return [{'id': f'fig10.{mechanism}.density0', 'figure': 10,
        'mechanism': mechanism, 'initial_democratic_share': 0.,
        'read_status': 'undefined_density_zero', 'interval': None,
        'in_family': False, 'reason': 'ratio_divides_by_initial_density'}
        for mechanism in MECHANISMS]


def fixture_table(read_status='readable'):
    slots = [{**slot, 'read_status': read_status,
        'interval': [0., 1.] if read_status == 'readable' else None,
        'attribution': 'synthetic_fixture', 'pixel_reading': None,
        'calibration_id': f"fig{slot['figure']}", 'note': 'Synthetic test only'}
        for slot in slot_roster()]
    return {'schema_version': 1, 'model': 'democratic_peace',
        'paper': 'papers/geopolitics/cederman-2001-jcr-democratic-peace-kantian-selection-process.pdf',
        'paper_sha256': PAPER_SHA256, 'source_equivalence': 'Unresolved',
        'extraction': {'classification': 'synthetic_fixture'}, 'figures': [],
        'slots': slots, 'structural_exclusions': structural_exclusions()}


def validate_table(table):
    fields = {'schema_version', 'model', 'paper', 'paper_sha256', 'source_equivalence',
        'extraction', 'figures', 'slots', 'structural_exclusions'}
    if set(table) != fields or type(table['schema_version']) is not int or table['schema_version'] != 1 or table['model'] != 'democratic_peace':
        raise ValueError('unknown or missing source table fields')
    if table['paper_sha256'] != PAPER_SHA256 or table['source_equivalence'] != 'Unresolved':
        raise ValueError('source identity or equivalence changed')
    roster = slot_roster()
    if len(table['slots']) != 105:
        raise ValueError('fixed source table requires 105 slots')
    from .records import _same_json_types
    for slot, expected in zip(table['slots'], roster):
        if set(slot) != SLOT_FIELDS or any(not _same_json_types(slot[k], v) for k, v in expected.items()):
            raise ValueError('source slot roster, ordering or fields changed')
        if slot['read_status'] not in READ_STATUSES:
            raise ValueError('invalid source read status')
        interval = slot['interval']
        if slot['read_status'] != 'readable':
            if interval is not None:
                raise ValueError('unreadable or absent source slot has numeric target')
        else:
            maximum = 1. if slot['metric'] == 'democratic_share' else 1 / slot['initial_democratic_share']
            if not isinstance(interval, list) or len(interval) != 2 or any(
                type(x) not in (int, float) or not math.isfinite(x) for x in interval
            ) or not 0 <= interval[0] <= interval[1] <= maximum:
                raise ValueError('invalid source extraction envelope')
    if table['structural_exclusions'] != structural_exclusions():
        raise ValueError('density-zero clustering exclusions changed')
    if table['extraction'].get('classification') != 'synthetic_fixture':
        validate_extraction_metadata(table)
    return table


def validate_extraction_metadata(table):
    figures = table['figures']
    if [f['id'] for f in figures] != ['fig9', 'fig10', 'fig11']:
        raise ValueError('missing figure calibration')
    for figure in figures:
        for key in ('image_path', 'image_sha256', 'image_size', 'render_command',
                    'axis_landmarks', 'axis_uncertainty_pixels', 'traces', 'independent_visual_check'):
            if key not in figure:
                raise ValueError(f'missing source calibration {key}')
        if not figure['independent_visual_check']:
            raise ValueError('source calibration lacks visual audit')
    for slot in table['slots']:
        if slot['read_status'] == 'readable' and (
            slot['pixel_reading'] is None or slot['attribution'] != 'inferred_curve_value'
        ):
            raise ValueError('readable curve value lacks pixels or honest attribution')


def validate_registered_table(table):
    validate_table(table)
    if table['extraction'].get('classification') != 'visually_audited_premeasurement' or any(
        not isinstance(figure['independent_visual_check'], dict) or
        figure['independent_visual_check'].get('second_reader_status') != 'reviewed'
        for figure in table['figures']
    ):
        raise ValueError('registered source audit is synthetic or lacks independent review')
    return table


def population_summary(histories, expected):
    complete = [h for h in histories if h['complete']]
    survivors = [h for h in complete if h['metrics']['clustering_ratio'] is not None]
    result = {'registered_count': expected, 'received_count': len(histories),
        'complete_count': len(complete), 'invalid_count': len(histories) - len(complete),
        'missing_count': max(0, expected - len(histories)),
        'extinction_count': sum(h['metrics']['democratic_extinction'] for h in complete),
        'clustering_defined_count': len(survivors),
        'clustering_undefined_count': len(complete) - len(survivors),
        'population_complete': len(histories) == expected and len(complete) == expected,
        'democratic_share_mean': None, 'clustering_ratio_mean': None,
        'all_democratic_probability': None, 'extinction_probability': None}
    if result['population_complete']:
        result['democratic_share_mean'] = math.fsum(h['metrics']['democratic_share'] for h in complete) / expected
        result['all_democratic_probability'] = sum(h['metrics']['all_democratic'] for h in complete) / expected
        result['extinction_probability'] = result['extinction_count'] / expected
        if survivors:
            result['clustering_ratio_mean'] = math.fsum(h['metrics']['clustering_ratio'] for h in survivors) / len(survivors)
    return result


def scope_verdict(targets):
    readable = [t for t in targets if t['read_status'] == 'readable']
    if any(t['holm_p'] is not None and t['holm_p'] < .05 for t in readable):
        return {'conditional_verdict': 'Incompatible', 'reason': None}
    if not readable:
        return {'conditional_verdict': 'Unresolved', 'reason': 'no_readable_source_targets'}
    if all(t['p'] is not None for t in readable):
        return {'conditional_verdict': 'Compatible', 'reason': None}
    return {'conditional_verdict': 'Unresolved', 'reason': 'ineligible_readable_source_targets'}


def source_findings(table, histories, jobs, *, precision_registered, draws=100000, minimum_defined=10000, fixture=False):
    from .manifest import job_rng
    validate_table(table)
    if fixture:
        if type(draws) is not int or not 1 <= draws <= 1000:
            raise ValueError('synthetic source fixture exceeds bounded draw gate')
    else:
        validate_registered_table(table)
        if draws != 100000 or minimum_defined != 10000:
            raise ValueError('registered source resampling contract changed')
    summaries = {arm: population_summary(values, 30 if arm.startswith('original.') else 100)
                 for arm, values in histories.items()}
    targets = []
    for slot in table['slots']:
        suffix = arm_suffix(slot['mobile_share'], slot['mechanism'], slot['initial_democratic_share'])
        original = summaries.get('original.' + suffix, population_summary([], 30))
        precision = summaries.get('precision.' + suffix, population_summary([], 100))
        metric = slot['metric']
        target = {**slot, 'source_equivalence': 'Unresolved',
            'original': original[metric + '_mean'], 'original_census': original,
            'precision_census': precision if precision_registered else None,
            'result': None, 'p': None, 'holm_p': None, 'unavailable_reason': None}
        reason = None
        if slot['read_status'] != 'readable':
            reason = slot['read_status']
        elif not precision_registered:
            reason = 'precision_not_registered'
        elif not original['population_complete']:
            reason = 'incomplete_original_history_population'
        elif not precision['population_complete']:
            reason = 'incomplete_precision_history_population'
        else:
            values = [h['metrics'][metric] for h in histories['precision.' + suffix]]
            prediction = numerics.predictive(values, job_rng(jobs['predictive.source.' + slot['id']]),
                draws=draws, minimum_defined=minimum_defined)
            target['result'] = {k: v for k, v in prediction.items() if k != 'replicates'}
            target['result']['conditional'] = metric == 'clustering_ratio'
            if prediction['status'] != 'Available':
                reason = prediction['reason']
            else:
                target['result'].update(numerics.maximize_interval_p(prediction['replicates'], slot['interval']))
                target['p'] = target['result']['p']
        target['unavailable_reason'] = reason
        targets.append(target)
    for target, adjusted in zip(targets, numerics.fixed_holm([t['p'] for t in targets], 105)):
        target['holm_p'] = adjusted
    scopes = {'all': scope_verdict(targets)}
    for figure in (9, 10, 11):
        scopes[f'fig{figure}'] = scope_verdict([t for t in targets if t['figure'] == figure])
    for name in scopes:
        subset = targets if name == 'all' else [t for t in targets if 'fig' + str(t['figure']) == name]
        scopes[name].update(source_equivalence='Unresolved', readable_count=sum(t['read_status'] == 'readable' for t in subset), potential_count=len(subset))
    return {'family_size': 105, 'readable_count': sum(t['read_status'] == 'readable' for t in targets),
        'targets': targets, 'scopes': scopes, 'populations': summaries,
        'structural_exclusions': table['structural_exclusions'],
        'finite_support_limit': 'outside_bootstrap_values_are_not_impossible_model_events'}
