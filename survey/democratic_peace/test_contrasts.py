import unittest
from survey.democratic_peace import contrasts, manifest


class PrecisionCensusTests(unittest.TestCase):
    def findings(self, histories, registered):
        jobs = {j['id']: j for j in manifest.analysis_jobs({})}
        return contrasts.findings(histories, jobs, precision_registered=registered, draws=8)

    def test_unregistered_precision_has_no_registration_or_missing_census(self):
        result = self.findings({}, False)
        for family in result.values():
            for component in family['components']:
                with self.subTest(component=component['id']):
                    self.assertIsNone(component['left_census'])
                    self.assertIsNone(component['right_census'])
                    self.assertEqual(component['required_precision_sample_size'], 100)
                    self.assertEqual(component['unavailable_reason'], 'precision_not_registered')

    def test_registered_missing_precision_retains_the_registered_sample_requirement(self):
        result = self.findings({}, True)
        for family in result.values():
            for component in family['components']:
                self.assertEqual(component['left_census']['registered_count'], 100)
                self.assertEqual(component['left_census']['missing_count'], 100)
                self.assertEqual(component['right_census']['missing_count'], 100)
                self.assertEqual(component['required_precision_sample_size'], 100)
                self.assertEqual(component['unavailable_reason'], 'incomplete_precision_history_population')

    def test_registered_incomplete_precision_keeps_received_invalid_and_missing_counts(self):
        histories = {'precision.mobile0_5.alliances.density0_1': [{'complete': False}]}
        component = self.findings(histories, True)['primary']['components'][0]
        self.assertEqual(component['left_census']['registered_count'], 100)
        self.assertEqual(component['left_census']['received_count'], 1)
        self.assertEqual(component['left_census']['invalid_count'], 1)
        self.assertEqual(component['left_census']['missing_count'], 99)
        self.assertIsNone(component['estimate'])

    def test_complete_synthetic_precision_censuses_remain_complete_with_bounded_draws(self):
        # Synthetic fixture values only; no worlds or registered observations.
        histories = {}
        for definition in manifest.contrast_definitions():
            for arm in (definition['left'], definition['right']):
                histories[arm] = [{'complete': True, 'metrics': {
                    'democratic_share': .5, 'all_democratic': False,
                    'clustering_ratio': 1., 'democratic_extinction': False}} for _ in range(100)]
        result = self.findings(histories, True)
        for family in result.values():
            for component in family['components']:
                self.assertTrue(component['left_census']['population_complete'])
                self.assertEqual(component['left_census']['missing_count'], 0)
                self.assertEqual(component['right_census']['complete_count'], 100)
                self.assertIsNone(component['unavailable_reason'])
                self.assertEqual(component['estimate'], 0.)
