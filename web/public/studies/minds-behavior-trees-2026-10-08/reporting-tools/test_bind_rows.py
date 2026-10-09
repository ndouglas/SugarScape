import copy
import hashlib
import json
from pathlib import Path
import unittest
from bind_rows import bind_rows
WRITER=Path(__file__).resolve().parents[1]
A=(WRITER/'analysis.json').read_bytes()
ANALYSIS=json.loads(A)
CHART=json.loads((WRITER/'native-chart-inputs.json').read_text())
SHA=hashlib.sha256(A).hexdigest()
class BindingTests(unittest.TestCase):
    def test_retains_all_native_values_exactly_and_maps_source_indices(self):
        groups=bind_rows(CHART,ANALYSIS,SHA)
        self.assertEqual(len(groups),16)
        for (family,metric),rows in groups.items():
            self.assertEqual(len(rows),16)
            self.assertEqual(sum(r['orientation']=='base' for r in rows),8)
            for r in rows:
                self.assertEqual(r['native'],ANALYSIS['estimates'][r['analysis_index']])
                self.assertEqual((r['native']['family'],r['native']['metric']),(family,metric))
    def test_rejects_missing_row(self):
        chart=copy.deepcopy(CHART);chart['estimates'].pop()
        with self.assertRaisesRegex(ValueError,'256'):bind_rows(chart,ANALYSIS,SHA)
    def test_rejects_duplicate_in_place_of_stratum(self):
        chart=copy.deepcopy(CHART);chart['estimates'][-1]=chart['estimates'][0]
        with self.assertRaisesRegex(ValueError,'duplicate'):bind_rows(chart,ANALYSIS,SHA)
    def test_rejects_changed_native_value(self):
        chart=copy.deepcopy(CHART);chart['estimates'][0]['summary']['mean']=0.125
        with self.assertRaisesRegex(ValueError,'native'):bind_rows(chart,ANALYSIS,SHA)
    def test_rejects_wrong_source_hash(self):
        with self.assertRaisesRegex(ValueError,'source'):bind_rows(CHART,ANALYSIS,'0'*64)
    def test_rejects_reduced_denominator_even_if_both_sources_changed(self):
        chart=copy.deepcopy(CHART);analysis=copy.deepcopy(ANALYSIS)
        chart['estimates'][0]['denominator']=39;analysis['estimates'][0]['denominator']=39
        with self.assertRaisesRegex(ValueError,'denominator'):bind_rows(chart,analysis,SHA)
    def test_rejects_unregistered_metric_even_if_both_sources_changed(self):
        chart=copy.deepcopy(CHART);analysis=copy.deepcopy(ANALYSIS)
        for x in [chart,analysis]:x['estimates'][0]['metric']='unrestricted_completion_ticks'
        with self.assertRaisesRegex(ValueError,'metric'):bind_rows(chart,analysis,SHA)
    def test_null_interval_is_retained_as_unavailable_not_zero(self):
        chart=copy.deepcopy(CHART);analysis=copy.deepcopy(ANALYSIS)
        for x in [chart,analysis]:x['estimates'][0]['summary']['ci95']=None
        groups=bind_rows(chart,analysis,SHA)
        rows=[r for rows in groups.values() for r in rows if r['native']['id']==chart['estimates'][0]['id']]
        self.assertEqual(len(rows),1)
        self.assertIsNone(rows[0]['native']['summary']['ci95'])
if __name__=='__main__':unittest.main()
