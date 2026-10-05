import json
import unittest
import dump


class FarolDumpTest(unittest.TestCase):
    def raw(self):
        def members(went):
            return [dict(id=i, memory=2, went=choice, selected=None if choice is None else 0,
                         strategies=[dict(label='strategy 1', score=0.0, forecast=None,
                                          attend=True, active=choice is not None)])
                    for i, choice in enumerate(went, 1)]
        return dict(format=1, model='farol', seed=9, ticks=1, every=5, agents=3,
                    game='minority', config=dict(game='minority', agents=3), stats={}, frames=[
                        dict(tick=0, attendance=0, capacity=1, crowded=None, winning_attend=None,
                             attendance_history=[], history_bits=None, agents=members([None]*3)),
                        dict(tick=5, attendance=1, capacity=1, crowded=False, winning_attend=True,
                             attendance_history=[2, 1, 2, 2], history_bits=6, agents=members([True, False, False]))])

    def test_actual_members_history_population_and_sampled_round_survive(self):
        d = dump.parse(json.dumps(self.raw()))
        self.assertEqual(d.model, 'farol')
        self.assertEqual(d.frames[1].period, 5)
        self.assertEqual(d.frames[1].farol['history_bits'], 6)
        self.assertEqual(d.frames[1].counts, [2, 1])
        self.assertEqual(d.frames[0].members[1]['went'], None)
        self.assertEqual(d.frames[1].members[1]['strategies'][0]['score'], 0.0)
        self.assertEqual(set(d.frames[1].agents), {1, 2, 3})
        self.assertEqual(d.frames[0].agents[2].x, d.frames[1].agents[2].x)
        self.assertEqual(len({(a.x, a.y) for a in d.frames[1].agents.values()}), 3)

    def test_duplicate_or_missing_ids_are_rejected(self):
        for bad in ([1, 1, 3], [1, 2]):
            raw = self.raw()
            raw['frames'][1]['agents'] = [dict(raw['frames'][1]['agents'][0], id=i) for i in bad]
            with self.assertRaises(ValueError):
                dump.parse(json.dumps(raw))

    def test_attendance_disagreement_is_rejected(self):
        raw = self.raw()
        raw['frames'][1]['attendance'] = 2
        with self.assertRaises(ValueError):
            dump.parse(json.dumps(raw))
