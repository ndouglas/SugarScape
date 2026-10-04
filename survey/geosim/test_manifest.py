"""Catch missing arms, source override loss and availability-dependent RNG order."""
import json
from pathlib import Path
import unittest
from survey.geosim import manifest

TABLE=Path(__file__).resolve().parents[2]/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'

class ManifestTests(unittest.TestCase):
    def setUp(self):self.table=json.loads(TABLE.read_text())
    def test_registered_keys_cover_all_four_populations_without_seed_overlap(self):
        m=manifest.build_manifest(self.table)
        keys=manifest.expected_keys(m)
        self.assertEqual((len(m['arms']),len(keys),len(set(keys))),(37,1490,1490))
        self.assertEqual(keys[-1],('artifact.reference_2017',370360015))
    def test_row7_retains_both_thresholds_and_shock_footnote(self):
        c=manifest.build_manifest(self.table)['arms'][6]['config_overrides']
        self.assertEqual(c,{'periods_per_tick':100,'superiority_threshold':2.5,'victory_threshold':2.5,'shock_shift':10})
    def test_artifact_is_a_rust_preset_reference_without_python_defaults(self):
        a=manifest.build_manifest(self.table)['arms'][-1]
        self.assertEqual((a['preset'],a['config_overrides']),('artifact_2017',{'periods_per_tick':100}))
    def test_missing_or_duplicate_source_row_is_rejected(self):
        self.table['rows'][1]['id']='base'
        with self.assertRaises(ValueError):manifest.build_manifest(self.table)
    def test_rng_jobs_reserve_unavailable_slots_and_preserve_later_child(self):
        m=manifest.build_manifest(self.table);jobs=manifest.analysis_jobs(m)
        self.assertEqual((len(jobs),jobs[13]['id'],jobs[71]['id']),(72,'ks.original.base','parameters.artifact.reference_2017'))
        self.assertEqual(manifest.job_rng(jobs[35]).integers(0,100,5).tolist(),manifest.job_rng(jobs[35]).integers(0,100,5).tolist())
    def test_method_contract_is_fixed_before_any_source_inventory_freeze(self):
        m=manifest.build_manifest(self.table)
        self.assertEqual((m['source_draws'],m['ks_draws'],m['analysis_seed']),(100000,1000,2026100301))
        self.assertEqual(m['provenance_status'],'provisional_unfrozen')

class MethodBytesTests(unittest.TestCase):
    def test_method_hash_uses_exact_supplied_string_and_parsed_contract(self):
        import hashlib
        source=json.loads(TABLE.read_text());m=manifest.build_manifest(source,TABLE.read_bytes())
        payload=m['method_contract_json']
        self.assertEqual(hashlib.sha256(payload.encode()).hexdigest(),m['method_contract_sha256'])
        self.assertEqual(json.loads(payload),m['method_contract'])

if __name__=='__main__':unittest.main()
