"""Source-exact cast and presentation transitions without Blender."""
import unittest
from types import SimpleNamespace
import animate
from retirement_visual import CastTimeline, smooth_visibility, eased_target


def member(slot, born=0, age=70, retired=False, group=0, network=()):
    return dict(id=slot, born=born, age=age, retired=retired, kind='imitator',
                group=group, network=list(network))


def recording(rows):
    return SimpleNamespace(ticks=len(rows)-1, frames=[SimpleNamespace(
        members={m['id']:m for m in row}, retirement={'eligibility':65}, period=i)
        for i,row in enumerate(rows)])


class TransitionTests(unittest.TestCase):
    def timeline(self, rows, candidates=None):
        d=recording(rows)
        return CastTimeline(d, animate.Timing(1), 61, [0,1],
                            candidates or (lambda f:list(f.members)))

    def test_retains_living_suitable_identity_when_candidate_order_changes(self):
        t=self.timeline([[member(0),member(1)], [member(0,retired=True),member(1)],
                         [member(0,retired=True),member(1)]],lambda f:[1,0])
        self.assertEqual(t.ids(31),[0,1])
        self.assertEqual(t.amount(31,0),.15625)

    def test_birth_replacement_has_zero_visibility_at_exact_source_switch(self):
        t=self.timeline([[member(0),member(1)], [member(0,born=1,age=20),member(1),member(2)],
                         [member(0,born=1,age=21),member(1),member(2)]])
        self.assertEqual(t.ids(15.999),[0,1])
        self.assertEqual(t.ids(16),[2,1])
        self.assertEqual(t.visibility(16,0),0)
        self.assertEqual(t.visibility(4,0),1)
        self.assertEqual(t.visibility(28,0),1)

    def test_repeated_retirement_is_a_new_onset_for_actual_new_birth(self):
        d=recording([[member(0)], [member(0,retired=True)],
                     [member(0,born=1)], [member(0,born=1,retired=True)]])
        t=CastTimeline(d,animate.Timing(1),91,[0],lambda f:[0],eligible=False)
        self.assertEqual(t.amount(46,0),0)
        self.assertEqual(t.amount(76,0),0)
        self.assertAlmostEqual(t.amount(91,0),.15625)
        self.assertAlmostEqual(t.amount(31,0),.15625)

    def test_group_cast_retains_holder_but_only_current_contacts_or_eligible_fillers(self):
        d=recording([[member(0,network=[1]),member(1,age=25,group=1),member(2)],
                     [member(0,network=[2]),member(1,age=26,group=1),member(2,group=1)]])
        t=CastTimeline(d,animate.Timing(1),31,[0,1],lambda f:[0]+f.members[0]['network'],groups=True)
        self.assertEqual(t.ids(31),[0,2])
        self.assertEqual(t.links(31),[(0,2)])

    def test_random_access_is_exact_and_unique(self):
        t=self.timeline([[member(0),member(1)], [member(0),member(1)], [member(0),member(1)]])
        forward=[(t.ids(f),t.amount(f,0),t.gaze(f,0,[(0,0,0),(1,0,0)])) for f in [1,16,31,61]]
        backward=[(t.ids(f),t.amount(f,0),t.gaze(f,0,[(0,0,0),(1,0,0)])) for f in [61,31,16,1]]
        self.assertEqual(forward,list(reversed(backward)))
        self.assertTrue(all(len(set(ids))==len(ids) for ids,_,_ in forward))

    def test_target_changes_are_continuous_and_plot_values_need_no_interpolation(self):
        events=[(1,0.),(16,.2),(31,-.2)]
        self.assertEqual(eased_target(events,16),0)
        self.assertAlmostEqual(eased_target(events,31-1e-5),eased_target(events,31),places=6)
        self.assertEqual(smooth_visibility(16,[16]),0)

    def test_dense_handoffs_have_a_full_visibility_hold_between_switches(self):
        self.assertEqual(smooth_visibility(16,[16,27,38]),0)
        self.assertEqual(smooth_visibility(21.5,[16,27,38]),1)
        self.assertEqual(smooth_visibility(32.5,[16,27,38]),1)

    def test_policy_switch_selects_current_newly_eligible_workers_without_retiring(self):
        d=recording([[member(0),member(1),member(2,age=62)],
                     [member(0),member(1),member(2,age=63)]])
        d.frames[1].retirement['eligibility']=62
        t=CastTimeline(d,animate.Timing(1),31,[0,1],
                       lambda f:[2,0,1] if f.retirement['eligibility']==62 else [0,1],policy=True)
        self.assertEqual(t.ids(16),[2,1])
        self.assertEqual(t.amount(16,0),0)
        self.assertEqual(t.visibility(16,0),0)

    def test_holder_replacement_does_not_duplicate_former_contact_seat(self):
        d=recording([[member(0,network=[1]),member(1,group=1,network=[0]),member(2)],
                     [member(0),member(1,group=1,network=[2]),member(2)]])
        t=CastTimeline(d,animate.Timing(1),31,[0,1],lambda f:[1,2],groups=True)
        self.assertEqual(t.ids(16),[1,2])
        self.assertEqual(t.visibility(16,0),0)

    def test_eligible_former_contact_is_replaced_by_actual_current_contact(self):
        d=recording([[member(0,network=[1]),member(1,group=1),member(2,group=1)],
                     [member(0,network=[2]),member(1,group=1),member(2,group=1)]])
        t=CastTimeline(d,animate.Timing(1),31,[0,1],lambda f:[0]+f.members[0]['network'],groups=True)
        self.assertEqual(t.ids(16),[0,2])
        self.assertEqual(t.links(16),[(0,2)])
