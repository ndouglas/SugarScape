import math
import random
import unittest

import episode

HOLDS = dict(vi2_late=837.0, vi3_late=821.0, vi2_dip=174.0, vi3_dip=123.0, vi2_min_late=725.0, vi3_max=914.0,
             vi3_cycle=False, ii6_travelers=0.02, iii12_mixed=0.0, iii14_pop=10.0, trades=31330.0,
             foresight_start=4.98, foresight_end=4.41, v2_end=0.0)
SWEEP = [(0, 0, 0, 0)] * 16


def rows(**changes):
    out = {s: dict(HOLDS, **changes) for s in range(1, 21)}
    out[20]["vi3_max"] = 1005.0  # one world that doubles
    for s in range(13, 21):
        out[s]["foresight_end"] = 5.2  # eight where foresight rises
    return out


class FinaleVerdictTest(unittest.TestCase):
    claims = episode.load_module("finale", "claims")
    NONE = "no crash after the first dip in 20 of 20 worlds; no doubling in 19; no 115-year cycles"
    UNSAID = "however we fill in what the book leaves unsaid, trade never turns a crash into a boom"

    def verdicts(self, r=None, sweep=SWEEP):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), sweep)}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_crash_after_the_dip_breaks_the_no_crash_count(self):
        r = rows()
        r[3]["vi2_min_late"] = 100.0
        self.assertFalse(self.verdicts(r)[self.NONE])

    def test_one_cycling_world_breaks_the_no_cycles_claim(self):
        r = rows()
        r[5]["vi3_cycle"] = True
        self.assertFalse(self.verdicts(r)[self.NONE])

    def test_a_reading_with_the_books_contrast_in_most_worlds_breaks_the_unsaid_claim(self):
        self.assertFalse(self.verdicts(sweep=[(0, 0, 0, 0)] * 15 + [(14, 12, 11, 0)])[self.UNSAID])
        self.assertTrue(self.verdicts(sweep=[(0, 0, 0, 0)] * 15 + [(14, 12, 10, 0)])[self.UNSAID])

    def test_never_meet_is_said_only_when_no_world_mixes(self):
        r = rows()
        r[4]["iii12_mixed"] = 0.01
        said = [c for c, _, _ in self.claims.verdicts(r, SWEEP) if "III-12" in c]
        self.assertEqual(said, ["two tribes that collide (III-12): in 19 of 20 worlds they never meet"])

    def test_travelling_waves_break_the_stay_put_claim(self):
        self.assertFalse(self.verdicts(rows(ii6_travelers=0.5))["waves that sweep across the land (II-6): here the Flumps stay put"])


class FinaleHelperTest(unittest.TestCase):
    claims = episode.load_module("finale", "claims")

    def test_the_cycle_finder_sees_a_115_tick_cycle_through_drift_and_not_noise(self):
        rng = random.Random(1)
        sine = [800 + 60 * math.sin(2 * math.pi * t / 115) + 0.3 * t + rng.gauss(0, 5) for t in range(1001)]
        noise = [800 + 0.3 * t + rng.gauss(0, 20) for t in range(1001)]
        eighty = [800 + 60 * math.sin(2 * math.pi * t / 80) + rng.gauss(0, 5) for t in range(1001)]
        self.assertTrue(self.claims._cycle(sine))
        self.assertFalse(self.claims._cycle(noise))
        self.assertFalse(self.claims._cycle(eighty))

    def test_distance_from_the_block_wraps(self):
        beyond = self.claims._beyond
        self.assertEqual(beyond(19, 30), 0)
        self.assertEqual(beyond(29, 30), 10)
        self.assertEqual(beyond(45, 30), 5)  # across the west edge
        self.assertEqual(beyond(5, 5), 6)  # across the south edge


if __name__ == "__main__":
    unittest.main()
