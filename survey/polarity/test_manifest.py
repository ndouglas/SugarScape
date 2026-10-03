import unittest
from manifest import build_manifest


class ManifestTests(unittest.TestCase):
    def source(self):
        return {'figures':[{'id':'book5.8','panels':[{'samples':[
            {'tax_rate':0.,'extraction_status':'recoverable'},
            {'tax_rate':.4,'extraction_status':'recoverable'},
            {'tax_rate':1.,'extraction_status':'recoverable'}]}]}]}

    def test_original_and_precision_counts_are_frozen(self):
        m=build_manifest(self.source())
        self.assertEqual(sum(a['sessions'] for a in m['arms'] if a['family']=='original'),640)
        self.assertEqual(sum(a['sessions'] for a in m['arms'] if a['family']=='precision'),6400)

    def test_overextension_continues_through_hegemony(self):
        a=next(a for a in build_manifest(self.source())['arms'] if a['family']=='overextension')
        self.assertEqual(a['config']['horizon'],4000)
        self.assertFalse(a['config']['stop_at_hegemony'])

    def test_allocation_interaction_holds_chapter5_damage_fixed(self):
        arms=[a for a in build_manifest(self.source())['arms'] if a['family']=='allocation']
        self.assertEqual(len(arms),64)
        self.assertTrue(all(a['config']['source_profile']=='chapter5' for a in arms))
        self.assertEqual({a['config']['allocation'] for a in arms},{'equal','pra'})

    def test_seed_blocks_do_not_overlap_and_configuration_is_reproducible(self):
        m=build_manifest(self.source())
        ranges=sorted((a['first_seed'],a['first_seed']+a['sessions']-1) for a in m['arms'])
        self.assertTrue(all(b[0]>a[1] for a,b in zip(ranges,ranges[1:])))
        self.assertEqual(m,build_manifest(self.source()))

    def test_stock_support_control_does_not_mix_with_main_interaction(self):
        m=build_manifest(self.source())
        control=[a for a in m['arms'] if a['family']=='support_control']
        self.assertEqual(len(control),16)
        self.assertTrue(all(a['sessions']==20 and a['config']['pra_alliance_support']=='stocks'
                            and a['config']['allocation']=='pra' and a['config']['alliances']
                            for a in control))

    def test_no_tax_knots_is_an_explicit_error(self):
        with self.assertRaises(ValueError):
            build_manifest({'figures':[]})

if __name__=='__main__':
    unittest.main()
