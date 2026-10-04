"""Two prespecified six-test families, with independent whole-history groups."""
from .manifest import contrast_definitions, job_rng
from . import numerics
from .source import population_summary


def verdict(estimate, adjusted):
    if estimate is None or adjusted is None:
        return 'Unresolved'
    if adjusted < .05 and estimate > 0:
        return 'Holds'
    if adjusted < .05 and estimate < 0:
        return 'Fails'
    return 'Inconclusive'


def aggregate(components):
    outcomes = [component['verdict'] for component in components]
    if 'Fails' in outcomes:
        return 'Fails'
    if all(outcome == 'Holds' for outcome in outcomes):
        return 'Holds'
    if 'Unresolved' in outcomes:
        return 'Unresolved'
    return 'Inconclusive'


def findings(histories, jobs, *, precision_registered, draws=100000):
    families = {'primary': [], 'secondary': []}
    for definition in contrast_definitions():
        left = histories.get(definition['left'], [])
        right = histories.get(definition['right'], [])
        left_census = population_summary(left, 100)
        right_census = population_summary(right, 100)
        reason = 'precision_not_registered' if not precision_registered else None
        if reason is None and (not left_census['population_complete'] or not right_census['population_complete']):
            reason = 'incomplete_precision_history_population'
        result = None
        if reason is None:
            metric = definition['metric']
            result = numerics.contrast([h['metrics'][metric] for h in left],
                [h['metrics'][metric] for h in right],
                job_rng(jobs['permutation.' + definition['id']]),
                job_rng(jobs['bootstrap.' + definition['id']]), draws=draws)
        families[definition['family']].append({**definition,
            'left_census': left_census, 'right_census': right_census,
            'estimate': None if result is None else result['estimate'],
            'p': None if result is None else result['p'], 'holm_p': None,
            'interval': None if result is None else result['interval'],
            'result': result, 'unavailable_reason': reason})
    output = {}
    for family, components in families.items():
        for component, adjusted in zip(components, numerics.fixed_holm([c['p'] for c in components], 6)):
            component['holm_p'] = adjusted
            component['verdict'] = verdict(component['estimate'], adjusted)
        output[family] = {'family_size': 6, 'components': components,
            'aggregate_verdict': aggregate(components), 'source_direction': 'positive',
            'interpretation': 'conditional_model_mechanism_not_historical_causal_inference'}
    return output
