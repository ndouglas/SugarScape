import copy
import unittest
from survey.democratic_peace import source, numerics, followup
try:
    from survey.democratic_peace import followup_analysis as analysis
except ImportError:
    analysis = None


def histories(count, value=0., extinct=False):
    return [{'complete': True, 'status': 'completed', 'metrics': {
        'democratic_share': value, 'all_democratic': value == 1.,
        'clustering_ratio': None if extinct else value,
        'democratic_extinction': extinct}} for _ in range(count)]


class FollowupAnalysisTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(analysis, 'composite follow-up analysis is not implemented')

    def test_prose_has_no_fictitious_original_and_literal_requires_original30(self):
        table = source.fixture_table()
        slot = table['slots'][0]; suffix = followup.baseline.arm_suffix(slot['mobile_share'], slot['mechanism'], slot['initial_democratic_share'])
        fresh = {'precision.'+suffix: histories(100)}
        jobs = {j['id']: j for j in followup.analysis_jobs('literal_precision')}
        left = analysis.source_findings(table, fresh, {}, jobs, 'literal_precision', fixture_draws=20)
        self.assertEqual(left['targets'][0]['unavailable_reason'], 'incomplete_historical_literal_original')
        jobs = {j['id']: j for j in followup.analysis_jobs('prose_precision')}
        right = analysis.source_findings(table, fresh, {}, jobs, 'prose_precision', fixture_draws=20)
        self.assertIsNotNone(right['targets'][0]['p'])
        self.assertNotIn('prose_original', right['targets'][0])
        self.assertEqual(right['targets'][0]['reference_population'], 'prose_reference_100')
        self.assertEqual(right['family_size'], 105)

    def test_literal_predictive_outputs_match_original_mathematics_exactly(self):
        table = source.fixture_table(); original = {}; fresh = {}
        for arm in followup.baseline.canonical_arms(False):
            suffix = arm['id'].removeprefix('original.')
            original[arm['id']] = histories(30, .25)
            fresh['precision.'+suffix] = histories(100, .25)
        jobs = {j['id']: j for j in followup.analysis_jobs('literal_precision')}
        actual = analysis.source_findings(table, fresh, original, jobs, 'literal_precision', fixture_draws=20)
        expected = source.source_findings(table, {**original, **fresh}, jobs, precision_registered=True, draws=20, minimum_defined=2, fixture=True)
        self.assertEqual([(t['p'], t['holm_p'], t['result']) for t in actual['targets']], [(t['p'], t['holm_p'], t['result']) for t in expected['targets']])

    def test_missing_invalid_extinct_undefined_counts_and_zero_are_preserved(self):
        values = histories(98, extinct=True) + [{'complete': False, 'status': 'invalid', 'metrics': None, 'partial_metrics': {'democratic_share': .2}}]
        census = analysis.census(values, 100)
        self.assertEqual((census['received_count'], census['complete_count'], census['invalid_count'], census['missing_count'], census['extinction_count'], census['clustering_undefined_count']), (99,98,1,1,98,98))
        self.assertIsNone(census['clustering_ratio_mean'])
        self.assertEqual(analysis.census(histories(100, extinct=True),100)['democratic_share_mean'],0.)

    def test_prose_contrasts_use_disjoint_rng_with_same_definitions(self):
        from survey.democratic_peace import contrasts
        fresh = {'precision.'+a['id'].removeprefix('original.'): histories(100, .25) for a in followup.baseline.canonical_arms(False)}
        jobs = {j['id']: j for j in followup.analysis_jobs('literal_precision')}
        actual = analysis.contrast_findings(fresh, jobs, 'literal_precision', fixture_draws=10)
        expected = contrasts.findings(fresh, jobs, precision_registered=True, draws=10)
        self.assertEqual(actual, expected)
        jobs = {j['id']: j for j in followup.analysis_jobs('prose_precision')}
        self.assertEqual(analysis.contrast_findings(fresh, jobs, 'prose_precision', fixture_draws=10)['primary']['family_size'],6)

    def test_sparse_survivor_and_endpoint_fixture_keeps_original_numerics(self):
        jobs = followup.analysis_jobs('prose_precision')
        result = numerics.predictive([1.] + [None]*99, followup.job_rng(jobs[0], 'prose_precision'), draws=20, minimum_defined=2)
        self.assertEqual(result['reason'], 'insufficient_conditional_reference')
        self.assertEqual(numerics.maximize_interval_p([0.,0.,1.,1.],[0.,1.])['p'],1.)
