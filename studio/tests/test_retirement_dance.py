"""Recorded identity, aging and modest musical motion."""
import dataclasses
import unittest
import retirement_dance as dance


def member(**changes):
    return dict(dict(id=7595, born=-75, age=98, retired=True, kind='imitator'), **changes)


class DanceTests(unittest.TestCase):
    def test_retired_steps_are_visible_at_crowd_scale(self):
        pose=dance.dance_pose(dance.BAR_SECONDS/4, member())
        self.assertGreaterEqual(pose.left_foot[2],.07)
        self.assertGreaterEqual(abs(pose.left_foot[1]),.06)
        self.assertGreaterEqual(abs(pose.offset[0]),.065)

    def test_pose_is_immutable(self):
        with self.assertRaises(dataclasses.FrozenInstanceError):
            dance.dance_pose(1, member()).yaw = 1

    def test_working_members_keep_both_feet_still(self):
        poses = [dance.dance_pose(t, member(retired=False)) for t in (0, .3, 2, 7)]
        self.assertTrue(all(p.left_foot == (0, 0, 0) and p.right_foot == (0, 0, 0) for p in poses))

    def test_bar_and_phrase_boundaries_are_continuous(self):
        for t in (dance.BAR_SECONDS, 8*dance.BAR_SECONDS, 16*dance.BAR_SECONDS):
            a, b = (dance.dance_pose(t+dt, member()) for dt in (-1e-6, 1e-6))
            self.assertLess(max(abs(x-y) for x,y in zip(dataclasses.astuple(a)[0], dataclasses.astuple(b)[0])), 1e-5)
            self.assertLess(abs(a.yaw-b.yaw), 1e-5)

    def test_aging_is_independent_of_retirement_and_renewal_removes_cues(self):
        self.assertEqual(dance.age_style(member()), dance.age_style(member(retired=False)))
        self.assertTrue(dance.age_style(member(), hero=True).spectacles)
        self.assertFalse(dance.age_style(member(age=20, born=1), hero=True).elderly)

    def test_birth_identity_changes_dance_style(self):
        self.assertNotEqual(dance.dance_pose(.37, member()), dance.dance_pose(.37, member(born=1)))

    def test_kind_selection_uses_actual_eligibility_and_stable_order(self):
        members = {3: member(id=3, age=20), 2: member(id=2, age=70), 1: member(id=1, age=68)}
        self.assertEqual(dance.eligible_member(members, 'imitator', 65)['id'], 1)
        with self.assertRaises(ValueError):
            dance.eligible_member(members, 'rational', 65)

class InvitationTests(unittest.TestCase):
    def test_join_endpoints_preserve_source_and_full_pose(self):
        source = member()
        self.assertEqual(dance.join_pose(3, source, 0), dance.dance_pose(3, member(retired=False)))
        self.assertEqual(dance.join_pose(3, source, 1), dance.dance_pose(3, source))
        self.assertTrue(source['retired'])

    def test_boundary_and_random_access_are_continuous(self):
        onset = 4.4
        before = dance.join_pose(25.4, member(retired=False), dance.join_amount(onset, onset))
        after = dance.join_pose(25.4, member(), dance.join_amount(onset, onset))
        self.assertEqual(before, after)
        samples = [dance.join_pose(21+t, member(), dance.join_amount(t, onset)) for t in (8, 4.4, 5, 2, 8)]
        self.assertEqual(samples[0], samples[-1])
        self.assertAlmostEqual(dance.join_amount(onset+1e-6, onset), 0, places=10)
        self.assertEqual(dance.join_amount(onset+2, onset), 1)

    def test_attention_eases_between_actual_targets_and_releases(self):
        targets = (.18, .27)
        self.assertEqual(dance.invitation_gaze(0, targets), 0)
        self.assertAlmostEqual(dance.invitation_gaze(1.2, targets), targets[0])
        self.assertAlmostEqual(dance.invitation_gaze(3.6, targets), targets[1])
        self.assertEqual(dance.invitation_gaze(7, targets, onset=4.4), 0)
        self.assertEqual(dance.invitation_gaze(2, ()), 0)

    def test_actual_counted_friend_positions_produce_distinct_gaze(self):
        host = (-1.65, 0, 0)
        angles = tuple(dance.attention_angle(host, friend) for friend in
                       ((.05, .25, 0), (1.7, .25, 0)))
        self.assertGreater(angles[1]-angles[0], .08)
        self.assertAlmostEqual(dance.invitation_gaze(3.6, angles), angles[1])
