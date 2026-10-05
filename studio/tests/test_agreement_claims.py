import unittest
import episode


class AgreementCountingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.c = episode.load_module('agreement', 'claims')

    def test_role_preserves_moderate_denominator_and_period_zero_baseline(self):
        agents = [dict(role='plus', start='.9', opinion='.1'),
                  dict(role='minus', start='-.9', opinion='-.1'),
                  dict(role='moderate', start='.85', opinion='.85'),
                  dict(role='moderate', start='0', opinion='.9')]
        r = self.c.count_agents(agents, dict(placement='drawn', extreme_margin=.1))
        self.assertEqual((r['p_plus'], r['baseline_plus'], r['extremist_abs']), (1, .5, .1))

    def test_cutoffs_use_initial_positions_even_when_extremists_move(self):
        agents = [dict(role='plus', start='.9', opinion='.1'),
                  dict(role='minus', start='-.9', opinion='-.1'),
                  dict(role='moderate', start='0', opinion='.8')]
        self.assertEqual(self.c.count_agents(agents, dict(placement='drawn', extreme_margin=.1))['p_plus'], 0)

    def test_printed_requires_direct_extremist_measurement(self):
        rows = {s: dict(y=0, extremist_abs=.8) for s in range(1, 51)}
        self.assertFalse(self.c.printed_holds(rows))

    def test_printed_requires_fifty_seeds(self):
        self.assertFalse(self.c.printed_holds({s: dict(y=0, extremist_abs=.1) for s in range(1, 21)}))


class AgreementProspectiveRuleTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.c = episode.load_module('agreement', 'claims')

    def data(self):
        data = {name: {s: dict(middle=.8, both=True, near_all=True, single=True,
                              sign=1 if s % 2 else -1, initial_plus=5, initial_minus=5,
                              drift=.8, tick=1200, stable=True, y=1, initial_agents=200, initial_extremists=10,
                              initial_uncertainties=[.1,1.4],rule='ra',pair_update='simultaneous',initial_pair_exists=True,
                              p_plus=.95, p_minus=0, extremist_abs=.1)
                       for s in range(1, 51)} for name in self.c.CONFIGS}
        data['middle']={s:dict(data['middle'][s],middle=.8 if s<=11 else .4) for s in range(1,21)}
        for name in ('early','size-big','lean-balanced','horizon-only','cutoff-only','neighbors'):
            for r in data[name].values():
                r.update(single=False,p_plus=.3,p_minus=.3)
        for r in data['lean'].values(): r.update(initial_plus=55,initial_minus=45,sign=1)
        for r in data['printed'].values(): r['y']=0
        return data

    def verdict(self, data, beat):
        return self.c.verdicts(data)[beat-1][1]

    def test_good_data_supports_all_empirical_rules(self):
        self.assertTrue(all(v[1] for v in self.c.verdicts(self.data())))

    def test_single_requires_forty_nearly_all_and_both_signs(self):
        data=self.data()
        for s in range(1,12): data['single'][s]['near_all']=False
        self.assertFalse(self.verdict(data,6))

    def test_population_requires_every_run_stable(self):
        data=self.data(); data['size-big'][1]['stable']=False
        self.assertFalse(self.verdict(data,10))

    def test_reply_combination_must_exceed_either_alone(self):
        data=self.data()
        for r in data['horizon-only'].values(): r['single']=True
        self.assertFalse(self.verdict(data,9))

    def test_lattice_rejects_a_seventy_percent_side(self):
        data=self.data(); data['neighbors'][1]['p_plus']=.7
        self.assertFalse(self.verdict(data,13))

    def test_opening_rejects_a_different_crowd_size(self):
        data=self.data(); data['crowd'][1]['initial_agents']=201
        self.assertFalse(self.verdict(data,1))

    def test_pair_rule_rejects_sequential_updates(self):
        data=self.data(); data['crowd'][1]['pair_update']='sequential'
        self.assertFalse(self.verdict(data,3))

    def test_revised_middle_caption_requires_exact_reported_frequency(self):
        data=self.data(); data['middle'][12]['middle']=.8
        self.assertFalse(self.verdict(data,4))
