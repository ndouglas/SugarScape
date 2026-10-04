use super::*;
use crate::model::Model;
fn small() -> DemocraticPeaceConfig {
    DemocraticPeaceConfig {
        width: 3,
        height: 2,
        horizon_periods: 7,
        ..Default::default()
    }
}
fn fixture(owners: &[usize], regimes: &[Regime]) -> DemocraticPeaceWorld {
    let mut w = DemocraticPeaceWorld::new(small(), 9).unwrap();
    w.engine.states.clear();
    for (i, &capital) in owners.iter().enumerate() {
        let id = StateId {
            capital_cell: capital,
            sovereignty_generation: 0,
        };
        w.engine.cells[i].owner = id;
        w.engine.cells[i].latent_regime = regimes[i];
        w.engine.states.entry(id).or_insert(State {
            id,
            regime: regimes[capital],
            resources: 100.,
            members: vec![],
        });
    }
    w.engine.rebuild().unwrap();
    w
}
#[test]
fn rounded_quota_assigns_exact_counts() {
    let w = DemocraticPeaceWorld::new(
        DemocraticPeaceConfig {
            assignment: Assignment::RoundedQuota,
            initial_democratic_share: 0.5,
            initial_resourced_share: 0.5,
            ..small()
        },
        3,
    )
    .unwrap();
    assert_eq!(w.setup.initial_democratic_cells, 3);
    assert_eq!(w.setup.initial_resourced_cells, 3);
}
#[test]
fn initial_all_democratic_first_passage_is_zero() {
    let w = DemocraticPeaceWorld::new(
        DemocraticPeaceConfig {
            initial_democratic_share: 1.,
            ..small()
        },
        3,
    )
    .unwrap();
    assert_eq!(w.snapshot().metrics.first_all_democratic_period, Some(0));
}
#[test]
fn grouped_periods_have_identical_state_and_rng() {
    let mut a = DemocraticPeaceWorld::new(small(), 3).unwrap();
    let mut b = DemocraticPeaceWorld::new(
        DemocraticPeaceConfig {
            periods_per_tick: 3,
            ..small()
        },
        3,
    )
    .unwrap();
    a.run(7);
    b.run(3);
    assert_eq!(a.economic_fingerprint(), b.economic_fingerprint());
    assert_eq!(a.outcome(), b.outcome());
}
#[test]
fn failed_period_preserves_engine_and_rng() {
    let mut w = DemocraticPeaceWorld::new(
        DemocraticPeaceConfig {
            zero_ratio: ZeroRatio::RejectZeroDenominator,
            initial_resourced_share: 0.,
            initial_democratic_share: 0.,
            ..small()
        },
        3,
    )
    .unwrap();
    let before = w.economic_fingerprint();
    w.run(1);
    assert_eq!(w.economic_fingerprint(), before);
    assert_eq!(w.completed_periods(), 0);
    assert_eq!(
        w.outcome().unwrap().invalid_phase.as_deref(),
        Some("alignments")
    );
}
#[test]
fn democracies_allocate_only_against_predatory_neighbors() {
    let mut w = fixture(
        &[0, 1, 2, 3, 4, 5],
        &[
            Regime::Democratic,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
        ],
    );
    w.engine.allocate(&w.config).unwrap();
    let id = w.engine.cells[0].owner;
    let front = w
        .engine
        .fronts
        .values()
        .find(|f| f.states.contains(&id) && f.states.contains(&w.engine.cells[3].owner))
        .unwrap();
    assert_eq!(front.commitments[0], 100.);
}
#[test]
fn extraction_uses_preclaim_territory() {
    let mut w = fixture(&[0, 0, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    w.engine.extract(&w.config).unwrap();
    assert_eq!(w.engine.states[&w.engine.cells[0].owner].resources, 19.5);
}
#[test]
fn capital_collapse_releases_each_primitive_cell() {
    let mut w = fixture(&[0, 0, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let winner = w.engine.cells[3].owner;
    let loser = w.engine.cells[0].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [0, 3],
            }],
        )
        .unwrap();
    assert_eq!(w.engine.states.len(), 6);
    assert_eq!(w.engine.cells[1].owner.capital_cell, 1);
    assert_eq!(w.engine.cells[0].owner.sovereignty_generation, 1);
}
#[test]
fn capturing_bridge_releases_disconnected_cells() {
    let mut w = fixture(&[0, 0, 0, 3, 4, 5], &[Regime::Predatory; 6]);
    let winner = w.engine.cells[4].owner;
    let loser = w.engine.cells[0].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [1, 4],
            }],
        )
        .unwrap();
    assert_eq!(w.engine.cells[2].owner.capital_cell, 2);
    assert_eq!(w.engine.cells[1].owner, winner);
}
#[test]
fn isolated_democracy_has_exposure_one() {
    let w = fixture(&[0; 6], &[Regime::Democratic; 6]);
    let m = w.engine.metrics(&w.config).unwrap();
    assert_eq!(m.democratic_exposure, Some(1.));
    assert_eq!(m.clustering_ratio, Some(10.));
}
#[test]
fn legitimate_undefined_clustering_is_blank_csv_and_nan_series() {
    let w = DemocraticPeaceWorld::new(
        DemocraticPeaceConfig {
            initial_democratic_share: 0.,
            ..small()
        },
        3,
    )
    .unwrap();
    assert!(w.series("clustering_ratio").unwrap()[0].is_nan());
    assert!(!w.series_csv().contains("NaN"));
}

#[test]
fn latent_democracy_can_return_after_extinction() {
    let mut w = fixture(
        &[0, 0, 2, 3, 4, 5],
        &[
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
        ],
    );
    w.engine.first_extinction = Some(0);
    let loser = w.engine.cells[0].owner;
    let winner = w.engine.cells[3].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [0, 3],
            }],
        )
        .unwrap();
    let m = w.engine.metrics(&w.config).unwrap();
    assert!(!m.democratic_extinction);
    assert_eq!(m.first_extinction_period, Some(0));
    assert_eq!(m.democratic_states, 1);
}
#[test]
fn internal_candidate_panic_is_terminal_and_preserves_committed_state() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let front = w.engine.fronts.values_mut().next().unwrap();
    front.path = Some([usize::MAX, 1]);
    front.previous = [true; 2];
    let before = w.economic_fingerprint();
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.run(1)));
    assert!(caught.is_ok());
    assert_eq!(w.economic_fingerprint(), before);
    assert_eq!(
        w.outcome().unwrap().invalid_phase.as_deref(),
        Some("interaction")
    );
    assert!(!w.outcome().unwrap().valid);
}
#[test]
fn probability_denominator_overflow_is_invalid() {
    assert!(resources::probability(
        10.,
        1e-320,
        2.2,
        30,
        ProbabilityDirection::PrintedDecreasing,
        ZeroRatio::EqualZeroNeutral
    )
    .is_err());
}
#[test]
fn late_generation_failure_rolls_back_prior_mutations_and_rng() {
    let mut w = fixture(&[0, 0, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    w.config.victory_exponent = 100;
    let loser = w.engine.cells[0].owner;
    w.engine.states.get_mut(&loser).unwrap().resources = 1e100;
    w.engine.cells[1].next_generation = u64::MAX;
    let k = territory::key(loser, w.engine.cells[3].owner);
    let f = w.engine.fronts.get_mut(&k).unwrap();
    f.previous = [true; 2];
    f.path = Some([0, 3]);
    let before = w.economic_fingerprint();
    w.run(1);
    assert_eq!(w.economic_fingerprint(), before);
    assert_eq!(
        w.outcome().unwrap().invalid_phase.as_deref(),
        Some("structural_change")
    );
}
#[test]
fn schema_paths_roundtrip_and_all_changes_require_reset() {
    crate::schema::check_schema(
        &schema(),
        &crate::model::ModelConfig::DemocraticPeace(small()),
        || {
            crate::model::ModelWorld::new(crate::model::ModelConfig::DemocraticPeace(small()), 3)
                .unwrap()
        },
    );
}
#[test]
fn independent_victories_can_record_opposing_claims() {
    let mut w = fixture(&[0, 0, 0, 3, 3, 3], &[Regime::Predatory; 6]);
    w.config.victory_threshold = 100.;
    w.config.victory_exponent = 100;
    let f = w.engine.fronts.values_mut().next().unwrap();
    f.actions = [true; 2];
    f.commitments = [1.; 2];
    f.path = Some([0, 3]);
    let claims = w.engine.combat(&w.config).unwrap();
    assert_eq!(claims.len(), 2);
    assert_eq!(w.engine.counters.opposing_claims, 1);
    assert_eq!(w.engine.fronts.values().next().unwrap().actions, [false; 2]);
}
#[test]
fn single_draw_victory_records_only_one_claim() {
    let mut w = fixture(&[0, 0, 0, 3, 3, 3], &[Regime::Predatory; 6]);
    w.config.victory_threshold = 100.;
    w.config.victory_exponent = 100;
    w.config.opposing_victories = OpposingVictories::SingleDraw;
    let f = w.engine.fronts.values_mut().next().unwrap();
    f.actions = [true; 2];
    f.commitments = [1.; 2];
    f.path = Some([0, 3]);
    assert_eq!(w.engine.combat(&w.config).unwrap().len(), 1);
}
#[test]
fn unilateral_defection_has_no_combat_consequence() {
    let mut w = fixture(&[0, 0, 0, 3, 3, 3], &[Regime::Predatory; 6]);
    let f = w.engine.fronts.values_mut().next().unwrap();
    f.actions = [true, false];
    f.commitments = [1.; 2];
    f.path = Some([0, 3]);
    assert!(w.engine.combat(&w.config).unwrap().is_empty());
    assert_eq!(w.engine.counters.mutual_d_front_periods, 0);
}
#[test]
fn conquest_yield_is_delayed_until_next_resource_update() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    w.engine.extract(&w.config).unwrap();
    let loser = w.engine.cells[1].owner;
    let winner = w.engine.cells[4].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [1, 4],
            }],
        )
        .unwrap();
    assert_eq!(w.engine.states[&winner].resources, 10.);
    w.engine.extract(&w.config).unwrap();
    assert_eq!(w.engine.states[&winner].resources, 19.5);
}
#[test]
fn overwrite_on_conquest_changes_released_latent_tag() {
    let mut w = fixture(
        &[0, 1, 2, 3, 4, 5],
        &[
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
            Regime::Predatory,
        ],
    );
    w.config.latent_regime = LatentRegime::OverwriteOnConquest;
    let loser = w.engine.cells[1].owner;
    let winner = w.engine.cells[4].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [1, 4],
            }],
        )
        .unwrap();
    assert_eq!(w.engine.cells[1].latent_regime, Regime::Predatory);
}
#[test]
fn capture_and_fragment_absorbs_compound_capital() {
    let mut w = fixture(&[0, 0, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    w.config.capital_capture = CapitalCapture::CaptureAndFragment;
    let loser = w.engine.cells[0].owner;
    let winner = w.engine.cells[3].owner;
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [loser, winner],
                side: 1,
                path: [0, 3],
            }],
        )
        .unwrap();
    assert_eq!(w.engine.cells[0].owner, winner);
    assert_eq!(w.engine.cells[1].owner.capital_cell, 1);
}
#[test]
fn collective_security_scope_changes_unaligned_democratic_obligation() {
    let mut w = fixture(
        &[0, 1, 2, 3, 4, 5],
        &[
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
        ],
    );
    let k = territory::key(w.engine.cells[0].owner, w.engine.cells[1].owner);
    w.engine.fronts.get_mut(&k).unwrap().previous = [true; 2];
    let target = territory::key(w.engine.cells[1].owner, w.engine.cells[4].owner);
    w.config.security_scope = SecurityScope::SameAlliance;
    w.engine.obligations(&w.config, false);
    assert!(!w.engine.fronts[&target].actions[1]);
    w.config.security_scope = SecurityScope::AllDemocracies;
    w.engine.obligations(&w.config, false);
    assert!(w.engine.fronts[&target].actions[1]);
}
#[test]
fn current_plan_obligations_do_not_read_previous_buffer() {
    let mut w = fixture(
        &[0, 1, 2, 3, 4, 5],
        &[
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
        ],
    );
    let k = territory::key(w.engine.cells[0].owner, w.engine.cells[1].owner);
    w.engine.fronts.get_mut(&k).unwrap().actions = [true; 2];
    w.engine.obligations(&w.config, false);
    assert!(w.engine.pariahs.is_empty());
    w.engine.obligations(&w.config, true);
    assert_eq!(w.engine.pariahs.len(), 1);
}
#[test]
fn territory_path_distance_follows_owned_cells() {
    let w = fixture(&[0, 1, 0, 0, 0, 0], &[Regime::Predatory; 6]);
    let id = w.engine.cells[0].owner;
    assert_eq!(
        w.engine
            .distance(
                &DemocraticPeaceConfig {
                    distance_metric: DistanceMetric::TerritorialPath,
                    ..w.config.clone()
                },
                id,
                2
            )
            .unwrap(),
        4.
    );
    assert_eq!(
        w.engine
            .distance(
                &DemocraticPeaceConfig {
                    distance_metric: DistanceMetric::Manhattan,
                    ..w.config.clone()
                },
                id,
                2
            )
            .unwrap(),
        2.
    );
}
fn threats_fixture() -> DemocraticPeaceWorld {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    for f in w.engine.fronts.values_mut() {
        f.commitments = [1.; 2];
    }
    for member in [0, 2, 4] {
        let id = w.engine.cells[1].owner;
        let k = territory::key(w.engine.cells[member].owner, id);
        w.engine.fronts.get_mut(&k).unwrap().commitments[territory::side(k, id)] = 9.;
    }
    w
}
#[test]
fn shared_primary_threat_forms_one_defensive_alliance() {
    let mut w = threats_fixture();
    w.engine.align(&w.config).unwrap();
    assert_eq!(w.engine.alliances.len(), 1);
    assert_eq!(
        w.engine.alliances[0]
            .members
            .iter()
            .map(|id| id.capital_cell)
            .collect::<Vec<_>>(),
        vec![0, 2, 4]
    );
    assert_eq!(w.engine.alliances[0].pooled_resources, 3.);
}
#[test]
fn persistent_alliance_keeps_old_threat_when_new_threat_is_stronger() {
    let mut w = threats_fixture();
    w.engine.align(&w.config).unwrap();
    let k = territory::key(w.engine.cells[0].owner, w.engine.cells[3].owner);
    w.engine.fronts.get_mut(&k).unwrap().commitments[1] = 10.;
    let mut rebuilt = w.clone();
    rebuilt.engine.align(&rebuilt.config).unwrap();
    assert_eq!(
        rebuilt.engine.alliances[0]
            .members
            .iter()
            .map(|id| id.capital_cell)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );
    w.config.alliance_maintenance = AllianceMaintenance::PersistWhileThreatened;
    w.engine.align(&w.config).unwrap();
    assert_eq!(
        w.engine.alliances[0]
            .members
            .iter()
            .map(|id| id.capital_cell)
            .collect::<Vec<_>>(),
        vec![0, 2, 4]
    );
}
#[test]
fn exact_threshold_does_not_count_as_threat() {
    let mut w = threats_fixture();
    w.config.min_threat = 9.;
    w.engine.align(&w.config).unwrap();
    assert!(w.engine.alliances.is_empty());
}
#[test]
fn lowest_id_breaks_equal_threat_tie() {
    let mut w = threats_fixture();
    let k = territory::key(w.engine.cells[0].owner, w.engine.cells[3].owner);
    w.engine.fronts.get_mut(&k).unwrap().commitments[1] = 9.;
    w.engine.align(&w.config).unwrap();
    assert_eq!(
        w.engine.alliances[0]
            .members
            .iter()
            .map(|id| id.capital_cell)
            .collect::<Vec<_>>(),
        vec![0, 2, 4]
    );
}
#[test]
fn border_exposure_counts_edges_not_distinct_states() {
    let mut w = fixture(
        &[0, 1, 2, 0, 1, 5],
        &[
            Regime::Democratic,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Democratic,
            Regime::Predatory,
        ],
    );
    assert!(
        (w.engine
            .metrics(&w.config)
            .unwrap()
            .democratic_exposure
            .unwrap()
            - 2. / 3.)
            .abs()
            < 1e-12
    );
    w.config.clustering_exposure = ClusteringExposure::BorderEdges;
    assert_eq!(
        w.engine.metrics(&w.config).unwrap().democratic_exposure,
        Some(0.75)
    );
}
#[test]
fn equal_state_weights_change_exposure_average() {
    let mut w = fixture(
        &[0, 1, 2, 0, 4, 5],
        &[
            Regime::Democratic,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Democratic,
            Regime::Predatory,
            Regime::Predatory,
        ],
    );
    assert!(
        (w.engine
            .metrics(&w.config)
            .unwrap()
            .democratic_exposure
            .unwrap()
            - 4. / 9.)
            .abs()
            < 1e-12
    );
    w.config.clustering_weights = ClusteringWeights::EqualStates;
    assert!(
        (w.engine
            .metrics(&w.config)
            .unwrap()
            .democratic_exposure
            .unwrap()
            - 5. / 12.)
            .abs()
            < 1e-12
    );
}
#[test]
fn simultaneous_proposals_count_one_initiated_front() {
    let mut w = fixture(&[0, 0, 0, 3, 3, 3], &[Regime::Predatory; 6]);
    w.config.superiority_threshold = 100.;
    w.config.superiority_exponent = 100;
    for f in w.engine.fronts.values_mut() {
        f.commitments = [1.; 2];
    }
    w.engine.decide(&w.config).unwrap();
    assert_eq!(w.engine.counters.initiated_fronts, 1);
    assert_eq!(
        w.engine.fronts.values().next().unwrap().initiations,
        [true; 2]
    );
}
#[test]
fn all_front_enemy_total_changes_active_commitment() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let k01 = territory::key(w.engine.cells[0].owner, w.engine.cells[1].owner);
    let k03 = territory::key(w.engine.cells[0].owner, w.engine.cells[3].owner);
    w.engine.fronts.get_mut(&k01).unwrap().previous = [true; 2];
    w.engine.fronts.get_mut(&k01).unwrap().old_commitments[1] = 15.;
    w.engine.fronts.get_mut(&k03).unwrap().old_commitments[1] = 25.;
    w.engine.allocate(&w.config).unwrap();
    assert_eq!(w.engine.fronts[&k01].commitments[0], 75.);
    w.config.enemy_total = EnemyTotal::AllFronts;
    w.engine.allocate(&w.config).unwrap();
    assert_eq!(w.engine.fronts[&k01].commitments[0], 43.75);
}
#[test]
fn repeated_first_inactive_term_changes_other_inactive_front() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    w.config.mobile_share = 1.;
    let ids: Vec<_> = w.engine.cells.iter().map(|c| c.owner).collect();
    for (neighbor, v) in [(0, 15.), (2, 20.), (4, 30.)] {
        let k = territory::key(ids[1], ids[neighbor]);
        let f = w.engine.fronts.get_mut(&k).unwrap();
        f.old_commitments[territory::side(k, ids[neighbor])] = v;
        if neighbor == 0 {
            f.previous = [true; 2];
        }
    }
    let k = territory::key(ids[1], ids[4]);
    w.engine.allocate(&w.config).unwrap();
    assert!((w.engine.fronts[&k].commitments[0] - 66.66666666666667).abs() < 1e-12);
    w.config.inactive_commitment = InactiveCommitment::FirstInactiveOpponent;
    w.engine.allocate(&w.config).unwrap();
    assert!((w.engine.fronts[&k].commitments[0] - 85.71428571428571).abs() < 1e-12);
}
#[test]
fn stalemate_ends_mutual_battle_without_claims() {
    let mut w = fixture(&[0, 0, 0, 3, 3, 3], &[Regime::Predatory; 6]);
    w.config.probability_direction = ProbabilityDirection::ProseIncreasing;
    w.config.victory_threshold = 1e10;
    w.config.victory_exponent = 100;
    w.config.stalemate_probability = 1.;
    let f = w.engine.fronts.values_mut().next().unwrap();
    f.actions = [true; 2];
    f.commitments = [1.; 2];
    f.path = Some([0, 3]);
    assert!(w.engine.combat(&w.config).unwrap().is_empty());
    assert_eq!(w.engine.counters.completed_stalemate_battles, 1);
    assert_eq!(w.engine.fronts.values().next().unwrap().actions, [false; 2]);
}
#[test]
fn probability_zero_boundaries_have_all_literal_limits() {
    for (direction, n, d, want) in [
        (ProbabilityDirection::PrintedDecreasing, 0., 0., 0.5),
        (ProbabilityDirection::PrintedDecreasing, 1., 0., 0.),
        (ProbabilityDirection::ProseIncreasing, 0., 0., 0.5),
        (ProbabilityDirection::ProseIncreasing, 0., 1., 0.),
        (ProbabilityDirection::ProseIncreasing, 1., 0., 1.),
    ] {
        assert_eq!(
            resources::probability(n, d, 2.2, 30, direction, ZeroRatio::EqualZeroNeutral).unwrap(),
            want
        );
    }
}
#[test]
fn threshold_probability_is_half_in_both_directions() {
    for direction in [
        ProbabilityDirection::PrintedDecreasing,
        ProbabilityDirection::ProseIncreasing,
    ] {
        assert_eq!(
            resources::probability(2.2, 1., 2.2, 30, direction, ZeroRatio::EqualZeroNeutral)
                .unwrap(),
            0.5
        );
    }
}
#[test]
fn cell_locks_allow_disjoint_claims_against_same_state() {
    let mut w = fixture(&[0, 0, 2, 0, 4, 5], &[Regime::Predatory; 6]);
    let ids: Vec<_> = w.engine.cells.iter().map(|c| c.owner).collect();
    let claims = vec![
        types::Claim {
            states: territory::key(ids[0], ids[2]),
            side: 1,
            path: [1, 2],
        },
        types::Claim {
            states: territory::key(ids[0], ids[4]),
            side: 1,
            path: [3, 4],
        },
    ];
    let mut states = w.clone();
    states.config.claim_locking = ClaimLocking::AffectedStates;
    w.engine.apply_claims(&w.config, claims.clone()).unwrap();
    states.engine.apply_claims(&states.config, claims).unwrap();
    assert_eq!(w.engine.counters.successful_claims, 2);
    assert_eq!(states.engine.counters.successful_claims, 1);
    assert_eq!(states.engine.counters.locked_claims, 1);
}
#[test]
fn prospective_release_footprint_honors_prior_locked_cells() {
    let mut saw_locked = false;
    for seed in 0..16 {
        let mut w = fixture(&[0, 0, 0, 0, 4, 5], &[Regime::Predatory; 6]);
        w.engine.rng = crate::rng::seeded(seed);
        let ids: Vec<_> = w.engine.cells.iter().map(|c| c.owner).collect();
        let claims = vec![
            types::Claim {
                states: territory::key(ids[0], ids[5]),
                side: 0,
                path: [2, 5],
            },
            types::Claim {
                states: territory::key(ids[0], ids[4]),
                side: 1,
                path: [1, 4],
            },
        ];
        w.engine.apply_claims(&w.config, claims).unwrap();
        assert_eq!(w.engine.counters.successful_claims, 1);
        saw_locked |= w.engine.counters.locked_claims == 1;
    }
    assert!(saw_locked);
}

#[test]
fn structural_pruning_recomputes_surviving_alliance_pool() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 3], &[Regime::Predatory; 6]);
    let threat = w.engine.cells[1].owner;
    let members = [
        w.engine.cells[0].owner,
        w.engine.cells[2].owner,
        w.engine.cells[4].owner,
    ];
    for member in members {
        let k = territory::key(member, threat);
        let i = territory::side(k, member);
        w.engine.fronts.get_mut(&k).unwrap().commitments[i] = 1.;
    }
    w.engine.alliances.push(Alliance {
        threat_id: threat,
        creation_period: 0,
        serial: 0,
        members: members.to_vec(),
        pooled_resources: 3.,
    });
    let attacker = w.engine.cells[3].owner;
    let conquered_member = members[2];
    w.engine
        .apply_claims(
            &w.config,
            vec![types::Claim {
                states: [attacker, conquered_member],
                side: 0,
                path: [3, 4],
            }],
        )
        .unwrap();

    assert_eq!(w.engine.alliances.len(), 1);
    assert_eq!(w.engine.alliances[0].members, members[..2]);
    assert_eq!(w.engine.alliances[0].pooled_resources, 2.);
}

#[test]
fn census_rejects_same_length_wrong_state_membership() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let state = w.engine.cells[0].owner;
    w.engine.states.get_mut(&state).unwrap().members = vec![1];

    let error = w.engine.validate().unwrap_err();
    assert!(error.contains("state"));
    assert!(error.contains("member list"));
}

#[test]
fn census_corruption_invalidates_candidate_without_advancing_world() {
    let mut w = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Democratic; 6]);
    let id = w.engine.cells[0].owner;
    w.engine
        .states
        .get_mut(&id)
        .unwrap()
        .id
        .sovereignty_generation = 1;
    let before = w.engine.canonical();

    w.run(1);

    assert_eq!(w.engine.canonical(), before);
    assert_eq!(w.completed_periods(), 0);
    let outcome = w.outcome().unwrap();
    assert_eq!(outcome.invalid_phase.as_deref(), Some("census"));
    assert!(outcome.invalid_reason.as_deref().unwrap().contains("state"));
}

#[test]
fn census_rejects_missing_owner_and_mismatched_cell_index_with_context() {
    let mut missing_owner = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    missing_owner.engine.cells[0].owner = StateId {
        capital_cell: 99,
        sovereignty_generation: 0,
    };
    let error = missing_owner.engine.validate().unwrap_err();
    assert!(error.contains("cell 0"));
    assert!(error.contains("missing state"));

    let mut wrong_index = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    wrong_index.engine.cells[0].id = 5;
    let error = wrong_index.engine.validate().unwrap_err();
    assert!(error.contains("vector index 0"));
    assert!(error.contains("cell id 5"));
}

#[test]
fn census_rejects_state_key_and_generation_counter_mismatches() {
    let mut wrong_key = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let id = wrong_key.engine.cells[0].owner;
    wrong_key
        .engine
        .states
        .get_mut(&id)
        .unwrap()
        .id
        .sovereignty_generation = 1;
    let error = wrong_key.engine.validate().unwrap_err();
    assert!(error.contains("state map key"));
    assert!(error.contains("stored state id"));

    let mut generation = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let old_id = generation.engine.cells[0].owner;
    let new_id = StateId {
        capital_cell: old_id.capital_cell,
        sovereignty_generation: 1,
    };
    let mut state = generation.engine.states.remove(&old_id).unwrap();
    state.id = new_id;
    generation.engine.states.insert(new_id, state);
    generation.engine.cells[0].owner = new_id;
    let error = generation.engine.validate().unwrap_err();
    assert!(error.contains("generation exceeds"));
    assert!(error.contains("cell 0"));
}

#[test]
fn census_rejects_front_endpoint_identity_and_missing_topology() {
    let mut wrong_identity = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    wrong_identity
        .engine
        .fronts
        .values_mut()
        .next()
        .unwrap()
        .states
        .reverse();
    let error = wrong_identity.engine.validate().unwrap_err();
    assert!(error.contains("front map key"));
    assert!(error.contains("endpoint ids"));

    let mut missing = fixture(&[0, 1, 2, 3, 4, 5], &[Regime::Predatory; 6]);
    let key = *missing.engine.fronts.keys().next().unwrap();
    missing.engine.fronts.remove(&key);
    let error = missing.engine.validate().unwrap_err();
    assert!(error.contains("front topology"));
    assert!(error.contains("missing territorial front"));
}
#[test]
fn random_threat_ties_are_reproducible_and_consume_stream() {
    use rand::RngCore;
    let mut w = threats_fixture();
    let k = territory::key(w.engine.cells[0].owner, w.engine.cells[3].owner);
    w.engine.fronts.get_mut(&k).unwrap().commitments[1] = 9.;
    let mut a = w.clone();
    a.config.threat_ties = ThreatTies::RandomTie;
    let mut b = a.clone();
    w.engine.align(&w.config).unwrap();
    a.engine.align(&a.config).unwrap();
    b.engine.align(&b.config).unwrap();
    assert_eq!(a.engine.alliances, b.engine.alliances);
    assert_eq!(a.engine.rng.next_u64(), b.engine.rng.next_u64());
    assert_ne!(a.engine.rng.next_u64(), w.engine.rng.next_u64());
}
