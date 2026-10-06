use super::super::{canonical_bits, Catalog, Error, WeightedPolicy, CATALOG_VERSION};
use crate::deduction::strategic_reporting::Policy;
use serde_json::{json, Value};

fn entry(bits: u32, weight: u8) -> WeightedPolicy {
    WeightedPolicy {
        policy: Policy { bits },
        weight,
    }
}

fn distinct_entries(count: u32, weight: u8) -> Vec<WeightedPolicy> {
    // Calibration is always false; these four low live rows are all reachable.
    (0..count).map(|live| entry(live << 2, weight)).collect()
}

#[test]
fn equivalent_unreachable_rows_cannot_gain_prior_mass() {
    let a = Policy::new(81942).unwrap();
    let b = Policy::new(88214).unwrap();
    assert_eq!(canonical_bits(&a).unwrap(), canonical_bits(&b).unwrap());
    assert!(Catalog::new(vec![
        WeightedPolicy {
            policy: a,
            weight: 1
        },
        WeightedPolicy {
            policy: b,
            weight: 1
        },
    ])
    .is_err());
}

#[test]
fn canonicalization_checks_every_live_bit_for_all_calibration_tables() {
    let reachable = [
        [0, 1, 2, 3, 8, 9, 10, 11],
        [4, 5, 6, 7, 8, 9, 10, 11],
        [0, 1, 2, 3, 12, 13, 14, 15],
        [4, 5, 6, 7, 12, 13, 14, 15],
    ];
    for (calibration, rows) in reachable.iter().enumerate() {
        for row in 0..16 {
            let calibration = calibration as u32;
            let live_bit = 1 << (2 + row);
            let policy = Policy::new(calibration | live_bit).unwrap();
            let expected = if rows.contains(&row) {
                calibration | live_bit
            } else {
                calibration
            };
            assert_eq!(
                canonical_bits(&policy).unwrap(),
                expected,
                "calibration {calibration}, row {row}"
            );
        }
    }
}

#[test]
fn canonical_policies_preserve_all_reachable_reports() {
    for bits in [0, 1, 2, 3, 81942, 88214, 87381, 174762, 262143] {
        let policy = Policy::new(bits).unwrap();
        let canonical = Policy::new(canonical_bits(&policy).unwrap()).unwrap();
        for signal in [false, true] {
            assert_eq!(canonical.calibration(signal), policy.calibration(signal));
            for truth in [false, true] {
                for live_signal in [false, true] {
                    let report = policy.calibration(signal);
                    assert_eq!(
                        canonical.live(signal, report, truth, live_signal),
                        policy.live(signal, report, truth, live_signal)
                    );
                }
            }
        }
    }
}

#[test]
fn invalid_public_policy_encoding_is_rejected_before_canonicalization() {
    for bits in [1 << 18, u32::MAX] {
        assert!(canonical_bits(&Policy { bits }).is_err());
        assert!(Catalog::new(vec![entry(bits, 1)]).is_err());
    }
}

#[test]
fn catalog_requires_one_through_eight_distinct_entries() {
    assert!(Catalog::new(vec![]).is_err());
    let catalog = Catalog::new(distinct_entries(8, 32)).unwrap();
    assert_eq!(catalog.entries().len(), 8);
    assert_eq!(catalog.total_weight(), 256);
    assert!(Catalog::new(distinct_entries(9, 1)).is_err());
}

#[test]
fn weights_have_inclusive_boundary_and_positive_total() {
    assert_eq!(Catalog::new(vec![entry(0, 32)]).unwrap().total_weight(), 32);
    assert!(Catalog::new(vec![entry(0, 33)]).is_err());
    assert!(Catalog::new(vec![entry(0, 0)]).is_err());
    assert!(Catalog::new(distinct_entries(8, 0)).is_err());
}

#[test]
fn zero_weight_entries_are_preserved_but_cannot_duplicate_behavior() {
    let catalog = Catalog::new(vec![entry(4, 0), entry(0, 1)]).unwrap();
    assert_eq!(catalog.entries(), &[entry(0, 1), entry(4, 0)]);
    assert_eq!(catalog.total_weight(), 1);
    assert!(Catalog::new(vec![entry(0, 1), entry(64, 0)]).is_err());
}

#[test]
fn named_priors_have_five_distinct_controls_and_disclosed_weights() {
    let controls = [
        Policy::copy(),
        Policy::invert(),
        Policy::positive(),
        Policy::negative(),
        Policy::calibration_copy_live_invert(),
    ];
    // Hand-derived sorted canonical encodings: negative, invert, live-invert,
    // copy, positive. The first four controls each retain weight one.
    for (catalog, weights, total) in [
        (Catalog::uniform(), [1, 1, 1, 1, 1], 5),
        (Catalog::optimization_informed(), [1, 1, 16, 1, 1], 20),
    ] {
        assert_eq!(catalog.total_weight(), total);
        assert_eq!(
            catalog
                .entries()
                .iter()
                .map(|e| e.policy.bits)
                .collect::<Vec<_>>(),
            [0, 5441, 81942, 163882, 246723]
        );
        assert_eq!(
            catalog
                .entries()
                .iter()
                .map(|e| e.weight)
                .collect::<Vec<_>>(),
            weights
        );
        for (i, control) in controls.iter().enumerate() {
            let bits = canonical_bits(control).unwrap();
            assert_eq!(
                catalog
                    .entries()
                    .iter()
                    .filter(|e| e.policy.bits == bits)
                    .count(),
                1
            );
            if i != 4 {
                assert_eq!(
                    catalog
                        .entries()
                        .iter()
                        .find(|e| e.policy.bits == bits)
                        .unwrap()
                        .weight,
                    1
                );
            }
        }
    }
}

#[test]
fn catalog_order_and_unreachable_bits_do_not_change_serialized_prior() {
    let entries = vec![entry(81942, 0), entry(0, 1), entry(174762, 32)];
    let forward = Catalog::new(entries.clone()).unwrap();
    let reverse = Catalog::new(entries.into_iter().rev().collect()).unwrap();
    assert_eq!(forward, reverse);
    assert_eq!(
        serde_json::to_string(&forward).unwrap(),
        serde_json::to_string(&reverse).unwrap()
    );
    assert_eq!(
        serde_json::to_value(Catalog::new(vec![entry(88214, 1)]).unwrap()).unwrap(),
        json!({"version": 1, "entries": [{"policy": {"bits": 81942}, "weight": 1}]})
    );
}

#[test]
fn singleton_and_zero_weight_catalogs_round_trip_with_explicit_version() {
    for catalog in [
        Catalog::new(vec![entry(0, 1)]).unwrap(),
        Catalog::new(vec![entry(4, 0), entry(0, 32)]).unwrap(),
        Catalog::uniform(),
        Catalog::optimization_informed(),
    ] {
        let wire = serde_json::to_value(&catalog).unwrap();
        assert_eq!(wire["version"], CATALOG_VERSION);
        assert_eq!(serde_json::from_value::<Catalog>(wire).unwrap(), catalog);
    }
}

#[test]
fn unknown_fields_are_rejected_at_each_catalog_nesting_level() {
    let base = json!({"version": 1, "entries": [{"policy": {"bits": 0}, "weight": 1}]});
    for path in [vec![], vec!["entries", "0"], vec!["entries", "0", "policy"]] {
        let mut wire = base.clone();
        let mut target = &mut wire;
        for part in path {
            target = if part == "0" {
                &mut target[0]
            } else {
                &mut target[part]
            };
        }
        target["extra"] = json!(true);
        assert!(serde_json::from_value::<Catalog>(wire).is_err());
    }
}

#[test]
fn wire_decoding_enforces_catalog_validation() {
    let cases = [
        json!({"version": 1, "entries": []}),
        json!({"version": 1, "entries": distinct_entries(9, 1)}),
        json!({"version": 1, "entries": [entry(0, 33)]}),
        json!({"version": 1, "entries": [entry(0, 0)]}),
        json!({"version": 1, "entries": [entry(1 << 18, 1)]}),
        json!({"version": 1, "entries": [entry(81942, 1), entry(88214, 0)]}),
    ];
    for wire in cases {
        assert!(
            serde_json::from_value::<Catalog>(wire.clone()).is_err(),
            "accepted {wire}"
        );
    }
}

#[test]
fn unsupported_versions_and_malformed_wire_payloads_fail() {
    for wire in [
        json!({"version": 0, "entries": [entry(0, 1)]}),
        json!({"version": 2, "entries": [entry(0, 1)]}),
        json!({"entries": [entry(0, 1)]}),
        json!({"version": 1}),
        json!({"version": 1, "entries": [{"policy": {"bits": -1}, "weight": 1}]}),
        json!({"version": 1, "entries": [{"policy": {"bits": 0}, "weight": 256}]}),
        json!({"version": 1, "entries": [{"policy": {"bits": 0}, "weight": 1.5}]}),
        json!({"version": 1, "entries": [{"policy": {}, "weight": 1}]}),
        json!({"version": 1, "entries": [{"policy": {"bits": 0}}]}),
        Value::Null,
    ] {
        assert!(
            serde_json::from_value::<Catalog>(wire.clone()).is_err(),
            "accepted {wire}"
        );
    }
}

#[test]
fn errors_explain_invalid_catalog_and_preserve_existing_cause() {
    let error = Catalog::new(vec![]).unwrap_err();
    assert!(matches!(error, Error::InvalidCatalog(_)));
    assert!(error.to_string().contains("catalog"));
    let error = canonical_bits(&Policy { bits: 1 << 18 }).unwrap_err();
    assert!(matches!(error, Error::Existing(_)));
    assert!(std::error::Error::source(&error).is_some());
}
