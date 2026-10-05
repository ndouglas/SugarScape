//! Fixed experiment. Integrity gates never require search improvement or superiority.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const DIAGNOSTIC_VERSION: &str = "strategic-reporting-diagnostic-v1";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCheck {
    pub quantity: String,
    pub expected_numerator: i64,
    pub actual_numerator: i64,
    pub denominator: u64,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelDescription {
    pub name: String,
    pub rules: Config,
    pub listeners: Vec<FrozenListener>,
    pub optimum: BestResponse,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelEvaluation {
    pub environment: String,
    pub name: String,
    pub policy: Policy,
    pub seed: Option<u64>,
    pub method: Option<SearchMethod>,
    pub fitness_numerator: i64,
    pub denominator: u64,
    pub reporter_regret_numerator: i64,
    pub listener_evaluations: Vec<Evaluation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactValue {
    pub numerator: i64,
    pub denominator: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchSummary {
    pub method: SearchMethod,
    pub environment: String,
    pub count: usize,
    pub minimum: ExactValue,
    pub median: ExactValue,
    pub maximum: ExactValue,
    pub mean: ExactValue,
    pub zero_reporter_regret_count: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedDifference {
    pub seed: u64,
    pub environment: String,
    pub utility_difference_numerator: i64,
    pub denominator: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub game_version: u16,
    pub protocol_version: u16,
    pub search_version: String,
    pub metadata: Value,
    pub environments: Vec<PanelDescription>,
    pub checks: Vec<DiagnosticCheck>,
    pub fixed_evaluations: Vec<PanelEvaluation>,
    pub runs: Vec<SearchRun>,
    pub frozen_evaluations: Vec<PanelEvaluation>,
    pub summaries: Vec<SearchSummary>,
    pub paired_differences: Vec<PairedDifference>,
    pub passed: bool,
}
fn check(
    checks: &mut Vec<DiagnosticCheck>,
    quantity: String,
    expected: i64,
    actual: i64,
    denominator: u64,
) {
    checks.push(DiagnosticCheck {
        quantity,
        expected_numerator: expected,
        actual_numerator: actual,
        denominator,
        passed: expected == actual,
    });
}
fn flag(checks: &mut Vec<DiagnosticCheck>, quantity: String, actual: bool) {
    check(checks, quantity, 1, i64::from(actual), 1);
}
fn rules(q: u16) -> Config {
    Config::standard(
        Probability {
            numerator: q,
            denominator: 5,
        },
        Probability {
            numerator: 3,
            denominator: 4,
        },
        UtilityTable::opposed(),
    )
}
fn frozen(algorithm: Listener) -> FrozenListener {
    FrozenListener {
        algorithm,
        assumed_copy_prior: Probability {
            numerator: 3,
            denominator: 4,
        },
    }
}
fn panels() -> Result<Vec<(String, TrainingPanel)>, Error> {
    let mut result = Vec::new();
    for (name, q, algorithms) in [
        ("training", 4, vec![Listener::Bayesian, Listener::Credulous]),
        (
            "training_pair_q3_5",
            3,
            vec![Listener::Bayesian, Listener::Credulous],
        ),
        ("skeptical_q4_5", 4, vec![Listener::Skeptical]),
        ("skeptical_q3_5", 3, vec![Listener::Skeptical]),
        (
            "evolved_q4_5",
            4,
            vec![Listener::Evolved(Genome {
                b: -3,
                u: 0,
                d: 2,
                k: 0,
            })],
        ),
        (
            "evolved_q3_5",
            3,
            vec![Listener::Evolved(Genome {
                b: -3,
                u: 0,
                d: 2,
                k: 0,
            })],
        ),
        ("passive", 4, vec![Listener::Passive]),
    ] {
        result.push((
            name.into(),
            TrainingPanel::new(&rules(q), algorithms.into_iter().map(frozen).collect())?,
        ));
    }
    Ok(result)
}
fn controls(optimum: &BestResponse) -> Vec<(String, Policy)> {
    vec![
        ("copy".into(), Policy::copy()),
        ("invert".into(), Policy::invert()),
        ("always_positive".into(), Policy::positive()),
        ("always_negative".into(), Policy::negative()),
        (
            "copy_calibration_invert_live".into(),
            Policy::calibration_copy_live_invert(),
        ),
        ("canonical_optimum".into(), optimum.policy.clone()),
    ]
}
/// Independent-reference checks without performing any GA or random search.
pub fn numerical_integrity() -> Result<Vec<DiagnosticCheck>, Error> {
    let fixtures: Value = serde_json::from_str(REFERENCES)
        .map_err(|_| Error::InvalidDiagnostic("invalid embedded independent reference"))?;
    let mut checks = Vec::new();
    for (name, panel) in panels()? {
        let distribution = enumerate(panel.config())?;
        let optimum = exact_best_response(&panel)?;
        if name == "passive" {
            check(
                &mut checks,
                "passive/optimum".into(),
                0,
                optimum.fitness_numerator,
                panel.denominator(),
            );
            for (control, policy) in controls(&optimum) {
                check(
                    &mut checks,
                    format!("passive/{control}"),
                    0,
                    panel.fitness(&policy)?,
                    panel.denominator(),
                );
            }
            continue;
        }
        let fixture = &fixtures[&name];
        flag(
            &mut checks,
            format!("{name}/denominators"),
            distribution.denominator() == fixture["world_denominator"].as_u64().unwrap()
                && panel.denominator() == fixture["panel_denominator"].as_u64().unwrap(),
        );
        check(
            &mut checks,
            format!("{name}/canonical_optimum"),
            fixture["canonical_bits"].as_i64().unwrap(),
            i64::from(optimum.policy.bits),
            1,
        );
        check(
            &mut checks,
            format!("{name}/optimal_utility"),
            fixture["fitness"].as_i64().unwrap(),
            optimum.fitness_numerator,
            panel.denominator(),
        );
        for (slot, listener) in panel.listeners().iter().enumerate() {
            for index in 0..32 {
                flag(
                    &mut checks,
                    format!("{name}/listener/{slot}/history/{index}/assumed_action"),
                    (listener.decide(&super::enumeration::observation(panel.config(), index))?
                        == DecisionAction::Intervene)
                        == fixture["actions"][slot][index].as_bool().unwrap(),
                );
            }
        }
        for control in fixture["controls"].as_array().unwrap() {
            let label = control["name"].as_str().unwrap();
            let policy = Policy::new(control["bits"].as_u64().unwrap() as u32)?;
            check(
                &mut checks,
                format!("{name}/{label}/utility"),
                control["fitness"].as_i64().unwrap(),
                panel.fitness(&policy)?,
                panel.denominator(),
            );
            let masses = histories(&distribution, &policy)?;
            flag(
                &mut checks,
                format!("{name}/{label}/normalization"),
                masses.iter().map(|h| h.total_mass).sum::<u64>() == distribution.denominator(),
            );
            for (index, h) in masses.iter().enumerate() {
                let reference = &control["masses"][index];
                flag(
                    &mut checks,
                    format!("{name}/{label}/history/{index}/actual_mass"),
                    h.total_mass == reference[0].as_u64().unwrap()
                        && h.true_mass == reference[1].as_u64().unwrap(),
                );
            }
            for (slot, listener) in panel.listeners().iter().enumerate() {
                let evaluation = evaluate(&distribution, &policy, listener)?;
                let expected = &control["metrics"][slot];
                for (metric, actual, column) in [
                    ("payoff", evaluation.payoff_numerator, 0),
                    ("utility", evaluation.utility_numerator, 1),
                    ("receiver_regret", evaluation.regret_numerator, 2),
                    ("intervention", evaluation.intervention_mass as i64, 3),
                    (
                        "false_intervention",
                        evaluation.false_intervention_mass as i64,
                        4,
                    ),
                    (
                        "missed_beneficial_intervention",
                        evaluation.missed_beneficial_intervention_mass as i64,
                        5,
                    ),
                ] {
                    check(
                        &mut checks,
                        format!("{name}/{label}/listener/{slot}/{metric}"),
                        expected[column].as_i64().unwrap(),
                        actual,
                        evaluation.denominator,
                    );
                }
                check(
                    &mut checks,
                    format!("{name}/{label}/policy_aware_payoff"),
                    control["reference_payoff"].as_i64().unwrap(),
                    evaluation.optimal_numerator,
                    evaluation.denominator,
                );
                check(
                    &mut checks,
                    format!("{name}/{label}/reference_regret"),
                    0,
                    evaluation.reference_regret_numerator,
                    evaluation.denominator,
                );
                for (metric, actual, column) in [
                    (
                        "calibration_truth_agreement",
                        evaluation.calibration_truth_agreement_mass,
                        0,
                    ),
                    (
                        "live_truth_agreement",
                        evaluation.live_truth_agreement_mass,
                        1,
                    ),
                    (
                        "calibration_signal_opposition",
                        evaluation.calibration_signal_opposition_mass,
                        2,
                    ),
                    (
                        "live_signal_opposition",
                        evaluation.live_signal_opposition_mass,
                        3,
                    ),
                ] {
                    check(
                        &mut checks,
                        format!("{name}/{label}/{metric}"),
                        control["agreements"][column].as_i64().unwrap(),
                        actual as i64,
                        evaluation.denominator,
                    );
                }
                for (index, h) in evaluation.histories.iter().enumerate() {
                    let posterior_matches = match &h.reference_posterior {
                        None => h.total_mass == 0,
                        Some(p) => {
                            h.total_mass > 0
                                && p.numerator * h.total_mass == h.true_mass * p.denominator
                        }
                    };
                    let expected_action = if h.total_mass == 0 {
                        None
                    } else {
                        Some(if h.true_mass > h.total_mass - h.true_mass {
                            DecisionAction::Intervene
                        } else {
                            DecisionAction::Abstain
                        })
                    };
                    flag(
                        &mut checks,
                        format!("{name}/{label}/listener/{slot}/history/{index}/actual_reference"),
                        posterior_matches
                            && h.reference_action == expected_action
                            && h.reference_regret_numerator == 0,
                    );
                }
            }
        }
    }
    Ok(checks)
}
fn panel_evaluation(
    name: &str,
    panel: &TrainingPanel,
    optimum: &BestResponse,
    label: String,
    policy: Policy,
    seed: Option<u64>,
    method: Option<SearchMethod>,
) -> Result<PanelEvaluation, Error> {
    let distribution = enumerate(panel.config())?;
    let listener_evaluations = panel
        .listeners()
        .iter()
        .map(|listener| evaluate(&distribution, &policy, listener))
        .collect::<Result<Vec<_>, _>>()?;
    let fitness_numerator = panel.fitness(&policy)?;
    Ok(PanelEvaluation {
        environment: name.into(),
        name: label,
        policy,
        seed,
        method,
        fitness_numerator,
        denominator: panel.denominator(),
        reporter_regret_numerator: optimum.fitness_numerator - fitness_numerator,
        listener_evaluations,
    })
}
fn method_name(method: SearchMethod) -> &'static str {
    match method {
        SearchMethod::Genetic => "genetic",
        SearchMethod::Random => "random",
    }
}
fn frozen_settings() -> SearchSettings {
    SearchSettings {
        version: 1,
        policy_bits: 18,
        population: 64,
        generations: 50,
        elites: 2,
        tournament: 3,
        crossover_numerator: 1,
        crossover_denominator: 2,
        mutation_numerator: 1,
        mutation_denominator: 18,
        evaluations: 3164,
        ranking: "higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order".into(),
    }
}
fn run_is_consistent(run: &SearchRun, training: &TrainingPanel) -> bool {
    run.version == SEARCH_VERSION
        && run.seed_derivation == SEARCH_SEED_DERIVATION
        && run.derived_seed == search_seed(run.seed, run.method)
        && run.settings == frozen_settings()
        && run.evaluations == 3164
        && run.curve.len() == 3164
        && run.denominator == training.denominator()
        && training.fitness(&run.champion).ok() == Some(run.fitness_numerator)
        && run.curve.last().is_some_and(|point| {
            point.champion == run.champion && point.best_fitness_numerator == run.fitness_numerator
        })
        && run.curve.iter().enumerate().all(|(index, point)| {
            point.evaluations as usize == index + 1
                && training.fitness(&point.champion).ok() == Some(point.best_fitness_numerator)
        })
        && run.curve.windows(2).all(|points| {
            points[1].best_fitness_numerator > points[0].best_fitness_numerator
                || (points[1].best_fitness_numerator == points[0].best_fitness_numerator
                    && points[1].champion.bits <= points[0].champion.bits)
        })
}
/// Recompute current report fields and exact references. Stored checks/flags are not authoritative.
pub fn report_integrity(report: &DiagnosticReport) -> bool {
    validate_report(report).unwrap_or(false)
}
fn validate_report(report: &DiagnosticReport) -> Result<bool, Error> {
    if report.version != DIAGNOSTIC_VERSION
        || report.game_version != GAME_VERSION
        || report.protocol_version != GAME_PROTOCOL_VERSION
        || report.search_version != SEARCH_VERSION
        || report.runs.len() != 40
        || report.environments.len() != 7
        || report.fixed_evaluations.len() != 42
        || report.frozen_evaluations.len() != 240
        || report.summaries.len() != 12
        || report.paired_differences.len() != 120
    {
        return Ok(false);
    }
    let panels = panels()?;
    let mut identities = BTreeSet::new();
    for (index, run) in report.runs.iter().enumerate() {
        let method = if index % 2 == 0 {
            SearchMethod::Genetic
        } else {
            SearchMethod::Random
        };
        if run.seed != (index / 2) as u64
            || run.method != method
            || !identities.insert((run.seed, method_name(run.method)))
            || !run_is_consistent(run, &panels[0].1)
        {
            return Ok(false);
        }
    }
    let mut expected = base_report(numerical_integrity()?, &panels)?;
    expected.runs = report.runs.clone();
    complete_report(&mut expected, &panels)?;
    Ok(expected.checks.iter().all(|check| check.passed)
        && report.metadata == expected.metadata
        && report.environments == expected.environments
        && report.checks == expected.checks
        && report.fixed_evaluations == expected.fixed_evaluations
        && report.frozen_evaluations == expected.frozen_evaluations
        && report.summaries == expected.summaries
        && report.paired_differences == expected.paired_differences)
}
pub fn diagnose() -> Result<DiagnosticReport, Error> {
    let checks = numerical_integrity()?;
    let panels = panels()?;
    let mut report = base_report(checks, &panels)?;
    if report.checks.iter().any(|check| !check.passed) {
        return Ok(report);
    }
    // All 40 owned champions are selected on the immutable training panel before any holdout evaluation.
    for seed in 0..20 {
        for method in [SearchMethod::Genetic, SearchMethod::Random] {
            report.runs.push(search(&panels[0].1, seed, method)?);
        }
    }
    complete_report(&mut report, &panels)?;
    report.passed = report_integrity(&report);
    Ok(report)
}
fn base_report(
    checks: Vec<DiagnosticCheck>,
    panels: &[(String, TrainingPanel)],
) -> Result<DiagnosticReport, Error> {
    let mut report = DiagnosticReport {
        version: DIAGNOSTIC_VERSION.into(),
        game_version: GAME_VERSION,
        protocol_version: GAME_PROTOCOL_VERSION,
        search_version: SEARCH_VERSION.into(),
        metadata: json!({
            "search_seeds":{"start_inclusive":0,"end_exclusive":20},
            "seed_derivation":{"version":SEARCH_SEED_DERIVATION,"multiplier":6364136223846793005u64,"offset_multiplier":1442695040888963407u64,"genetic_identity":3,"random_identity":4},
            "search_settings":{"policy_bits":18,"population":64,"generations":50,"elites":2,"tournament":3,"crossover":"1/2 independently per bit","mutation":"1/18 independently per bit","evaluations":3164,"count_repeated_candidates":true,"rank":"higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order"},
            "training":"equally weighted frozen Bayesian assumed-channel and Credulous algorithms; q=4/5; actual fixed copy prior=3/4; opposed reporter utility",
            "assumptions":"Legacy listeners continue to model both reporters as persistent copy/invert channels. Their assumed prior is separate from actual generative rules; no assumed-model belief is labeled an actual-policy posterior.",
            "policy_information":"calibration signal; own calibration report; verified C; live signal. Other calibration report is publicly visible but ignored by this restricted 18-bit family; unreachable rows are retained in search encodings.",
            "reference_information":"policy-aware reference knows candidate policy and actual generative rules, never private realized signals or truths",
            "freeze":"all 40 champions are selected on training alone and owned before any champion holdout evaluation",
            "inference":"actual-policy posterior is exact true/total history mass; zero-mass histories have no posterior or action; exact decision ties abstain",
            "gate":"correctness and integrity only; no GA superiority or search success requirement",
            "archives":"privileged unauthenticated replay records",
            "reference_provenance":serde_json::from_str::<Value>(REFERENCE_PROVENANCE).expect("independent reference provenance literal"),
            "scope":"finite one-calibration/one-live game; expected utility and regret, not guaranteed realized losses, psychological simplicity or equilibrium; no new interactive host mode"
        }),
        environments: Vec::new(),
        checks,
        fixed_evaluations: Vec::new(),
        runs: Vec::new(),
        frozen_evaluations: Vec::new(),
        summaries: Vec::new(),
        paired_differences: Vec::new(),
        passed: false,
    };
    for (name, panel) in panels {
        let optimum = exact_best_response(panel)?;
        for (label, policy) in controls(&optimum) {
            report.fixed_evaluations.push(panel_evaluation(
                name, panel, &optimum, label, policy, None, None,
            )?);
        }
        report.environments.push(PanelDescription {
            name: name.clone(),
            rules: panel.config().clone(),
            listeners: panel.listeners().to_vec(),
            optimum,
        });
    }
    Ok(report)
}
fn complete_report(
    report: &mut DiagnosticReport,
    panels: &[(String, TrainingPanel)],
) -> Result<(), Error> {
    let frozen_runs = report.runs.clone();
    for run in &report.runs {
        flag(
            &mut report.checks,
            format!("{}/{}/training_fitness", method_name(run.method), run.seed),
            panels[0].1.fitness(&run.champion)? == run.fitness_numerator,
        );
        flag(
            &mut report.checks,
            format!("{}/{}/curve", method_name(run.method), run.seed),
            run.curve.iter().enumerate().all(|(index, point)| {
                point.evaluations as usize == index + 1
                    && panels[0].1.fitness(&point.champion).ok()
                        == Some(point.best_fitness_numerator)
            }) && run.curve.windows(2).all(|points| {
                points[1].best_fitness_numerator > points[0].best_fitness_numerator
                    || (points[1].best_fitness_numerator == points[0].best_fitness_numerator
                        && points[1].champion.bits <= points[0].champion.bits)
            }),
        );
        for (index, (name, panel)) in panels.iter().take(6).enumerate() {
            let evaluation = panel_evaluation(
                name,
                panel,
                &report.environments[index].optimum,
                method_name(run.method).into(),
                run.champion.clone(),
                Some(run.seed),
                Some(run.method),
            )?;
            flag(
                &mut report.checks,
                format!(
                    "{name}/{}/{}/exact_fitness",
                    method_name(run.method),
                    run.seed
                ),
                evaluation
                    .listener_evaluations
                    .iter()
                    .map(|e| e.utility_numerator)
                    .sum::<i64>()
                    == evaluation.fitness_numerator,
            );
            flag(
                &mut report.checks,
                format!(
                    "{name}/{}/{}/reference_regret",
                    method_name(run.method),
                    run.seed
                ),
                evaluation
                    .listener_evaluations
                    .iter()
                    .all(|e| e.reference_regret_numerator == 0 && e.error_mass == 0),
            );
            report.frozen_evaluations.push(evaluation);
        }
    }
    flag(
        &mut report.checks,
        "champions_and_curves_frozen".into(),
        report.runs == frozen_runs,
    );
    for (index, (name, panel)) in panels.iter().take(6).enumerate() {
        for method in [SearchMethod::Genetic, SearchMethod::Random] {
            let mut values: Vec<i64> = report
                .frozen_evaluations
                .iter()
                .filter(|e| e.environment == *name && e.method == Some(method))
                .map(|e| e.fitness_numerator)
                .collect();
            values.sort();
            let denominator = panel.denominator();
            report.summaries.push(SearchSummary {
                method,
                environment: name.clone(),
                count: values.len(),
                minimum: ExactValue {
                    numerator: values[0],
                    denominator,
                },
                median: ExactValue {
                    numerator: values[9] + values[10],
                    denominator: 2 * denominator,
                },
                maximum: ExactValue {
                    numerator: values[19],
                    denominator,
                },
                mean: ExactValue {
                    numerator: values.iter().sum(),
                    denominator: 20 * denominator,
                },
                zero_reporter_regret_count: values
                    .iter()
                    .filter(|&&value| value == report.environments[index].optimum.fitness_numerator)
                    .count(),
            });
        }
        for seed in 0..20 {
            let fitness = |method| {
                report
                    .frozen_evaluations
                    .iter()
                    .find(|e| {
                        e.environment == *name && e.method == Some(method) && e.seed == Some(seed)
                    })
                    .expect("complete fixed collection")
                    .fitness_numerator
            };
            report.paired_differences.push(PairedDifference {
                seed,
                environment: name.clone(),
                utility_difference_numerator: fitness(SearchMethod::Genetic)
                    - fitness(SearchMethod::Random),
                denominator: panel.denominator(),
            });
        }
    }
    Ok(())
}

// Independent Fraction worlds/actions and exhaustive/decomposed optima; no Rust oracle call.
const REFERENCE_PROVENANCE: &str = r#"{"bit_encoding":"calibration bits 0,1; live row r at bit 2+r","boolean_order":[false,true],"history_order":["C","calibration_report_strategic","calibration_report_fixed","live_report_strategic","live_report_fixed"],"holdout_extension_script_sha256":"c6c4b48bfe15770b1564a68e6062ee985391e1173ad264ab82d26bde97935a63","live_row":"8*calibration_signal + 4*own_calibration_report + 2*C + live_signal","oracle":"Python standard-library Fraction independent latent-variable enumeration","random_search_or_ga_run":false,"rust_called_or_imported":false,"script_sha256":"dac061c8ce428b998a4db9d6a190ab09139d99282a56a28938d92e55f5127006","tie_rule":"abstain on posterior equality; false report on row utility equality; lowest unsigned optimal encoding","unsupported_assumed_history":"null posterior/action, never fabricated evidence","world_order":["C","T","fixed_copy","calibration_signal_strategic","calibration_signal_fixed","live_signal_strategic","live_signal_fixed"]}"#;
const REFERENCES: &str = r#"{"training":{"world_denominator":10000,"panel_denominator":20000,"canonical_bits":81942,"fitness":1260,"actions":[[false,false,false,true,false,false,true,true,false,true,false,true,true,false,false,false,true,false,false,false,false,true,false,true,false,false,true,true,false,false,false,true],[false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":-900,"masses":[[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[900,-900,900,2500,800,3300],[0,0,1800,0,0,5000]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-3150,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225]],"metrics":[[1650,-1650,150,5000,1675,1675],[1500,-1500,300,5000,1750,1750]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":81942,"fitness":1260,"masses":[[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196]],"metrics":[[-510,510,3510,3138,1824,3686],[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[8000,2000,0,10000]},{"name":"copy","bits":174762,"fitness":-4770,"masses":[[848,64],[452,196],[452,256],[848,784],[332,76],[368,64],[368,304],[332,256],[212,16],[113,49],[113,64],[212,196],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[212,16],[113,49],[113,64],[212,196],[332,76],[368,64],[368,304],[332,256],[848,64],[452,196],[452,256],[848,784]],"metrics":[[2520,-2520,480,3912,696,1784],[2250,-2250,750,2950,350,2400]],"reference_payoff":3000,"agreements":[8000,8000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":1260,"masses":[[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196]],"metrics":[[-510,510,3510,3138,1824,3686],[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[8000,2000,0,10000]},{"name":"invert","bits":87381,"fitness":-810,"masses":[[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49]],"metrics":[[1560,-1560,1440,3912,1176,2264],[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[2000,2000,10000,10000]}]},"evolved_q3_5":{"world_denominator":10000,"panel_denominator":10000,"canonical_bits":98342,"fitness":310,"actions":[[false,false,false,true,false,false,true,true,false,true,false,true,true,false,false,false,true,false,false,false,false,true,false,true,false,false,true,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":-100,"masses":[[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[100,-100,400,2500,1200,3700]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-350,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775]],"metrics":[[350,-350,150,5000,2325,2325]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":98342,"fitness":310,"masses":[[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[282,96],[268,124],[268,144],[282,186],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[282,96],[268,124],[268,144],[282,186],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186]],"metrics":[[-310,310,1310,3710,2010,3300]],"reference_payoff":1000,"agreements":[6000,4800,0,6000]},{"name":"copy","bits":174762,"fitness":-560,"masses":[[423,144],[402,186],[402,216],[423,279],[342,126],[333,144],[333,189],[342,216],[282,96],[268,124],[268,144],[282,186],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[282,96],[268,124],[268,144],[282,186],[342,126],[333,144],[333,189],[342,216],[423,144],[402,186],[402,216],[423,279]],"metrics":[[560,-560,440,3752,1596,2844]],"reference_payoff":1000,"agreements":[6000,6000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":130,"masses":[[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186]],"metrics":[[-130,130,1130,3698,1914,3216]],"reference_payoff":1000,"agreements":[6000,4000,0,10000]},{"name":"invert","bits":87381,"fitness":-80,"masses":[[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124]],"metrics":[[80,-80,920,3752,1836,3084]],"reference_payoff":1000,"agreements":[4000,4000,10000,10000]}]},"evolved_q4_5":{"world_denominator":10000,"panel_denominator":10000,"canonical_bits":98342,"fitness":720,"actions":[[false,false,false,true,false,false,true,true,false,true,false,true,true,false,false,false,true,false,false,false,false,true,false,true,false,false,true,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":-900,"masses":[[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[900,-900,900,2500,800,3300]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-1650,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225]],"metrics":[[1650,-1650,150,5000,1675,1675]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":98342,"fitness":720,"masses":[[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[212,16],[113,49],[113,64],[212,196],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[212,16],[113,49],[113,64],[212,196],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196]],"metrics":[[-720,720,3720,3120,1920,3800]],"reference_payoff":3000,"agreements":[8000,3200,0,8000]},{"name":"copy","bits":174762,"fitness":-2520,"masses":[[848,64],[452,196],[452,256],[848,784],[332,76],[368,64],[368,304],[332,256],[212,16],[113,49],[113,64],[212,196],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[212,16],[113,49],[113,64],[212,196],[332,76],[368,64],[368,304],[332,256],[848,64],[452,196],[452,256],[848,784]],"metrics":[[2520,-2520,480,3912,696,1784]],"reference_payoff":3000,"agreements":[8000,8000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":510,"masses":[[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196]],"metrics":[[-510,510,3510,3138,1824,3686]],"reference_payoff":3000,"agreements":[8000,2000,0,10000]},{"name":"invert","bits":87381,"fitness":-1560,"masses":[[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49]],"metrics":[[1560,-1560,1440,3912,1176,2264]],"reference_payoff":3000,"agreements":[2000,2000,10000,10000]}]},"skeptical_q3_5":{"world_denominator":10000,"panel_denominator":10000,"canonical_bits":5140,"fitness":250,"actions":[[false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":0,"masses":[[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[0,0,500,0,0,5000]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-500,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775]],"metrics":[[500,-500,0,5000,2250,2250]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":5140,"fitness":250,"masses":[[670,360],[705,465],[705,240],[670,310],[555,315],[570,360],[570,210],[555,240],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[555,315],[570,360],[570,210],[555,240],[670,360],[705,465],[705,240],[670,310],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[5000,4000,5000,10000]},{"name":"copy","bits":174762,"fitness":-750,"masses":[[423,144],[402,186],[402,216],[423,279],[342,126],[333,144],[333,189],[342,216],[282,96],[268,124],[268,144],[282,186],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[282,96],[268,124],[268,144],[282,186],[342,126],[333,144],[333,189],[342,216],[423,144],[402,186],[402,216],[423,279]],"metrics":[[750,-750,250,2550,900,3350]],"reference_payoff":1000,"agreements":[6000,6000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":250,"masses":[[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186]],"metrics":[[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[6000,4000,0,10000]},{"name":"invert","bits":87381,"fitness":250,"masses":[[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124]],"metrics":[[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[4000,4000,10000,10000]}]},"skeptical_q4_5":{"world_denominator":10000,"panel_denominator":10000,"canonical_bits":5140,"fitness":750,"actions":[[false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":0,"masses":[[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[0,0,1800,0,0,5000]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-1500,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1625,400],[1625,1225],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[875,475],[875,400],[0,0],[0,0],[1625,400],[1625,1225]],"metrics":[[1500,-1500,300,5000,1750,1750]],"reference_payoff":1800,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":5140,"fitness":750,"masses":[[565,320],[1060,980],[1060,80],[565,245],[460,380],[415,320],[415,95],[460,80],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[460,380],[415,320],[415,95],[460,80],[565,320],[1060,980],[1060,80],[565,245],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[5000,2000,5000,10000]},{"name":"copy","bits":174762,"fitness":-2250,"masses":[[848,64],[452,196],[452,256],[848,784],[332,76],[368,64],[368,304],[332,256],[212,16],[113,49],[113,64],[212,196],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[212,16],[113,49],[113,64],[212,196],[332,76],[368,64],[368,304],[332,256],[848,64],[452,196],[452,256],[848,784]],"metrics":[[2250,-2250,750,2950,350,2400]],"reference_payoff":3000,"agreements":[8000,8000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":750,"masses":[[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196]],"metrics":[[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[8000,2000,0,10000]},{"name":"invert","bits":87381,"fitness":750,"masses":[[113,64],[212,196],[212,16],[113,49],[92,76],[83,64],[83,19],[92,16],[452,256],[848,784],[848,64],[452,196],[368,304],[332,256],[332,76],[368,64],[368,304],[332,256],[332,76],[368,64],[452,256],[848,784],[848,64],[452,196],[92,76],[83,64],[83,19],[92,16],[113,64],[212,196],[212,16],[113,49]],"metrics":[[-750,750,3750,2050,1400,4350]],"reference_payoff":3000,"agreements":[2000,2000,10000,10000]}]},"training_pair_q3_5":{"world_denominator":10000,"panel_denominator":20000,"canonical_bits":81942,"fitness":500,"actions":[[false,false,false,true,false,false,true,true,false,true,false,true,false,false,false,true,false,false,false,true,false,true,false,true,false,false,true,true,false,false,false,true],[false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true,false,false,false,true]],"controls":[{"name":"always_negative","bits":0,"fitness":-175,"masses":[[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0]],"metrics":[[175,-175,325,1375,600,4225],[0,0,500,0,0,5000]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"always_positive","bits":262143,"fitness":-925,"masses":[[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1375,600],[1375,775],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[0,0],[1125,525],[1125,600],[0,0],[0,0],[1375,600],[1375,775]],"metrics":[[425,-425,75,6125,2850,1725],[500,-500,0,5000,2250,2250]],"reference_payoff":500,"agreements":[5000,5000,5000,5000]},{"name":"canonical_optimum","bits":81942,"fitness":500,"masses":[[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186]],"metrics":[[-250,250,1250,3698,1974,3276],[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[6000,4000,0,10000]},{"name":"copy","bits":174762,"fitness":-1550,"masses":[[423,144],[402,186],[402,216],[423,279],[342,126],[333,144],[333,189],[342,216],[282,96],[268,124],[268,144],[282,186],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[282,96],[268,124],[268,144],[282,186],[342,126],[333,144],[333,189],[342,216],[423,144],[402,186],[402,216],[423,279]],"metrics":[[800,-800,200,3752,1476,2724],[750,-750,250,2550,900,3350]],"reference_payoff":1000,"agreements":[6000,6000,0,0]},{"name":"copy_calibration_invert_live","bits":87382,"fitness":500,"masses":[[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186]],"metrics":[[-250,250,1250,3698,1974,3276],[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[6000,4000,0,10000]},{"name":"invert","bits":87381,"fitness":350,"masses":[[268,144],[282,186],[282,96],[268,124],[222,126],[228,144],[228,84],[222,96],[402,216],[423,279],[423,144],[402,186],[333,189],[342,216],[342,126],[333,144],[333,189],[342,216],[342,126],[333,144],[402,216],[423,279],[423,144],[402,186],[222,126],[228,144],[228,84],[222,96],[268,144],[282,186],[282,96],[268,124]],"metrics":[[-100,100,1100,3752,1926,3174],[-250,250,1250,2450,1350,3900]],"reference_payoff":1000,"agreements":[4000,4000,10000,10000]}]}}"#;

#[cfg(test)]
mod tests {
    use super::*;
    fn complete_fixture() -> DiagnosticReport {
        let panels = panels().unwrap();
        let mut report = DiagnosticReport {
            version: DIAGNOSTIC_VERSION.into(),
            game_version: 1,
            protocol_version: 1,
            search_version: SEARCH_VERSION.into(),
            metadata: json!({
                "search_seeds":{"start_inclusive":0,"end_exclusive":20},
                "seed_derivation":{"version":SEARCH_SEED_DERIVATION,"multiplier":6364136223846793005u64,"offset_multiplier":1442695040888963407u64,"genetic_identity":3,"random_identity":4},
                "search_settings":{"policy_bits":18,"population":64,"generations":50,"elites":2,"tournament":3,"crossover":"1/2 independently per bit","mutation":"1/18 independently per bit","evaluations":3164,"count_repeated_candidates":true,"rank":"higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order"},
                "training":"equally weighted frozen Bayesian assumed-channel and Credulous algorithms; q=4/5; actual fixed copy prior=3/4; opposed reporter utility",
                "assumptions":"Legacy listeners continue to model both reporters as persistent copy/invert channels. Their assumed prior is separate from actual generative rules; no assumed-model belief is labeled an actual-policy posterior.",
                "policy_information":"calibration signal; own calibration report; verified C; live signal. Other calibration report is publicly visible but ignored by this restricted 18-bit family; unreachable rows are retained in search encodings.",
                "reference_information":"policy-aware reference knows candidate policy and actual generative rules, never private realized signals or truths",
                "freeze":"all 40 champions are selected on training alone and owned before any champion holdout evaluation",
                "inference":"actual-policy posterior is exact true/total history mass; zero-mass histories have no posterior or action; exact decision ties abstain",
                "gate":"correctness and integrity only; no GA superiority or search success requirement",
                "archives":"privileged unauthenticated replay records",
                "reference_provenance":serde_json::from_str::<Value>(REFERENCE_PROVENANCE).expect("independent reference provenance literal"),
                "scope":"finite one-calibration/one-live game; expected utility and regret, not guaranteed realized losses, psychological simplicity or equilibrium; no new interactive host mode"
            }),
            environments: vec![],
            checks: numerical_integrity().unwrap(),
            fixed_evaluations: vec![],
            runs: vec![],
            frozen_evaluations: vec![],
            summaries: vec![],
            paired_differences: vec![],
            passed: false,
        };
        for (name, panel) in &panels {
            let optimum = exact_best_response(panel).unwrap();
            for (label, policy) in controls(&optimum) {
                report.fixed_evaluations.push(
                    panel_evaluation(name, panel, &optimum, label, policy, None, None).unwrap(),
                );
            }
            report.environments.push(PanelDescription {
                name: name.clone(),
                rules: panel.config().clone(),
                listeners: panel.listeners().to_vec(),
                optimum,
            });
        }
        // Synthetic fixed candidates, not a search: every curve point has the same exact policy.
        let settings = SearchSettings {
            version: 1,
            policy_bits: 18,
            population: 64,
            generations: 50,
            elites: 2,
            tournament: 3,
            crossover_numerator: 1,
            crossover_denominator: 2,
            mutation_numerator: 1,
            mutation_denominator: 18,
            evaluations: 3164,
            ranking: "higher_exact_fitness_then_lower_unsigned_encoding_then_stable_order".into(),
        };
        for seed in 0..20 {
            for method in [SearchMethod::Genetic, SearchMethod::Random] {
                let champion = Policy::negative();
                let fitness_numerator = panels[0].1.fitness(&champion).unwrap();
                let curve = (1..=3164)
                    .map(|evaluations| ProgressPoint {
                        evaluations,
                        best_fitness_numerator: fitness_numerator,
                        champion: champion.clone(),
                    })
                    .collect();
                report.runs.push(SearchRun {
                    version: SEARCH_VERSION.into(),
                    seed,
                    derived_seed: search_seed(seed, method),
                    method,
                    seed_derivation: SEARCH_SEED_DERIVATION.into(),
                    settings: settings.clone(),
                    champion,
                    fitness_numerator,
                    denominator: 20000,
                    evaluations: 3164,
                    curve,
                });
            }
        }
        for run in &report.runs {
            flag(
                &mut report.checks,
                format!("{}/{}/training_fitness", method_name(run.method), run.seed),
                true,
            );
            flag(
                &mut report.checks,
                format!("{}/{}/curve", method_name(run.method), run.seed),
                true,
            );
            for (index, (name, panel)) in panels.iter().take(6).enumerate() {
                report.frozen_evaluations.push(
                    panel_evaluation(
                        name,
                        panel,
                        &report.environments[index].optimum,
                        method_name(run.method).into(),
                        run.champion.clone(),
                        Some(run.seed),
                        Some(run.method),
                    )
                    .unwrap(),
                );
                flag(
                    &mut report.checks,
                    format!(
                        "{name}/{}/{}/exact_fitness",
                        method_name(run.method),
                        run.seed
                    ),
                    true,
                );
                flag(
                    &mut report.checks,
                    format!(
                        "{name}/{}/{}/reference_regret",
                        method_name(run.method),
                        run.seed
                    ),
                    true,
                );
            }
        }
        flag(
            &mut report.checks,
            "champions_and_curves_frozen".into(),
            true,
        );
        for (index, (name, panel)) in panels.iter().take(6).enumerate() {
            let value = panel.fitness(&Policy::negative()).unwrap();
            let denominator = panel.denominator();
            for method in [SearchMethod::Genetic, SearchMethod::Random] {
                report.summaries.push(SearchSummary {
                    method,
                    environment: name.clone(),
                    count: 20,
                    minimum: ExactValue {
                        numerator: value,
                        denominator,
                    },
                    median: ExactValue {
                        numerator: 2 * value,
                        denominator: 2 * denominator,
                    },
                    maximum: ExactValue {
                        numerator: value,
                        denominator,
                    },
                    mean: ExactValue {
                        numerator: 20 * value,
                        denominator: 20 * denominator,
                    },
                    zero_reporter_regret_count: if value
                        == report.environments[index].optimum.fitness_numerator
                    {
                        20
                    } else {
                        0
                    },
                });
            }
            for seed in 0..20 {
                report.paired_differences.push(PairedDifference {
                    seed,
                    environment: name.clone(),
                    utility_difference_numerator: 0,
                    denominator,
                });
            }
        }
        report.passed = true;
        report
    }
    #[test]
    fn complete_synthetic_report_passes_without_search() {
        assert!(report_integrity(&complete_fixture()));
    }
    #[test]
    fn altered_frozen_settings_fail_even_with_stale_checks() {
        let original = complete_fixture();
        for field in 0..13 {
            let mut report = original.clone();
            let s = &mut report.runs[0].settings;
            match field {
                0 => s.version += 1,
                1 => s.policy_bits += 1,
                2 => s.population += 1,
                3 => s.generations += 1,
                4 => s.elites += 1,
                5 => s.tournament += 1,
                6 => s.crossover_numerator += 1,
                7 => s.crossover_denominator += 1,
                8 => s.mutation_numerator += 1,
                9 => s.mutation_denominator += 1,
                10 => s.ranking = "changed".into(),
                11 => s.evaluations += 1,
                _ => report.runs[0].denominator += 1,
            };
            assert!(!report_integrity(&report), "settings field {field}");
        }
    }
    #[test]
    fn altered_evaluation_utility_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.frozen_evaluations[0].fitness_numerator += 1;
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_evaluation_regret_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.frozen_evaluations[0].listener_evaluations[0].regret_numerator += 1;
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_summary_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.summaries[0].median.numerator += 1;
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_pair_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.paired_differences[0].utility_difference_numerator += 1;
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_curve_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.runs[0].curve[0].best_fitness_numerator += 1;
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_metadata_fails_with_stale_checks() {
        let mut report = complete_fixture();
        report.metadata["search_settings"]["mutation"] = "changed".into();
        assert!(!report_integrity(&report));
    }
    #[test]
    fn altered_reference_check_cannot_be_self_certified() {
        let mut report = complete_fixture();
        report.checks[0].quantity = "changed".into();
        assert!(!report_integrity(&report));
    }
}
