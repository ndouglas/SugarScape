import copy,json,unittest
from pathlib import Path
from survey.geosim import source,manifest
from survey.geosim import test_records as record_fixtures

TABLE=Path(__file__).resolve().parents[2]/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'

class SourceFindingsTests(unittest.TestCase):
    def setUp(self):
        self.table=json.loads(TABLE.read_bytes());self.manifest=manifest.build_manifest(self.table)
        self.jobs={j['id']:j for j in self.manifest['analysis_jobs']}
        self.histories={}
        for row in self.table['rows']:
            for family,n in [('original',15),('precision',100)]:
                self.histories[f"{family}.{row['id']}"]=[{'availability':{'complete':True},'vector':[-.55,.991,4.2,204]} for _ in range(n)]
    def test_all88_source_slots_remain_when_precision_is_missing(self):
        del self.histories['precision.base']
        result=source.source_findings(self.table,self.histories,self.jobs,draws=16)
        self.assertEqual(len(result['targets']),88)
        self.assertEqual(result['arms']['base']['conditional_verdict'],'Unresolved')
        self.assertTrue(all(r['p'] is None for r in result['targets'][:8]))
        self.assertEqual(result['family_size'],88)
    def test_incomplete_original_cannot_be_compatible_even_with_complete_precision(self):
        self.histories['original.base'].pop()
        result=source.source_findings(self.table,self.histories,self.jobs,draws=16)
        self.assertEqual(result['arms']['base']['conditional_verdict'],'Unresolved')
        self.assertTrue(all(r['p'] is not None for r in result['targets'][:8]))
    def test_source_equivalence_stays_unresolved_when_conditional_complete(self):
        result=source.source_findings(self.table,self.histories,self.jobs,draws=16)
        self.assertEqual(result['arms']['base']['conditional_verdict'],'Compatible')
        self.assertEqual(result['arms']['base']['source_equivalence'],'Unresolved')
    def test_one_invalid_precision_history_never_reduces_sample_size(self):
        self.histories['precision.base'][0]={'availability':{'complete':False},'vector':None}
        result=source.source_findings(self.table,self.histories,self.jobs,draws=16)
        self.assertTrue(all(r['p'] is None for r in result['targets'][:8]))
    def test_six_contrast_slots_preserve_missing_precision_arm(self):
        del self.histories['precision.shock0']
        result=source.contrast_findings(self.histories,self.jobs,draws=16)
        self.assertEqual(len(result),6)
        self.assertTrue(all(r['verdict']=='Unresolved' for r in result[:3]))
        self.assertTrue(all(r['p'] is not None for r in result[3:]))
    def test_history_source_never_promotes_invalid_partial_completion(self):
        row,_,_=record_fixtures.EnvelopeTests().fixture();row['attempt']['status']='invalid';o=row['outcome']
        o.update(valid=False,periods=521,attempted_period=522,finish_reason='invalid',invalid_reason='fixture')
        bad=copy.deepcopy(o['completed_wars'][0]);bad.update(id=1,end_period=522,elapsed_periods=22)
        o['completed_wars'].append(bad);o['legacy_visible_wars'].append(copy.deepcopy(bad))
        result=source.history_source(row)
        self.assertIsNone(result['vector'])
        self.assertEqual(result['partial_diagnostic']['war_count'],1)
        self.assertEqual(result['census']['excluded'][0]['reasons'],['invalid_partial_period_completion'])

if __name__=='__main__':unittest.main()
