//! Frozen deduction-v1 experiment; never tune after first collection.
use super::*;
use crate::rng::{seeded, SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};
pub const EXPERIMENT_VERSION: &str = "deduction-v1";
pub const POLICY_VERSION: u16 = 1;
pub fn policy_seed(world: u64, kind: PolicyKind, agent: AgentId) -> u64 {
    controller_seed(
        world,
        match kind {
            PolicyKind::Evidence => 1,
            PolicyKind::Random => 2,
            PolicyKind::Reckless => 3,
            PolicyKind::Passive => 4,
        },
        agent,
    )
}
fn controller_seed(world: u64, identity: u64, agent: AgentId) -> u64 {
    world
        .wrapping_mul(6364136223846793005)
        .wrapping_add(identity.wrapping_mul(1442695040888963407))
        .wrapping_add(u64::from(agent))
}
#[derive(Clone, Debug)]
pub struct ExperimentThreatController {
    rng: SimRng,
}
impl ExperimentThreatController {
    pub fn new(world: u64, agent: AgentId) -> Self {
        Self {
            rng: seeded(controller_seed(world, 5, agent)),
        }
    }
}
impl Controller for ExperimentThreatController {
    fn respond(&mut self, r: &TurnRequest) -> TurnResponse {
        let action = if r.phase == Phase::Act
            && r.legal.can_use
            && r.legal
                .capabilities
                .contains(&r.observation.objectives.capability)
            && !r.legal.targets.is_empty()
        {
            Action::Use {
                capability: r.observation.objectives.capability,
                target: r.legal.targets[self.rng.gen_range(0..r.legal.targets.len())],
            }
        } else {
            Action::Pass
        };
        TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub frozen_config: ScenarioConfig,
    pub controller_seed_derivation: String,
    pub experiment_version: String,
    pub rules_version: u16,
    pub policy_version: u16,
    pub seed_start: u64,
    pub seed_end_exclusive: u64,
    pub opponent_behavior: String,
    pub perception: Vec<PerceptionDiagnostic>,
    pub inference: InferenceDiagnostic,
    pub decision: DecisionDiagnostic,
    pub games: Vec<GameDiagnostic>,
    pub paired: Vec<PairedDiagnostic>,
    pub totals: Vec<PolicyTotals>,
    pub evidence_more_captures_than_reckless: bool,
    pub evidence_fewer_false_accusations_than_reckless: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PerceptionDiagnostic {
    pub layer: String,
    pub detection_per_mille: u16,
    pub eligible: u64,
    pub detected: u64,
    pub missed: u64,
    pub criterion: String,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InferenceDiagnostic {
    pub layer: String,
    pub posterior: Vec<f64>,
    pub reference: Vec<f64>,
    pub max_absolute_error: f64,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionDiagnostic {
    pub layer: String,
    pub chosen: Option<usize>,
    pub chosen_utility: f64,
    pub reference_utility: f64,
    pub regret: f64,
    pub uniform_abstains: bool,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameDiagnostic {
    pub seed: u64,
    pub policy: PolicyKind,
    pub captures: u64,
    pub threat_wins: u64,
    pub civilian_losses: u64,
    pub false_accusations: u64,
    pub rounds: u64,
    pub direct_evidence_available: u64,
    pub evidence_supported_accusations: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyTotals {
    pub policy: PolicyKind,
    pub captures: u64,
    pub threat_wins: u64,
    pub civilian_losses: u64,
    pub false_accusations: u64,
    pub rounds: u64,
    pub direct_evidence_available: u64,
    pub evidence_supported_accusations: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedDiagnostic {
    pub seed: u64,
    pub comparison: PolicyKind,
    pub captures: i64,
    pub threat_wins: i64,
    pub civilian_losses: i64,
    pub false_accusations: i64,
    pub rounds: i64,
}
fn perception(detection: u16) -> PerceptionDiagnostic {
    let mut config = wink_config(6);
    config.assignment = None;
    config.agents[0].grants = vec![0];
    config.agents[0].objective = ObjectiveTeam::Threat;
    config.observation.detection_per_mille = detection;
    let mut e = Engine::new(config, 0).expect("frozen valid fixture");
    let mut detected = 0;
    while let Some(r) = e.request() {
        if r.phase == Phase::Discuss {
            if r.actor == 1 {
                detected = u64::from(
                    r.observation
                        .events
                        .iter()
                        .any(|v| matches!(v.content, EventContent::Use { source: 0, .. })),
                );
            }
            break;
        }
        let action = match (r.phase, r.actor) {
            (Phase::Attention, 1) => Action::Watch { agents: vec![0] },
            (Phase::Act, 0) => Action::Use {
                capability: 0,
                target: 2,
            },
            _ => Action::Pass,
        };
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action,
        })
        .unwrap();
    }
    // Advance to observer 1 without collecting another round.
    while let Some(r) = e.request() {
        if r.phase != Phase::Discuss {
            break;
        }
        if r.actor == 1 {
            detected = u64::from(
                r.observation
                    .events
                    .iter()
                    .any(|v| matches!(v.content, EventContent::Use { source: 0, .. })),
            );
            break;
        }
        e.submit(TurnResponse {
            request_id: r.request_id,
            actor: r.actor,
            action: Action::Pass,
        })
        .unwrap();
    }
    PerceptionDiagnostic {
        layer: "perception".into(),
        detection_per_mille: detection,
        eligible: 1,
        detected,
        missed: 1 - detected,
        criterion: format!(
            "one eligible watched use: expected {} detection",
            u64::from(detection == 1000)
        ),
        passed: detected == u64::from(detection == 1000),
    }
}
fn game(seed: u64, policy: PolicyKind) -> GameDiagnostic {
    let mut e = Engine::new(wink_config(6), seed).unwrap();
    let holder = e.agents.iter().position(|a| a.grants.contains(&0)).unwrap() as AgentId;
    let mut civilians: Vec<_> = (0..6)
        .map(|id| BuiltinController::new(policy, policy_seed(seed, policy, id)))
        .collect();
    let mut threat = ExperimentThreatController::new(seed, holder);
    let mut g = GameDiagnostic {
        seed,
        policy,
        captures: 0,
        threat_wins: 0,
        civilian_losses: 0,
        false_accusations: 0,
        rounds: 0,
        direct_evidence_available: 0,
        evidence_supported_accusations: 0,
    };
    while let Some(r) = e.request() {
        let sources: Vec<_> = r
            .observation
            .events
            .iter()
            .filter_map(|v| match v.content {
                EventContent::Use {
                    source, capability, ..
                } if capability == 0 && r.legal.targets.contains(&source) => Some(source),
                _ => None,
            })
            .collect();
        if r.actor != holder && r.phase == Phase::Discuss && !sources.is_empty() {
            g.direct_evidence_available += 1;
        }
        let response = if r.actor == holder {
            threat.respond(&r)
        } else {
            civilians[usize::from(r.actor)].respond(&r)
        };
        if let Action::Accuse { target } = response.action {
            if target != holder {
                g.false_accusations += 1;
            }
            if sources.contains(&target) {
                g.evidence_supported_accusations += 1;
            }
        }
        e.submit(response).expect("controller contract");
    }
    let outcome = e.outcome().unwrap();
    g.captures = u64::from(outcome.winner == ObjectiveTeam::Accuser);
    g.threat_wins = 1 - g.captures;
    g.rounds = u64::from(outcome.completed_rounds);
    g.civilian_losses = e
        .agents
        .iter()
        .enumerate()
        .filter(|(id, a)| *id != usize::from(holder) && !a.status.is_active())
        .count() as u64;
    g
}
pub fn diagnose() -> DiagnosticReport {
    let p = posterior(&[1.0 / 3.0; 3], &[0.75, 0.25, 0.25]).unwrap();
    let reference = vec![0.6, 0.2, 0.2];
    let error = p
        .iter()
        .zip(&reference)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    let chosen = best_accusation(&p, 1.0, -1.0, 0.0);
    let utilities: Vec<f64> = (0..3)
        .map(|choice| {
            reference
                .iter()
                .enumerate()
                .map(|(truth, mass)| mass * if truth == choice { 1.0 } else { -1.0 })
                .sum()
        })
        .collect();
    let optimal = utilities.iter().copied().fold(0.0, f64::max);
    let utility = chosen.map(|i| utilities[i]).unwrap_or(0.0);
    let abstains = best_accusation(&[1.0 / 3.0; 3], 1.0, -1.0, 0.0).is_none();
    let mut games = Vec::new();
    let mut paired = Vec::new();
    for seed in 0..128 {
        let evidence = game(seed, PolicyKind::Evidence);
        for kind in [PolicyKind::Reckless, PolicyKind::Passive] {
            let other = game(seed, kind);
            paired.push(PairedDiagnostic {
                seed,
                comparison: kind,
                captures: evidence.captures as i64 - other.captures as i64,
                threat_wins: evidence.threat_wins as i64 - other.threat_wins as i64,
                civilian_losses: evidence.civilian_losses as i64 - other.civilian_losses as i64,
                false_accusations: evidence.false_accusations as i64
                    - other.false_accusations as i64,
                rounds: evidence.rounds as i64 - other.rounds as i64,
            });
            games.push(other);
        }
        games.push(evidence);
    }
    let totals: Vec<_> = [
        PolicyKind::Evidence,
        PolicyKind::Reckless,
        PolicyKind::Passive,
    ]
    .into_iter()
    .map(|policy| {
        let selected: Vec<_> = games.iter().filter(|g| g.policy == policy).collect();
        PolicyTotals {
            policy,
            captures: selected.iter().map(|g| g.captures).sum(),
            threat_wins: selected.iter().map(|g| g.threat_wins).sum(),
            civilian_losses: selected.iter().map(|g| g.civilian_losses).sum(),
            false_accusations: selected.iter().map(|g| g.false_accusations).sum(),
            rounds: selected.iter().map(|g| g.rounds).sum(),
            direct_evidence_available: selected.iter().map(|g| g.direct_evidence_available).sum(),
            evidence_supported_accusations: selected
                .iter()
                .map(|g| g.evidence_supported_accusations)
                .sum(),
        }
    })
    .collect();
    DiagnosticReport {
        frozen_config: wink_config(6),
        controller_seed_derivation: "wrapping u64: world * 6364136223846793005 + identity * 1442695040888963407 + agent; Evidence=1, Random=2, Reckless=3, Passive=4, threat=5; separate PCG streams".into(),
        experiment_version: EXPERIMENT_VERSION.into(),
        rules_version: RULES_VERSION,
        policy_version: POLICY_VERSION,
        seed_start: 0,
        seed_end_exclusive: 128,
        opponent_behavior:
            "uniform random active other target; no watch, talk, or accusation; identity 5".into(),
        perception: vec![perception(0), perception(1000)],
        inference: InferenceDiagnostic {
            layer: "finite noisy-label inference".into(),
            posterior: p,
            reference,
            max_absolute_error: error,
            passed: error < 1e-12,
        },
        decision: DecisionDiagnostic {
            layer: "finite decision".into(),
            chosen,
            chosen_utility: utility,
            reference_utility: optimal,
            regret: optimal - utility,
            uniform_abstains: abstains,
            passed: (optimal - utility).abs() < 1e-12 && abstains,
        },
        evidence_more_captures_than_reckless: totals[0].captures > totals[1].captures,
        evidence_fewer_false_accusations_than_reckless: totals[0].false_accusations
            < totals[1].false_accusations,
        totals,
        games,
        paired,
    }
}
