import copy
import unittest
from survey.democratic_peace import manifest, source


def histories(table):
    result = {}
    for slot in table['slots']:
        suffix = manifest.arm_suffix(slot['mobile_share'], slot['mechanism'], slot['initial_democratic_share'])
        for family, count in [('original', 30), ('precision', 100)]:
            result[family + '.' + suffix] = [{'complete': True, 'metrics': {
                'democratic_share': .5, 'clustering_ratio': 1.,
                'democratic_extinction': False, 'all_democratic': False}}
                for _ in range(count)]
    return result


class SourceTests(unittest.TestCase):
    def setUp(self):
        self.table = source.fixture_table()
        self.jobs = {j['id']: j for j in manifest.build_manifest(self.table)['analysis_jobs']}
        self.histories = histories(self.table)

    def result(self, **kwargs):
        return source.source_findings(self.table, self.histories, self.jobs,
            precision_registered=True, draws=20, minimum_defined=2, fixture=True, **kwargs)

    def test_missing_original_history_prevents_precision_inference(self):
        arm = 'original.' + manifest.arm_suffix(.5, 'tagging', .1)
        self.histories[arm].pop()
        result = self.result()
        target = next(t for t in result['targets'] if t['id'] == 'fig9.tagging.density0_1')
        self.assertIsNone(target['p'])
        self.assertEqual(target['unavailable_reason'], 'incomplete_original_history_population')

    def test_invalid_precision_history_is_not_dropped(self):
        arm = 'precision.' + manifest.arm_suffix(.5, 'tagging', .1)
        self.histories[arm][0] = {'complete': False, 'metrics': None}
        target = next(t for t in self.result()['targets'] if t['id'] == 'fig9.tagging.density0_1')
        self.assertEqual(target['precision_census']['invalid_count'], 1)
        self.assertIsNone(target['p'])

    def test_all_extinction_preserves_territory_inference_but_not_clustering(self):
        for family in ('original', 'precision'):
            arm = family + '.' + manifest.arm_suffix(.5, 'tagging', .1)
            for h in self.histories[arm]:
                h['metrics'].update(democratic_share=0., clustering_ratio=None, democratic_extinction=True)
        targets = {t['id']: t for t in self.result()['targets']}
        self.assertIsNotNone(targets['fig9.tagging.density0_1']['p'])
        self.assertEqual(targets['fig10.tagging.density0_1']['unavailable_reason'], 'insufficient_conditional_reference')
        self.assertEqual(targets['fig10.tagging.density0_1']['original_census']['extinction_count'], 30)

    def test_zero_readable_scope_never_vacuously_compatible(self):
        for slot in self.table['slots']:
            slot.update(read_status='absent_source_point', interval=None)
        result = self.result()
        self.assertEqual(result['scopes']['all']['conditional_verdict'], 'Unresolved')
        self.assertEqual(result['scopes']['all']['reason'], 'no_readable_source_targets')
        self.assertEqual(len(result['targets']), 105)

    def test_original_only_never_substitutes_thirty_for_hundred(self):
        result = source.source_findings(self.table, self.histories, self.jobs,
            precision_registered=False, draws=20, fixture=True)
        self.assertTrue(all(t['unavailable_reason'] == 'precision_not_registered' for t in result['targets']))


if __name__ == '__main__':
    unittest.main()
