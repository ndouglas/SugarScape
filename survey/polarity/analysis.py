"""Offline, preregistered EPM analysis. Uses only Python's standard library."""
from collections import Counter, defaultdict
from pathlib import Path
import argparse
import bisect
import json
import math
import itertools
import random


def category(n):
    if not isinstance(n, int) or not 1 <= n <= 100:
        raise ValueError("source category requires integer polarity in 1..100")
    return 0 if n == 1 else 1 if n == 2 else 2 if n <= 10 else 3 if n <= 90 else 4


def holm(values):
    if any(not math.isfinite(p) or not 0 <= p <= 1 for p in values):
        raise ValueError("p-values must be finite probabilities")
    result = [0.] * len(values)
    last = 0.
    for rank, i in enumerate(sorted(range(len(values)), key=values.__getitem__)):
        last = min(1., max(last, values[i]*(len(values)-rank)))
        result[i] = last
    return result


def probabilities(counts):
    if len(counts) != 5 or any(not isinstance(c, int) or c < 0 for c in counts):
        raise ValueError("five nonnegative integer category counts required")
    return [(c+.5)/(sum(counts)+2.5) for c in counts]


def contrast(groups, weights, seed, draws=100_000):
    if len(groups) != len(weights) or not groups or any(not g for g in groups) or draws < 1:
        raise ValueError("contrast needs nonempty groups, matching weights and positive draws")
    if any(not math.isfinite(x) for g in groups for x in g):
        raise ValueError("contrast cannot drop nonfinite sessions")
    rng = random.Random(seed)
    estimate = sum(w*sum(g)/len(g) for w, g in zip(weights, groups))
    histograms = [sorted(Counter(g).items()) for g in groups]
    samples = sorted(sum(w*bootstrap_mean(h, len(g), rng) for w, h, g in
                         zip(weights, histograms, groups)) for _ in range(draws))
    below = bisect.bisect_right(samples, 0.)
    above = draws-bisect.bisect_left(samples, 0.)
    return {'estimate': estimate, 'interval': [quantile(samples,.025),quantile(samples,.975)],
            'p': min(1., 2*(min(below, above)+1)/(draws+1)),
            'analysis_seed': seed, 'resamples': draws}


def compatible_counts(boundaries):
    if len(boundaries) != 4:
        raise ValueError("four cumulative boundaries required")
    candidates = []
    for bounds in boundaries:
        lo, hi = bounds
        if not (math.isfinite(lo) and math.isfinite(hi) and 0 <= lo <= hi <= 20):
            raise ValueError("invalid cumulative interval")
        candidates.append(range(math.ceil(lo), math.floor(hi)+1))
    vectors = []
    for b in itertools.product(*candidates):
        if list(b) == sorted(b):
            edges = (0, *b, 20)
            vectors.append([edges[i+1]-edges[i] for i in range(5)])
    if not vectors:
        raise ValueError("no admissible twenty-run count vector")
    return vectors


def read_sessions(path):
    rows = defaultdict(list)
    seen = set()
    manifests = set()
    for number, line in enumerate(Path(path).read_text().splitlines(), 1):
        if not line.strip():
            continue
        record = json.loads(line)
        key = (record['arm'], record['seed'])
        if key in seen:
            raise ValueError(f"duplicate session {key} at line {number}")
        seen.add(key)
        if 'manifest_sha256' in record:
            manifests.add(record['manifest_sha256'])
        rows[record['arm']].append(record)
    if len(manifests) > 1:
        raise ValueError("raw sessions mix different registered manifests")
    return dict(rows)


def quantile(sorted_values, p):
    x = (len(sorted_values)-1)*p
    lo, hi = math.floor(x), math.ceil(x)
    return sorted_values[lo] + (sorted_values[hi]-sorted_values[lo])*(x-lo)


def multinomial(n, probs, rng):
    counts = []
    remaining_p = 1.
    remaining_n = n
    for p in probs[:-1]:
        q = max(0., min(1., p/remaining_p)) if remaining_p else 0.
        if hasattr(rng, 'binomialvariate'):
            c = rng.binomialvariate(remaining_n, q)
        else:
            c = sum(rng.random() < q for _ in range(remaining_n))
        counts.append(c)
        remaining_n -= c
        remaining_p -= p
    return counts+[remaining_n]


def bootstrap_mean(histogram, n, rng):
    if len(histogram) == 1:
        return histogram[0][0]
    counts = multinomial(n, [c/n for _, c in histogram], rng)
    return sum(x*c for (x, _), c in zip(histogram, counts))/n


def predictive_mean(values, interval, seed, draws=100_000):
    """Largest inclusive two-tail p over the complete digitization interval."""
    rng = random.Random(seed)
    histogram = sorted(Counter(values).items())
    # Source batches have size20; probabilities use the entire reconstruction arm.
    probs = [c/len(values) for _, c in histogram]
    batches = sorted(sum(x*c for (x, _), c in zip(histogram, multinomial(20, probs, rng)))/20
                     for _ in range(draws))
    lo, hi = interval
    candidates = {lo, hi}
    left, right = bisect.bisect_left(batches, lo), bisect.bisect_right(batches, hi)
    if left < right:
        candidates.add(batches[max(left, min(right-1, draws//2))])
    def tail(target):
        below = bisect.bisect_right(batches, target)
        above = draws-bisect.bisect_left(batches, target)
        return min(1., 2*(min(below, above)+1)/(draws+1))
    return {'p':max(map(tail,candidates)), 'source_interval':interval,
            'predictive_interval':[quantile(batches,.025),quantile(batches,.975)],
            'reconstruction_mean':sum(values)/len(values), 'source_batch_size':20,
            'analysis_seed':seed,'resamples':draws}


def deviance(counts, probs):
    return 2*sum(c*math.log(c/(20*p)) for c,p in zip(counts,probs) if c)


def wilson(count, n):
    z=1.959963984540054
    p=count/n
    denominator=1+z*z/n
    center=(p+z*z/(2*n))/denominator
    half=z*math.sqrt(p*(1-p)/n+z*z/(4*n*n))/denominator
    return [max(0.,center-half),min(1.,center+half)]


def predictive_categories(values, targets, seed, draws=100_000):
    counts = [sum(category(v)==k for v in values) for k in range(5)]
    probs = probabilities(counts)
    rng = random.Random(seed)
    reference = sorted(deviance(multinomial(20,probs,rng),probs) for _ in range(draws))
    ps = [(draws-bisect.bisect_left(reference,deviance(t,probs))+1)/(draws+1)
          for t in targets]
    empirical = [c/len(values) for c in counts]
    return {'p':max(ps),'target_p_values':ps,'admissible_source_vectors':targets,
            'counts':counts,'jeffreys_probabilities':probs,
            'reconstruction_category_intervals':[wilson(c,len(values)) for c in counts],
            'reconstruction_category_interval_method':'Marginal Wilson95%; descriptive finite-sample uncertainty, not simultaneous intervals',
            'total_variation_interval':[min(sum(abs(t[k]/20-empirical[k]) for k in range(5))/2 for t in targets),
                                        max(sum(abs(t[k]/20-empirical[k]) for k in range(5))/2 for t in targets)],
            'source_batch_size':20,'analysis_seed':seed,'resamples':draws}


def metric(outcomes, name):
    if name=='mean_polarity':return [o['sovereign_count'] for o in outcomes]
    if name=='hegemony':return [int(o['sovereign_count']==1) for o in outcomes]
    if name=='power_politics':return [int(2<=o['sovereign_count']<=10) for o in outcomes]
    if name.startswith('category_'):
        k=int(name.split('_')[1]);return [int(category(o['sovereign_count'])==k) for o in outcomes]
    raise ValueError(name)


def report(manifest, source, records, manifest_bytes):
    """Retain integrity failures and mark affected comparisons unresolved."""
    import hashlib
    import sys
    expected_hash = hashlib.sha256(manifest_bytes).hexdigest()
    draws = manifest.get('analysis_draws',100_000)
    seed = manifest.get('analysis_seed',2026100202)
    if draws < 1:raise ValueError('analysis_draws must be positive')
    registered={a['id']:a for a in manifest['arms']}
    grouped=defaultdict(list)
    for row in records:grouped[row.get('arm','<missing arm>')].append(row)
    findings=[]; summaries=[]; clean={}; serial=0
    source_ok = ('source_sha256' not in manifest or
                 manifest['source_sha256']==hashlib.sha256(json.dumps(source,sort_keys=True).encode()).hexdigest())
    def next_seed():
        nonlocal serial
        serial+=1
        return seed+serial
    def finding(claim, family, citation, judge, result=None, limits=None, issues=None):
        f={'claim':claim,'family':family,'source_citation':citation,'judge':judge,
           'result':result,'uncertainty':None if result is None else result.get('interval',result.get('predictive_interval',result.get('total_variation_interval'))),
           'verdict':'Unresolved' if issues else 'Inconclusive','limitations':limits or [],'issues':issues or []}
        findings.append(f);return f
    for aid,arm in registered.items():
        rows=grouped.get(aid,[]);issues=[]
        expected=set(range(arm['first_seed'],arm['first_seed']+arm['sessions']))
        actual=[r.get('seed') for r in rows]
        if len(set(actual))!=len(actual):issues.append('duplicate session seeds retained')
        if set(actual)!=expected or len(rows)!=arm['sessions']:issues.append('incomplete arm or unexpected seeds')
        outcomes=[];invalid=[]
        for row in rows:
            if row.get('manifest_sha256')!=expected_hash:issues.append('manifest hash mismatch or missing')
            config=row.get('config',{})
            if config.get('model','polarity')!='polarity':issues.append('config model mismatch')
            if any(config.get(k)!=v for k,v in arm['config'].items() if k!='model'):issues.append('config mismatch')
            out=row.get('outcome',{})
            if not out.get('valid',False):invalid.append(out.get('invalid_reason','unspecified invalidity'));continue
            try:category(out.get('sovereign_count'))
            except ValueError:issues.append('invalid terminal category');invalid.append('invalid terminal category');continue
            outcomes.append(out)
        if invalid:issues.append('invalid sessions retained outside valid category denominator')
        # A resolved field may not vary within an arm even when omitted in the source config.
        if len({json.dumps(r.get('config',{}),sort_keys=True) for r in rows})>1:issues.append('mixed resolved configurations')
        issues=sorted(set(issues))
        counts=[sum(category(o['sovereign_count'])==k for o in outcomes) for k in range(5)]
        summary={'id':aid,'family':arm['family'],'received_sessions':len(rows),'registered_sessions':arm['sessions'],
                 'valid_sessions':len(outcomes),'invalid_sessions':len(invalid),'invalid_reasons':dict(Counter(invalid)),
                 'category_counts':counts,'category_denominator':len(outcomes),'issues':issues,
                 'verdict':'Unresolved' if issues else 'Descriptive',
                 'mean_polarity':sum(o['sovereign_count'] for o in outcomes)/len(outcomes) if outcomes else None,
                 'config':arm['config'],'invalid_fraction':len(invalid)/len(rows) if rows else None}
        # Descriptions use all available valid records, explicitly conditional on validity.
        for field in ['destruction','signed_creation','periods']:
            vals=[o[field] for o in outcomes if isinstance(o.get(field),(int,float)) and math.isfinite(o[field])]
            summary[field]={'mean':sum(vals)/len(vals) if vals else None,'available_sessions':len(vals)}
        summary['events_totals']=dict(sum((Counter(o.get('events',{})) for o in outcomes),Counter()))
        summary['episode_count']=sum(len(o.get('episodes',[])) for o in outcomes)
        episodes=[e for o in outcomes for e in o.get('episodes',[])]
        summary['episode_end_causes']=dict(Counter(e.get('end_cause',e.get('end_reason','unreported')) for e in episodes))
        summary['censored_episodes']=sum(bool(e.get('censored',False)) for e in episodes)
        summary['finish_reasons']=dict(Counter(o.get('finish_reason','unreported') for o in outcomes))
        summaries.append(summary)
        if not issues:clean[aid]=outcomes
    unknown=sorted(set(grouped)-set(registered))
    global_integrity=bool(unknown) or any(r.get('manifest_sha256')!=expected_hash for r in records)
    def select(family, **config):
        return [a['id'] for a in manifest['arms'] if a['family']==family and
                all(a['config'].get(k,{'allocation':'equal','alliances':False,'predator_share':.2}.get(k))==v for k,v in config.items())]
    def single(family, **config):
        ids=select(family,**config)
        return ids[0] if len(ids)==1 else None
    # Original figure checks use its separately registered precision distribution.
    for figure in source.get('figures',[]):
        fid=figure['id'];citation=f"{figure['source_id']} printed{figure['printed_page']} {fid}"
        for panel in figure['panels']:
            for sample in panel['samples']:
                family='two_level_precision' if 'tax_rate' in sample else ('allocation' if fid.endswith('5.6') else 'precision')
                cfg={'tax_rate':sample['tax_rate']} if 'tax_rate' in sample else {'predator_share':sample['predator_share'],'superiority':panel['ratio'],'alliances':panel['alliances']}
                if family=='allocation':cfg['allocation']='pra'
                aid=single(family,**cfg)
                primary_family='two_level' if family=='two_level_precision' else ('pra' if family=='allocation' else 'original')
                primary=single(primary_family,**cfg)
                check_family='tax' if family=='two_level_precision' else ('pra' if family=='allocation' else 'original')
                is_mean='mean_interval' in sample
                ff=check_family+('_means' if is_mean else '_categories')
                issues=[] if aid in clean and primary in clean and source_ok else ['missing/invalid source-size or precision arm, or source hash mismatch']
                if sample.get('extraction_status','').startswith('unresolved'):issues.append('unrecoverable source target')
                targets=sample.get('admissible_category_count_vectors',[])
                if not is_mean and not targets:issues.append('no admissible source vectors')
                result=None
                if not issues:
                    values=metric(clean[aid],'mean_polarity')
                    result=predictive_mean(values,sample['mean_interval'],next_seed(),draws) if is_mean else predictive_categories(values,targets,next_seed(),draws)
                    result['distribution_arm']=aid
                    result['distribution_size']=len(values)
                    if family=='two_level_precision':
                        result['tax_knot_status']=sample.get('knot_status','inferred source setting')
                        result['tax_abscissa_interval']=sample.get('abscissa_interval')
                        result['complete_source_knot_list_available']=source.get('tax_knot_recovery',{}).get('complete_source_experiment_knot_list_available',False)
                    # Empirical distribution uncertainty is distinct from source-size variability.
                    uncertainty=contrast([values],[1],next_seed(),draws)
                    result['reconstruction_mean_interval']=uncertainty['interval']
                finding(f'{fid}/{panel["id"]}/{cfg}',ff,citation,
                        'size20 unsmoothed bootstrap means' if is_mean else 'size20 Jeffreys multinomial deviance; maximum p across admissible vectors',
                        result,['Predictive check conditions on estimated reconstruction distribution; compatibility is not equivalence.',
                                'Tax abscissae visually inferred; absent precision knots remain unresolved.' if family=='two_level_precision' else
                                'PRA uses separately registered matching200-session no-alliance PRA factorial arm; original20 sessions retained separately.' if family=='allocation' else
                                'Original20-run arms remain separate descriptive literal-size samples.'],issues)
    metrics=['mean_polarity','hegemony','power_politics']
    shares=sorted({a['config'].get('predator_share') for a in manifest['arms'] if a['family']=='precision'})
    # Registered directional families, all strata and uniformly pooled nonzero-share aggregate.
    for treatment in ['defense','alliance']:
        contrasts=[]
        strata= [(share,a) for share in shares for a in [False,True]] if treatment=='defense' else [(share,r) for share in shares for r in [2,3]]
        for share,other in strata:
            cfg={'predator_share':share}
            left=single('precision',**cfg,superiority=3,alliances=other) if treatment=='defense' else single('precision',**cfg,superiority=other,alliances=True)
            right=single('precision',**cfg,superiority=2,alliances=other) if treatment=='defense' else single('precision',**cfg,superiority=other,alliances=False)
            contrasts.append((share,other,left,right))
            for name in metrics:
                issues=[] if left in clean and right in clean else ['missing or invalid contrast arms']
                result=contrast([metric(clean[left],name),metric(clean[right],name)],[1,-1],next_seed(),draws) if not issues else None
                finding(f'{treatment}/{share}/{other}/{name}',treatment,'Cederman1994 Figs10/11/13',
                        'session bootstrap treatment-minus-control; two-sided inclusive tails',result,
                        ['Positive/negative mean and hegemony contrasts have separate interpretations.'],issues)
        for name in metrics:
            nonzero=[c for c in contrasts if c[0]!=0]
            issues=[] if nonzero and all(c[2] in clean and c[3] in clean for c in nonzero) else ['incomplete registered nonzero-share aggregate']
            result=None
            if not issues:
                groups=[metric(clean[aid],name) for c in nonzero for aid in c[2:]]
                result=contrast(groups,[w/len(nonzero) for _ in nonzero for w in [1,-1]],next_seed(),draws)
            finding(f'{treatment}/registered_nonzero_aggregate/{name}',treatment,'Cederman1994 Figs10/11/13',
                    'Uniform weighting of seven nonzero shares and both counterpart settings',result,
                    ['Aggregate cannot hide conflicting strata.'],issues)
    # Category-wise difference of alliance contrasts under PRA versus equal, same chapter5 damage.
    for share in sorted({a['config'].get('predator_share') for a in manifest['arms'] if a['family']=='allocation'}):
        for ratio in [2,3]:
            ids=[single('allocation',predator_share=share,superiority=ratio,allocation=allocation,alliances=alliance)
                 for allocation,alliance in [('pra',True),('pra',False),('equal',True),('equal',False)]]
            for k in range(5):
                issues=[] if all(a in clean for a in ids) else ['missing or invalid factorial arms']
                result=contrast([metric(clean[a],f'category_{k}') for a in ids],[1,-1,-1,1],next_seed(),draws) if not issues else None
                finding(f'allocation_interaction/{share}/{ratio}/category_{k}','allocation_interaction','Cederman1997 printed121 fn5',
                        '(PRA alliance-minus-none)-(equal alliance-minus-none)',result,
                        ['Author-motivated robustness, no published factorial target; no detected difference is inconclusive.'],issues)
    # Named stocks-support controls have separate arms and cannot change the primary factorial.
    for arm in manifest['arms']:
        if arm['family']!='support_control':continue
        aid=arm['id'];cfg=arm['config']
        baseline=arm.get('baseline') or single('allocation',predator_share=cfg['predator_share'],
                    superiority=cfg['superiority'],allocation='pra',alliances=True)
        for name in metrics+[f'category_{k}' for k in range(5)]:
            issues=[] if aid in clean and baseline in clean else ['missing or invalid support-control arm/baseline']
            result=contrast([metric(clean[aid],name),metric(clean[baseline],name)],[1,-1],next_seed(),draws) if not issues else None
            finding(f'support_control/{aid}/{name}','support_control','Cederman1997 printed121 fn5; explicit support reconstruction',
                    'Stocks-support minus front-commitment-support; independent whole-session bootstrap',result,
                    ['Separate reconstruction control; primary allocation interaction uses front commitments.',
                     'Same seed labels imply pairing only when identical initialization is verified; bootstrap does not assume paired tapes.'],issues)
    # Survivors are capital types, session-level difference from realized initial share.
    for arm in manifest['arms']:
        share=arm['config'].get('predator_share')
        if share is None or not 0<share<1:continue
        aid=arm['id'];outs=clean.get(aid,[])
        missing=not outs or any(not isinstance(o.get('predator_capital_share'),(int,float)) or
                                not isinstance(o.get('initial_predator_share'),(int,float)) or
                                not math.isfinite(o['predator_capital_share']) or
                                not math.isfinite(o['initial_predator_share']) for o in outs)
        result=None
        if not missing:
            values=[o['predator_capital_share']-o['initial_predator_share'] for o in outs]
            result=contrast([values],[1],next_seed(),draws)
        finding(f'selection/{aid}','selection','Cederman1994 predator selection discussion',
                'Session predator-capital share minus initial share',result,
                ['Pure-type settings cannot test selection; dependent provinces are not counted as sovereign capitals.'],
                ['missing or invalid selection measurements'] if missing else [])
    if global_integrity:
        for f in findings:
            f['result']=None;f['uncertainty']=None;f['verdict']='Unresolved'
            f['issues'].append('dataset contains unregistered or mismatched-manifest records; no pooled judge')
    # Holm includes unavailable registered comparisons at p1; no family shrinks because of invalidity.
    for family in {f['family'] for f in findings}:
        rows=[f for f in findings if f['family']==family]
        corrected=holm([f['result']['p'] if f['result'] is not None else 1 for f in rows])
        for f,p in zip(rows,corrected):
            if f['result'] is None:continue
            f['result']['holm_p']=p
            if family.endswith('_means') or family.endswith('_categories'):
                f['verdict']='Incompatible' if p<.05 else 'Compatible'
            else:
                lo,hi=f['result']['interval']
                if family=='allocation_interaction':f['verdict']='Dependence detected' if p<.05 and (lo>0 or hi<0) else 'Inconclusive'
                else:f['verdict']='Holds' if lo>0 and p<.05 else 'Fails' if hi<0 and p<.05 else 'Inconclusive'
    # Initial hypotheses and observed source directions have independent roles.
    def source_power(share, ratio, alliance):
        if not source_ok:return None
        fid='figure_13' if alliance else 'figure_11'
        matches=[sample for figure in source.get('figures',[]) if figure['id']==fid
                 for panel in figure['panels'] if panel['ratio']==ratio
                 for sample in panel['samples'] if sample.get('predator_share')==share]
        if len(matches)!=1:return None
        vectors=matches[0].get('admissible_category_count_vectors',[])
        if not vectors:return None
        values=[(v[1]+v[2])/20 for v in vectors]
        return min(values),max(values)
    for f in list(findings):
        if f['family'] not in ['defense','alliance'] or not f['claim'].endswith('/power_politics'):continue
        label='P2' if f['family']=='defense' else 'P3'
        f['claim_role']='initial stabilization hypothesis '+label
        parts=f['claim'].split('/')
        if parts[1]=='registered_nonzero_aggregate':
            pairs=[(share,other) for share in shares if share!=0 for other in
                   ([False,True] if f['family']=='defense' else [2,3])]
        else:
            pairs=[(float(parts[1]),parts[2]=='True' if f['family']=='defense' else int(parts[2]))]
        bounds=[]
        for share,other in pairs:
            left=source_power(share,3,other) if f['family']=='defense' else source_power(share,other,True)
            right=source_power(share,2,other) if f['family']=='defense' else source_power(share,other,False)
            if left is None or right is None:break
            bounds.append((left[0]-right[1],left[1]-right[0]))
        source_interval=[sum(v[k] for v in bounds)/len(bounds) for k in [0,1]] if len(bounds)==len(pairs) and bounds else None
        copy={**f,'claim':'reported_source_direction/'+f['claim'],'claim_role':'reported counterexample or source direction',
              'judge':f['judge']+'; compare with recoverable source direction',
              'source_contrast_interval':source_interval}
        if source_interval is None or source_interval[0]<=0<=source_interval[1]:
            copy['verdict']='Unresolved';copy['issues']=f['issues']+['source contrast sign unavailable or digitization spans zero']
        elif source_interval[1]<0:
            copy['claim_role']='reported negative counterexample to '+label
            if f['result'] is not None:
                copy['verdict']='Holds' if f['verdict']=='Fails' else 'Fails' if f['verdict']=='Holds' else f['verdict']
        else:
            copy['claim_role']='reported positive source contrast; not a negative counterexample'
        findings.append(copy)
    for claim,limit in [('frequent_power_politics','No recovered numeric frequency threshold.'),('exponential_takeoff','No preregistered curvature or unknown-seed trajectory target.'),('representative_overextension','Single unknown-seed narrative is illustrative; report continuation/collapse descriptively.'),('stoermer_75_percent','Near-corner region has no recovered radius/cutoff; exact inertness is a distinct new measure.'),('duffy_effect_sign','Concurrent one-conflict protocol and model differ; no EPM numeric target.')]:
        f=finding(claim,'unquantified','Approved design evidence boundaries','No quantitative judge',limits=[limit]);f['verdict']='Untestable'
    configurations=[]
    for summary in summaries:
        if summary['family']!='stoermer':continue
        outs=clean.get(summary['id'])
        row={'arm':summary['id'],'verdict':summary['verdict'],'mean_terminal_count':summary['mean_polarity'],
             'valid_sessions':summary['valid_sessions'],'registered_repeats':summary['registered_sessions']}
        if outs:
            required=['conquests','capital_collapses','disconnections']
            if all(all(k in o.get('events',{}) for k in required) for o in outs):
                row['exact_zero_structural_event_fraction']=sum(all(o['events'][k]==0 for k in required) for o in outs)/len(outs)
                row['mean_structural_events']=sum(sum(o['events'][k] for k in required) for o in outs)/len(outs)
            else:row['structural_events_status']='unavailable counter definitions'
        configurations.append(row)
    aliases={'original_means':'source-fit','original_categories':'source-fit','pra_categories':'pra-fit','pra_means':'pra-fit','tax_categories':'tax-fit','tax_means':'tax-fit','allocation_interaction':'allocation'}
    rows=[{'id':f['claim'],'family':aliases.get(f['family'],f['family']),'type':f.get('claim_role',f['family']),
           'verdict':f['verdict'],'citation':f['source_citation'],'judge':f['judge'],
           'measured':f['result'],'uncertainty':f['uncertainty'],'limitations':f['limitations'],'issues':f['issues'],
           'source_contrast_interval':f.get('source_contrast_interval')} for f in findings]
    return {'schema_version':1,'manifest_sha256':expected_hash,'rows':rows,'raw_record_count':len(records),
            'unknown_arms':unknown,'integrity_issue_counts':dict(Counter(issue for arm in summaries for issue in arm['issues'])),
            'unregistered_record_count':sum(len(grouped[a]) for a in unknown),'global_issues':([] if source_ok else ['source hash mismatch'])+(['unregistered records retained'] if unknown else []),
            'analysis_seed':seed,'analysis_draws':draws,'analysis_runtime':{'python':sys.version,'binomial_sampler':'random.binomialvariate' if hasattr(random.Random(),'binomialvariate') else 'Bernoulli fallback'},'arms':summaries,'findings':findings,
            'stoermer':{'unit':'configuration mean across registered repeats','configuration_count':len(configurations),
                       'configurations':configurations,'limitations':['Do not treat1100 repeats as independent configurations; no75% exact-inert gate.']},
            'raw_records':records}


def markdown_report(result):
    lines=['# Emergent polarity findings','',f"Raw records: {result['raw_record_count']}; analysis draws: {result['analysis_draws']}; analysis seed: {result['analysis_seed']}.",'',
           'Invalid records, duplicate records, incomplete arms and integrity failures are retained. Descriptive arm means condition on available valid sessions and do not resolve source compatibility.','',
           '| Claim | Source | Judge | Result and uncertainty | Verdict | Limitations |',
           '| --- | --- | --- | --- | --- | --- |']
    def cell(value):return str(value).replace('|','/').replace('\n',' ')
    for f in result['findings']:
        res=f['result'];brief='Unavailable' if res is None else json.dumps({k:res[k] for k in ['estimate','interval','reconstruction_mean','source_interval','predictive_interval','total_variation_interval','p','holm_p'] if k in res},sort_keys=True)
        lines.append('| '+' | '.join(map(cell,[f['claim'],f['source_citation'],f['judge'],brief,f['verdict'],'; '.join(f['limitations']+f['issues'])]))+' |')
    lines+=['','## Arm accounting','','| Arm | Received / registered | Valid / invalid | Mean polarity | Status | Issues |','| --- | --- | --- | --- | --- | --- |']
    for a in result['arms']:
        lines.append('| '+' | '.join(map(cell,[a['id'],f"{a['received_sessions']} / {a['registered_sessions']}",f"{a['valid_sessions']} / {a['invalid_sessions']}",a['mean_polarity'],a['verdict'],'; '.join(a['issues'])]))+' |')
    lines+=['','Störmer results are configuration means across registered repeats. Scatter coordinates and exact-zero structural-event fractions are descriptive; no near-corner cutoff was recovered.','']
    return '\n'.join(lines)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['manifest','source','sessions','output']:parser.add_argument('--'+name,required=True,type=Path)
    args=parser.parse_args()
    manifest_bytes=args.manifest.read_bytes()
    records=[json.loads(line) for line in args.sessions.read_text().splitlines() if line.strip()]
    result=report(json.loads(manifest_bytes),json.loads(args.source.read_text()),records,manifest_bytes)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    Path(str(args.output)+'.json').write_text(json.dumps(result,indent=2,sort_keys=True,allow_nan=False)+'\n')
    Path(str(args.output)+'.md').write_text(markdown_report(result))
    print(f"Retained {len(records)} records; wrote {args.output}.json and {args.output}.md")


if __name__=='__main__':main()
