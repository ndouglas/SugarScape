"""Strict compact exports; presentation never calculates scientific judges."""
import json
import math
import numpy as np

PRIVATE_FIELDS={'replicates','raw_sizes','logpdf','eligibility_counts','refitted_xmins'}


def _json_value(value,path='$'):
    if isinstance(value,dict):return {k:_json_value(v,f'{path}.{k}') for k,v in value.items() if k not in PRIVATE_FIELDS}
    if isinstance(value,(list,tuple)):return [_json_value(v,f'{path}[{i}]') for i,v in enumerate(value)]
    if isinstance(value,np.ndarray):return _json_value(value.tolist(),path)
    if isinstance(value,np.generic):return _json_value(value.item(),path)
    if isinstance(value,float) and not math.isfinite(value):raise ValueError(f'nonfinite report value at {path}')
    return value


def serialize_report(report):
    return json.dumps(_json_value(report),indent=2,allow_nan=False)+'\n'


def markdown_report(report):
    def cell(value):
        if value is None:return 'Unavailable'
        return str(value).replace('|','\\|').replace('\n',' ')
    lines=['# GeoSim offline findings','',f"Classification: {report['classification']}. Received {report['received_histories']} / {report['registered_histories']} registered history slots.",
           '', 'Source equivalence remains Unresolved where the printed severity scale/range/count definitions are unverified. Conditional reconstruction compatibility is distinct.',
           '', '| Source arm | Conditional verdict | Source equivalence | Original joint eligible | Precision joint eligible |',
           '|---|---|---|---:|---:|']
    for name,a in report['source']['arms'].items():
        lines.append(f"| {name} | {a['conditional_verdict']} | {a['source_equivalence']} | {a['original_joint_eligible']}/15 | {a['precision_joint_eligible']}/100 |")
    lines+=['','Fixed source family: 88 targets; mechanism contrasts: six targets. Unavailable slots remain in multiplicity. Bootstrap extrema have finite support, not impossible-model-event meaning.',
            '', '| Source target | Printed rounding interval | Original estimate | Predictive 95% interval | Raw p | Holm p | Availability reason |',
            '|---|---|---:|---|---:|---:|---|']
    for t in report['source']['targets']:
        values=(t['id'],t['source_interval'],t['original'],(t['result'] or {}).get('predictive_interval'),t['p'],t['holm_p'],t['unavailable_reason'])
        lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','| Source contrast | Verdict/reason | Estimate | 95% interval | Raw p | Holm p |','|---|---|---:|---|---:|---:|']
    for f in report['source_contrasts']:
        values=(f['id'],f['verdict']+((': '+f['unavailable_reason']) if f['unavailable_reason'] else ''),f['estimate'],f['interval'],f['p'],f['holm_p'])
        lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','Modern pooled fits and generated/refitted tests are iid diagnostics; nonrejection does not validate dependent histories. The declared half-chi-square cutoff calibration remains unvalidated, especially for alpha<=3.',
            '', '| Modern pool | Fit availability/reason | Alpha | xmin | Tail n | KS D | KS test/reason | KS p | 95% MC interval | KS verdict |',
            '|---|---|---:|---:|---:|---:|---|---:|---|---|']
    for name,p in report['modern_pools'].items():
        ks=p['ks_test']
        values=(name,p['status']+((': '+p['reason']) if p.get('reason') else ''),p.get('alpha'),p.get('xmin'),p.get('n_tail'),p.get('ks'),
                ks['status']+((': '+ks['reason']) if ks.get('reason') else ''),ks.get('p'),ks.get('interval'),ks.get('verdict'))
        lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','| Pool | Alternative | Fit status/reason | LR R | LR p | Preference |','|---|---|---|---:|---:|---|']
    for name,p in report['modern_pools'].items():
        for alternative,a in p.get('alternatives',{}).items():
            ratio=a.get('ratio',{})
            values=(name,alternative,a['status']+((': '+a['reason']) if a.get('reason') else ''),ratio.get('R'),ratio.get('p'),a.get('diagnostic_verdict'))
            lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','Whole-history parameter estimates and 95% intervals sample registered rows with eligibility masks. Conditional nonempty intervals are descriptive when empty draws prevent primary availability.',
            '', '| Arm | Availability/reason | Eligible/registered histories | Empty draws | Metric | Estimate | Primary 95% interval | Conditional nonempty 95% interval |',
            '|---|---|---:|---:|---|---:|---|---|']
    for name,p in report['modern_parameters'].items():
        for metric in ('alpha_mean','alpha_median','xmin_mean','xmin_median'):
            values=(name,p['status']+((': '+p['reason']) if p.get('reason') else ''),f"{p['eligible_history_count']}/{p['registered_history_count']}",
                    p['empty_replicates'],metric,p['estimates'].get(metric),p['intervals'].get(metric),p.get('conditional_nonempty_intervals',{}).get(metric))
            lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','| Parameter contrast | Availability/reason | Base/control eligible histories | Metric | Difference estimate | Descriptive 95% interval |',
            '|---|---|---:|---|---:|---|']
    for name,p in report['modern_parameter_contrasts'].items():
        for metric in ('alpha_mean','alpha_median','xmin_mean','xmin_median'):
            values=(name,p['status']+((': '+p['reason']) if p.get('reason') else ''),f"{p['base_eligible']}/{p['base_registered']} versus {p['control_eligible']}/{p['control_registered']}",metric,p['estimate'].get(metric),p['intervals'].get(metric))
            lines.append('| '+' | '.join(cell(v) for v in values)+' |')
    lines+=['','Individual fits retain all four alternative outcomes, insufficiency and optimizer failures. Parameter contrasts use independent whole-history replicate arrays and descriptive intervals, with no invented p-family.',
            '', 'Exact Clauset Supplement S1 reproduction is Unresolved; CoW missing fatalities (-9 for Thailand, war170) are unknown, not zero. Cederman historical input is separate. Runnable GeoSim2 reference is not full archived-mechanics or seed-identical docking.', '']
    return '\n'.join(lines)
