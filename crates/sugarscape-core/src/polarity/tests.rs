use super::{config::*, decision::FrontKey, territory, world::*};
use crate::model::Model;
use std::collections::BTreeMap;
#[cfg(test)]
mod clock_contracts {
    use super::*;
    use crate::model::Model;
    #[test]
    fn no_predator_world_stays_peaceful_and_finishes_at_horizon() {
        let c = PolarityConfig {
            width: 3,
            height: 3,
            predator_share: 0.0,
            horizon: 4,
            initial_mean: 50.0,
            initial_sd: 0.0,
            harvest_mean: 2.0,
            harvest_sd: 0.0,
            ..Default::default()
        };
        let mut w = PolarityWorld::new(c, 7).unwrap();
        w.run(20);
        assert_eq!(w.period, 4);
        assert_eq!(w.population(), 9);
        assert_eq!(w.cells.iter().map(|c| c.stock).sum::<f64>(), 522.0);
        assert_eq!(w.totals.attacks, 0);
        let before = w.fingerprint();
        w.run(1);
        assert_eq!(before, w.fingerprint());
    }
    #[test]
    fn grouped_periods_have_identical_economic_state_and_random_stream() {
        let c = PolarityConfig {
            width: 4,
            height: 4,
            horizon: 12,
            stop_at_hegemony: false,
            ..Default::default()
        };
        let mut one = PolarityWorld::new(c.clone(), 17).unwrap();
        one.run(12);
        let mut c = c;
        c.periods_per_tick = 5;
        let mut grouped = PolarityWorld::new(c, 17).unwrap();
        grouped.run(3);
        assert_eq!(one.economic_fingerprint(), grouped.economic_fingerprint());
        assert_eq!(grouped.stats.latest().unwrap().last_tick_periods, 2);
    }
    #[test]
    fn provincial_negative_harvest_tax_is_signed_and_zero_stock_exempt() {
        let c = PolarityConfig {
            width: 2,
            height: 2,
            predator_share: 0.0,
            initial_mean: 50.0,
            initial_sd: 0.0,
            harvest_mean: -10.0,
            harvest_sd: 0.0,
            tax_rate: 0.4,
            horizon: 1,
            ..PolarityConfig::for_variant(Variant::TwoLevel)
        };
        let mut w = PolarityWorld::new(c, 9).unwrap();
        w.cells[1].capital = 0;
        w.cells[1].stock = 20.0;
        w.rebuild();
        w.run(1);
        assert_eq!((w.cells[0].stock, w.cells[1].stock), (36.0, 14.0));
        assert_eq!(w.totals.harvest, -40.0);
        assert_eq!(w.totals.taxes, -4.0);
    }
    #[test]
    fn rejecting_nonpositive_stock_retains_invalid_outcome() {
        let c = PolarityConfig {
            width: 2,
            height: 2,
            predator_share: 0.0,
            initial_mean: 1.0,
            initial_sd: 0.0,
            harvest_mean: -2.0,
            harvest_sd: 0.0,
            resource_policy: ResourcePolicy::RejectNonpositive,
            ..Default::default()
        };
        let mut w = PolarityWorld::new(c, 4).unwrap();
        w.run(1);
        let outcome = w.outcome().unwrap();
        assert!(!outcome.valid);
        assert!(outcome.terminal_category.is_none());
        assert!(!w.holds_when_finished());
    }
}
#[cfg(test)]
mod source_motifs {
    use super::*;
    fn fixture() -> PolarityWorld {
        PolarityWorld::new(
            PolarityConfig {
                width: 3,
                height: 2,
                predator_share: 0.0,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                stop_at_hegemony: false,
                ..Default::default()
            },
            1,
        )
        .unwrap()
    }
    #[test]
    fn equal_allocation_counts_sovereign_neighbors_not_border_edges() {
        let mut w = fixture();
        w.cells[1].capital = 0;
        w.cells[0].stock = 120.0;
        w.cells[1].stock = 0.0;
        w.rebuild();
        let a = w.allocate(&w.cells.iter().map(|c| c.stock).collect::<Vec<_>>());
        assert_eq!(a[&FrontKey::foreign(0, 2)][0], 40.0);
    }
    #[test]
    fn signed_damage_ledger_separates_creation_from_positive_destruction() {
        let mut w = fixture();
        w.cells[0].stock = -60.0;
        w.cells[1].stock = -60.0;
        let k = FrontKey::foreign(0, 1);
        w.fronts.get_mut(&k).unwrap().previous = [true, true];
        w.fronts.get_mut(&FrontKey::foreign(2, 5)).unwrap().previous = [true, true];
        w.run(1);
        assert!(w.totals.signed_creation > 0.0);
        assert!(w.totals.destruction > 0.0);
    }
    #[test]
    fn coalition_obligations_do_not_recursively_trigger_another_coalition() {
        let mut w = fixture();
        w.config.alliances = true;
        w.coalitions = BTreeMap::from([(0, vec![1, 3]), (1, vec![2, 4])]);
        let k = FrontKey::foreign(0, 1);
        w.fronts.get_mut(&k).unwrap().initiated = [true, false];
        w.fronts.get_mut(&k).unwrap().actions = [true, false];
        w.obligations();
        assert!(w.fronts[&FrontKey::foreign(0, 3)].actions[1]);
        assert!(!w.fronts[&FrontKey::foreign(1, 2)].actions[1]);
    }
    #[test]
    fn intra_coalition_initiator_is_expelled_before_obligations() {
        let mut w = fixture();
        w.config.alliances = true;
        w.coalitions = BTreeMap::from([(4, vec![0, 1, 3])]);
        w.fronts
            .get_mut(&FrontKey::foreign(0, 1))
            .unwrap()
            .initiated = [true, false];
        w.obligations();
        assert_eq!(w.coalitions[&4], vec![1, 3]);
    }
    #[test]
    fn source_horizon_censors_open_episode_instead_of_inventing_end() {
        let mut w = fixture();
        w.config.horizon = 1;
        w.config.victory = 100.0;
        w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap().previous = [true, true];
        w.run(1);
        let ep = &w.outcome().unwrap().episodes[0];
        assert_eq!(ep.duration, 1);
        assert!(ep.censored);
        assert!(ep.end_cause.is_none());
    }
    #[test]
    fn inspect_and_render_leave_state_and_rng_unchanged() {
        let w = fixture();
        let before = w.fingerprint();
        let _ = w.inspect_json(3, 3).unwrap();
        for mode in ["territory", "resources", "strategy", "coalitions"] {
            w.render(mode, "", &mut vec![]).unwrap();
        }
        assert_eq!(before, w.fingerprint());
    }
    #[test]
    fn logs_have_monotonic_ids_and_report_dropped_events() {
        let mut w = fixture();
        w.config.event_log = true;
        w.config.event_log_limit = 2;
        for _ in 0..3 {
            w.log("test", vec![0], None);
        }
        assert_eq!(
            w.events.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(w.events_dropped, 1);
    }
    #[test]
    fn tax_distance_discount_is_cardinal_distance() {
        let w = fixture();
        assert_eq!(w.distance(0, 5), 3);
        assert_eq!(discount(0.7, 3), 0.3429999999999999);
    }
}

#[cfg(test)]
mod added_rules {
    use super::*;
    #[test]
    fn pra_coalition_deterrence_distinguishes_front_commitments_from_stocks() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 3,
                height: 2,
                predator_share: 0.0,
                initial_mean: 60.0,
                initial_sd: 0.0,
                alliances: true,
                allocation: Allocation::Pra,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.coalitions = BTreeMap::from([(0, vec![1, 4]), (1, vec![0, 2])]);
        assert_eq!(w.deterrence_commitments(0, 1), [60.0, 20.0]);
        w.config.pra_alliance_support = PraAllianceSupport::Stocks;
        assert_eq!(w.deterrence_commitments(0, 1), [90.0, 80.0]);
    }
    #[test]
    fn reject_policy_stops_at_damage_before_harvest_can_repair_stock() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 1.0,
                initial_sd: 0.0,
                harvest_mean: 100.0,
                harvest_sd: 0.0,
                damage_rate: 1.0,
                resource_policy: ResourcePolicy::RejectNonpositive,
                victory: 100.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.cells[1].stock = 10.0;
        w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap().previous = [true, true];
        w.run(1);
        assert!(!w.outcome().unwrap().valid);
        assert_eq!(w.totals.harvest, 0.0);
    }
}
#[cfg(test)]
mod invalid_arithmetic {
    use super::*;
    #[test]
    fn aggregate_overflow_is_invalid_even_when_every_stock_is_finite() {
        let w = PolarityWorld::new(
            PolarityConfig {
                initial_mean: 1e308,
                initial_sd: 0.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        assert!(!w.outcome().unwrap().valid);
        assert!(w
            .outcome()
            .unwrap()
            .invalid_reason
            .as_ref()
            .unwrap()
            .contains("aggregate"));
    }
    #[test]
    fn flooring_does_not_conceal_nonfinite_aggregate() {
        let w = PolarityWorld::new(
            PolarityConfig {
                initial_mean: 1e308,
                initial_sd: 0.0,
                resource_policy: ResourcePolicy::FloorZero,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        assert!(!w.outcome().unwrap().valid);
    }
    #[test]
    fn overextension_continues_a_one_state_period() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                horizon: 2,
                ..PolarityConfig::for_variant(Variant::Overextension)
            },
            1,
        )
        .unwrap();
        for c in &mut w.cells {
            c.capital = 0;
        }
        w.rebuild();
        w.run(1);
        assert_eq!(w.period, 1);
        assert!(w.outcome().is_none());
    }
}
#[cfg(test)]
mod further_contracts {
    use super::*;
    #[test]
    fn all_presets_validate_and_roundtrip_resolved_fields() {
        for p in crate::polarity::presets() {
            let crate::model::ModelConfig::Polarity(c) = p.config else {
                panic!("wrong preset kind")
            };
            c.validate().unwrap();
            let encoded = serde_json::to_string(&c).unwrap();
            let copy: PolarityConfig = serde_json::from_str(&encoded).unwrap();
            assert_eq!(copy, c);
        }
    }
    #[test]
    fn all_schema_paths_exist_in_exported_configuration() {
        let c = serde_json::to_value(PolarityConfig::default()).unwrap();
        for p in crate::polarity::schema() {
            assert!(c.get(p.path).is_some(), "{}", p.path);
            assert_eq!(p.apply, crate::schema::Apply::Reset);
        }
    }
    #[test]
    fn next_period_obligations_are_deferred() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 3,
                height: 2,
                alliances: true,
                obligation_timing: ObligationTiming::NextPeriod,
                predator_share: 0.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.coalitions = BTreeMap::from([(0, vec![1, 3])]);
        w.fronts
            .get_mut(&FrontKey::foreign(0, 1))
            .unwrap()
            .initiated = [true, false];
        w.obligations();
        assert!(!w.fronts[&FrontKey::foreign(0, 3)].actions[1]);
        assert!(w.pending.contains(&(3, 0)));
        w.prepare();
        assert!(w.fronts[&FrontKey::foreign(0, 3)].actions[1]);
    }
    #[test]
    fn prime_threat_requires_strictly_negative_enough_trust() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                alliances: true,
                trust_initial: 0.0,
                threat_threshold: 0.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.form_coalitions();
        assert!(w.threats.is_empty());
        w.trust.insert((1, 0), -1.0);
        w.trust.insert((2, 0), -1.0);
        w.form_coalitions();
        assert_eq!(w.coalitions[&0], vec![1, 2]);
    }
    #[test]
    fn sequential_claims_share_period_locks() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..Default::default()
        };
        let mut cells: Vec<territory::Cell> = (0..6)
            .map(|id| territory::Cell {
                id,
                capital: if id < 3 { 0 } else { id },
                stock: if id == 0 {
                    90.0
                } else if id < 3 {
                    0.0
                } else {
                    50.0
                },
                predator: false,
            })
            .collect();
        let mut locks = std::collections::BTreeSet::new();
        let a = territory::Claim {
            winner: 3,
            loser: 0,
            target: 2,
            domestic: false,
            source: None,
        };
        let b = territory::Claim {
            target: 1,
            ..a.clone()
        };
        territory::apply_locked(&c, &mut cells, &[a], &mut locks);
        assert_eq!(
            territory::apply_locked(&c, &mut cells, &[b], &mut locks).locked,
            1
        );
    }
    #[test]
    fn floor_zero_records_signed_clipping_without_hiding_harvest() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 1.0,
                initial_sd: 0.0,
                harvest_mean: -2.0,
                harvest_sd: 0.0,
                resource_policy: ResourcePolicy::FloorZero,
                horizon: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.run(1);
        assert_eq!(w.totals.harvest, -8.0);
        assert_eq!(w.totals.clipping, 4.0);
        assert_eq!(w.cells.iter().map(|c| c.stock).sum::<f64>(), 0.0);
    }
    #[test]
    fn harvest_tax_is_exempt_at_zero_province_stock() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: -10.0,
                harvest_sd: 0.0,
                horizon: 1,
                ..PolarityConfig::for_variant(Variant::TwoLevel)
            },
            1,
        )
        .unwrap();
        w.cells[1].capital = 0;
        w.cells[1].stock = 0.0;
        w.rebuild();
        w.run(1);
        assert_eq!((w.cells[0].stock, w.cells[1].stock), (40.0, -10.0));
        assert_eq!(w.totals.taxes, 0.0);
    }
}
#[cfg(test)]
mod ordered_source_motifs {
    use super::*;
    fn fight(timing: VictoryTiming) -> PolarityWorld {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                victory_timing: timing,
                damage_rate: 1.0,
                stop_at_hegemony: false,
                horizon: 1,
                ..Default::default()
            },
            7,
        )
        .unwrap();
        w.cells[0].stock = 90.0;
        w.cells[1].stock = 20.0;
        let f = w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap();
        f.previous = [true, true];
        f.path = Some([0, 1]);
        w
    }
    #[test]
    fn simultaneous_damage_uses_frozen_opponent_commitments() {
        let mut w = fight(VictoryTiming::AfterDamage);
        w.run(1);
        assert_eq!((w.cells[0].stock, w.cells[1].stock), (80.0, -25.0));
        assert_eq!(w.totals.destruction, 55.0);
    }
    #[test]
    fn before_damage_victory_uses_frozen_ratio() {
        let mut w = fight(VictoryTiming::BeforeDamage);
        w.run(1);
        assert_eq!(w.cells[1].capital, 0);
        assert_eq!(w.cells[0].stock, 55.0);
    }
    #[test]
    fn after_damage_victory_can_differ_from_before_damage() {
        let mut w = fight(VictoryTiming::AfterDamage);
        w.run(1);
        assert_eq!(w.cells[1].capital, 1);
        assert_eq!(w.totals.conquests, 0);
    }
    #[test]
    fn after_harvest_recomputes_stocks_before_victory() {
        let mut w = fight(VictoryTiming::AfterHarvest);
        w.config.damage_rate = 0.0;
        w.config.harvest_mean = 100.0;
        w.run(1);
        assert_eq!(w.cells[1].capital, 1);
        assert_eq!((w.cells[0].stock, w.cells[1].stock), (190.0, 120.0));
    }
    #[test]
    fn defender_victory_captures_attacking_border_cell() {
        let mut w = fight(VictoryTiming::BeforeDamage);
        w.config.damage_rate = 0.0;
        w.cells[0].stock = 20.0;
        w.cells[1].stock = 90.0;
        w.run(1);
        assert_eq!(w.cells[0].capital, 1);
        assert_eq!(w.cells[1].stock, 110.0);
    }
    #[test]
    fn cc_cannot_generate_claim_despite_strict_resource_superiority() {
        let mut w = fight(VictoryTiming::BeforeDamage);
        w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap().previous = [false, false];
        w.run(1);
        assert_eq!(w.totals.conquests, 0);
        assert_eq!(w.cells[1].capital, 1);
    }
    #[test]
    fn domestic_revolt_strict_equality_does_not_initiate() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 60.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                horizon: 1,
                ..PolarityConfig::for_variant(Variant::TwoLevel)
            },
            1,
        )
        .unwrap();
        w.cells[1].capital = 0;
        w.cells[1].stock = 120.0;
        w.rebuild();
        w.run(1);
        assert_eq!(w.totals.revolts, 0);
        assert_eq!(w.cells[1].capital, 0);
    }
    #[test]
    fn winning_province_gains_independence_without_stock_transfer() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 60.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                damage_rate: 0.0,
                horizon: 1,
                ..PolarityConfig::for_variant(Variant::TwoLevel)
            },
            1,
        )
        .unwrap();
        w.cells[1].capital = 0;
        w.cells[1].stock = 121.0;
        w.rebuild();
        w.run(1);
        assert_eq!(w.totals.revolts, 1);
        assert_eq!(
            (w.cells[1].capital, w.cells[1].stock, w.cells[0].stock),
            (1, 121.0, 60.0)
        );
    }
    #[test]
    fn two_stage_path_samples_from_border_cell_support() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 3,
                height: 2,
                predator_share: 0.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.cells[3].capital = 0;
        w.cells[4].capital = 1;
        w.rebuild();
        let key = FrontKey::foreign(0, 1);
        for _ in 0..32 {
            let p = w.sample_path(key, 0);
            assert!([0, 3].contains(&p[0]));
            assert!([1, 4].contains(&p[1]));
            assert!(territory::adjacent(&w.config, p[0]).contains(&p[1]));
        }
    }
    #[test]
    fn territory_path_distance_uses_only_owned_cells() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 3,
                height: 2,
                tax_distance: TaxDistance::TerritorialPath,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        for i in [1, 2, 5, 3] {
            w.cells[i].capital = 0;
        }
        assert_eq!(w.distance(0, 5), 3);
    }
}
#[cfg(test)]
mod sequential_control {
    use super::*;
    #[test]
    fn later_actor_cannot_initiate_against_already_resolved_dyads() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                update: Update::Sequential,
                initial_mean: 10.0,
                initial_sd: 0.0,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.cells[3].predator = true;
        w.cells[3].stock = 100.0;
        w.resolved_fronts.insert(FrontKey::foreign(1, 3));
        w.resolved_fronts.insert(FrontKey::foreign(2, 3));
        w.prepare();
        let mut ledger = crate::polarity::Ledger::default();
        w.decide_actor(3, &mut ledger);
        assert_eq!(ledger.attacks, 0);
        assert!(!w.fronts.values().any(|f| f.initiated[0] || f.initiated[1]));
    }
    #[test]
    fn sequential_runs_a_full_harvest_period_once_per_cell() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                update: Update::Sequential,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: 2.0,
                harvest_sd: 0.0,
                horizon: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.run(1);
        assert_eq!(w.totals.harvest, 8.0);
        assert_eq!(w.cells.iter().map(|c| c.stock).sum::<f64>(), 208.0);
    }
}
#[cfg(test)]
mod variant_validation {
    use super::*;
    #[test]
    fn two_level_rejects_decay_but_overextension_accepts_it() {
        let mut c = PolarityConfig::for_variant(Variant::TwoLevel);
        c.tax_discount = 0.7;
        assert!(c
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "tax_discount"));
        c.variant = Variant::Overextension;
        c.stop_at_hegemony = false;
        assert!(c.validate().is_ok());
    }
}
#[cfg(test)]
mod completed_clock {
    use super::*;
    #[test]
    fn invalid_partial_period_is_attempted_but_not_completed() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 1.0,
                initial_sd: 0.0,
                harvest_mean: -2.0,
                harvest_sd: 0.0,
                resource_policy: ResourcePolicy::RejectNonpositive,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.run(1);
        let o = w.outcome().unwrap();
        assert_eq!((o.periods, o.attempted_period), (0, 1));
        let s = w.stats.latest().unwrap();
        assert_eq!(
            (s.periods, s.attempted_period, s.last_tick_periods),
            (0, 1, 0)
        );
    }
}
#[cfg(test)]
mod final_numeric_guards {
    use super::*;
    #[test]
    fn finite_nonzero_denominator_ratio_overflow_retains_invalid_session() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 1.0,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                damage_rate: 0.0,
                horizon: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.cells[0].stock = 1e300;
        w.cells[1].stock = 1e-300;
        w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap().previous = [true, true];
        w.run(1);
        assert!(!w.outcome().unwrap().valid);
        assert!(w
            .outcome()
            .unwrap()
            .invalid_reason
            .as_ref()
            .unwrap()
            .contains("ratio"));
    }
    #[test]
    fn cumulative_ledger_overflow_is_invalid_with_finite_stocks() {
        let mut w = PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                initial_mean: 1e307,
                initial_sd: 0.0,
                harvest_mean: 0.0,
                harvest_sd: 0.0,
                damage_rate: 1.0,
                victory: 100.0,
                horizon: 1,
                ..Default::default()
            },
            1,
        )
        .unwrap();
        w.totals.destruction = f64::MAX;
        w.fronts.get_mut(&FrontKey::foreign(0, 1)).unwrap().previous = [true, true];
        w.run(1);
        assert!(w.cells.iter().all(|c| c.stock.is_finite()));
        assert!(!w.outcome().unwrap().valid);
        assert!(w
            .outcome()
            .unwrap()
            .invalid_reason
            .as_ref()
            .unwrap()
            .contains("ledger"));
    }
}

#[cfg(test)]
mod review_round1 {
    use super::*;
    use rand::Rng;

    fn world(update: Update, seed: u64) -> PolarityWorld {
        PolarityWorld::new(
            PolarityConfig {
                width: 2,
                height: 2,
                predator_share: 0.0,
                update,
                initial_mean: 50.0,
                initial_sd: 0.0,
                harvest_mean: 7.0,
                harvest_sd: 0.0,
                horizon: 1,
                stop_at_hegemony: false,
                ..Default::default()
            },
            seed,
        )
        .unwrap()
    }

    fn next_rng_value(world: &PolarityWorld) -> u64 {
        let mut rng = world.rng.clone();
        rng.gen()
    }

    fn seed_with_actor_order(before: usize, after: &[usize]) -> u64 {
        (1..10_000)
            .find(|&seed| {
                let w = world(Update::Sequential, seed);
                let mut capitals = territory::capitals(&w.cells);
                let mut rng = w.rng.clone();
                super::super::world::shuffle(&mut capitals, &mut rng);
                let before_position = capitals.iter().position(|&x| x == before).unwrap();
                after.iter().all(|&actor| {
                    before_position < capitals.iter().position(|&x| x == actor).unwrap()
                })
            })
            .unwrap()
    }

    #[test]
    fn sequential_invalid_decision_aborts_before_harvest_and_extra_random_draws() {
        let seed = seed_with_actor_order(0, &[1, 2]);
        let mut w = world(Update::Sequential, seed);
        w.cells[0].predator = true;
        w.cells[0].stock = f64::MAX;
        w.cells[1].stock = 1e-300;

        let mut expected_rng = w.rng.clone();
        let mut capitals = territory::capitals(&w.cells);
        super::super::world::shuffle(&mut capitals, &mut expected_rng);
        let expected_next = expected_rng.gen::<u64>();

        w.run(1);

        let outcome = w.outcome().unwrap();
        assert!(!outcome.valid);
        assert_eq!((outcome.periods, outcome.attempted_period), (0, 1));
        assert_eq!(w.totals.harvest, 0.0);
        assert_eq!(w.cells[0].stock, f64::MAX);
        assert_eq!(w.cells[1].stock, 1e-300);
        assert_eq!(next_rng_value(&w), expected_next);
    }

    #[test]
    fn sequential_invalid_before_damage_victory_aborts_before_damage_and_harvest() {
        let mut w = world(Update::Sequential, 32);
        w.cells[0].stock = f64::MAX;
        w.cells[1].stock = 1e-300;
        let key = FrontKey::foreign(0, 1);
        let front = w.fronts.get_mut(&key).unwrap();
        front.previous = [false, true];
        front.path = Some([0, 1]);

        let mut expected_rng = w.rng.clone();
        let mut capitals = territory::capitals(&w.cells);
        super::super::world::shuffle(&mut capitals, &mut expected_rng);
        let expected_next = expected_rng.gen::<u64>();

        w.run(1);

        let outcome = w.outcome().unwrap();
        assert!(!outcome.valid);
        assert_eq!((outcome.periods, outcome.attempted_period), (0, 1));
        assert_eq!(w.cells[0].stock, f64::MAX);
        assert_eq!(w.cells[1].stock, 1e-300);
        assert_eq!(w.totals.harvest, 0.0);
        assert_eq!(next_rng_value(&w), expected_next);
    }

    #[test]
    fn snapshot_invalid_before_damage_victory_aborts_before_damage_and_harvest() {
        let mut w = world(Update::Snapshot, 33);
        w.cells[0].stock = f64::MAX;
        w.cells[1].stock = 1e-300;
        let key = FrontKey::foreign(0, 1);
        let front = w.fronts.get_mut(&key).unwrap();
        front.previous = [false, true];
        front.path = Some([0, 1]);
        let expected_next = next_rng_value(&w);

        w.run(1);

        let outcome = w.outcome().unwrap();
        assert!(!outcome.valid);
        assert_eq!((outcome.periods, outcome.attempted_period), (0, 1));
        assert_eq!(w.cells[0].stock, f64::MAX);
        assert_eq!(w.cells[1].stock, 1e-300);
        assert_eq!(w.totals.harvest, 0.0);
        assert_eq!(next_rng_value(&w), expected_next);
    }

    #[test]
    fn late_same_period_obligation_does_not_change_an_already_resolved_front() {
        let config = PolarityConfig {
            width: 2,
            height: 2,
            predator_share: 0.0,
            update: Update::Sequential,
            alliances: true,
            obligation_timing: ObligationTiming::SamePeriod,
            initial_mean: 50.0,
            initial_sd: 0.0,
            harvest_mean: 0.0,
            harvest_sd: 0.0,
            horizon: 1,
            stop_at_hegemony: false,
            ..Default::default()
        };
        let (mut w, seed) = (1..10_000)
            .find_map(|seed| {
                let w = PolarityWorld::new(config.clone(), seed).unwrap();
                let mut capitals = territory::capitals(&w.cells);
                let mut rng = w.rng.clone();
                super::super::world::shuffle(&mut capitals, &mut rng);
                (capitals.iter().position(|&x| x == 2) < capitals.iter().position(|&x| x == 0)
                    && capitals.iter().position(|&x| x == 0)
                        < capitals.iter().position(|&x| x == 1))
                .then_some((w, seed))
            })
            .unwrap();
        w.cells[0].predator = true;
        w.cells[0].stock = 150.0;
        w.cells[1].stock = 20.0;
        w.cells[2].stock = 50.0;
        w.config.victory = 100.0;
        w.coalitions = BTreeMap::from([(1, vec![1, 2])]);
        let resolved = FrontKey::foreign(0, 2);
        assert!(!w.fronts[&resolved].actions[1]);

        w.run(1);

        let late_attack = FrontKey::foreign(0, 1);
        let resolved_front = &w.fronts[&resolved];
        assert_eq!(w.totals.attacks, 1, "seed {seed}");
        assert_eq!(
            w.totals.dd_encounters,
            1,
            "seed {seed}, fronts {:?}",
            w.fronts
                .iter()
                .map(|(k, f)| (k, f.actions, f.previous, f.initiated))
                .collect::<Vec<_>>()
        );
        assert_eq!(w.fronts[&late_attack].previous, [true, true]);
        assert_eq!(resolved_front.actions, [false, false]);
        assert_eq!(resolved_front.previous, [false, false]);
        assert!(resolved_front.episode.is_none());
        assert!(w.pending.is_empty());
    }
}

#[cfg(test)]
mod panic_export {
    use super::*;
    #[test]
    fn panic_inside_second_batched_period_keeps_first_completed_and_partial_ledger() {
        let c = PolarityConfig {
            predator_share: 0.0,
            initial_sd: 0.0,
            harvest_mean: 2.0,
            harvest_sd: 0.0,
            horizon: 3,
            periods_per_tick: 3,
            ..Default::default()
        };
        let mut w = PolarityWorld::new(c, 7).unwrap();
        w.panic_on_period = Some(2);
        w.run(1);
        let o = w.outcome().unwrap();
        assert!(!o.valid);
        assert_eq!((o.periods, o.attempted_period), (1, 2));
        assert_eq!(o.events.harvest, 400.0);
        assert_eq!(o.finish_reason, "panic");
        assert!(o.invalid_reason.as_ref().unwrap().contains("injected"));
        assert_eq!(o.seed, 7);
        let frozen = o.clone();
        w.invalidate_after_panic("later host panic".into());
        assert_eq!(w.outcome().unwrap(), &frozen);
    }
}
