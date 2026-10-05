//! Frozen testimony-v1: rational references are independent of Belief.
//! Accuracy 4/5, uniform truth, persistent copy/invert priors 3/4,1/4;
//! decision costs +1/-1/0 and absolute tolerance 1e-12.
use super::*;
use serde::Serialize;
const TOLERANCE: f64 = 1e-12;
#[derive(Clone, Debug, Serialize)]
pub struct TestimonyReport {
    pub version: String,
    pub tolerance: f64,
    pub fixtures: Vec<TestimonyFixtureResult>,
    pub decisions: Vec<TestimonyDecisionResult>,
    pub passed: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct TestimonyFixtureResult {
    pub name: String,
    pub model: TestimonyModel,
    pub records: Vec<EvidenceRecord>,
    pub references: Vec<TestimonyReferenceCheck>,
    pub checks: Vec<TestimonyBooleanCheck>,
    pub passed: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct TestimonyReferenceCheck {
    pub quantity: String,
    pub expected_exact: String,
    pub expected: f64,
    pub actual: f64,
    pub absolute_error: f64,
    pub passed: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct TestimonyBooleanCheck {
    pub quantity: String,
    pub expected: bool,
    pub actual: bool,
    pub passed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestimonyDecision {
    Accuse,
    Abstain,
}
#[derive(Clone, Debug, Serialize)]
pub struct TestimonyDecisionResult {
    pub fixture: String,
    pub posterior_true: f64,
    pub chosen_action: TestimonyDecision,
    pub chosen_utility: f64,
    pub best_reference_utility_exact: String,
    pub best_reference_utility: f64,
    pub regret: f64,
    pub credulous_utility: f64,
    pub credulous_regret: f64,
    pub passed: bool,
}
fn reference(quantity: &str, exact: &str, expected: f64, actual: f64) -> TestimonyReferenceCheck {
    let absolute_error = (expected - actual).abs();
    TestimonyReferenceCheck {
        quantity: quantity.into(),
        expected_exact: exact.into(),
        expected,
        actual,
        absolute_error,
        passed: absolute_error <= TOLERANCE,
    }
}
fn check(quantity: &str, actual: bool) -> TestimonyBooleanCheck {
    TestimonyBooleanCheck {
        quantity: quantity.into(),
        expected: true,
        actual,
        passed: actual,
    }
}
fn model(
    uncertain: bool,
    channel: [f64; 2],
    accuracy: f64,
    witnesses: bool,
    transfer: bool,
) -> TestimonyModel {
    let propositions: Vec<_> = (0..if transfer { 2 } else { 1 })
        .map(|id| Proposition { id, label: None })
        .collect();
    let speakers = if witnesses { vec![0, 1] } else { vec![0] };
    let profiles = if uncertain {
        vec![
            ReportingProfile {
                id: 0,
                positive_given_signal: [0.0, 1.0],
            },
            ReportingProfile {
                id: 1,
                positive_given_signal: [1.0, 0.0],
            },
        ]
    } else {
        vec![ReportingProfile {
            id: 0,
            positive_given_signal: channel,
        }]
    };
    let mut hypotheses = Vec::new();
    for bits in 0..(1 << propositions.len()) {
        for profile in 0..profiles.len() {
            hypotheses.push(TestimonyHypothesis {
                id: hypotheses.len() as u16,
                prior: if uncertain {
                    [0.75, 0.25][profile]
                } else {
                    1.0
                } / (1 << propositions.len()) as f64,
                propositions: propositions
                    .iter()
                    .map(|p| PropositionValue {
                        proposition: p.id,
                        value: bits & (1 << p.id) != 0,
                    })
                    .collect(),
                profiles: speakers
                    .iter()
                    .map(|&speaker| SpeakerProfile {
                        speaker,
                        profile: profile as u16,
                    })
                    .collect(),
            });
        }
    }
    let groups = (0..3)
        .map(|id| SignalGroup {
            id,
            proposition: if transfer && id == 2 { 1 } else { 0 },
            accuracy,
        })
        .collect();
    TestimonyModel {
        propositions,
        speakers,
        profiles,
        hypotheses,
        groups,
    }
}
fn report(id: u64, group: u16, speaker: u16, positive: bool) -> EvidenceRecord {
    EvidenceRecord {
        id,
        content: TestimonyEvidence::Report {
            group,
            speaker,
            positive,
        },
    }
}
fn fixture(
    name: &str,
    model: TestimonyModel,
    records: Vec<EvidenceRecord>,
    expected: &[(usize, &str, f64)],
    profile: Option<(&str, f64)>,
    special: bool,
) -> Result<TestimonyFixtureResult, TestimonyError> {
    let mut belief = Belief::new(model.clone())?;
    let mut checks = Vec::new();
    for (i, record) in records.iter().enumerate() {
        let before = belief.clone();
        let result = belief.observe(record.clone());
        if special && i == records.len() - 1 {
            if name == "model_contradiction" {
                checks.push(check(
                    "zero_evidence",
                    result == Err(TestimonyError::ZeroEvidence),
                ));
            } else {
                checks.push(check("duplicate_success", result.is_ok()));
            }
            checks.push(check("no_mutation", belief == before));
        } else {
            result?;
        }
    }
    let snapshot = belief.snapshot();
    let mut references: Vec<_> = expected
        .iter()
        .map(|&(index, exact, value)| {
            reference(
                &format!("proposition_{index}_true"),
                exact,
                value,
                snapshot.propositions[index].probability_true,
            )
        })
        .collect();
    if let Some((exact, value)) = profile {
        references.push(reference(
            "speaker_0_copy",
            exact,
            value,
            snapshot.speakers[0].profiles[0].probability,
        ));
    }
    if name == "empty_evidence" {
        checks.push(check(
            "joint_equals_prior",
            snapshot
                .hypotheses
                .iter()
                .zip(&model.hypotheses)
                .all(|(p, h)| (p.probability - h.prior).abs() <= TOLERANCE),
        ));
    }
    let passed = fixture_passed(&references, &checks);
    Ok(TestimonyFixtureResult {
        name: name.into(),
        model,
        records,
        references,
        checks,
        passed,
    })
}
/// Enumerate the frozen fixture matrix without consulting any game oracle.
pub fn diagnose_testimony() -> Result<TestimonyReport, TestimonyError> {
    let uncertain = || model(true, [0.0, 1.0], 0.8, false, false);
    let known = |channel, q, witnesses| model(false, channel, q, witnesses, false);
    let positive = report(1, 0, 0, true);
    let verified = EvidenceRecord {
        id: 2,
        content: TestimonyEvidence::Verified {
            proposition: 0,
            value: true,
        },
    };
    let fixtures = vec![
        fixture(
            "empty_evidence",
            uncertain(),
            vec![],
            &[(0, "1/2", 0.5)],
            Some(("3/4", 0.75)),
            false,
        )?,
        fixture(
            "uncertain_speaker",
            uncertain(),
            vec![positive.clone()],
            &[(0, "13/20", 13.0 / 20.0)],
            Some(("3/4", 0.75)),
            false,
        )?,
        fixture(
            "two_signals",
            uncertain(),
            vec![positive.clone(), report(2, 1, 0, true)],
            &[(0, "49/68", 49.0 / 68.0)],
            None,
            false,
        )?,
        fixture(
            "shared_signal",
            uncertain(),
            vec![positive.clone(), report(2, 0, 0, true)],
            &[(0, "13/20", 13.0 / 20.0)],
            None,
            false,
        )?,
        fixture(
            "duplicate_record",
            uncertain(),
            vec![positive.clone(), positive.clone()],
            &[],
            None,
            true,
        )?,
        fixture(
            "independent_witnesses",
            known([0.0, 1.0], 0.8, true),
            vec![positive.clone(), report(2, 1, 1, true)],
            &[(0, "16/17", 16.0 / 17.0)],
            None,
            false,
        )?,
        fixture(
            "common_signal_witnesses",
            known([0.0, 1.0], 0.8, true),
            vec![positive.clone(), report(2, 0, 1, true)],
            &[(0, "4/5", 4.0 / 5.0)],
            None,
            false,
        )?,
        fixture(
            "inversion",
            known([1.0, 0.0], 0.8, false),
            vec![positive.clone()],
            &[(0, "1/5", 1.0 / 5.0)],
            None,
            false,
        )?,
        fixture(
            "always_positive",
            known([1.0, 1.0], 0.8, false),
            vec![positive.clone()],
            &[(0, "1/2", 0.5)],
            None,
            false,
        )?,
        fixture(
            "half_accuracy",
            known([0.0, 1.0], 0.5, false),
            vec![positive.clone()],
            &[(0, "1/2", 0.5)],
            None,
            false,
        )?,
        fixture(
            "verified_truth",
            model(true, [0.0, 1.0], 0.8, false, true),
            vec![positive.clone(), verified.clone()],
            &[(0, "1", 1.0), (1, "1/2", 0.5)],
            Some(("12/13", 12.0 / 13.0)),
            false,
        )?,
        fixture(
            "transfer",
            model(true, [0.0, 1.0], 0.8, false, true),
            vec![positive.clone(), verified, report(3, 2, 0, true)],
            &[(1, "49/65", 49.0 / 65.0)],
            None,
            false,
        )?,
        fixture(
            "model_contradiction",
            known([0.0, 1.0], 1.0, false),
            vec![positive, report(2, 0, 0, false)],
            &[],
            None,
            true,
        )?,
    ];
    let mut decisions = Vec::new();
    for (name, action, best_exact, best, credulous_expected, credulous_regret_expected) in [
        (
            "uncertain_speaker",
            TestimonyDecision::Accuse,
            "3/10",
            0.3,
            0.3,
            0.0,
        ),
        ("inversion", TestimonyDecision::Abstain, "0", 0.0, -0.6, 0.6),
        (
            "empty_evidence",
            TestimonyDecision::Abstain,
            "0",
            0.0,
            0.0,
            0.0,
        ),
    ] {
        let posterior_true = fixtures
            .iter()
            .find(|f| f.name == name)
            .expect("frozen fixture")
            .references[0]
            .actual;
        let chosen_action = if best_accusation(
            &[posterior_true, 1.0 - posterior_true],
            1.0,
            -1.0,
            0.0,
        ) == Some(0)
        {
            TestimonyDecision::Accuse
        } else {
            TestimonyDecision::Abstain
        };
        let credulous_utility = 2.0 * posterior_true - 1.0;
        let chosen_utility = if chosen_action == TestimonyDecision::Accuse {
            credulous_utility
        } else {
            0.0
        };
        let regret = best - chosen_utility;
        let credulous_regret = best - credulous_utility;
        let passed = chosen_action == action
            && regret.abs() <= TOLERANCE
            && (credulous_utility - credulous_expected).abs() <= TOLERANCE
            && (credulous_regret - credulous_regret_expected).abs() <= TOLERANCE;
        decisions.push(TestimonyDecisionResult {
            fixture: name.into(),
            posterior_true,
            chosen_action,
            chosen_utility,
            best_reference_utility_exact: best_exact.into(),
            best_reference_utility: best,
            regret,
            credulous_utility,
            credulous_regret,
            passed,
        });
    }
    let passed = report_passed(&fixtures, &decisions);
    Ok(TestimonyReport {
        version: "testimony-v1".into(),
        tolerance: TOLERANCE,
        fixtures,
        decisions,
        passed,
    })
}

fn fixture_passed(
    references: &[TestimonyReferenceCheck],
    checks: &[TestimonyBooleanCheck],
) -> bool {
    references.iter().all(|r| r.passed) && checks.iter().all(|c| c.passed)
}
fn report_passed(
    fixtures: &[TestimonyFixtureResult],
    decisions: &[TestimonyDecisionResult],
) -> bool {
    fixtures.iter().all(|f| f.passed) && decisions.iter().all(|d| d.passed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn testimony_comparison_failures_close_fixture_and_report_gates() {
        let wrong_numeric = reference("truth", "1/2", 0.5, 0.6);
        assert!(!wrong_numeric.passed);
        let wrong_boolean = check("no_mutation", false);
        assert!(!wrong_boolean.passed);
        assert!(!fixture_passed(&[wrong_numeric], &[]));
        assert!(!fixture_passed(&[], &[wrong_boolean]));
        let mut report = diagnose_testimony().unwrap();
        report.fixtures[0].references[0] = reference("truth", "1/2", 0.5, 0.6);
        report.fixtures[0].passed =
            fixture_passed(&report.fixtures[0].references, &report.fixtures[0].checks);
        assert!(!report_passed(&report.fixtures, &report.decisions));
        let mut report = diagnose_testimony().unwrap();
        report.fixtures[0]
            .checks
            .push(check("joint_equals_prior", false));
        report.fixtures[0].passed =
            fixture_passed(&report.fixtures[0].references, &report.fixtures[0].checks);
        assert!(!report_passed(&report.fixtures, &report.decisions));
        let mut report = diagnose_testimony().unwrap();
        report.decisions[0].passed = false;
        assert!(!report_passed(&report.fixtures, &report.decisions));
    }
}
