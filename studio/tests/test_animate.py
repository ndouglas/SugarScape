import pathlib
import unittest

import animate
import dump

FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tiny.frames.json"


def track(first, cells, death=None, sugar=None):
    return dump.Track(1, first, cells, sugar or [10.0] * len(cells), death, "starvation" if death else None)


class TimingTest(unittest.TestCase):
    def test_frame_and_tick_are_inverse(self):
        t = animate.Timing(ticks_per_second=2, lead_in=1.0, end_tick=10)
        self.assertEqual(t.frame(0), 31)
        self.assertEqual(t.frame(1), 46)
        self.assertAlmostEqual(t.tick_at(t.frame(3.5)), 3.5)

    def test_tick_at_clamps(self):
        t = animate.Timing(ticks_per_second=2, start_tick=4, end_tick=6)
        self.assertEqual(t.tick_at(-100), 4)
        self.assertEqual(t.tick_at(10_000), 6)


class BoardTest(unittest.TestCase):
    def test_flat_board_is_flat_and_cells_are_centered(self):
        corners = animate.corner_heights([2.0] * 16, 4, 4)
        self.assertEqual(len(corners), 25)
        self.assertAlmostEqual(animate.cell_height(corners, 1, 2, 4), 2 * animate.HEIGHT_PER_SUGAR)
        self.assertEqual(animate.cell_center(0, 0, 4, 4), (-1.5, 1.5))
        self.assertEqual(animate.cell_center(3, 3, 4, 4), (1.5, -1.5))

    def test_levels_before_start_are_frame_zero(self):
        d = dump.load(FIXTURE)
        self.assertEqual(animate.levels_at(d, -3), d.frames[0].sugar)
        self.assertEqual(animate.levels_at(d, 99), d.frames[-1].sugar)

    def test_eaten_sugar_snaps_at_landing_and_growback_is_linear(self):
        d = dump.load(FIXTURE)
        before, after = d.frames[0].sugar, d.frames[1].sugar
        eaten = [i for i, (a, b) in enumerate(zip(before, after)) if b < a]
        self.assertTrue(eaten)
        i = eaten[0]
        self.assertEqual(animate.levels_at(d, 0.8)[i], before[i])
        self.assertEqual(animate.levels_at(d, 0.9)[i], after[i])
        grown = [i for i, (a, b) in enumerate(zip(d.frames[1].sugar, d.frames[2].sugar)) if b > a]
        self.assertTrue(grown)
        j = grown[0]
        mid = (d.frames[1].sugar[j] + d.frames[2].sugar[j]) / 2
        self.assertAlmostEqual(animate.levels_at(d, 1.5)[j], mid)


class PoseTest(unittest.TestCase):
    corners = animate.corner_heights([0.0] * 64, 8, 8)
    # Half a second a tick: shorter than HOP_SECONDS, so a hop fills its tick.
    timing = animate.Timing(ticks_per_second=2, lead_in=1.0)

    def at(self, t, frame):
        return animate.pose(t, self.timing, frame, self.corners, 8, 8)

    def test_still_agent_rests_on_its_cell(self):
        p = self.at(track(0, [(2, 2), (2, 2)]), self.timing.frame(0.5))
        self.assertEqual((p.x, p.y), animate.cell_center(2, 2, 8, 8))
        self.assertAlmostEqual(p.z, 0)
        self.assertTrue(p.visible)

    def test_hop_arcs_between_cells_and_keeps_volume(self):
        t = track(0, [(2, 2), (4, 2)])
        mid = self.at(t, self.timing.frame(0.5))
        self.assertGreater(mid.z, 0.3)
        halfway = (animate.cell_center(2, 2, 8, 8)[0] + animate.cell_center(4, 2, 8, 8)[0]) / 2
        self.assertAlmostEqual(mid.x, halfway, places=1)
        self.assertAlmostEqual(mid.sx * mid.sy * mid.sz, 1, places=6)
        landed = self.at(t, self.timing.frame(1))
        self.assertEqual((landed.x, landed.y), animate.cell_center(4, 2, 8, 8))

    def test_slow_ticks_stand_still_then_hop_quickly_at_the_end(self):
        slow = animate.Timing(ticks_per_second=0.25, lead_in=1.0)  # 4 s a tick
        t = track(0, [(2, 2), (4, 2)])
        standing = animate.pose(t, slow, slow.frame(0.5), self.corners, 8, 8)
        self.assertEqual((standing.x, standing.y, standing.sz), (*animate.cell_center(2, 2, 8, 8), 1.0))
        hop_start = slow.frame(1) - animate.HOP_SECONDS * slow.fps
        airborne = animate.pose(t, slow, hop_start + 0.5 * animate.HOP_SECONDS * slow.fps, self.corners, 8, 8)
        self.assertGreater(airborne.z, 0.3)

    def test_eaten_sugar_snaps_when_a_slow_hop_lands(self):
        d = dump.load(FIXTURE)
        before, after = d.frames[0].sugar, d.frames[1].sugar
        i = next(i for i, (a, b) in enumerate(zip(before, after)) if b < a)
        hop = 0.25  # the hop takes the last quarter of the tick
        self.assertEqual(animate.levels_at(d, 0.9, hop)[i], before[i])
        self.assertEqual(animate.levels_at(d, 0.97, hop)[i], after[i])

    def test_wrap_move_shrinks_instead_of_gliding(self):
        t = track(0, [(0, 3), (7, 3)])
        half = self.at(t, self.timing.frame(0.5))
        self.assertLess(half.sz, 0.05)
        early = self.at(t, self.timing.frame(0.3))
        self.assertEqual(early.x, animate.cell_center(0, 3, 8, 8)[0])
        late = self.at(t, self.timing.frame(0.7))
        self.assertEqual(late.x, animate.cell_center(7, 3, 8, 8)[0])

    def test_spawn_and_poof_bound_visibility(self):
        t = track(2, [(1, 1), (1, 1)], death=4)
        self.assertFalse(self.at(t, self.timing.frame(2) - animate.SPAWN_FRAMES - 1).visible)
        self.assertTrue(self.at(t, self.timing.frame(2)).visible)
        self.assertTrue(self.at(t, self.timing.frame(3.4)).visible)
        self.assertFalse(self.at(t, self.timing.frame(3.5) + animate.POOF_FRAMES + 1).visible)

    def test_fast_beats_never_show_a_flump_outside_its_life(self):
        # At 40 ticks a second a tick is 0.75 frames: a fixed-length spawn
        # or poof would show Flumps for many ticks before birth or after death.
        fast = animate.Timing(ticks_per_second=40, lead_in=1.0)
        t = track(10, [(1, 1), (1, 1)], death=12)
        tick_frames = fast.fps / fast.ticks_per_second
        for frame10 in range(0, 1000):
            frame = frame10 / 10
            p = animate.pose(t, fast, frame, self.corners, 8, 8)
            if p.visible:
                self.assertGreaterEqual(frame, fast.frame(10) - tick_frames - 1e-9, frame)
                self.assertLessEqual(frame, fast.frame(12) + 1e-9, frame)

    def test_single_frame_track_poses(self):
        t = track(0, [(5, 5)], death=1)
        self.assertTrue(self.at(t, self.timing.frame(0.2)).visible)
        self.assertFalse(self.at(t, self.timing.frame(3)).visible)

    def test_hunger_rises_as_sugar_falls(self):
        full = self.at(track(0, [(1, 1)] * 2, sugar=[12.0, 12.0]), self.timing.frame(0))
        empty = self.at(track(0, [(1, 1)] * 2, sugar=[0.5, 0.5]), self.timing.frame(0))
        self.assertEqual(full.hunger, 0)
        self.assertGreater(empty.hunger, 0.9)


class HelpersTest(unittest.TestCase):
    def test_blink_is_mostly_open_and_deterministic(self):
        values = [animate.blink(7, f) for f in range(600)]
        self.assertGreater(sum(v == 1 for v in values), 550)
        self.assertLess(min(values), 0.2)
        self.assertEqual(values, [animate.blink(7, f) for f in range(600)])

    def test_sight_cells_wrap_nearest_first(self):
        n, e, s, w = animate.sight_cells(0, 0, 2, 5, 5)
        self.assertEqual(n, [(0, 4), (0, 3)])
        self.assertEqual(e, [(1, 0), (2, 0)])
        self.assertEqual(s, [(0, 1), (0, 2)])
        self.assertEqual(w, [(4, 0), (3, 0)])

    def test_shares_split_the_sugar_by_rank(self):
        # Ten Flumps: the poorest five hold 1 each, the next four 2, the richest 11.
        values = [1] * 5 + [2] * 4 + [11]
        poor, middle, rich = animate.shares(values)
        self.assertAlmostEqual(poor, 5 / 24)
        self.assertAlmostEqual(middle, 8 / 24)
        self.assertAlmostEqual(rich, 11 / 24)

    def test_shares_of_nothing_are_equal(self):
        self.assertEqual(animate.shares([0, 0, 0, 0]), (0.5, 0.4, 0.1))
        self.assertEqual(animate.shares([]), (0.5, 0.4, 0.1))

    def test_histogram_bins_and_clamps(self):
        self.assertEqual(animate.histogram([0, 1, 4.9, 5, 9.9, 100], 2, 10), [3, 3])


if __name__ == "__main__":
    unittest.main()
