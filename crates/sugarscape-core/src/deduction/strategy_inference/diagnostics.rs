//! Search-free frozen protocol. Payload integrity is checked against a fresh reconstruction.
use super::{
    canonical_bits, evaluate_fixed, evaluate_mixture, CalibrationBelief, CalibrationView, Catalog,
    Error, EvaluatedListener, Evaluation, InferenceDecision, Model, REPORT_VERSION,
};
use crate::deduction::strategic_reporting::{
    Config, DecisionObservation, FrozenListener, Genome, Listener, Policy, Probability,
    UtilityTable,
};
use serde::{Deserialize, Serialize};

mod references;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub passed: bool,
    pub metadata: Metadata,
    pub catalogs: Vec<CatalogRow>,
    pub environments: Vec<EnvironmentRow>,
    pub policy_provenance: Vec<PolicyProvenance>,
    pub inference_models: Vec<InferenceModelRow>,
    pub mixture_evaluations: Vec<MixtureRow>,
    pub fixed_evaluations: Vec<FixedRow>,
    pub checks: Vec<DiagnosticCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub utility: UtilityTable,
    pub prior_motivation: String,
    pub former_panel_target: String,
    pub inference_rule: String,
    pub support_rule: String,
    pub tie_rule: String,
    pub optimization: bool,
    pub policy_selection: String,
    pub belief_metric: String,
    pub oracle_source_sha256: String,
    pub reference_sha256: String,
    pub fixture_sha256: String,
    pub champion_source_commit: String,
    pub champion_source_report_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRow {
    pub id: String,
    pub catalog: Catalog,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentRow {
    pub id: String,
    pub rules: Config,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyProvenance {
    pub id: String,
    pub method: String,
    pub seed: Option<u64>,
    pub raw_bits: u32,
    pub canonical_bits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListenerIdentity {
    pub id: String,
    pub catalog_id: Option<String>,
    pub legacy: Option<FrozenListener>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationRow {
    pub view: CalibrationView,
    pub belief: CalibrationBelief,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceHistoryRow {
    pub observation: DecisionObservation,
    pub decision: InferenceDecision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InferenceModelRow {
    pub environment_id: String,
    pub catalog_id: String,
    pub listener: ListenerIdentity,
    pub calibrations: Vec<CalibrationRow>,
    pub histories: Vec<InferenceHistoryRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixtureRow {
    pub environment_id: String,
    pub catalog_id: String,
    pub listener: ListenerIdentity,
    pub evaluation: Evaluation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedRow {
    pub environment_id: String,
    pub policy_id: String,
    pub canonical_bits: u32,
    pub listener: ListenerIdentity,
    pub evaluation: Evaluation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCheck {
    pub name: String,
    pub expected: Option<ExactValue>,
    pub actual: Option<ExactValue>,
    pub passed: bool,
}

/// Exact scalar checks may include counts and scores as well as probabilities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExactValue {
    pub numerator: i64,
    pub denominator: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactValueWire {
    numerator: i64,
    denominator: u64,
}
impl<'de> Deserialize<'de> for ExactValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ExactValueWire::deserialize(deserializer)?;
        if wire.denominator == 0 {
            return Err(serde::de::Error::custom(
                "exact check denominator must be positive",
            ));
        }
        Ok(Self {
            numerator: wire.numerator,
            denominator: wire.denominator,
        })
    }
}
const RANDOM_CHAMPIONS: [u32; 20] = [
    88214, 86550, 86358, 85142, 91158, 82838, 86486, 84246, 82198, 82902, 89494, 83542, 83542,
    85782, 88342, 94934, 83862, 82070, 82198, 83542,
];
fn policies() -> [(&'static str, Policy); 6] {
    [
        ("copy", Policy::copy()),
        ("invert", Policy::invert()),
        ("always_positive", Policy::positive()),
        ("always_negative", Policy::negative()),
        (
            "copy_calibration_invert_live",
            Policy::calibration_copy_live_invert(),
        ),
        (
            "evolved_target_optimum",
            Policy::new(98342).expect("frozen encoding"),
        ),
    ]
}
fn listener_identity(id: &str) -> ListenerIdentity {
    let catalog_id = id.strip_prefix("strategy_").map(str::to_owned);
    let legacy = match id {
        "bayesian" => Some(Listener::Bayesian),
        "credulous" => Some(Listener::Credulous),
        "skeptical" => Some(Listener::Skeptical),
        "evolved" => Some(Listener::Evolved(Genome {
            b: -3,
            u: 0,
            d: 2,
            k: 0,
        })),
        "passive" => Some(Listener::Passive),
        _ => None,
    }
    .map(|algorithm| FrozenListener {
        algorithm,
        assumed_copy_prior: Probability {
            numerator: 3,
            denominator: 4,
        },
    });
    ListenerIdentity {
        id: id.into(),
        catalog_id,
        legacy,
    }
}
fn evaluated_listener(
    rules: &Config,
    identity: &ListenerIdentity,
    catalogs: &[CatalogRow],
) -> Result<EvaluatedListener, Error> {
    if let Some(id) = &identity.catalog_id {
        let catalog = catalogs
            .iter()
            .find(|row| &row.id == id)
            .ok_or(Error::InvalidObservation("unknown frozen catalog"))?;
        Ok(EvaluatedListener::Strategy(Model::new(
            rules,
            &catalog.catalog,
        )?))
    } else {
        Ok(EvaluatedListener::Legacy(identity.legacy.clone().ok_or(
            Error::InvalidObservation("unknown frozen listener"),
        )?))
    }
}
/// Reconstruct every frozen setting, row and independent reference comparison.
pub fn diagnose() -> Result<DiagnosticReport, Error> {
    let catalogs = vec![
        CatalogRow {
            id: "uniform".into(),
            catalog: Catalog::uniform(),
        },
        CatalogRow {
            id: "optimization_informed".into(),
            catalog: Catalog::optimization_informed(),
        },
    ];
    let environments = [4, 3]
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
        .collect::<Vec<_>>();
    let mut policy_provenance = Vec::new();
    for (id, policy) in policies() {
        policy_provenance.push(PolicyProvenance {
            id: id.into(),
            method: if policy.bits == 98342 {
                "withheld"
            } else {
                "control"
            }
            .into(),
            seed: None,
            raw_bits: policy.bits,
            canonical_bits: canonical_bits(&policy)?,
        });
    }
    for (method, encodings) in [("genetic", [81942; 20]), ("random", RANDOM_CHAMPIONS)] {
        for (seed, bits) in encodings.into_iter().enumerate() {
            policy_provenance.push(PolicyProvenance {
                id: format!("{method}_{seed}"),
                method: method.into(),
                seed: Some(seed as u64),
                raw_bits: bits,
                canonical_bits: canonical_bits(&Policy::new(bits)?)?,
            });
        }
    }
    let mut report = DiagnosticReport {
        version:REPORT_VERSION.into(),passed:false,
        metadata: Metadata {
            utility:UtilityTable::opposed(),
            prior_motivation:"Uniform named-control assumptions versus supplied optimization-informed weights [1,1,1,1,16] in named-control order; weights are not learned objective rationality.".into(),
            former_panel_target:"Previous q=4/5, rho=3/4 opposed-utility optimization targeted equally weighted frozen Bayesian and Credulous listeners; champions are behavioral clones, not independent challenges.".into(),
            inference_rule:"Exact joint marginalization over a once-selected policy, truths, fixed profile and signal noise; calibration uses no live evidence; complete history conditions on public evidence once.".into(),
            support_rule:"Positive actual mass without prior evidence is unsupported: no action or unconditional metrics. Zero actual mass has no action and is not unsupported.".into(),
            tie_rule:"Intervene only for posterior T > 1/2; exact ties abstain.".into(),
            optimization:false,
            policy_selection:"Policy selected once independently of truths, profile and noise and retained across both phases; no RNG, search or reporter optimization is run.".into(),
            belief_metric:"Signed inferred T posterior minus actual-distribution T posterior; maximum absolute error over supported positive-mass histories. Legacy listeners provide actions only.".into(),
            oracle_source_sha256:"087096ca6ab0e286a0782a5f58484b2fa4fb3bb0d73ac65b414b74c787f57024".into(),
            reference_sha256:"54bcf0b50e1a0fc8a5d5f326a3c914499296170ad2690fa8eb8b79f03acac7b0".into(),
            fixture_sha256:"b4f89c97aa9b5c6cdb5d68399fac66d3621c4552e533b04cad6d8a5ef7e74d4a".into(),
            champion_source_commit:"eaf59cab3db3a44905c65e5c6ca8babf6a3994de".into(),
            champion_source_report_sha256:"d8668ef024023d3ba99770b0129e418cdad5dedcf1f1415d92e14882fa9e4937".into(),
        },catalogs,environments,policy_provenance,inference_models:vec![],mixture_evaluations:vec![],fixed_evaluations:vec![],checks:vec![],
    };
    for env in &report.environments {
        for prior in &report.catalogs {
            let id = listener_identity(&format!("strategy_{}", prior.id));
            let model = Model::new(&env.rules, &prior.catalog)?;
            let mut row = InferenceModelRow {
                environment_id: env.id.clone(),
                catalog_id: prior.id.clone(),
                listener: id.clone(),
                calibrations: vec![],
                histories: vec![],
            };
            for index in 0..8 {
                let view = CalibrationView {
                    rules: env.rules.clone(),
                    calibration_truth: index & 4 != 0,
                    calibration_reports: [index & 2 != 0, index & 1 != 0],
                };
                let belief = model.calibration(&view)?;
                row.calibrations.push(CalibrationRow { view, belief });
            }
            for index in 0..32 {
                let observation = DecisionObservation {
                    rules: env.rules.clone(),
                    calibration_truth: index & 16 != 0,
                    calibration_reports: [index & 8 != 0, index & 4 != 0],
                    live_reports: [index & 2 != 0, index & 1 != 0],
                };
                let decision = model.decide(&observation)?;
                row.histories.push(InferenceHistoryRow {
                    observation,
                    decision,
                });
            }
            report.inference_models.push(row);
            for listener in [
                id,
                listener_identity("bayesian"),
                listener_identity("credulous"),
                listener_identity("skeptical"),
                listener_identity("evolved"),
                listener_identity("passive"),
            ] {
                let evaluation = evaluate_mixture(
                    &env.rules,
                    &prior.catalog,
                    &evaluated_listener(&env.rules, &listener, &report.catalogs)?,
                )?;
                report.mixture_evaluations.push(MixtureRow {
                    environment_id: env.id.clone(),
                    catalog_id: prior.id.clone(),
                    listener,
                    evaluation,
                });
            }
        }
        for (policy_id, policy) in policies() {
            let bits = canonical_bits(&policy)?;
            for id in [
                "strategy_uniform",
                "strategy_optimization_informed",
                "bayesian",
                "credulous",
                "skeptical",
                "evolved",
                "passive",
            ] {
                let listener = listener_identity(id);
                let evaluation = evaluate_fixed(
                    &env.rules,
                    &Policy::new(bits)?,
                    &evaluated_listener(&env.rules, &listener, &report.catalogs)?,
                )?;
                report.fixed_evaluations.push(FixedRow {
                    environment_id: env.id.clone(),
                    policy_id: policy_id.into(),
                    canonical_bits: bits,
                    listener,
                    evaluation,
                });
            }
        }
    }
    report.checks = references::checks(&report)?;
    report.passed = report.checks.iter().all(|check| check.passed);
    Ok(report)
}
/// Validate the entire current payload against fresh frozen-protocol calculations.
/// Every reference check and setting is reconstructed; stored flags never establish success.
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error> {
    let expected = diagnose()?;
    Ok(expected.passed && report == &expected)
}
