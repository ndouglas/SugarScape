"""Diagnostic telemetry has denominators separate from registered scientific judges."""
import hashlib
import json
import unittest
from stock_diagnostics import summarize


class StockDiagnosticsTests(unittest.TestCase):
    def fixture(self):
        config={'model':'polarity','width':10,'height':10,'variant':'epm','horizon':1}
        manifest={'schema_version':1,'arms':[{'id':'a','family':'original','config':config,'first_seed':1,'sessions':2}]}
        raw=json.dumps(manifest).encode();digest=hashlib.sha256(raw).hexdigest()
        resolved={'schema_version':1,'manifest_sha256':digest,'arms':[{'id':'a','config':config}]}
        def obs(count,denominator,period):
            return {'available':True,'nonpositive_stocks':count,'stock_count':denominator,'periods':period,'attempted_period':period,'unavailable_reason':None}
        rows=[]
        for seed,n,k in [(1,2,1),(2,8,0)]:
            rows.append({'arm':'a','seed':seed,'manifest_sha256':digest,'config':config,
                'outcome':{'valid':True,'periods':1,'attempted_period':1,'sovereign_count':n},
                'stock_diagnostics':{'schema_version':1,'unit':'sovereign_capital_stocks','initial':obs(0,100,0),'terminal':obs(k,n,1)}})
        return manifest,resolved,rows,raw

    def test_pooled_stock_and_session_frequencies_have_distinct_denominators(self):
        m,a,r,b=self.fixture();s=summarize(m,a,r,b)['arms'][0]['terminal']['all_sessions']
        self.assertEqual((s['nonpositive_stocks'],s['stock_count'],s['stock_frequency'],s['sessions_with_any'],s['available_sessions'],s['session_frequency']),(1,10,.1,1,2,.5))

    def test_exact_session_bytes_and_manifest_are_bound_to_summary(self):
        m,a,r,b=self.fixture();data=b"raw bytes with exact newline\n"
        s=summarize(m,a,r,b,data)
        self.assertEqual((s['manifest_sha256'],s['session_file_sha256']),(hashlib.sha256(b).hexdigest(),hashlib.sha256(data).hexdigest()))

    def test_family_pool_uses_stock_counts_rather_than_mean_of_frequencies(self):
        m,a,r,b=self.fixture()
        s=summarize(m,a,r,b)['pools_by_family_policy_unit'][0]['terminal']['all_sessions']
        self.assertEqual((s['nonpositive_stocks'],s['stock_count'],s['stock_frequency'],s['session_frequency']),(1,10,.1,.5))

    def test_unavailable_terminal_is_retained_without_measured_zero(self):
        m,a,r,b=self.fixture();r[1]['outcome']['valid']=False
        r[1]['stock_diagnostics']['terminal']={'available':False,'nonpositive_stocks':None,'stock_count':None,'periods':None,'attempted_period':None,'unavailable_reason':'stale snapshot'}
        s=summarize(m,a,r,b)['arms'][0]['terminal']['all_sessions']
        self.assertEqual((s['available_sessions'],s['unavailable_sessions'],s['stock_count'],s['stock_frequency']),(1,1,2,.5))

    def test_stale_clocks_are_not_accepted_as_terminal_observation(self):
        m,a,r,b=self.fixture();r[0]['stock_diagnostics']['terminal']['attempted_period']=0
        s=summarize(m,a,r,b)['arms'][0]
        self.assertEqual(s['terminal']['all_sessions']['invalid_telemetry_sessions'],1)

    def test_provincial_denominator_is_all_primitive_cells(self):
        m,a,r,b=self.fixture();m['arms'][0]['config']['variant']='two_level'
        b=json.dumps(m).encode();a['manifest_sha256']=hashlib.sha256(b).hexdigest()
        for x in r:
            x['manifest_sha256']=a['manifest_sha256'];x['stock_diagnostics']['unit']='primitive_cell_stocks';x['stock_diagnostics']['terminal']['stock_count']=100
        s=summarize(m,a,r,b)['arms'][0]['terminal']['all_sessions']
        self.assertEqual((s['stock_count'],s['stock_frequency']),(200,.005))

    def test_counts_outside_denominator_and_mismatched_config_are_retained_invalid(self):
        m,a,r,b=self.fixture();r[0]['stock_diagnostics']['terminal']['nonpositive_stocks']=3;r[1]['config']={}
        s=summarize(m,a,r,b)['arms'][0]
        self.assertEqual((s['received_sessions'],s['terminal']['all_sessions']['invalid_telemetry_sessions']),(2,2))

    def test_duplicate_and_manifest_mixture_remain_visible(self):
        m,a,r,b=self.fixture();r.append(dict(r[0]));r[1]['manifest_sha256']='other'
        s=summarize(m,a,r,b)
        self.assertEqual((s['raw_record_count'],bool(s['integrity_issues'])),(3,True))

    def test_invalid_session_counts_have_their_own_denominator(self):
        m,a,r,b=self.fixture();r[0]['outcome']['valid']=False
        s=summarize(m,a,r,b)['arms'][0]['terminal']
        self.assertEqual((s['valid_sessions']['stock_frequency'],s['invalid_sessions']['stock_frequency']),(0,.5))


if __name__=='__main__':
    unittest.main()
