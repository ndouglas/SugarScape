//! Independent full-joint Fraction reference checks, reduced only at the comparison boundary.
use super::*;
use crate::deduction::strategic_reporting::DecisionAction;

type Fraction = (i64, u64);
type InferenceHistory = (Option<Fraction>, Option<String>, Option<Vec<Fraction>>);
type Calibration = (Option<Vec<Fraction>>, Option<Vec<[Fraction; 2]>>);
type EvaluationHistory = (
    Fraction,
    Fraction,
    Option<Fraction>,
    Option<Fraction>,
    Option<Fraction>,
    Option<String>,
    bool,
    Fraction,
    Fraction,
);
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    oracle_source_sha256: String,
    reference_sha256: String,
    history_order: Vec<String>,
    inference: Vec<InferenceCase>,
    evaluations: Vec<EvaluationCase>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InferenceCase {
    id: String,
    accuracy: [u16; 2],
    fixed_copy_prior: [u16; 2],
    entries: Vec<(u32, u8)>,
    rows: Vec<InferenceHistory>,
    calibrations: Vec<Calibration>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvaluationCase {
    listener: String,
    accuracy: [u16; 2],
    fixed_copy_prior: [u16; 2],
    actual: Vec<(u32, u8)>,
    fixed: bool,
    rows: Vec<EvaluationHistory>,
    aggregates: Aggregates,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Aggregates {
    supported_probability: Fraction,
    unsupported_probability: Fraction,
    supported_payoff: Fraction,
    supported_regret: Fraction,
    supported_reporter_utility: Fraction,
    payoff: Option<Fraction>,
    decision_regret: Option<Fraction>,
    reporter_utility: Option<Fraction>,
    maximum_belief_error: Option<Fraction>,
    reference_payoff: Fraction,
}
fn scalar(numerator: i64, denominator: u64) -> Option<ExactValue> {
    Some(ExactValue {
        numerator,
        denominator,
    })
}
fn expected(value: Option<Fraction>) -> Option<ExactValue> {
    value.map(|(numerator, denominator)| ExactValue {
        numerator,
        denominator,
    })
}
fn ratio(value: Option<&super::super::Ratio>) -> Option<ExactValue> {
    value.and_then(|r| scalar(i64::try_from(r.numerator).ok()?, r.denominator))
}
fn signed(value: Option<&super::super::SignedRatio>) -> Option<ExactValue> {
    value.map(|r| ExactValue {
        numerator: r.numerator,
        denominator: r.denominator,
    })
}
fn action(value: Option<DecisionAction>) -> Option<ExactValue> {
    value.and_then(|action| scalar(i64::from(action == DecisionAction::Intervene), 1))
}
fn expected_action(value: &Option<String>) -> Result<Option<ExactValue>, Error> {
    match value.as_deref() {
        None => Ok(None),
        Some("intervene") => Ok(scalar(1, 1)),
        Some("abstain") => Ok(scalar(0, 1)),
        _ => Err(Error::InvalidObservation("invalid reference action")),
    }
}
fn add(
    checks: &mut Vec<DiagnosticCheck>,
    name: String,
    actual: Option<ExactValue>,
    expected: Option<ExactValue>,
) {
    let passed = match (&actual, &expected) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.denominator > 0
                && b.denominator > 0
                && i128::from(a.numerator) * i128::from(b.denominator)
                    == i128::from(b.numerator) * i128::from(a.denominator)
        }
        _ => false,
    };
    checks.push(DiagnosticCheck {
        name,
        expected,
        actual,
        passed,
    });
}
fn require(condition: bool) -> Result<(), Error> {
    if condition {
        Ok(())
    } else {
        Err(Error::InvalidObservation(
            "independent reference identity or shape mismatch",
        ))
    }
}
pub(super) fn checks(report: &DiagnosticReport) -> Result<Vec<DiagnosticCheck>, Error> {
    let reference: Reference =
        serde_json::from_str(include_str!("../diagnostic-reference.json"))
            .map_err(|_| Error::InvalidObservation("invalid embedded independent reference"))?;
    require(
        reference.oracle_source_sha256 == report.metadata.oracle_source_sha256
            && reference.reference_sha256 == report.metadata.reference_sha256,
    )?;
    require(
        reference.history_order
            == [
                "verified_C",
                "strategic_calibration_report",
                "fixed_calibration_report",
                "strategic_live_report",
                "fixed_live_report",
            ],
    )?;
    require(reference.inference.len() == 4 && reference.evaluations.len() == 108)?;
    let mut checks = Vec::new();
    for model in &report.inference_models {
        let name = format!("{}_{}", model.environment_id, model.catalog_id);
        let fixture = reference
            .inference
            .iter()
            .find(|case| case.id == name)
            .ok_or(Error::InvalidObservation("missing inference reference"))?;
        let catalog = report
            .catalogs
            .iter()
            .find(|c| c.id == model.catalog_id)
            .ok_or(Error::InvalidObservation("missing reference catalog"))?;
        let rules = &model.histories[0].observation.rules;
        require(
            fixture.accuracy == [rules.accuracy.numerator, rules.accuracy.denominator]
                && fixture.fixed_copy_prior
                    == [
                        rules.fixed_copy_prior.numerator,
                        rules.fixed_copy_prior.denominator,
                    ],
        )?;
        require(
            fixture.entries
                == catalog
                    .catalog
                    .entries()
                    .iter()
                    .map(|entry| (entry.policy.bits, entry.weight))
                    .collect::<Vec<_>>(),
        )?;
        require(fixture.rows.len() == 32 && fixture.calibrations.len() == 8)?;
        for (index, (row, expected_row)) in model.histories.iter().zip(&fixture.rows).enumerate() {
            let prefix = format!("inference/{name}/history/{index}");
            add(
                &mut checks,
                format!("{prefix}/posterior_true"),
                ratio(Some(&row.decision.posterior_true)),
                expected(expected_row.0),
            );
            add(
                &mut checks,
                format!("{prefix}/action"),
                action(Some(row.decision.action)),
                expected_action(&expected_row.1)?,
            );
            let policies = expected_row.2.as_ref().ok_or(Error::InvalidObservation(
                "unsupported default reference history",
            ))?;
            require(policies.len() == row.decision.policies.len())?;
            for (p, (actual, exp)) in row.decision.policies.iter().zip(policies).enumerate() {
                add(
                    &mut checks,
                    format!("{prefix}/policy/{p}"),
                    ratio(Some(&actual.probability)),
                    expected(Some(*exp)),
                );
            }
        }
        for (index, (row, (policies, live))) in model
            .calibrations
            .iter()
            .zip(&fixture.calibrations)
            .enumerate()
        {
            let prefix = format!("inference/{name}/calibration/{index}");
            let policies = policies.as_ref().ok_or(Error::InvalidObservation(
                "unsupported default reference calibration",
            ))?;
            let live = live
                .as_ref()
                .ok_or(Error::InvalidObservation("missing reference predictions"))?;
            require(
                policies.len() == row.belief.policies.len() && live.len() == row.belief.live.len(),
            )?;
            for (p, (actual, exp)) in row.belief.policies.iter().zip(policies).enumerate() {
                add(
                    &mut checks,
                    format!("{prefix}/policy/{p}"),
                    ratio(Some(&actual.probability)),
                    expected(Some(*exp)),
                );
            }
            for (p, (actual, exp)) in row.belief.live.iter().zip(live).enumerate() {
                add(
                    &mut checks,
                    format!("{prefix}/live/{p}/mass"),
                    ratio(Some(&actual.probability)),
                    expected(Some(exp[0])),
                );
                add(
                    &mut checks,
                    format!("{prefix}/live/{p}/true_mass"),
                    ratio(Some(&actual.truth_and_reports_probability)),
                    expected(Some(exp[1])),
                );
            }
        }
    }
    for fixture in &reference.evaluations {
        let env = format!("q{}_{}", fixture.accuracy[0], fixture.accuracy[1]);
        require(fixture.fixed_copy_prior == [3, 4])?;
        let (id, evaluation) = if fixture.fixed {
            require(fixture.actual.len() == 1 && fixture.actual[0].1 == 1)?;
            let bits = canonical_bits(&Policy::new(fixture.actual[0].0)?)?;
            let row = report
                .fixed_evaluations
                .iter()
                .find(|row| {
                    row.environment_id == env
                        && row.canonical_bits == bits
                        && row.listener.id == fixture.listener
                })
                .ok_or(Error::InvalidObservation("missing fixed reference row"))?;
            (
                format!("fixed/{env}/{}/{}", row.policy_id, fixture.listener),
                &row.evaluation,
            )
        } else {
            let actual = Catalog::new(
                fixture
                    .actual
                    .iter()
                    .map(|&(bits, weight)| {
                        Ok(super::super::WeightedPolicy {
                            policy: Policy::new(bits)?,
                            weight,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            )?;
            let catalog = report
                .catalogs
                .iter()
                .find(|row| row.catalog == actual)
                .ok_or(Error::InvalidObservation(
                    "missing actual mixture reference",
                ))?;
            let row = report
                .mixture_evaluations
                .iter()
                .find(|row| {
                    row.environment_id == env
                        && row.catalog_id == catalog.id
                        && row.listener.id == fixture.listener
                })
                .ok_or(Error::InvalidObservation("missing mixture reference row"))?;
            (
                format!("mixture/{env}/{}/{}", catalog.id, fixture.listener),
                &row.evaluation,
            )
        };
        evaluation_checks(&mut checks, &id, evaluation, fixture)?;
    }
    for (name, actual, exp) in [
        (
            "protocol/inference_models",
            report.inference_models.len(),
            4,
        ),
        (
            "protocol/mixture_rows",
            report.mixture_evaluations.len(),
            24,
        ),
        ("protocol/fixed_rows", report.fixed_evaluations.len(), 84),
        ("protocol/provenance", report.policy_provenance.len(), 46),
    ] {
        add(
            &mut checks,
            name.into(),
            scalar(actual as i64, 1),
            scalar(exp, 1),
        );
    }
    for entry in &report.policy_provenance {
        let canonical = canonical_bits(&Policy::new(entry.raw_bits)?)?;
        add(
            &mut checks,
            format!("provenance/{}/canonical", entry.id),
            scalar(i64::from(entry.canonical_bits), 1),
            scalar(i64::from(canonical), 1),
        );
    }
    for catalog in &report.catalogs {
        add(
            &mut checks,
            format!("protocol/{}/withheld_absent", catalog.id),
            scalar(
                i64::from(
                    catalog
                        .catalog
                        .entries()
                        .iter()
                        .all(|entry| entry.policy.bits != 98342),
                ),
                1,
            ),
            scalar(1, 1),
        );
    }
    Ok(checks)
}
fn evaluation_checks(
    checks: &mut Vec<DiagnosticCheck>,
    id: &str,
    evaluation: &Evaluation,
    fixture: &EvaluationCase,
) -> Result<(), Error> {
    require(fixture.rows.len() == 32 && evaluation.histories.len() == 32)?;
    let a = &fixture.aggregates;
    let d = evaluation.denominator;
    for (name, n, exp) in [
        (
            "supported_probability",
            evaluation.supported_mass as i64,
            a.supported_probability,
        ),
        (
            "unsupported_probability",
            evaluation.unsupported_mass as i64,
            a.unsupported_probability,
        ),
        (
            "supported_payoff",
            evaluation.supported_payoff_numerator,
            a.supported_payoff,
        ),
        (
            "supported_regret",
            evaluation.supported_regret_numerator,
            a.supported_regret,
        ),
        (
            "supported_reporter_utility",
            evaluation.supported_reporter_utility_numerator,
            a.supported_reporter_utility,
        ),
    ] {
        add(
            checks,
            format!("{id}/{name}"),
            scalar(n, d),
            expected(Some(exp)),
        );
    }
    for (name, actual, exp) in [
        ("payoff", signed(evaluation.payoff.as_ref()), a.payoff),
        (
            "decision_regret",
            ratio(evaluation.decision_regret.as_ref()),
            a.decision_regret,
        ),
        (
            "reporter_utility",
            signed(evaluation.reporter_utility.as_ref()),
            a.reporter_utility,
        ),
        (
            "maximum_belief_error",
            ratio(evaluation.maximum_belief_error.as_ref()),
            a.maximum_belief_error,
        ),
    ] {
        add(checks, format!("{id}/{name}"), actual, expected(exp));
    }
    let mut reference_payoff = 0;
    for (index, (row, exp)) in evaluation.histories.iter().zip(&fixture.rows).enumerate() {
        let prefix = format!("{id}/history/{index}");
        add(
            checks,
            format!("{prefix}/mass"),
            scalar(row.actual_mass as i64, d),
            expected(Some(exp.0)),
        );
        add(
            checks,
            format!("{prefix}/true_mass"),
            scalar(row.actual_true_mass as i64, d),
            expected(Some(exp.1)),
        );
        add(
            checks,
            format!("{prefix}/actual_posterior"),
            ratio(row.actual_posterior.as_ref()),
            expected(exp.2),
        );
        add(
            checks,
            format!("{prefix}/listener_posterior"),
            ratio(row.listener_posterior.as_ref()),
            expected(exp.3),
        );
        add(
            checks,
            format!("{prefix}/belief_error"),
            signed(row.belief_error.as_ref()),
            expected(exp.4),
        );
        add(
            checks,
            format!("{prefix}/action"),
            action(row.action),
            expected_action(&exp.5)?,
        );
        add(
            checks,
            format!("{prefix}/unsupported"),
            scalar(i64::from(row.unsupported), 1),
            scalar(i64::from(exp.6), 1),
        );
        let gain = 2 * row.actual_true_mass as i64 - row.actual_mass as i64;
        let payoff = if row.action == Some(DecisionAction::Intervene) {
            gain
        } else {
            0
        };
        let regret = if row.unsupported {
            0
        } else {
            gain.max(0) - payoff
        };
        add(
            checks,
            format!("{prefix}/payoff"),
            scalar(payoff, d),
            expected(Some(exp.7)),
        );
        add(
            checks,
            format!("{prefix}/regret"),
            scalar(regret, d),
            expected(Some(exp.8)),
        );
        reference_payoff += gain.max(0);
    }
    add(
        checks,
        format!("{id}/reference_payoff"),
        scalar(reference_payoff, d),
        expected(Some(a.reference_payoff)),
    );
    Ok(())
}
