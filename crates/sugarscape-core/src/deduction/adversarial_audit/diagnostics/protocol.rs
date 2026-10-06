use super::*;
use crate::deduction::strategic_reporting::{enumerate, histories};

pub(super) fn controllers() -> [ControllerKind; 4] {
    [
        ControllerKind::StrategyUniform,
        ControllerKind::StrategyOptimizationInformed,
        ControllerKind::FixedOnly,
        ControllerKind::Passive,
    ]
}
fn environments() -> Vec<EnvironmentRow> {
    [4, 3]
        .into_iter()
        .map(|q| EnvironmentRow {
            id: format!("q{q}_5"),
            rules: Config::standard(
                Probability {
                    numerator: q,
                    denominator: 5,
                },
                Probability {
                    numerator: 3,
                    denominator: 4,
                },
                UtilityTable::opposed(),
            ),
        })
        .collect()
}
fn fixed_model(env: &EnvironmentRow) -> Result<FixedModelRow, Error> {
    let model = FixedModel::new(&env.rules)?;
    let distribution = enumerate(&env.rules)?;
    let mut rows: Vec<_> = (0..8)
        .map(|index| FixedPosteriorRow {
            index,
            view: FixedView {
                rules: env.rules.clone(),
                calibration_truth: index & 4 != 0,
                calibration_report: index & 2 != 0,
                live_report: index & 1 != 0,
            },
            total_mass: 0,
            true_mass: 0,
            decision: FrozenDecision {
                posterior_true: None,
                action: crate::deduction::strategic_reporting::DecisionAction::Abstain,
            },
        })
        .collect();
    for h in histories(&distribution, &Policy::positive())? {
        let index = (usize::from(h.observation.calibration_truth) << 2)
            | (usize::from(h.observation.calibration_reports[1]) << 1)
            | usize::from(h.observation.live_reports[1]);
        let row = &mut rows[index];
        row.total_mass = row
            .total_mass
            .checked_add(h.total_mass)
            .ok_or(Error::ArithmeticOverflow)?;
        row.true_mass = row
            .true_mass
            .checked_add(h.true_mass)
            .ok_or(Error::ArithmeticOverflow)?;
    }
    for row in &mut rows {
        row.decision = model.decide(&row.view)?;
    }
    Ok(FixedModelRow {
        environment_id: env.id.clone(),
        rules: env.rules.clone(),
        denominator: model.denominator(),
        rows,
    })
}
fn evaluation_score(rules: &Config, e: &AuditEvaluation) -> Score {
    Score {
        rules: rules.clone(),
        denominator: e.denominator,
        payoff_numerator: e.payoff_numerator,
        utility_numerator: e.utility_numerator,
    }
}
fn ratio(n: u64, d: u64) -> Ratio {
    Ratio {
        numerator: n,
        denominator: d,
    }
}

fn metadata() -> Result<Metadata, Error> {
    let f = references::diagnostic_fixture()?;
    Ok(Metadata {
        threat_model:"Reporter knows the public controller rule and selects one policy before private state; allowed inputs are its own calibration signal/report, genuine C verification and live signal. Ignore the other reporter's calibration report.".into(),
        trusted_components:"Known q/rho, independent noise, persistent fixed copy/invert channel, genuine C verification, and opposed utility remain trusted.".into(),
        private_view_boundary:"Controllers receive only public rules and permitted observations; no actual policy, private world, seed/archive or current-game evaluator score enters controller decisions.".into(),
        prior_assumptions:"Supplied named-control weights are frozen assumptions, not learned rationality. Both nominal populations are evaluated separately for every controller.".into(),
        named_control_order:["copy","invert","always_positive","always_negative","copy_calibration_invert_live"].map(str::to_owned),
        uniform_prior:[1,1,1,1,1], optimization_informed_prior:[1,1,1,1,16],
        tie_rule:"Posterior T=1/2 abstains; zero deltas choose false; optimum ties choose the smallest unsigned canonical encoding.".into(),
        decomposition:"Four all-false-live calibration bases plus 32 reachable single-row flips; recombine independent live rows and verify all 1,024 canonical fitnesses.".into(),
        query_count_per_controller:36, raw_encodings:262144, canonical_behaviors:1024, aliases_per_behavior:256,
        prior_published_benchmark_predictions:[ratio(9,50),ratio(1,20)],
        guarantee_scope:"Expected payoff minimax within this finite game: fixed-only attains F for every allowed policy; a constant report removes strategic evidence and limits any receiver to F. No realized-game, empirical, richer-family, corruption, collusion or equilibrium guarantee.".into(),
        clone_provenance:"Forty earlier champion seed/method/raw encodings are behavioral clones of copy_calibration_invert_live, not independent challenges. Their former target was equally weighted frozen Bayesian and Credulous listeners at q=4/5, rho=3/4 with opposed utility.".into(),
        rng:false, learning:false, search:false,
        oracle_source_sha256:f.source_sha256,
        reference_sha256:f.reference_sha256,
        dto_extraction_source_sha256:"f2ab44611004444dc935ea78489b49a41853fa7d3c4cc47010e804ca74b7b656".into(),
        dto_reference_sha256:"0b57a1b11f7ca9d6ea263cd7ef2421276179b2bc993d8cf9a6b0c80fdefa5c81".into(),
        diagnostic_fixture_sha256:"68aaa522536276229f014414ebe979755431580e459f680115cb77ba05ed17c4".into(),
        attack_fixture_sha256:"49d2bb6bee7899a73b6bf5974438196ed9008f5f2d2be820c7d9f855e668db1c".into(),
        evaluation_fixture_sha256:"38d839d7e9976154df9136f36bc4bf1b906cbe89f8af7b8e3393620719128e76".into(),
        policy_coordinate_source_sha256:"351ca357fb2be0aa6c16bcfdbc7a2f0a98c54224758c3f57a4a7cf80a3377c43".into(),
        prior_definition_source_sha256:"042f40cda9b928a84935da4223cc997ff5e76d00b73d8400bec2bae72347815b".into(),
        provenance_source_report_sha256:f.provenance_source_report_sha256,
        champion_source_commit:f.champion_source_commit,
        champion_source_report_sha256:f.champion_source_report_sha256,
    })
}

pub(super) fn rebuild() -> Result<DiagnosticReport, Error> {
    let environments = environments();
    // All eight controller snapshots are frozen before any attack basis exists.
    let mut frozen = Vec::new();
    let mut controller_snapshots = Vec::new();
    let mut fixed_only_models = Vec::new();
    for env in &environments {
        fixed_only_models.push(fixed_model(env)?);
        for controller in controllers() {
            let actions = FrozenActions::freeze(&env.rules, controller)?;
            controller_snapshots.push(SnapshotRow {
                environment_id: env.id.clone(),
                snapshot: actions.snapshot(),
            });
            frozen.push(actions);
        }
    }
    let policy_provenance = references::diagnostic_fixture()?.policy_provenance;
    let mut r = DiagnosticReport {
        version: REPORT_VERSION.into(),
        passed: false,
        metadata: metadata()?,
        environments,
        controller_snapshots,
        fixed_only_models,
        policy_provenance,
        fitness_tables: Vec::new(),
        targeted_audits: Vec::new(),
        control_evaluations: Vec::new(),
        nominal_evaluations: Vec::new(),
        cross_target_evaluations: Vec::new(),
        bound_checks: Vec::new(),
        checks: Vec::new(),
    };
    for (ei, env) in r.environments.iter().enumerate() {
        let actions = &frozen[ei * 4..ei * 4 + 4];
        let fixed = score(&actions[2], &Policy::negative())?;
        let mut witnesses = Vec::new();
        for a in actions {
            let basis = AttackBasis::build(a)?;
            let rows = fitness_table(&basis)?;
            let witness = exact_best_response(&basis)?;
            let canonical_tie_count = u32::try_from(
                rows.iter()
                    .filter(|row| row.utility_numerator == witness.score.utility_numerator)
                    .count(),
            )
            .map_err(|_| Error::ArithmeticOverflow)?;
            let raw_tie_count = canonical_tie_count
                .checked_mul(256)
                .ok_or(Error::ArithmeticOverflow)?;
            let evaluation = evaluate_fixed(a, &witness.policy)?;
            let guarantee_shortfall = guarantee_shortfall(&fixed, &witness.score)?;
            r.fitness_tables.push(FitnessTable {
                environment_id: env.id.clone(),
                controller: *a.controller(),
                denominator: basis.denominator(),
                basis: basis.rows().clone(),
                rows,
            });
            r.targeted_audits.push(TargetedAudit {
                environment_id: env.id.clone(),
                controller: *a.controller(),
                witness: witness.clone(),
                canonical_tie_count,
                raw_tie_count,
                evaluation,
                guarantee_shortfall,
            });
            witnesses.push(witness);
            // Six distinct controls; preserve clones only in the provenance list.
            for p in r.policy_provenance.iter().take(6) {
                r.control_evaluations.push(ControlEvaluation {
                    environment_id: env.id.clone(),
                    controller: *a.controller(),
                    policy_id: p.id.clone(),
                    raw_bits: p.raw_bits,
                    canonical_bits: p.canonical_bits,
                    evaluation: evaluate_fixed(a, &Policy::new(p.raw_bits)?)?,
                });
            }
            for (population, catalog) in [
                ("strategy_uniform", Catalog::uniform()),
                (
                    "strategy_optimization_informed",
                    Catalog::optimization_informed(),
                ),
            ] {
                let evaluation = evaluate_mixture(a, &catalog)?;
                let nominal_minus_fixed_only =
                    nominal_difference(&evaluation_score(&env.rules, &evaluation), &fixed)?;
                r.nominal_evaluations.push(NominalEvaluation {
                    environment_id: env.id.clone(),
                    controller: *a.controller(),
                    population: population.into(),
                    catalog,
                    evaluation,
                    nominal_minus_fixed_only,
                });
            }
        }
        for (target, witness) in controllers().into_iter().zip(&witnesses) {
            for a in actions {
                r.cross_target_evaluations.push(CrossTargetEvaluation {
                    environment_id: env.id.clone(),
                    controller: *a.controller(),
                    target_controller: target,
                    witness_bits: witness.policy.bits,
                    evaluation: evaluate_fixed(a, &witness.policy)?,
                });
            }
        }
        let negative = evaluate_fixed(&actions[2], &Policy::negative())?;
        let positive = evaluate_fixed(&actions[2], &Policy::positive())?;
        let expected_f = &r.metadata.prior_published_benchmark_predictions[ei];
        let mut fixed_policy_count = 0;
        let mut passive_policy_count = 0;
        for p in canonical_policies() {
            if score(&actions[2], &p)? == fixed {
                fixed_policy_count += 1;
            }
            if score(&actions[3], &p)?.payoff_numerator == 0 {
                passive_policy_count += 1;
            }
        }
        let mut passed = references::fraction_equal(
            i128::from(fixed.payoff_numerator),
            fixed.denominator,
            i128::from(expected_f.numerator),
            expected_f.denominator,
        )? && fixed_policy_count == 1024
            && passive_policy_count == 1024;
        for a in actions {
            for p in [Policy::negative(), Policy::positive()] {
                let e = evaluate_fixed(a, &p)?;
                passed &= e.optimal_numerator == fixed.payoff_numerator
                    && e.payoff_numerator <= fixed.payoff_numerator;
            }
        }
        r.bound_checks.push(BoundCheck {
            environment_id: env.id.clone(),
            fixed_guarantee: expected_f.clone(),
            constant_negative_informed_reference: ratio(
                u64::try_from(negative.optimal_numerator).map_err(|_| Error::ArithmeticOverflow)?,
                negative.denominator,
            ),
            constant_positive_informed_reference: ratio(
                u64::try_from(positive.optimal_numerator).map_err(|_| Error::ArithmeticOverflow)?,
                positive.denominator,
            ),
            upper_bound: expected_f.clone(),
            fixed_policy_count,
            passive_policy_count,
            passed,
        });
    }
    r.checks = references::checks(&r)?;
    r.passed = r.checks.iter().all(|c| c.passed) && r.bound_checks.iter().all(|b| b.passed);
    Ok(r)
}
