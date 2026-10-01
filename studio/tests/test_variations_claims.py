import unittest

import episode

CONFIG = {"changed": {"pv-flat": "movers anyone, movement best", "gvn-segregated": "movers anyone",
                      "svw-small": "start deleted_checkerboard", "zhang-checkerboard": "movement swap"},
          "pv": ("anyone", "best"), "gvn": ("anyone", "random"), "ring": "p100",
          "zhang": (10000, 10000, "swap", "tent")}
ROW = dict(flat_clusters=2.0, flat_split=1.0, flat_first_split=2.0, p50_clusters=2.0, p50_split=1.0,
           p50_first_split=2.0, p100_clusters=4.5, p100_split=0.0, p100_first_split=99999.0, spiked_clusters=7.0,
           spiked_split=0.0, spiked_first_split=300.0, ring_groups=2.0, ring_first_two=3.0, frozen_s=0.025,
           frozen_still=1.0, frozen_clusters=415.0, segregated_s=1.0, segregated_still=0.0, segregated_clusters=2.0,
           mixed_s=0.05, mixed_still=0.0, mixed_clusters=290.0, small_clusters=2.0, small_rest=3.0,
           large_clusters=57.0, large_rest=9.0, zhang_start=20000.0, zhang_100=3600.0, zhang_end=2000.0,
           zhang_first_8000=10.0)


def rows(**changes):
    """20 seeds, 17 of them split under flat preferences and 19 under p50."""
    r = {s: dict(ROW, **changes) for s in range(1, 21)}
    for s in (1, 2, 3):
        r[s]["flat_split"], r[s]["flat_clusters"] = 0.0, 3.0
    r[1]["p50_split"], r[1]["p50_clusters"] = 0.0, 3.0
    return r


class VariationsVerdictTest(unittest.TestCase):
    claims = episode.load_module("variations", "claims")

    def verdicts(self, r=None, **config):
        return {c: h for c, h, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_p50_splitting_no_more_often_breaks_even_more_often(self):
        r = rows()
        for s in (2, 3):
            r[s]["p50_split"] = 0.0
        self.assertFalse(self.verdicts(r)["wanting a mixed street, up to half and half, splits them even more "
                                          "often"])

    def test_one_ring_in_three_groups_breaks_the_ring(self):
        r = rows()
        r[5]["ring_groups"] = 3.0
        self.assertFalse(self.verdicts(r)["in a ring, even Flumps who like half and half best end in two groups"])

    def test_a_board_still_moving_breaks_frozen(self):
        r = rows()
        r[2]["frozen_still"] = 0.0
        self.assertFalse(self.verdicts(r)["tolerate too little, and nobody can move"])

    def test_a_mixed_board_that_sorts_breaks_stays_mixed(self):
        self.assertFalse(self.verdicts(rows(mixed_s=0.6))["tolerate most, and the town stays mixed"])

    def test_a_big_city_with_few_clusters_breaks_dozens(self):
        v = self.verdicts(rows(large_clusters=6.0))
        self.assertFalse(v["in a city of 100 by 100, dozens; Schelling's striking picture is a small-town effect"])

    def test_spiked_like_p100_breaks_the_misses(self):
        v = self.verdicts(rows(spiked_clusters=4.6))
        self.assertFalse(v["two claims don't reproduce here: Zhang's waiting times, and that wanting only a perfect "
                           "mix acts like wanting half and half"])

    def test_zhang_as_slow_as_his_figure_breaks_the_misses(self):
        v = self.verdicts(rows(zhang_first_8000=4000.0))
        self.assertFalse(v["two claims don't reproduce here: Zhang's waiting times, and that wanting only a perfect "
                           "mix acts like wanting half and half"])
