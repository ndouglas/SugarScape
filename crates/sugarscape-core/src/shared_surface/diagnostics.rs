//! Versioned exhaustive reports. Privileged evaluator records never enter Agent views.
use super::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
pub(super) mod protocol;
pub(super) mod references;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub protocol_version: u16,
    pub supplied_structure: Vec<String>,
    pub traces: Vec<AgentTrace>,
    pub primary: Vec<PanelReport>,
    pub secondary: Vec<PanelReport>,
    pub checks: Vec<DiagnosticCheck>,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelScope {
    pub sequence_denominator: u16,
    pub catalog_contains_actual_environment: bool,
    pub initial_priors: [OwnPrior; 2],
    pub valid_mass: Probability,
    pub failed_mass: Probability,
    pub unconditional_terminal_metrics: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelReport {
    pub kind: PanelKind,
    pub protocol: Protocol,
    /// Privileged report-only actual environment, never an Agent observation.
    pub environment: Environment,
    pub origin: Option<TransferOrigin>,
    pub scope: PanelScope,
    pub episodes: Vec<EpisodeReference>,
    pub aggregate: Aggregate,
    pub sensitivity: Vec<Sensitivity>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelProjection {
    pub visibility: Probability,
    pub retention: Probability,
    pub useful_channel: Probability,
    /// Evaluator-only score, unavailable for NoCommunication and DataFlip.
    pub true_model_probability: Option<Probability>,
    pub unique_true_identification: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeReference {
    /// Privileged task-bit index, not passed to controllers.
    #[serde(deserialize_with = "deserialize_sequence")]
    pub sequence: u16,
    pub agent_traces: [u64; 2],
    /// Evaluator-only events and lineage, kept outside the local trace library.
    pub privileged: Vec<PrivilegedStep>,
    pub metrics: [AgentMetrics; 2],
    pub model_projections: [Option<ModelProjection>; 2],
    pub failure: Option<FailureTrace>,
    pub group_net_utility: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PanelKind {
    Primary,
    Asymmetric,
    Restart,
    Stale,
    DataFlip,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

fn deserialize_sequence<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    let value = u16::deserialize(deserializer)?;
    if value >= 256 {
        return Err(serde::de::Error::custom(
            "episode sequence must be in 0..256",
        ));
    }
    Ok(value)
}
fn invalid(message: &str) -> Error {
    Error::InvalidReport(message.into())
}

pub(super) fn scope(panel: &Panel) -> Result<PanelScope, Error> {
    let first = panel
        .episodes
        .first()
        .ok_or_else(|| invalid("empty panel"))?;
    Ok(PanelScope {
        sequence_denominator: 256,
        catalog_contains_actual_environment: matches!(panel.environment, Environment::InFamily(_)),
        initial_priors: [first.local[0].own_prior, first.local[1].own_prior],
        valid_mass: Probability::new(u64::from(panel.aggregate.valid), 256)?,
        failed_mass: panel.aggregate.failure_mass,
        unconditional_terminal_metrics: true,
    })
}
pub(super) fn projections(
    metrics: &[AgentMetrics; 2],
) -> Result<[Option<ModelProjection>; 2], Error> {
    let project = |m: &AgentMetrics| -> Result<Option<ModelProjection>, Error> {
        m.final_belief
            .as_ref()
            .map(|belief| {
                Ok(ModelProjection {
                    visibility: belief.models[0].checked_add(belief.models[1])?,
                    retention: belief.models[0].checked_add(belief.models[2])?,
                    useful_channel: belief.models[0],
                    true_model_probability: m.true_model_probability,
                    unique_true_identification: m.unique_true_identification,
                })
            })
            .transpose()
    };
    Ok([project(&metrics[0])?, project(&metrics[1])?])
}

// The last prefix is only a search bucket. Complete typed trace equality decides
// identity, so differing posteriors, earlier clocks or decisions never alias.
type TraceKey = (Role, OwnPrior, Option<LocalPrefix>);
fn trace_key(trace: &AgentTrace) -> TraceKey {
    (
        trace.role,
        trace.own_prior,
        trace.steps.last().map(|s| s.prefix.clone()),
    )
}
#[derive(Default)]
pub(super) struct TraceLibrary {
    pub(super) traces: Vec<AgentTrace>,
    buckets: BTreeMap<TraceKey, Vec<usize>>,
}
impl TraceLibrary {
    pub(super) fn intern(&mut self, trace: AgentTrace) -> Result<u64, Error> {
        let key = trace_key(&trace);
        if let Some(indices) = self.buckets.get(&key) {
            if let Some(index) = indices.iter().find(|&&i| self.traces[i] == trace) {
                return u64::try_from(*index).map_err(|_| Error::ArithmeticOverflow);
            }
        }
        // Each frozen episode contributes at most one complete trace per role.
        let max = protocol::PANEL_COUNT
            .checked_mul(protocol::EPISODES)
            .and_then(|n| n.checked_mul(2))
            .ok_or(Error::ArithmeticOverflow)?;
        if self.traces.len() >= max {
            return Err(invalid("trace count exceeds frozen episode projection"));
        }
        let index = self.traces.len();
        self.traces.push(trace);
        self.buckets.entry(key).or_default().push(index);
        u64::try_from(index).map_err(|_| Error::ArithmeticOverflow)
    }
    pub(super) fn panel(
        &mut self,
        panel: Panel,
        kind: PanelKind,
        origin: Option<TransferOrigin>,
    ) -> Result<PanelReport, Error> {
        if panel.episodes.len() != protocol::EPISODES {
            return Err(invalid("panel needs 256 episodes"));
        }
        let scope = scope(&panel)?;
        let mut episodes = Vec::with_capacity(protocol::EPISODES);
        for episode in panel.episodes {
            let [a, b] = episode.local;
            episodes.push(EpisodeReference {
                sequence: episode.sequence,
                agent_traces: [self.intern(a)?, self.intern(b)?],
                model_projections: projections(&episode.metrics)?,
                privileged: episode.privileged,
                metrics: episode.metrics,
                failure: episode.failure,
                group_net_utility: episode.group_net_utility,
            });
        }
        Ok(PanelReport {
            kind,
            protocol: panel.protocol,
            environment: panel.environment,
            origin,
            scope,
            episodes,
            aggregate: panel.aggregate,
            sensitivity: panel.sensitivity,
        })
    }
}

pub(super) fn validate_panel_payload(
    panel: &PanelReport,
    traces: &[AgentTrace],
    expected: &Panel,
) -> Result<bool, Error> {
    if panel.protocol != expected.protocol
        || panel.environment != expected.environment
        || panel.scope != scope(expected)?
        || panel.episodes.len() != 256
        || panel.aggregate != expected.aggregate
        || panel.sensitivity != expected.sensitivity
    {
        return Ok(false);
    }
    for (sequence, (row, episode)) in panel.episodes.iter().zip(&expected.episodes).enumerate() {
        if usize::from(row.sequence) != sequence
            || row.sequence != episode.sequence
            || row.privileged != episode.privileged
            || row.metrics != episode.metrics
            || row.model_projections != projections(&episode.metrics)?
            || row.failure != episode.failure
            || row.group_net_utility != episode.group_net_utility
        {
            return Ok(false);
        }
        for (role, &index) in row.agent_traces.iter().enumerate() {
            let Ok(index) = usize::try_from(index) else {
                return Ok(false);
            };
            if traces.get(index) != Some(&episode.local[role]) {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn checks(passed: bool) -> Vec<DiagnosticCheck> {
    [
        ("frozen_protocol", "48 primary and 172 secondary settings; 56,320 uniformly weighted episodes in canonical order."),
        ("complete_current_payload", "Replay checks every local history, posterior, action, prediction, cost, privileged lineage, failure, projection, aggregate, sensitivity row and transfer origin."),
        ("independent_references", "All settings match independently authored exact numerical references and designated complete local-history projections."),
        ("canonical_trace_library", "Every role trace reconstructs exactly; references use stable first occurrence with no missing, unused or duplicate traces."),
        ("renaming_invariance", "All 12,288 primary episodes rerun with consistently changed opaque identifiers and compare after normalization."),
    ].map(|(name,detail)| DiagnosticCheck {name:name.into(),passed,detail:detail.into()}).to_vec()
}

/// One immutable ensemble per complete Protocol, never one build per episode.
pub(super) struct ReplayCache {
    ensembles: BTreeMap<Protocol, Ensemble>,
    origins: BTreeMap<Mechanism, TransferOrigin>,
}
impl ReplayCache {
    pub(super) fn new() -> Self {
        Self {
            ensembles: BTreeMap::new(),
            origins: BTreeMap::new(),
        }
    }
    pub(super) fn panel(
        &mut self,
        protocol: &Protocol,
        environment: Environment,
    ) -> Result<Panel, Error> {
        if !self.ensembles.contains_key(protocol) {
            self.ensembles
                .insert(protocol.clone(), Ensemble::build(protocol)?);
        }
        evaluate_panel_with_ensemble(protocol, environment, &self.ensembles[protocol])
    }
    pub(super) fn origin(
        &mut self,
        mechanism: Option<Mechanism>,
    ) -> Result<Option<TransferOrigin>, Error> {
        mechanism
            .map(|m| {
                if let std::collections::btree_map::Entry::Vacant(entry) = self.origins.entry(m) {
                    entry.insert(acquire_transfer_origin(m, &protocol::ids())?);
                }
                Ok(self.origins[&m].clone())
            })
            .transpose()
    }
}

pub(super) fn normalized_renamed_panel(
    panel: &Panel,
    cache: &mut ReplayCache,
) -> Result<Panel, Error> {
    let mut renamed_protocol = panel.protocol.clone();
    renamed_protocol.ids = Ids {
        surface: "renamed-object-Q".into(),
        agents: ["renamed-role-Z".into(), "renamed-role-X".into()],
    };
    let mut renamed = cache.panel(&renamed_protocol, panel.environment)?;
    renamed.protocol.ids = panel.protocol.ids.clone();
    for episode in &mut renamed.episodes {
        for trace in &mut episode.local {
            for step in &mut trace.steps {
                step.prefix.ids = panel.protocol.ids.clone();
            }
        }
        if let Some(failure) = &mut episode.failure {
            failure.prefix.ids = panel.protocol.ids.clone();
        }
    }
    Ok(renamed)
}
pub(super) fn renamed_matches(panel: &Panel, cache: &mut ReplayCache) -> Result<bool, Error> {
    Ok(normalized_renamed_panel(panel, cache)? == *panel)
}

/// Produce the complete frozen report. Collection must follow source/reference review.
pub fn diagnose() -> Result<DiagnosticReport, Error> {
    let mut cache = ReplayCache::new();
    let mut library = TraceLibrary::default();
    let mut primary = Vec::with_capacity(protocol::PRIMARY_COUNT);
    let mut secondary = Vec::with_capacity(protocol::PANEL_COUNT - protocol::PRIMARY_COUNT);
    let mut passed = true;
    for (kind, protocol, environment, old) in protocol::settings() {
        let panel = cache.panel(&protocol, environment)?;
        passed &= references::matches_panel(&panel, kind.clone(), old)?;
        if kind == PanelKind::Primary {
            passed &= renamed_matches(&panel, &mut cache)?;
        }
        let origin = cache.origin(old)?;
        let report = library.panel(panel, kind.clone(), origin)?;
        if kind == PanelKind::Primary {
            primary.push(report)
        } else {
            secondary.push(report)
        }
    }
    Ok(DiagnosticReport {
        version: protocol::VERSION.into(),
        protocol_version: 1,
        supplied_structure: protocol::supplied_structure(),
        traces: library.traces,
        primary,
        secondary,
        checks: checks(passed),
        passed,
    })
}

/// Recompute integrity from the current payload and frozen protocol, never from
/// cached flags or by recursively constructing another diagnostic report.
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error> {
    if report.version != protocol::VERSION
        || report.protocol_version != 1
        || report.supplied_structure != protocol::supplied_structure()
        || report.primary.len() != protocol::PRIMARY_COUNT
        || report.secondary.len() != protocol::PANEL_COUNT - protocol::PRIMARY_COUNT
        || !report.passed
        || report.checks != checks(true)
    {
        return Ok(false);
    }
    let max_traces = protocol::PANEL_COUNT
        .checked_mul(protocol::EPISODES)
        .and_then(|v| v.checked_mul(2))
        .ok_or(Error::ArithmeticOverflow)?;
    if report.traces.len() > max_traces {
        return Ok(false);
    }
    // Maximal schedule: 12 calibration slots + 48 live slots. Both roles
    // retain before/after public clocks, four predictions, and EpisodeStart.
    let public_slots = 4usize
        .checked_mul(3 + 3 * usize::from(TRIALS))
        .ok_or(Error::ArithmeticOverflow)?;
    let max_steps = public_slots
        .checked_mul(2)
        .and_then(|v| v.checked_add(1 + usize::from(TRIALS)))
        .ok_or(Error::ArithmeticOverflow)?;
    let max_entries = public_slots / 2 + 1 + 2 * usize::from(TRIALS);
    let ids = protocol::ids();
    if report.traces.iter().any(|trace| {
        trace.steps.is_empty()
            || trace.steps.len() > max_steps
            || trace.steps.iter().any(|step| {
                step.prefix.entries.len() > max_entries
                    || step.prefix.ids != ids
                    || step.prefix.role != trace.role
                    || step.prefix.own_prior != trace.own_prior
            })
    }) {
        return Ok(false);
    }
    let settings = protocol::settings();
    // Cheap exact count/order gates precede any allocation or simulation driven by
    // a payload. Allocation bounds below come only from the frozen protocol.
    for (panel, (kind, protocol, environment, _)) in report
        .primary
        .iter()
        .chain(&report.secondary)
        .zip(&settings)
    {
        if &panel.kind != kind
            || &panel.protocol != protocol
            || &panel.environment != environment
            || panel.episodes.len() != 256
            || panel.sensitivity.len() != 256 * 2 * 4
        {
            return Ok(false);
        }
    }
    if !canonical_library(report)? {
        return Ok(false);
    }
    let mut cache = ReplayCache::new();
    for (panel, (_, protocol, environment, old)) in report
        .primary
        .iter()
        .chain(&report.secondary)
        .zip(&settings)
    {
        if panel.origin != cache.origin(*old)? {
            return Ok(false);
        }
        let expected = cache.panel(protocol, *environment)?;
        if !validate_panel_payload(panel, &report.traces, &expected)?
            || !references::matches_panel(&expected, panel.kind.clone(), *old)?
        {
            return Ok(false);
        }
        if panel.kind == PanelKind::Primary && !renamed_matches(&expected, &mut cache)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn canonical_library(report: &DiagnosticReport) -> Result<bool, Error> {
    let mut buckets: BTreeMap<TraceKey, Vec<usize>> = BTreeMap::new();
    let mut next = 0usize;
    for panel in report.primary.iter().chain(&report.secondary) {
        for (sequence, row) in panel.episodes.iter().enumerate() {
            if usize::from(row.sequence) != sequence {
                return Ok(false);
            }
            for &index in &row.agent_traces {
                let Ok(index) = usize::try_from(index) else {
                    return Ok(false);
                };
                let Some(trace) = report.traces.get(index) else {
                    return Ok(false);
                };
                if index > next {
                    return Ok(false);
                }
                if index == next {
                    let key = trace_key(trace);
                    let bucket = buckets.entry(key).or_default();
                    if bucket.iter().any(|&other| report.traces[other] == *trace) {
                        return Ok(false);
                    }
                    bucket.push(index);
                    next = next.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
                }
            }
        }
    }
    Ok(next == report.traces.len())
}
