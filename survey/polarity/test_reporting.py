"""Reporting fixtures are synthetic; no model studies run here."""
import hashlib
import itertools
import json
import unittest
from analysis import report as actual_report, predictive_mean, predictive_categories


def resolved_fixture(manifest, raw):
    """Synthetic authority for fixtures; product resolution comes from native Rust."""
    return {'schema_version':1,'manifest_sha256':hashlib.sha256(raw).hexdigest(),
        'arms':[{'id':a['id'],'config':{'model':'polarity','resource_policy':'signed',
            'horizon':1,'stop_at_hegemony':True,'width':10,'height':10,**a['config']}} for a in manifest['arms']]}


def complete_fixture(manifest, raw, records):
    authority=resolved_fixture(manifest,raw)
    expected={a['id']:a['config'] for a in authority['arms']}
    records=json.loads(json.dumps(records))
    for row in records:
        row['config']={**expected.get(row['arm'],expected[authority['arms'][0]['id']]),**row.get('config',{})}
        out=row.setdefault('outcome',{})
        out.setdefault('config',{k:v for k,v in row['config'].items() if k!='model'})
        out.setdefault('seed',row['seed'])
        out.setdefault('attempted_period',out.get('periods',1))
        out.setdefault('finish_reason','horizon' if out.get('valid') else 'invalid')
        out.setdefault('invalid_reason',None if out.get('valid') else 'invalid')
        out.setdefault('terminal_category','two' if out.get('valid') else None)
        out.setdefault('destruction',0);out.setdefault('signed_creation',0);out.setdefault('episodes',[])
        out.setdefault('events',{})
        for key in ['attacks','dd_encounters','conquests','capital_collapses','disconnections','revolts',
            'stale_claims','locked_claims','path_collisions','double_successes','destruction',
            'signed_creation','harvest','taxes','transfers','clipping']:out['events'].setdefault(key,0)
    return authority,records


def report(manifest,source,records,raw):
    authority,records=complete_fixture(manifest,raw,records)
    return actual_report(manifest,source,records,raw,authority)


class ReportingTests(unittest.TestCase):
    def fixture(self):
        manifest={'analysis_draws':99,'analysis_seed':7,'arms':[{'id':'original.a','family':'original','config':{'predator_share':.2,'superiority':2,'alliances':False},'first_seed':1,'sessions':2}]}
        raw=json.dumps(manifest).encode(); digest=hashlib.sha256(raw).hexdigest()
        rows=[{'arm':'original.a','seed':i,'manifest_sha256':digest,'config':manifest['arms'][0]['config'],'outcome':{'valid':True,'sovereign_count':2,'initial_predator_share':.2,'predator_capital_share':.5,'events':{'conquests':0,'capital_collapses':0,'disconnections':0},'periods':1}} for i in [1,2]]
        return manifest,raw,rows

    def test_duplicate_is_retained_and_arm_unresolved(self):
        m,b,r=self.fixture(); out=report(m,{},r+[r[0]],b)
        self.assertEqual(out['arms'][0]['verdict'],'Unresolved')
        self.assertEqual(out['raw_record_count'],3)

    def test_invalid_session_stays_outside_valid_category_denominator(self):
        m,b,r=self.fixture();r[0]['outcome'].update(valid=False,invalid_reason='negative')
        a=report(m,{},r,b)['arms'][0]
        self.assertEqual((a['valid_sessions'],a['invalid_sessions'],a['verdict']),(1,1,'Unresolved'))

    def test_wrong_manifest_and_missing_seed_unresolved(self):
        m,b,r=self.fixture();r[0]['manifest_sha256']='wrong'
        self.assertEqual(report(m,{},r[:1],b)['arms'][0]['verdict'],'Unresolved')

    def test_config_mismatch_is_not_silently_pooled(self):
        m,b,r=self.fixture();r[0]['config']={**r[0]['config'],'superiority':3}
        self.assertIn('config mismatch', ' '.join(report(m,{},r,b)['arms'][0]['issues']))

    def test_constant_mean_interval_uses_most_compatible_target(self):
        result=predictive_mean([2]*10,[1,2],seed=7,draws=99)
        self.assertEqual(result['p'],1)
        self.assertEqual(result['predictive_interval'],[2,2])

    def test_category_uncertainty_retains_all_targets(self):
        result=predictive_categories([2]*10,[[20,0,0,0,0],[0,20,0,0,0]],seed=7,draws=99)
        self.assertEqual(result['p'],max(result['target_p_values']))

    def test_stoermer_reports_configuration_means_not_pooled_runs(self):
        m,b,r=self.fixture();m['arms'][0]['family']='stoermer';b=json.dumps(m).encode()
        for row in r:row['manifest_sha256']=hashlib.sha256(b).hexdigest()
        out=report(m,{},r,b)
        self.assertEqual(out['stoermer']['unit'],'configuration mean across registered repeats')
        self.assertEqual(out['stoermer']['configuration_count'],1)

    def test_dataset_manifest_mixture_invalidates_otherwise_complete_selection(self):
        m,b,r=self.fixture()
        other={**m['arms'][0],'id':'original.other','first_seed':3}
        m['arms'].append(other);b=json.dumps(m).encode();digest=hashlib.sha256(b).hexdigest()
        for row in r:row['manifest_sha256']=digest
        wrong={**r[0],'arm':'original.other','seed':3,'manifest_sha256':'foreign'}
        out=report(m,{},r+[wrong],b)
        selected=[f for f in out['rows'] if f['id']=='selection/original.a']
        self.assertEqual(selected[0]['verdict'],'Unresolved')

    def test_source_pra_uses_registered_matching_factorial_precision(self):
        m,b,r=self.fixture()
        m['arms'][0]['family']='pra'
        m['arms'][0]['config']['allocation']='pra'
        precision={**m['arms'][0],'id':'allocation.pra','family':'allocation','first_seed':3}
        m['arms'].append(precision);b=json.dumps(m).encode();digest=hashlib.sha256(b).hexdigest()
        for row in r:row['manifest_sha256']=digest
        r += [{**row,'arm':'allocation.pra','seed':row['seed']+2} for row in list(r)]
        source={'figures':[{'id':'figure_5.6','source_id':'book','printed_page':120,'panels':[{'id':'offense','ratio':2,'alliances':False,'samples':[{'predator_share':.2,'admissible_category_count_vectors':[[0,20,0,0,0]]}]}]}]}
        out=report(m,source,r,b)
        fit=[f for f in out['rows'] if f['family']=='pra-fit'][0]
        self.assertEqual(fit['measured']['distribution_arm'],'allocation.pra')

    def test_empty_sessions_are_retained_as_incomplete_not_zero_outcomes(self):
        m,b,r=self.fixture();out=report(m,{},[],b)
        self.assertEqual(out['arms'][0]['mean_polarity'],None)
        self.assertEqual(out['arms'][0]['verdict'],'Unresolved')

    def source_fixture(self):
        m,b,r=self.fixture()
        m['arms'].append({**m['arms'][0],'id':'precision.a','family':'precision','first_seed':3})
        b=json.dumps(m).encode();digest=hashlib.sha256(b).hexdigest()
        for row in r:row['manifest_sha256']=digest
        r += [{**row,'arm':'precision.a','seed':row['seed']+2} for row in list(r)]
        source={'figures':[{'id':'figure_11','source_id':'article','printed_page':10,
            'panels':[{'id':'offense','ratio':2,'alliances':False,'samples':[
                {'predator_share':.2,'admissible_category_count_vectors':[[0,20,0,0,0],[0,19,1,0,0]]}]}]}]}
        return m,b,r,source

    def test_missing_precision_cannot_substitute_source_size_sessions(self):
        m,b,r,s=self.source_fixture()
        fit=report(m,s,r[:2],b)['rows'][0]
        self.assertEqual((fit['family'],fit['verdict'],fit['measured']),('source-fit','Unresolved',None))

    def test_missing_literal_source_size_arm_is_not_hidden_by_precision(self):
        m,b,r,s=self.source_fixture();fit=report(m,s,r[2:],b)['rows'][0]
        self.assertEqual((fit['verdict'],fit['measured']),('Unresolved',None))

    def test_missing_source_vectors_cannot_be_compatible(self):
        m,b,r,s=self.source_fixture()
        del s['figures'][0]['panels'][0]['samples'][0]['admissible_category_count_vectors']
        fit=report(m,s,r,b)['rows'][0]
        self.assertEqual((fit['verdict'],fit['measured']),('Unresolved',None))
        self.assertIn('no admissible source vectors',fit['issues'])

    def test_inferred_tax_knot_without_registered_precision_remains_unresolved(self):
        m,b,r,s=self.source_fixture();m['arms']=m['arms'][:1]
        m['arms'][0].update(family='two_level',config={'tax_rate':.3})
        b=json.dumps(m).encode();r=r[:2]
        for row in r:row.update(config={'tax_rate':.3},manifest_sha256=hashlib.sha256(b).hexdigest())
        s['figures'][0]['id']='figure_5.8'
        s['figures'][0]['panels'][0]['samples']=[{'tax_rate':.3,'knot_status':'inferred',
            'admissible_category_count_vectors':[[0,20,0,0,0]]}]
        fit=report(m,s,r,b)['rows'][0]
        self.assertEqual((fit['family'],fit['verdict'],fit['measured']),('tax-fit','Unresolved',None))

    def test_source_fit_keeps_all_admissible_vectors_and_separate_samples(self):
        m,b,r,s=self.source_fixture();fit=report(m,s,r,b)['rows'][0]
        measured=fit['measured']
        self.assertEqual(measured['admissible_source_vectors'],[[0,20,0,0,0],[0,19,1,0,0]])
        self.assertEqual((measured['distribution_arm'],measured['distribution_size'],measured['source_batch_size']),('precision.a',2,20))
        self.assertEqual(measured['p'],max(measured['target_p_values']))

    def test_source_hash_mismatch_remains_unresolved(self):
        m,b,r,s=self.source_fixture();m['source_sha256']='wrong';b=json.dumps(m).encode()
        for row in r:row['manifest_sha256']=hashlib.sha256(b).hexdigest()
        out=report(m,s,r,b)
        self.assertEqual(out['rows'][0]['verdict'],'Unresolved')
        self.assertIn('source hash mismatch',out['global_issues'])

    def test_duplicate_precision_records_disable_source_judge_and_retain_raw(self):
        m,b,r,s=self.source_fixture();out=report(m,s,r+[r[-1]],b)
        self.assertEqual((out['rows'][0]['verdict'],out['raw_record_count']),('Unresolved',5))

    def test_invalid_precision_records_keep_reasons_fraction_and_denominator(self):
        m,b,r,s=self.source_fixture();r[-1]['outcome']={**r[-1]['outcome'],
            'valid':False,'invalid_reason':'aggregate overflow','periods':0,'attempted_period':0}
        out=report(m,s,r,b);arm=out['arms'][1]
        self.assertEqual((arm['category_denominator'],arm['invalid_fraction'],arm['invalid_reasons']),
            (1,.5,{'aggregate overflow':1}))
        self.assertEqual(out['rows'][0]['verdict'],'Unresolved')
        self.assertEqual(out['raw_records'][-1]['outcome']['periods'],0)

    def test_unregistered_records_disable_all_pooled_judges(self):
        m,b,r,s=self.source_fixture();foreign={**r[0],'arm':'unregistered'}
        out=report(m,s,r+[foreign],b)
        self.assertEqual((out['unknown_arms'],out['unregistered_record_count']),(['unregistered'],1))
        self.assertTrue(all(f['verdict']=='Unresolved' and f['measured'] is None
            for f in out['rows'] if f['family']!='unquantified'))

    def test_uncertain_source_direction_cannot_be_reported_as_signed_counterexample(self):
        m,b,r,s=self.source_fixture();out=report(m,s,r,b)
        rows=[f for f in out['rows'] if f['id'].startswith('reported_source_direction/')]
        self.assertTrue(rows)
        self.assertTrue(all(f['verdict']=='Unresolved' for f in rows))

    def test_missing_comparisons_remain_in_holm_family(self):
        m,b,r,s=self.source_fixture();out=report(m,s,r,b)
        selection=[f for f in out['rows'] if f['family']=='selection']
        self.assertEqual(len(selection),2)
        # Two constant positive selection contrasts have p=.02; Holm has two registered members.
        self.assertEqual([f['measured']['holm_p'] for f in selection],[.04,.04])
        incomplete=report(m,s,r[:2],b)
        remaining=[f for f in incomplete['rows'] if f['family']=='selection']
        self.assertEqual([f['measured']['holm_p'] if f['measured'] else None for f in remaining],[.04,None])


    def test_uniform_wrong_omitted_default_and_missing_model_are_unresolved(self):
        m,b,r,s=self.source_fixture();authority,r=complete_fixture(m,b,r)
        for row in r:row['config']['resource_policy']='floor_zero'
        self.assertTrue(all(a['verdict']=='Unresolved' for a in actual_report(m,s,r,b,authority)['arms']))
        authority,r=complete_fixture(m,b,self.source_fixture()[2])
        for row in r:del row['config']['model']
        self.assertTrue(all(a['verdict']=='Unresolved' for a in actual_report(m,s,r,b,authority)['arms']))

    def test_missing_resolved_field_and_missing_authority_are_unresolved(self):
        m,b,r,s=self.source_fixture();authority,r=complete_fixture(m,b,r)
        for row in r:del row['config']['resource_policy']
        self.assertTrue(all(a['verdict']=='Unresolved' for a in actual_report(m,s,r,b,authority)['arms']))
        self.assertTrue(all(a['verdict']=='Unresolved' for a in actual_report(m,s,r,b)['arms']))

    def test_embedded_seed_config_clocks_status_and_category_are_checked(self):
        m,b,r,s=self.source_fixture();authority,rows=complete_fixture(m,b,r)
        changes=[{'seed':999},{'config':{}},{'periods':0,'attempted_period':1},
            {'valid':'true'},{'terminal_category':'one'},{'finish_reason':'hegemony'}]
        for change in changes:
            with self.subTest(change=change):
                changed=json.loads(json.dumps(rows));changed[0]['outcome'].update(change)
                out=actual_report(m,s,changed,b,authority)
                self.assertEqual(out['arms'][0]['verdict'],'Unresolved')
                self.assertEqual(out['raw_record_count'],4)

    def test_construction_panic_retains_unknown_state_without_measured_zero(self):
        m,b,r=self.fixture();authority,r=complete_fixture(m,b,r)
        out=r[0]['outcome'];out.update(valid=False,finish_reason='panic',
            invalid_reason='native construction panic: injected',state_available=False)
        for key in ['periods','attempted_period','sovereign_count','terminal_category',
            'initial_predator_share','predator_capital_share','destruction','signed_creation','events','episodes']:out[key]=None
        result=actual_report(m,{},r,b,authority)
        self.assertEqual(result['arms'][0]['invalid_sessions'],1)
        self.assertEqual(result['arms'][0]['category_denominator'],1)
        self.assertEqual(result['raw_records'][0]['outcome']['periods'],None)
        self.assertNotIn('construction panic availability mismatch',result['arms'][0]['issues'])

    def test_signed_ledger_totals_preserve_negative_zero_and_record_order(self):
        m,b,r=self.fixture()
        r[0]['outcome']['events']={'harvest':-10,'taxes':-4,'transfers':-3,'conquests':0}
        r[1]['outcome']['events']={'harvest':4,'taxes':4,'transfers':1,'conquests':0}
        forward=report(m,{},r,b)['arms'][0]['events_totals']
        reverse=report(m,{},list(reversed(r)),b)['arms'][0]['events_totals']
        self.assertEqual({k:forward[k] for k in ['harvest','taxes','transfers','conquests']},{'harvest':-6,'taxes':0,'transfers':-2,'conquests':0})
        self.assertEqual(reverse,forward)


    def extreme_ledger_fixture(self, values):
        m,_,r=self.fixture();m['arms'][0]['sessions']=len(values)
        b=json.dumps(m).encode();digest=hashlib.sha256(b).hexdigest()
        rows=[]
        for seed,value in enumerate(values,1):
            row=json.loads(json.dumps(r[0]));row.update(seed=seed,manifest_sha256=digest)
            row['outcome']['events']['harvest']=value
            rows.append(row)
        return m,b,rows

    def test_extreme_cancelling_signed_ledgers_have_identical_total_for_all_orderings(self):
        cases=[([1e308,1e308,-1e308],1e308),([-1e308,-1e308,1e308],-1e308),
               ([1e308,1e308,-1e308,-1e308],0.0)]
        for values,expected in [(v,e) for base,e in cases for v in set(itertools.permutations(base))]:
            with self.subTest(values=values):
                m,b,r=self.extreme_ledger_fixture(values);result=report(m,{},r,b)
                self.assertEqual(result['arms'][0]['events_totals']['harvest'],expected)
                self.assertEqual(result['arms'][0]['verdict'],'Descriptive')
                json.dumps(result,allow_nan=False)

    def test_true_signed_total_overflow_is_unavailable_raw_retained_and_judges_unresolved(self):
        cases=[[1e308,1e308,-1.0],[-1e308,-1e308,1.0]]
        for values in {v for base in cases for v in itertools.permutations(base)}:
            with self.subTest(values=values):
                m,b,r=self.extreme_ledger_fixture(values);result=report(m,{},r,b)
                arm=result['arms'][0]
                self.assertIsNone(arm['events_totals']['harvest'])
                self.assertEqual(arm['verdict'],'Unresolved')
                self.assertEqual((arm['valid_sessions'],arm['invalid_sessions'],result['raw_record_count']),(3,0,3))
                self.assertIn('harvest',arm['events_totals_unavailable'])
                selection=[f for f in result['rows'] if f['family']=='selection']
                self.assertEqual(selection[0]['verdict'],'Unresolved')
                self.assertEqual([row['outcome']['events']['harvest'] for row in result['raw_records']],list(values))
                json.dumps(result,allow_nan=False)

    def test_extreme_descriptive_ledger_mean_stays_finite_when_total_overflows(self):
        m,b,r=self.extreme_ledger_fixture([1e308,1e308])
        for row in r:
            row['outcome']['destruction']=1e308
            row['outcome']['events']['destruction']=1e308
        result=report(m,{},r,b)
        self.assertEqual(result['arms'][0]['destruction']['mean'],1e308)
        json.dumps(result,allow_nan=False)

if __name__=='__main__':unittest.main()
