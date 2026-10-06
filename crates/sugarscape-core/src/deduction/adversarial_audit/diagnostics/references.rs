//! Portable independent oracle extracts; these are expected values, never report caches.
use super::*;
use crate::deduction::strategic_reporting::DecisionAction;
use crate::deduction::strategy_inference::canonical_bits;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiagnosticFixture {
    pub source_sha256: String,
    pub reference_sha256: String,
    pub provenance_source_report_sha256: String,
    pub champion_source_commit: String,
    pub champion_source_report_sha256: String,
    pub policy_provenance: Vec<PolicyProvenance>,
    fixed_models: Vec<FixedReference>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixedReference {
    q: [u16; 2],
    rows: Vec<FixedRowReference>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixedRowReference {
    index: u8,
    intervene: bool,
    mass: [u64; 2],
    posterior_true: [u64; 2],
    true_mass: [u64; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackReference {
    canonical_bits: Vec<u32>,
    environments: Vec<AttackEnvironment>,
    reference_sha256: String,
    source_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AttackEnvironment {
    q: [u16; 2],
    denominator: u64,
    action_tables: BTreeMap<String, Vec<ActionReference>>,
    basis: BTreeMap<String, [BasisRow; 4]>,
    fitness_utility_numerators: BTreeMap<String, Vec<i64>>,
    optima: BTreeMap<String, OptimumReference>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionReference {
    index: u8,
    intervene: bool,
    posterior_true: Option<[u64; 2]>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OptimumReference {
    calibration_candidates: [u32; 4],
    canonical_bits: u32,
    canonical_tie_count: u32,
    raw_tie_count: u32,
    utility: [i64; 2],
    utility_numerator: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationReference {
    source_sha256: String,
    reference_sha256: String,
    metric_names: Vec<String>,
    history_names: Vec<String>,
    environments: Vec<EvaluationEnvironment>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationEnvironment {
    q: [u16; 2],
    #[serde(rename = "fixed_only_F")]
    fixed_only_f: [u64; 2],
    strict_mixture_counterexamples: serde_json::Value,
    evaluations: Vec<EvaluationRow>,
}
type HistoryReference = (
    u8,
    [u8; 5],
    u64,
    u64,
    Option<[u64; 2]>,
    Option<bool>,
    Option<bool>,
    i64,
    i64,
);
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationRow {
    controller: ControllerKind,
    policy_bits: Option<u32>,
    mixture: Option<String>,
    target_controller: Option<ControllerKind>,
    group: String,
    guarantee_shortfall: Option<[u64; 2]>,
    nominal_minus_fixed_only: Option<[i64; 2]>,
    metrics: Vec<i64>,
    histories: Vec<HistoryReference>,
}
fn parse<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, Error> {
    serde_json::from_str(s)
        .map_err(|_| Error::InvalidReport("invalid embedded independent reference"))
}
pub(super) fn diagnostic_fixture() -> Result<DiagnosticFixture, Error> {
    parse(include_str!("../tests/diagnostic-reference.json"))
}
pub(super) fn fraction_equal(a: i128, ad: u64, b: i128, bd: u64) -> Result<bool, Error> {
    if ad == 0 || bd == 0 {
        return Err(Error::InvalidReport(
            "zero independent reference denominator",
        ));
    }
    Ok(a.checked_mul(i128::from(bd))
        .ok_or(Error::ArithmeticOverflow)?
        == b.checked_mul(i128::from(ad))
            .ok_or(Error::ArithmeticOverflow)?)
}
fn posterior_equal(actual: Option<&Ratio>, expected: Option<[u64; 2]>) -> Result<bool, Error> {
    match (actual, expected) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => fraction_equal(
            i128::from(a.numerator),
            a.denominator,
            i128::from(b[0]),
            b[1],
        ),
        _ => Ok(false),
    }
}
fn name(kind: ControllerKind) -> Result<String, Error> {
    serde_json::to_value(kind)
        .map_err(|_| Error::InvalidReport("controller serialization failed"))?
        .as_str()
        .map(str::to_owned)
        .ok_or(Error::InvalidReport("invalid controller name"))
}
fn lookup<'a, T>(map: &'a BTreeMap<String, T>, key: &str) -> Result<&'a T, Error> {
    map.get(key).ok_or(Error::InvalidReport(
        "independent reference missing controller",
    ))
}
struct Comparison {
    quantity: &'static str,
    expected: u64,
    actual: u64,
}
impl Comparison {
    fn new(quantity: &'static str) -> Self {
        Self {
            quantity,
            expected: 0,
            actual: 0,
        }
    }
    fn add(&mut self, passed: bool) -> Result<(), Error> {
        self.expected = self
            .expected
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        self.actual = self
            .actual
            .checked_add(u64::from(passed))
            .ok_or(Error::ArithmeticOverflow)?;
        Ok(())
    }
    fn finish(self) -> DiagnosticCheck {
        DiagnosticCheck {
            quantity: self.quantity.into(),
            expected_numerator: self.expected,
            actual_numerator: self.actual,
            denominator: 1,
            passed: self.expected > 0 && self.actual == self.expected,
        }
    }
}
fn evaluation_equal(
    actual: &AuditEvaluation,
    expected: &EvaluationRow,
    names: &[String],
) -> Result<bool, Error> {
    if names.len() != 15
        || expected.metrics.len() != 15
        || expected.histories.len() != 32
        || actual.histories.len() != 32
    {
        return Err(Error::InvalidReport(
            "independent evaluation shape mismatch",
        ));
    }
    let d = u64::try_from(expected.metrics[3])
        .map_err(|_| Error::InvalidReport("invalid reference evaluation denominator"))?;
    let value = serde_json::to_value(actual)
        .map_err(|_| Error::InvalidReport("evaluation serialization failed"))?;
    let mut passed = true;
    for (field, n) in names.iter().zip(&expected.metrics) {
        let a = value[field].as_i64().ok_or(Error::InvalidReport(
            "unknown independent evaluation metric",
        ))?;
        passed &= fraction_equal(i128::from(a), actual.denominator, i128::from(*n), d)?;
    }
    for (i, (h, e)) in actual.histories.iter().zip(&expected.histories).enumerate() {
        passed &= usize::from(e.0) == i && history_index(&h.observation) == i;
        passed &= [
            u8::from(h.observation.calibration_truth),
            u8::from(h.observation.calibration_reports[0]),
            u8::from(h.observation.calibration_reports[1]),
            u8::from(h.observation.live_reports[0]),
            u8::from(h.observation.live_reports[1]),
        ] == e.1;
        for (a, b) in [
            (i128::from(h.total_mass), i128::from(e.2)),
            (i128::from(h.true_mass), i128::from(e.3)),
            (i128::from(h.regret_numerator), i128::from(e.7)),
            (i128::from(h.reference_regret_numerator), i128::from(e.8)),
        ] {
            passed &= fraction_equal(a, actual.denominator, b, d)?;
        }
        let posterior = h.reference_posterior.as_ref().map(|p| Ratio {
            numerator: p.numerator,
            denominator: p.denominator,
        });
        passed &= posterior_equal(posterior.as_ref(), e.4)?;
        passed &= h.reference_action.map(|a| a == DecisionAction::Intervene) == e.5;
        passed &= h.listener_action.map(|a| a == DecisionAction::Intervene) == e.6;
    }
    Ok(passed)
}

pub(super) fn checks(r: &DiagnosticReport) -> Result<Vec<DiagnosticCheck>, Error> {
    let attack: AttackReference = parse(include_str!("../tests/attack-reference.json"))?;
    let evaluation: EvaluationReference =
        parse(include_str!("../tests/evaluation-reference.json"))?;
    let diagnostic = diagnostic_fixture()?;
    let mut sources = Comparison::new("independent_reference_provenance");
    sources.add(
        attack.reference_sha256 == r.metadata.reference_sha256
            && attack.source_sha256 == r.metadata.oracle_source_sha256
            && evaluation.reference_sha256 == r.metadata.reference_sha256
            && evaluation.source_sha256 == r.metadata.dto_extraction_source_sha256,
    )?;
    sources.add(
        evaluation.history_names
            == [
                "index",
                "observation_bits",
                "total_mass",
                "true_mass",
                "reference_posterior",
                "reference_action",
                "listener_action",
                "regret_numerator",
                "reference_regret_numerator",
            ],
    )?;
    let mut actions = Comparison::new("independent_action_rows");
    let mut fixed = Comparison::new("independent_fixed_posterior_rows");
    let mut bases = Comparison::new("independent_basis_rows");
    let mut fitness = Comparison::new("independent_canonical_fitness_rows");
    let mut optimum = Comparison::new("independent_optima_and_ties");
    let mut evaluations = Comparison::new("independent_complete_evaluations");
    let mut provenance = Comparison::new("preserved_policy_provenance");
    let mut bounds = Comparison::new("constant_report_minimax_bounds");
    if attack.environments.len() != 2
        || evaluation.environments.len() != 2
        || diagnostic.fixed_models.len() != 2
        || attack.canonical_bits.len() != 1024
    {
        return Err(Error::InvalidReport("independent reference shape mismatch"));
    }
    sources.add(
        canonical_policies()
            .iter()
            .map(|p| p.bits)
            .collect::<Vec<_>>()
            == attack.canonical_bits,
    )?;
    for (ei, env) in r.environments.iter().enumerate() {
        let a = &attack.environments[ei];
        let e = &evaluation.environments[ei];
        let f = &diagnostic.fixed_models[ei];
        let q = [env.rules.accuracy.numerator, env.rules.accuracy.denominator];
        sources.add(
            a.q == q
                && e.q == q
                && f.q == q
                && f.rows.len() == 8
                && e.evaluations.len() == 52
                && e.strict_mixture_counterexamples.is_array(),
        )?;
        let model = &r.fixed_only_models[ei];
        for (row, expected) in model.rows.iter().zip(&f.rows) {
            fixed.add(
                row.index == expected.index
                    && row.decision.action
                        == (if expected.intervene {
                            DecisionAction::Intervene
                        } else {
                            DecisionAction::Abstain
                        })
                    && posterior_equal(
                        row.decision.posterior_true.as_ref(),
                        Some(expected.posterior_true),
                    )?
                    && fraction_equal(
                        i128::from(row.total_mass),
                        model.denominator,
                        i128::from(expected.mass[0]),
                        expected.mass[1],
                    )?
                    && fraction_equal(
                        i128::from(row.true_mass),
                        model.denominator,
                        i128::from(expected.true_mass[0]),
                        expected.true_mass[1],
                    )?,
            )?;
        }
        for i in 0..4 {
            let snapshot = &r.controller_snapshots[ei * 4 + i].snapshot;
            let key = name(snapshot.controller)?;
            let expected = lookup(&a.action_tables, &key)?;
            if expected.len() != 32 {
                return Err(Error::InvalidReport("independent action shape mismatch"));
            }
            for (row, e) in snapshot.rows.iter().zip(expected) {
                actions.add(
                    row.index == e.index
                        && row.decision.action
                            == (if e.intervene {
                                DecisionAction::Intervene
                            } else {
                                DecisionAction::Abstain
                            })
                        && posterior_equal(row.decision.posterior_true.as_ref(), e.posterior_true)?,
                )?;
            }
            let table = &r.fitness_tables[ei * 4 + i];
            for (row, e) in table.basis.iter().zip(lookup(&a.basis, &key)?) {
                bases.add(row == e)?;
            }
            let expected = lookup(&a.fitness_utility_numerators, &key)?;
            if expected.len() != 1024 {
                return Err(Error::InvalidReport("independent fitness shape mismatch"));
            }
            for ((row, e), bits) in table.rows.iter().zip(expected).zip(&attack.canonical_bits) {
                fitness.add(
                    row.canonical_bits == *bits
                        && fraction_equal(
                            i128::from(row.utility_numerator),
                            table.denominator,
                            i128::from(*e),
                            a.denominator,
                        )?,
                )?;
            }
            let target = &r.targeted_audits[ei * 4 + i];
            let expected = lookup(&a.optima, &key)?;
            optimum.add(
                target.witness.policy.bits == expected.canonical_bits
                    && target.canonical_tie_count == expected.canonical_tie_count
                    && target.raw_tie_count == expected.raw_tie_count
                    && fraction_equal(
                        i128::from(target.witness.score.utility_numerator),
                        target.witness.score.denominator,
                        i128::from(expected.utility[0]),
                        u64::try_from(expected.utility[1]).map_err(|_| {
                            Error::InvalidReport("invalid optimum reference denominator")
                        })?,
                    )?
                    && fraction_equal(
                        i128::from(target.witness.score.utility_numerator),
                        target.witness.score.denominator,
                        i128::from(expected.utility_numerator),
                        a.denominator,
                    )?,
            )?;
            // All four independent calibration candidates must attain their basis-specific optimum.
            for (calibration, bits) in expected.calibration_candidates.iter().enumerate() {
                let row = &table.basis[calibration];
                let mut candidate = calibration as u32;
                for (live, delta) in row.deltas.iter().enumerate() {
                    if delta.is_some_and(|d| d > 0) {
                        candidate |= 1 << (2 + live);
                    }
                }
                optimum.add(candidate == *bits)?;
            }
        }
        for expected in &e.evaluations {
            let (actual, extra) = match expected.group.as_str() {
                "targeted_audits" => {
                    let row = r
                        .targeted_audits
                        .iter()
                        .find(|row| {
                            row.environment_id == env.id
                                && row.controller == expected.controller
                                && Some(row.witness.policy.bits) == expected.policy_bits
                        })
                        .ok_or(Error::InvalidReport("missing targeted reference identity"))?;
                    let shortfall = expected
                        .guarantee_shortfall
                        .ok_or(Error::InvalidReport("missing reference shortfall"))?;
                    (
                        &row.evaluation,
                        posterior_equal(Some(&row.guarantee_shortfall), Some(shortfall))?,
                    )
                }
                "control_evaluations" => {
                    let row = r
                        .control_evaluations
                        .iter()
                        .find(|row| {
                            row.environment_id == env.id
                                && row.controller == expected.controller
                                && Some(row.canonical_bits) == expected.policy_bits
                        })
                        .ok_or(Error::InvalidReport("missing control reference identity"))?;
                    (&row.evaluation, true)
                }
                "nominal_evaluations" => {
                    let row = r
                        .nominal_evaluations
                        .iter()
                        .find(|row| {
                            row.environment_id == env.id
                                && row.controller == expected.controller
                                && Some(&row.population) == expected.mixture.as_ref()
                        })
                        .ok_or(Error::InvalidReport("missing nominal reference identity"))?;
                    let n = expected
                        .nominal_minus_fixed_only
                        .ok_or(Error::InvalidReport("missing reference nominal difference"))?;
                    (
                        &row.evaluation,
                        fraction_equal(
                            i128::from(row.nominal_minus_fixed_only.numerator),
                            row.nominal_minus_fixed_only.denominator,
                            i128::from(n[0]),
                            u64::try_from(n[1]).map_err(|_| {
                                Error::InvalidReport("invalid nominal reference denominator")
                            })?,
                        )?,
                    )
                }
                "cross_target_evaluations" => {
                    let row = r
                        .cross_target_evaluations
                        .iter()
                        .find(|row| {
                            row.environment_id == env.id
                                && row.controller == expected.controller
                                && Some(row.witness_bits) == expected.policy_bits
                                && Some(row.target_controller) == expected.target_controller
                        })
                        .ok_or(Error::InvalidReport(
                            "missing cross-target reference identity",
                        ))?;
                    (&row.evaluation, true)
                }
                _ => return Err(Error::InvalidReport("unknown independent evaluation group")),
            };
            evaluations
                .add(extra && evaluation_equal(actual, expected, &evaluation.metric_names)?)?;
        }
        let bound = &r.bound_checks[ei];
        bounds.add(
            bound.passed
                && posterior_equal(Some(&bound.fixed_guarantee), Some(e.fixed_only_f))?
                && posterior_equal(Some(&bound.upper_bound), Some(e.fixed_only_f))?
                && posterior_equal(
                    Some(&bound.constant_negative_informed_reference),
                    Some(e.fixed_only_f),
                )?
                && posterior_equal(
                    Some(&bound.constant_positive_informed_reference),
                    Some(e.fixed_only_f),
                )?,
        )?;
    }
    provenance.add(
        r.policy_provenance == diagnostic.policy_provenance && r.policy_provenance.len() == 46,
    )?;
    for p in &r.policy_provenance {
        provenance.add(canonical_bits(&Policy::new(p.raw_bits)?)? == p.canonical_bits)?;
    }
    provenance.add(
        r.policy_provenance
            .iter()
            .filter(|p| p.method == "genetic" || p.method == "random")
            .count()
            == 40,
    )?;
    Ok(vec![
        sources.finish(),
        actions.finish(),
        fixed.finish(),
        bases.finish(),
        fitness.finish(),
        optimum.finish(),
        evaluations.finish(),
        provenance.finish(),
        bounds.finish(),
    ])
}
