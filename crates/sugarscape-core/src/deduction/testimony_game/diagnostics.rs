//! Frozen finite benchmark. Correctness gates are independent of search outcomes.
use super::*;
use serde::Serialize;
use serde_json::{json, Value};
const TOLERANCE: f64 = 1e-12;
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReferenceCheck {
    pub quantity: String,
    pub expected_exact: String,
    pub expected: f64,
    pub actual: f64,
    pub error: f64,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IntegrityCheck {
    pub quantity: String,
    pub expected: bool,
    pub actual: bool,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NamedEnvironment {
    pub name: String,
    pub distribution: Distribution,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NamedEvaluation {
    pub environment: String,
    pub policy: String,
    pub seed: Option<u64>,
    pub genome: Option<Genome>,
    pub payoff: f64,
    pub optimal_payoff: f64,
    pub regret: f64,
    pub intervention_probability: f64,
    pub correct_intervention_probability: f64,
    pub incorrect_intervention_probability: f64,
    pub evaluation: Evaluation,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SearchSummary {
    pub method: SearchMethod,
    pub environment: String,
    pub count: usize,
    pub minimum: f64,
    pub median: f64,
    pub maximum: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PairedDifference {
    pub seed: u64,
    pub environment: String,
    pub payoff_numerator: i64,
    pub denominator: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GameReport {
    pub version: String,
    pub protocol_version: u16,
    pub game_version: u16,
    pub listener_version: String,
    pub search_version: String,
    pub tolerance: f64,
    pub metadata: Value,
    pub environments: Vec<NamedEnvironment>,
    pub checks: Vec<ReferenceCheck>,
    pub integrity_checks: Vec<IntegrityCheck>,
    pub fixed_evaluations: Vec<NamedEvaluation>,
    pub runs: Vec<SearchRun>,
    pub frozen_evaluations: Vec<NamedEvaluation>,
    pub summaries: Vec<SearchSummary>,
    pub paired_differences: Vec<PairedDifference>,
    pub passed: bool,
}
fn rational(s: &str) -> f64 {
    let (a, b) = s.split_once('/').unwrap_or((s, "1"));
    a.parse::<f64>().expect("literal numerator") / b.parse::<f64>().expect("literal denominator")
}
fn reference(quantity: String, expected_exact: &str, actual: f64) -> ReferenceCheck {
    let expected = rational(expected_exact);
    let error = (expected - actual).abs();
    ReferenceCheck {
        quantity,
        expected_exact: expected_exact.into(),
        expected,
        actual,
        error,
        passed: actual.is_finite() && error <= TOLERANCE,
    }
}
fn integrity(quantity: String, actual: bool) -> IntegrityCheck {
    IntegrityCheck {
        quantity,
        expected: true,
        actual,
        passed: actual,
    }
}
fn gates(checks: &[ReferenceCheck], integrity: &[IntegrityCheck]) -> bool {
    checks.iter().all(|c| c.passed) && integrity.iter().all(|c| c.passed)
}
fn policy_name(method: SearchMethod) -> &'static str {
    match method {
        SearchMethod::Genetic => "genetic",
        SearchMethod::Random => "random",
    }
}
pub fn diagnose_testimony_game() -> Result<GameReport, Error> {
    let refs: Value =
        serde_json::from_str(REFERENCES).expect("checked-in independent Fraction literals");
    let mut environments = Vec::new();
    for (name, qn, qd, rn, rd) in [
        ("training", 4, 5, 3, 4),
        ("inversion", 4, 5, 1, 4),
        ("less_accurate", 3, 5, 3, 4),
        ("uninformative", 1, 2, 3, 4),
    ] {
        environments.push(NamedEnvironment {
            name: name.into(),
            distribution: enumerate(&Config::standard(
                Probability {
                    numerator: qn,
                    denominator: qd,
                },
                Probability {
                    numerator: rn,
                    denominator: rd,
                },
            ))?,
        });
    }
    let mut checks = Vec::new();
    let mut integrity_checks = Vec::new();
    let mut fixed_evaluations = Vec::new();
    for env in &environments {
        let d = &env.distribution;
        let r = &refs[&env.name];
        integrity_checks.push(integrity(
            format!("{}/reference_denominator", env.name),
            d.denominator == r["denominator"].as_u64().unwrap(),
        ));
        integrity_checks.push(integrity(
            format!("{}/normalization", env.name),
            d.histories.iter().map(|h| h.total_mass).sum::<u64>() == d.denominator,
        ));
        for (i, h) in d.histories.iter().enumerate() {
            let x = &r["histories"][i];
            let v = &h.observation;
            integrity_checks.push(integrity(
                format!("{}/history/{i}/observation_order", env.name),
                json!([
                    v.calibration_truth,
                    v.calibration_reports[0],
                    v.calibration_reports[1],
                    v.live_reports[0],
                    v.live_reports[1]
                ]) == x["key"],
            ));
            integrity_checks.push(integrity(
                format!("{}/history/{i}/exact_mass", env.name),
                h.total_mass == x["total_mass"].as_u64().unwrap()
                    && h.true_mass == x["true_mass"].as_u64().unwrap(),
            ));
        }
        for (name, listener) in [
            ("Bayesian", Listener::Bayesian),
            ("Credulous", Listener::Credulous),
            ("Skeptical", Listener::Skeptical),
            ("DirectEvidence", Listener::DirectEvidence),
            ("Passive", Listener::Passive),
        ] {
            let e = evaluate(d, &listener)?;
            let reference_name = if name == "DirectEvidence" {
                "Passive"
            } else {
                name
            };
            for (metric, actual) in [
                ("payoff", e.payoff_numerator as f64 / e.denominator as f64),
                ("regret", e.regret_numerator as f64 / e.denominator as f64),
            ] {
                checks.push(reference(
                    format!("{}/{name}/{metric}", env.name),
                    r[metric][reference_name].as_str().unwrap(),
                    actual,
                ));
            }
            if name != "Passive" {
                for (metric, actual) in [
                    ("squared_error", e.posterior_squared_error.unwrap()),
                    ("brier", e.brier.unwrap()),
                    (
                        "intervention",
                        e.intervention_mass as f64 / e.denominator as f64,
                    ),
                    (
                        "correct",
                        e.correct_intervention_mass as f64 / e.denominator as f64,
                    ),
                    (
                        "incorrect",
                        e.incorrect_intervention_mass as f64 / e.denominator as f64,
                    ),
                ] {
                    checks.push(reference(
                        format!("{}/{name}/{metric}", env.name),
                        r["metrics"][name][metric].as_str().unwrap(),
                        actual,
                    ));
                }
            }
            if name == "Bayesian" {
                for (i, h) in e.histories.iter().enumerate() {
                    checks.push(reference(
                        format!("{}/Bayesian/history/{i}/posterior", env.name),
                        r["histories"][i]["posterior"].as_str().unwrap(),
                        h.decision.posterior_true.unwrap(),
                    ));
                    checks.push(reference(
                        format!("{}/Bayesian/history/{i}/regret", env.name),
                        "0",
                        h.conditional_regret,
                    ));
                }
            }
            fixed_evaluations.push(NamedEvaluation {
                environment: env.name.clone(),
                policy: name.into(),
                seed: None,
                genome: None,
                payoff: e.payoff_numerator as f64 / e.denominator as f64,
                optimal_payoff: e.optimal_numerator as f64 / e.denominator as f64,
                regret: e.regret_numerator as f64 / e.denominator as f64,
                intervention_probability: e.intervention_mass as f64 / e.denominator as f64,
                correct_intervention_probability: e.correct_intervention_mass as f64
                    / e.denominator as f64,
                incorrect_intervention_probability: e.incorrect_intervention_mass as f64
                    / e.denominator as f64,
                evaluation: e,
            });
        }
    }
    // This owned immutable collection is complete before any holdout evaluation.
    let runs: Vec<SearchRun> = (0..20)
        .map(|seed| {
            Ok([
                search_ga(&environments[0].distribution, seed)?,
                search_random(&environments[0].distribution, seed)?,
            ])
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .flatten()
        .collect();
    let frozen_champions: Vec<_> = runs.iter().map(|r| r.champion.clone()).collect();
    let mut frozen_evaluations = Vec::new();
    for run in &runs {
        integrity_checks.push(integrity(
            format!("{}/{}/budget", policy_name(run.method), run.seed),
            run.evaluations == 3164 && run.settings.evaluations == 3164 && run.curve.len() == 3164,
        ));
        integrity_checks.push(integrity(
            format!("{}/{}/training_fitness", policy_name(run.method), run.seed),
            fitness(&environments[0].distribution, &run.champion)? == run.fitness_numerator,
        ));
        integrity_checks.push(integrity(
            format!("{}/{}/curve_champion", policy_name(run.method), run.seed),
            run.curve.last().is_some_and(|p| {
                p.champion == run.champion && p.best_fitness_numerator == run.fitness_numerator
            }),
        ));
        for env in &environments {
            let e = evaluate(&env.distribution, &Listener::Evolved(run.champion.clone()))?;
            if env.name == "uninformative" {
                checks.push(reference(
                    format!(
                        "uninformative/{}/{}/payoff",
                        policy_name(run.method),
                        run.seed
                    ),
                    "0",
                    e.payoff_numerator as f64 / e.denominator as f64,
                ));
            }
            integrity_checks.push(integrity(
                format!(
                    "{}/{}/{}/no_probability",
                    env.name,
                    policy_name(run.method),
                    run.seed
                ),
                e.posterior_max_error.is_none()
                    && e.posterior_squared_error.is_none()
                    && e.brier.is_none()
                    && e.histories
                        .iter()
                        .all(|h| h.decision.posterior_true.is_none()),
            ));
            frozen_evaluations.push(NamedEvaluation {
                environment: env.name.clone(),
                policy: policy_name(run.method).into(),
                seed: Some(run.seed),
                genome: Some(run.champion.clone()),
                payoff: e.payoff_numerator as f64 / e.denominator as f64,
                optimal_payoff: e.optimal_numerator as f64 / e.denominator as f64,
                regret: e.regret_numerator as f64 / e.denominator as f64,
                intervention_probability: e.intervention_mass as f64 / e.denominator as f64,
                correct_intervention_probability: e.correct_intervention_mass as f64
                    / e.denominator as f64,
                incorrect_intervention_probability: e.incorrect_intervention_mass as f64
                    / e.denominator as f64,
                evaluation: e,
            });
        }
    }
    integrity_checks.push(integrity(
        "champions_frozen".into(),
        runs.iter().map(|r| r.champion.clone()).collect::<Vec<_>>() == frozen_champions,
    ));
    let mut summaries = Vec::new();
    let mut paired_differences = Vec::new();
    for env in &environments {
        for method in [SearchMethod::Genetic, SearchMethod::Random] {
            let mut payoffs: Vec<f64> = frozen_evaluations
                .iter()
                .filter(|e| e.environment == env.name && e.policy == policy_name(method))
                .map(|e| e.evaluation.payoff_numerator as f64 / e.evaluation.denominator as f64)
                .collect();
            payoffs.sort_by(f64::total_cmp);
            summaries.push(SearchSummary {
                method,
                environment: env.name.clone(),
                count: payoffs.len(),
                minimum: payoffs[0],
                median: (payoffs[9] + payoffs[10]) / 2.0,
                maximum: payoffs[19],
            });
        }
        for seed in 0..20 {
            let find = |policy| {
                &frozen_evaluations
                    .iter()
                    .find(|e| {
                        e.environment == env.name && e.policy == policy && e.seed == Some(seed)
                    })
                    .unwrap()
                    .evaluation
            };
            let ga = find("genetic");
            let random = find("random");
            integrity_checks.push(integrity(
                format!("{env_name}/{seed}/paired_denominator", env_name = env.name),
                ga.denominator == random.denominator,
            ));
            paired_differences.push(PairedDifference {
                seed,
                environment: env.name.clone(),
                payoff_numerator: ga.payoff_numerator - random.payoff_numerator,
                denominator: ga.denominator,
            });
        }
    }
    for (i, e) in fixed_evaluations
        .iter()
        .chain(&frozen_evaluations)
        .enumerate()
    {
        integrity_checks.push(integrity(
            format!("evaluation/{i}/finite_valid"),
            [
                e.payoff,
                e.optimal_payoff,
                e.regret,
                e.intervention_probability,
                e.correct_intervention_probability,
                e.incorrect_intervention_probability,
            ]
            .iter()
            .all(|v| v.is_finite())
                && e.evaluation.histories.iter().all(|h| {
                    h.reference_posterior.is_finite()
                        && h.conditional_regret.is_finite()
                        && h.conditional_regret >= 0.0
                })
                && e.evaluation.regret_numerator >= 0
                && e.evaluation.intervention_mass
                    == e.evaluation.correct_intervention_mass
                        + e.evaluation.incorrect_intervention_mass,
        ));
    }
    for pair in runs.as_chunks::<2>().0 {
        integrity_checks.push(integrity(
            format!("seed/{}/budget_parity", pair[0].seed),
            pair[0].seed == pair[1].seed
                && pair[0].settings == pair[1].settings
                && pair[0].evaluations == pair[1].evaluations,
        ));
    }
    let passed = gates(&checks, &integrity_checks);
    Ok(GameReport {
        version: "testimony-game-v1".into(),
        protocol_version: GAME_PROTOCOL_VERSION,
        game_version: GAME_VERSION,
        listener_version: "testimony-listener-v1".into(),
        search_version: "testimony-search-v1".into(),
        tolerance: TOLERANCE,
        metadata: json!({"training_environment":"training","search_seeds":{"start_inclusive":0,"end_exclusive":20},
            "seed_derivation":{"version":SEARCH_SEED_DERIVATION,"formula":"wrapping(seed * 6364136223846793005 + identity * 1442695040888963407)","genetic_identity":1,"random_identity":2},
            "channels":{"truth_priors":"independent Bernoulli(1/2)","profile_prior":"independent persistent copy with rho, invert otherwise","signals":"four independent conditional channels, accuracy q","propositions":[0,1],"signal_groups":[0,1,2,3],"calibration_verified":true,"live_verified_before_decision":false},
            "payoffs":{"intervene_true":1,"intervene_false":-1,"abstain":0},
            "genome":{"version":"testimony-genome-v1","bounds":{"b":[-16,16],"u":[0,16],"d":[0,16],"k":[-16,16]},"decode":"c_s=sigmoid(logit(rho)+b/4+(match?u/4:-d/4)); rho endpoints exact; score=sum_s (2q-1)(2c_s-1)(2report_s-1); intervene iff score>k/8","probability_output":false},
            "operators":{"initialization":"uniform independent integer genes","selection":"tournament 3 with replacement","crossover":"each gene independently from either parent with probability 1/2","mutation":"each gene independently probability 1/4, equal +/-1, clamp","ranking":"exact payoff descending; integer L1 ascending; gene tuple lexicographic ascending; stable exact ties","elites":"2 unchanged; count every candidate including repeats","fitness":"complete exact world enumeration; training only","budget":"64 + 50 * 62 = 3164 each method"},
            "exploratory_outcomes_gate_passed":false}),
        environments,
        checks,
        integrity_checks,
        fixed_evaluations,
        runs,
        frozen_evaluations,
        summaries,
        paired_differences,
        passed,
    })
}
// Independent precollection Python Fraction evidence, copied as source literals.
const REFERENCES: &str = r#"{"training":{"metrics":{"Bayesian":{"squared_error":"0","brier":"11676289037/66161654102","intervention":"489/1250","correct":"387/1250","incorrect":"51/625"},"Credulous":{"squared_error":"850180627143/17995969915744","brier":"2069/9248","intervention":"109/400","correct":"169/800","incorrect":"49/800"},"Skeptical":{"squared_error":"464924251575/14423240594236","brier":"91/436","intervention":"109/400","correct":"169/800","incorrect":"49/800"},"DirectEvidence":{"squared_error":"9728248977/132323308204","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"}},"denominator":40000,"histories":[{"key":[false,false,false,false,false],"total":"2657/40000","true":"4/625","total_mass":2657,"true_mass":256,"posterior":"256/2657"},{"key":[false,false,false,false,true],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[false,false,false,true,false],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[false,false,false,true,true],"total":"2657/40000","true":"2401/40000","total_mass":2657,"true_mass":2401,"posterior":"2401/2657"},{"key":[false,false,true,false,false],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[false,false,true,false,true],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[false,false,true,true,false],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[false,false,true,true,true],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[false,true,false,false,false],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[false,true,false,false,true],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[false,true,false,true,false],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[false,true,false,true,true],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[false,true,true,false,false],"total":"617/40000","true":"361/40000","total_mass":617,"true_mass":361,"posterior":"361/617"},{"key":[false,true,true,false,true],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[false,true,true,true,false],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[false,true,true,true,true],"total":"617/40000","true":"4/625","total_mass":617,"true_mass":256,"posterior":"256/617"},{"key":[true,false,false,false,false],"total":"617/40000","true":"361/40000","total_mass":617,"true_mass":361,"posterior":"361/617"},{"key":[true,false,false,false,true],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[true,false,false,true,false],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[true,false,false,true,true],"total":"617/40000","true":"4/625","total_mass":617,"true_mass":256,"posterior":"256/617"},{"key":[true,false,true,false,false],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[true,false,true,false,true],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[true,false,true,true,false],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[true,false,true,true,true],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[true,true,false,false,false],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[true,true,false,false,true],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[true,true,false,true,false],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[true,true,false,true,true],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[true,true,true,false,false],"total":"2657/40000","true":"4/625","total_mass":2657,"true_mass":256,"posterior":"256/2657"},{"key":[true,true,true,false,true],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[true,true,true,true,false],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[true,true,true,true,true],"total":"2657/40000","true":"2401/40000","total_mass":2657,"true_mass":2401,"posterior":"2401/2657"}],"payoff":{"Bayesian":"57/250","Credulous":"3/20","Skeptical":"3/20","Passive":"0"},"regret":{"Bayesian":"0","Credulous":"39/500","Skeptical":"39/500","Passive":"57/250"}},"inversion":{"metrics":{"Bayesian":{"squared_error":"0","brier":"11676289037/66161654102","intervention":"489/1250","correct":"387/1250","incorrect":"51/625"},"Credulous":{"squared_error":"5613819722487/17995969915744","brier":"4517/9248","intervention":"109/400","correct":"49/800","incorrect":"169/800"},"Skeptical":{"squared_error":"464924251575/14423240594236","brier":"91/436","intervention":"109/400","correct":"169/800","incorrect":"49/800"},"DirectEvidence":{"squared_error":"9728248977/132323308204","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"}},"denominator":40000,"histories":[{"key":[false,false,false,false,false],"total":"617/40000","true":"4/625","total_mass":617,"true_mass":256,"posterior":"256/617"},{"key":[false,false,false,false,true],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[false,false,false,true,false],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[false,false,false,true,true],"total":"617/40000","true":"361/40000","total_mass":617,"true_mass":361,"posterior":"361/617"},{"key":[false,false,true,false,false],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[false,false,true,false,true],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[false,false,true,true,false],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[false,false,true,true,true],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[false,true,false,false,false],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[false,true,false,false,true],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[false,true,false,true,false],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[false,true,false,true,true],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[false,true,true,false,false],"total":"2657/40000","true":"2401/40000","total_mass":2657,"true_mass":2401,"posterior":"2401/2657"},{"key":[false,true,true,false,true],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[false,true,true,true,false],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[false,true,true,true,true],"total":"2657/40000","true":"4/625","total_mass":2657,"true_mass":256,"posterior":"256/2657"},{"key":[true,false,false,false,false],"total":"2657/40000","true":"2401/40000","total_mass":2657,"true_mass":2401,"posterior":"2401/2657"},{"key":[true,false,false,false,true],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[true,false,false,true,false],"total":"49/1250","true":"49/2500","total_mass":1568,"true_mass":784,"posterior":"1/2"},{"key":[true,false,false,true,true],"total":"2657/40000","true":"4/625","total_mass":2657,"true_mass":256,"posterior":"256/2657"},{"key":[true,false,true,false,false],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[true,false,true,false,true],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[true,false,true,true,false],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[true,false,true,true,true],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[true,true,false,false,false],"total":"17/625","true":"49/2500","total_mass":1088,"true_mass":784,"posterior":"49/68"},{"key":[true,true,false,false,true],"total":"1187/40000","true":"4/625","total_mass":1187,"true_mass":256,"posterior":"256/1187"},{"key":[true,true,false,true,false],"total":"1187/40000","true":"931/40000","total_mass":1187,"true_mass":931,"posterior":"931/1187"},{"key":[true,true,false,true,true],"total":"17/625","true":"19/2500","total_mass":1088,"true_mass":304,"posterior":"19/68"},{"key":[true,true,true,false,false],"total":"617/40000","true":"4/625","total_mass":617,"true_mass":256,"posterior":"256/617"},{"key":[true,true,true,false,true],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[true,true,true,true,false],"total":"19/1250","true":"19/2500","total_mass":608,"true_mass":304,"posterior":"1/2"},{"key":[true,true,true,true,true],"total":"617/40000","true":"361/40000","total_mass":617,"true_mass":361,"posterior":"361/617"}],"payoff":{"Bayesian":"57/250","Credulous":"-3/20","Skeptical":"3/20","Passive":"0"},"regret":{"Bayesian":"0","Credulous":"189/500","Skeptical":"39/500","Passive":"57/250"}},"less_accurate":{"metrics":{"Bayesian":{"squared_error":"0","brier":"903530373/3693847508","intervention":"469/1250","correct":"4327/20000","incorrect":"3177/20000"},"Credulous":{"squared_error":"1859769829/384160140832","brier":"1349/5408","intervention":"101/400","correct":"121/800","incorrect":"81/800"},"Skeptical":{"squared_error":"83079075/186539299154","brier":"99/404","intervention":"101/400","correct":"121/800","incorrect":"81/800"},"DirectEvidence":{"squared_error":"4982876/923461877","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"}},"denominator":40000,"histories":[{"key":[false,false,false,false,false],"total":"1537/40000","true":"9/625","total_mass":1537,"true_mass":576,"posterior":"576/1537"},{"key":[false,false,false,false,true],"total":"93/2500","true":"93/5000","total_mass":1488,"true_mass":744,"posterior":"1/2"},{"key":[false,false,false,true,false],"total":"93/2500","true":"93/5000","total_mass":1488,"true_mass":744,"posterior":"1/2"},{"key":[false,false,false,true,true],"total":"1537/40000","true":"961/40000","total_mass":1537,"true_mass":961,"posterior":"961/1537"},{"key":[false,false,true,false,false],"total":"39/1250","true":"63/5000","total_mass":1248,"true_mass":504,"posterior":"21/52"},{"key":[false,false,true,false,true],"total":"1227/40000","true":"9/625","total_mass":1227,"true_mass":576,"posterior":"192/409"},{"key":[false,false,true,true,false],"total":"1227/40000","true":"651/40000","total_mass":1227,"true_mass":651,"posterior":"217/409"},{"key":[false,false,true,true,true],"total":"39/1250","true":"93/5000","total_mass":1248,"true_mass":744,"posterior":"31/52"},{"key":[false,true,false,false,false],"total":"39/1250","true":"63/5000","total_mass":1248,"true_mass":504,"posterior":"21/52"},{"key":[false,true,false,false,true],"total":"1227/40000","true":"651/40000","total_mass":1227,"true_mass":651,"posterior":"217/409"},{"key":[false,true,false,true,false],"total":"1227/40000","true":"9/625","total_mass":1227,"true_mass":576,"posterior":"192/409"},{"key":[false,true,false,true,true],"total":"39/1250","true":"93/5000","total_mass":1248,"true_mass":744,"posterior":"31/52"},{"key":[false,true,true,false,false],"total":"1017/40000","true":"441/40000","total_mass":1017,"true_mass":441,"posterior":"49/113"},{"key":[false,true,true,false,true],"total":"63/2500","true":"63/5000","total_mass":1008,"true_mass":504,"posterior":"1/2"},{"key":[false,true,true,true,false],"total":"63/2500","true":"63/5000","total_mass":1008,"true_mass":504,"posterior":"1/2"},{"key":[false,true,true,true,true],"total":"1017/40000","true":"9/625","total_mass":1017,"true_mass":576,"posterior":"64/113"},{"key":[true,false,false,false,false],"total":"1017/40000","true":"441/40000","total_mass":1017,"true_mass":441,"posterior":"49/113"},{"key":[true,false,false,false,true],"total":"63/2500","true":"63/5000","total_mass":1008,"true_mass":504,"posterior":"1/2"},{"key":[true,false,false,true,false],"total":"63/2500","true":"63/5000","total_mass":1008,"true_mass":504,"posterior":"1/2"},{"key":[true,false,false,true,true],"total":"1017/40000","true":"9/625","total_mass":1017,"true_mass":576,"posterior":"64/113"},{"key":[true,false,true,false,false],"total":"39/1250","true":"63/5000","total_mass":1248,"true_mass":504,"posterior":"21/52"},{"key":[true,false,true,false,true],"total":"1227/40000","true":"651/40000","total_mass":1227,"true_mass":651,"posterior":"217/409"},{"key":[true,false,true,true,false],"total":"1227/40000","true":"9/625","total_mass":1227,"true_mass":576,"posterior":"192/409"},{"key":[true,false,true,true,true],"total":"39/1250","true":"93/5000","total_mass":1248,"true_mass":744,"posterior":"31/52"},{"key":[true,true,false,false,false],"total":"39/1250","true":"63/5000","total_mass":1248,"true_mass":504,"posterior":"21/52"},{"key":[true,true,false,false,true],"total":"1227/40000","true":"9/625","total_mass":1227,"true_mass":576,"posterior":"192/409"},{"key":[true,true,false,true,false],"total":"1227/40000","true":"651/40000","total_mass":1227,"true_mass":651,"posterior":"217/409"},{"key":[true,true,false,true,true],"total":"39/1250","true":"93/5000","total_mass":1248,"true_mass":744,"posterior":"31/52"},{"key":[true,true,true,false,false],"total":"1537/40000","true":"9/625","total_mass":1537,"true_mass":576,"posterior":"576/1537"},{"key":[true,true,true,false,true],"total":"93/2500","true":"93/5000","total_mass":1488,"true_mass":744,"posterior":"1/2"},{"key":[true,true,true,true,false],"total":"93/2500","true":"93/5000","total_mass":1488,"true_mass":744,"posterior":"1/2"},{"key":[true,true,true,true,true],"total":"1537/40000","true":"961/40000","total_mass":1537,"true_mass":961,"posterior":"961/1537"}],"payoff":{"Bayesian":"23/400","Credulous":"1/20","Skeptical":"1/20","Passive":"0"},"regret":{"Bayesian":"0","Credulous":"3/400","Skeptical":"3/400","Passive":"23/400"}},"uninformative":{"metrics":{"Bayesian":{"squared_error":"0","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"},"Credulous":{"squared_error":"0","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"},"Skeptical":{"squared_error":"0","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"},"DirectEvidence":{"squared_error":"0","brier":"1/4","intervention":"0","correct":"0","incorrect":"0"}},"denominator":1024,"histories":[{"key":[false,false,false,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,false,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,false,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,false,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,true,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,true,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,true,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,false,true,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,false,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,false,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,false,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,false,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,true,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,true,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,true,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[false,true,true,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,false,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,false,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,false,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,false,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,true,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,true,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,true,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,false,true,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,false,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,false,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,false,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,false,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,true,false,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,true,false,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,true,true,false],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"},{"key":[true,true,true,true,true],"total":"1/32","true":"1/64","total_mass":32,"true_mass":16,"posterior":"1/2"}],"payoff":{"Bayesian":"0","Credulous":"0","Skeptical":"0","Passive":"0"},"regret":{"Bayesian":"0","Credulous":"0","Skeptical":"0","Passive":"0"}}}"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comparator_failures_reach_aggregate_gate() {
        assert!(!gates(
            &[reference("deliberate numeric failure".into(), "1/2", 0.6)],
            &[]
        ));
        assert!(!gates(
            &[],
            &[integrity("deliberate integrity failure".into(), false)]
        ));
        assert!(gates(
            &[reference("exact".into(), "1/2", 0.5)],
            &[integrity("true".into(), true)]
        ));
    }
}
