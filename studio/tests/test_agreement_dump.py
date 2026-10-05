import collections
import json
import pathlib
import unittest

import dump


class AgreementDumpTest(unittest.TestCase):
    def setUp(self):
        self.raw = dict(format=1, model="agreement", seed=7, ticks=2, every=3, agents=6,
                        starts=[-1, 1, 0, -0.5, 0.5, 0], roles=["minus", "plus"] + ["moderate"] * 4,
                        config={"model": "agreement"}, stats={}, frames=[
                            dict(tick=k * 3, opinions=values, uncertainties=[0.1, 0.1, 1.4, 1.4, 1.4, 1.4])
                            for k, values in enumerate([[-1, 1, 0, -0.5, 0.5, 0], [-1, 1, 1, 1, 1, 1], [-1, 1, 1, 1, 1, 1]])])
        self.d = dump.parse(json.dumps(self.raw))

    def test_kept_frame_clock_and_exact_state(self):
        self.assertEqual([f.tick for f in self.d.frames], [0, 1, 2])
        self.assertEqual([f.period for f in self.d.frames], [0, 3, 6])
        for f, raw in zip(self.d.frames, self.raw["frames"]):
            self.assertEqual(f.opinions, dict(enumerate(raw["opinions"])))
            self.assertEqual(f.uncertainties, dict(enumerate(raw["uncertainties"])))

    def test_initial_roles_and_start_colors_are_preserved(self):
        self.assertEqual(self.d.roles, dict(enumerate(self.raw["roles"])))
        self.assertEqual(self.d.frames[0].groups, {0: 0, 1: 9, 2: 5, 3: 2, 4: 7, 5: 5})
        self.assertEqual(self.d.start_rank, {0: 0, 3: 1, 2: 2, 5: 3, 4: 4, 1: 5})

    def test_endpoint_mapping_and_no_missing_or_duplicated_agents(self):
        columns = dump.OPINION_COLUMNS // 2
        for f in self.d.frames:
            self.assertEqual(set(f.agents), set(range(6)))
            self.assertEqual(len({(a.x, a.y) for a in f.agents.values()}), 6)
            for i, a in f.agents.items():
                expected = min(int((f.opinions[i] + 1) / 2 * columns), columns - 1)
                self.assertEqual(a.x // dump.BLOCK, expected)
            bins = collections.defaultdict(list)
            for a in f.agents.values():
                bins[a.x // dump.BLOCK].append((self.d.height - 1 - a.y) * dump.BLOCK + a.x % dump.BLOCK)
            for slots in bins.values():
                self.assertEqual(sorted(slots), list(range(len(slots))))

    def test_stable_repeated_frames_have_identical_layout(self):
        self.assertEqual(self.d.frames[1].agents, self.d.frames[2].agents)

    def test_engine_fixture_loads_exact_values_and_roles(self):
        path = pathlib.Path(__file__).parent / "fixtures" / "agreement.frames.json"
        raw = json.loads(path.read_text())
        d = dump.load(path)
        self.assertEqual(d.roles, dict(enumerate(raw["roles"])))
        self.assertEqual(d.ticks, 4)
        self.assertEqual(d.frames[-1].period, 12)
        self.assertEqual(d.frames[0].opinions, dict(enumerate(raw["starts"])))
        for f, expected in zip(d.frames, raw["frames"]):
            self.assertEqual(f.opinions, dict(enumerate(expected["opinions"])))
            self.assertEqual(f.uncertainties, dict(enumerate(expected["uncertainties"])))
