"""Hand-selected static trace checks for reporting counts, never effect estimates."""
import unittest
import diagnostics_p4


def frame(tick, actions=(), choices=(), observations=()):
    return dict(tick=tick, roles=[dict(id=1,pos=dict(x=3,y=3))], actions=list(actions),
                choices=list(choices), observations=list(observations), deaths=[], restrictions={})


class DiagnosticsTests(unittest.TestCase):
    def test_empty_arrival_separates_false_estimate_inspection_and_physical_transfer(self):
        choice = dict(actor=2,target=dict(x=5,y=6),remembered_value=12.,actual_value=0.,
                      arrived=True,raid_amount=0.,wasted=True,inspection=dict(x=5,y=6),
                      inspected_stock=0.,target_occupant=None,source_recovered=False)
        group = dict(id='fixture',members=['sham-ambiguous-seen-off-route-cost0-m0 seed 20001'],
                     frames=[frame(0),frame(33,choices=[choice])])
        result = diagnostics_p4.summarize(group)
        self.assertIsInstance(result, dict)
        self.assertEqual(result['receiver_false_display_choices'], 1)
        self.assertEqual(result['receiver_display_arrivals'], 1)
        self.assertEqual(result['receiver_display_gross_raid'], 0.)
        self.assertEqual(result['receiver_wasted_display_inspections'], 1)

    def test_blocked_target_does_not_count_as_an_arrival(self):
        choice = dict(actor=2,target=dict(x=5,y=6),remembered_value=12.,actual_value=0.,
                      arrived=False,raid_amount=0.,wasted=False,inspection=dict(x=4,y=6),
                      inspected_stock=0.,target_occupant=1,source_recovered=False)
        group = dict(id='fixture',members=['sham-ambiguous-seen-off-route-cost3-m0 seed 20001'],
                     frames=[frame(0),frame(33,choices=[choice])])
        result = diagnostics_p4.summarize(group)
        self.assertIsInstance(result, dict)
        self.assertEqual(result['receiver_display_arrivals'], 0)
        self.assertEqual(result['receiver_occupied_display_choices'], 1)

    def test_physical_source_dig_precedes_departure_flag_and_retains_body_choices(self):
        recovery = dict(actor=1,phase='ordinary',action='ordinary',pos=dict(x=3,y=3),
                        target=dict(x=3,y=3),harvest=0.,dug=12.,buried=0.,effort=0.,
                        metabolic_demand=1.,metabolic_consumed=1.,cancellation=None,
                        walk_outcome=None,target_occupant=None,source_recovered=False)
        departure = dict(recovery,phase='departure',action='walk_and_gather',
                         pos=dict(x=3,y=2),target=dict(x=3,y=2),dug=0.,
                         walk_outcome='arrived',source_recovered=True)
        receiver = dict(recovery,actor=2,pos=dict(x=4,y=3),target=dict(x=4,y=3),
                        dug=0.,source_recovered=False)
        stay = dict(actor=2,target=dict(x=4,y=3),remembered_value=0.,actual_value=0.,
                    arrived=True,raid_amount=0.,wasted=False,inspection=dict(x=4,y=3),
                    inspected_stock=0.,target_occupant=None,source_recovered=False)
        recovery_frame = frame(39,actions=[recovery,receiver],choices=[stay])
        recovery_frame['roles'].append(dict(id=2,pos=dict(x=4,y=3)))
        group = dict(id='fixture',members=['sham-ambiguous-seen-off-route-cost0-m0 seed 20002'],
                     frames=[frame(0),recovery_frame,frame(40,actions=[departure])])
        result = diagnostics_p4.summarize(group)
        self.assertEqual(result['owner_source_recovery_tick'],39)
        self.assertEqual(result['owner_post_recovery_departure_tick'],40)
        self.assertEqual(result['recovery_actions'][0]['source_recovered'],False)
        self.assertTrue(result['physical_recovery_frames'][0]['owner_occupies_source_at_frame_end'])
        self.assertEqual(result['physical_recovery_frames'][0]['receiver_choices'][0]['target'],dict(x=4,y=3))
        self.assertTrue(result['physical_recovery_frames'][0]['owner_action_precedes_receiver_action'])

    def test_recovery_and_effort_are_separate_actual_action_sums(self):
        action = dict(actor=1,phase='ordinary',action='ordinary',pos=dict(x=3,y=3),
                      target=None,harvest=4.,dug=12.,buried=0.,effort=3.,
                      metabolic_demand=1.,metabolic_consumed=1.,cancellation=None,
                      walk_outcome=None,target_occupant=None,source_recovered=True)
        group = dict(id='fixture',members=['sham-ambiguous-seen-off-route-cost3-m0 seed 20001'],
                     frames=[frame(0),frame(35,actions=[action])])
        result = diagnostics_p4.summarize(group)
        self.assertIsInstance(result, dict)
        self.assertEqual(result['owner_totals']['dug'], 12.)
        self.assertEqual(result['owner_totals']['effort'], 3.)
        self.assertEqual(result['owner_source_recovery_tick'], 35)


if __name__ == '__main__':
    unittest.main()
