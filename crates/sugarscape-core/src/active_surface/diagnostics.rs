//! Compact reconstructible records and independent current-payload validation.
pub(super) mod protocol;
pub(super) mod references;

use super::*;
pub use protocol::{frozen_settings, DIAGNOSTIC_VERSION, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixRef {
    pub history: u64,
    pub entry_count: u64,
    pub checkpoint: Checkpoint,
}
impl PrefixRef {
    /// Reconstruct only one Agent's own prefix, including its IDs and supplied prior.
    pub fn reconstruct(&self, histories: &[Prefix]) -> Result<Prefix, Error> {
        let index = usize::try_from(self.history).map_err(|_| Error::ArithmeticOverflow)?;
        let count = usize::try_from(self.entry_count).map_err(|_| Error::ArithmeticOverflow)?;
        let history = histories
            .get(index)
            .ok_or_else(|| Error::InvalidReport("history reference is out of range".into()))?;
        let entries = history.entries.get(..count).ok_or_else(|| {
            Error::InvalidReport("entry reference discloses a suffix beyond the history".into())
        })?;
        let prefix = Prefix {
            role: history.role,
            ids: history.ids.clone(),
            own_prior: history.own_prior,
            checkpoint: self.checkpoint,
            entries: entries.to_vec(),
        };
        prefix.validate()?;
        Ok(prefix)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactDecision {
    pub prefix: PrefixRef,
    pub decision: Decision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeliefRef {
    pub prefix: PrefixRef,
    pub belief: Belief,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureRef {
    pub role: Role,
    pub prefix: PrefixRef,
    pub last_supported: Option<Belief>,
    pub spent: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeRef {
    #[serde(deserialize_with = "deserialize_sequence")]
    pub sequence: u16,
    pub histories: [u64; 2],
    pub decisions: Vec<CompactDecision>,
    pub metrics: [AgentMetrics; 2],
    pub beliefs: Vec<BeliefRef>,
    pub model_projections: [Option<ModelProjection>; 2],
    pub predictions: [Vec<bool>; 2],
    pub prediction_beliefs: [Vec<Belief>; 2],
    /// Privileged host chronology and origin; never an Agent View.
    pub privileged: Vec<PrivilegedStep>,
    pub failure: Option<FailureRef>,
    pub catalog_discoveries: [Option<CatalogDiscovery>; 2],
}
fn deserialize_sequence<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<u16, D::Error> {
    let sequence = u16::deserialize(deserializer)?;
    EpisodeBits::from_index(sequence).map_err(serde::de::Error::custom)?;
    Ok(sequence)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelReport {
    pub protocol: Protocol,
    pub environment: Environment,
    pub episodes: Vec<EpisodeRef>,
    pub aggregate: Aggregate,
    pub search_stats: SearchStats,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub protocol_version: u16,
    pub supplied_structure: Vec<String>,
    pub histories: Vec<Prefix>,
    pub panels: Vec<PanelReport>,
    pub checks: Vec<Check>,
    pub passed: bool,
}

#[derive(Default)]
pub(super) struct HistoryLibrary {
    pub(super) histories: Vec<Prefix>,
    indices: BTreeMap<Prefix, u64>,
}
impl HistoryLibrary {
    pub(super) fn intern(&mut self, prefix: Prefix) -> Result<u64, Error> {
        if let Some(index) = self.indices.get(&prefix) {
            return Ok(*index);
        }
        let index = u64::try_from(self.histories.len()).map_err(|_| Error::ArithmeticOverflow)?;
        self.indices.insert(prefix.clone(), index);
        self.histories.push(prefix);
        Ok(index)
    }
    fn reference(&self, history: u64, prefix: &Prefix) -> Result<PrefixRef, Error> {
        let reference = PrefixRef {
            history,
            entry_count: u64::try_from(prefix.entries.len())
                .map_err(|_| Error::ArithmeticOverflow)?,
            checkpoint: prefix.checkpoint,
        };
        if reference.reconstruct(&self.histories)? != *prefix {
            return Err(Error::InvalidReport(
                "prefix is not part of its final own history".into(),
            ));
        }
        Ok(reference)
    }
    pub(super) fn compact(
        &mut self,
        episode: &Episode,
        replay: &Replay,
    ) -> Result<EpisodeRef, Error> {
        let histories = [
            self.intern(episode.histories[0].clone())?,
            self.intern(episode.histories[1].clone())?,
        ];
        let mut decisions = Vec::with_capacity(episode.decisions.len());
        let mut beliefs = Vec::new();
        for trace in &episode.decisions {
            let prefix = self.reference(histories[trace.prefix.role.index()], &trace.prefix)?;
            beliefs.push(BeliefRef {
                prefix: prefix.clone(),
                belief: replay.infer(&trace.prefix)?,
            });
            decisions.push(CompactDecision {
                prefix,
                decision: trace.decision.clone(),
            });
        }
        // Prediction snapshots share raw entries with other checkpoints, so the
        // exact Prediction clock is part of every reference, never inferred away.
        for role in [Role::A, Role::B] {
            let i = role.index();
            for (trial, belief) in episode.prediction_beliefs[i].iter().enumerate() {
                let trial = u8::try_from(trial).map_err(|_| Error::ArithmeticOverflow)?;
                let history = &episode.histories[i];
                let count = history.entries.iter().position(|e| matches!(e, Entry::Physical(LocalEntry::Reset { phase: Phase::Live { trial: next } }) if *next == trial + 1)).unwrap_or(history.entries.len());
                beliefs.push(BeliefRef {
                    prefix: PrefixRef {
                        history: histories[i],
                        entry_count: u64::try_from(count).map_err(|_| Error::ArithmeticOverflow)?,
                        checkpoint: Checkpoint::Prediction { trial },
                    },
                    belief: belief.clone(),
                });
            }
        }
        let failure = episode
            .failure
            .as_ref()
            .map(|f| -> Result<FailureRef, Error> {
                Ok(FailureRef {
                    role: f.role,
                    prefix: self.reference(histories[f.role.index()], &f.prefix)?,
                    last_supported: f.last_supported.clone(),
                    spent: f.spent,
                })
            })
            .transpose()?;
        Ok(EpisodeRef {
            sequence: episode.sequence,
            histories,
            decisions,
            metrics: episode.metrics.clone(),
            beliefs,
            model_projections: episode.model_projections.clone(),
            predictions: episode.predictions.clone(),
            prediction_beliefs: episode.prediction_beliefs.clone(),
            privileged: episode.privileged.clone(),
            failure,
            catalog_discoveries: episode.catalog_discoveries.clone(),
        })
    }
}

pub fn reconstruct_episode(histories: &[Prefix], row: &EpisodeRef) -> Result<Episode, Error> {
    EpisodeBits::from_index(row.sequence)?;
    let own_history = |role: Role| -> Result<Prefix, Error> {
        let index =
            usize::try_from(row.histories[role.index()]).map_err(|_| Error::ArithmeticOverflow)?;
        let prefix = histories.get(index).ok_or_else(|| {
            Error::InvalidReport("final history reference is out of range".into())
        })?;
        if prefix.role != role {
            return Err(Error::InvalidReport(
                "final history has the wrong Agent role".into(),
            ));
        }
        prefix.validate()?;
        Ok(prefix.clone())
    };
    Ok(Episode {
        sequence: row.sequence,
        histories: [own_history(Role::A)?, own_history(Role::B)?],
        decisions: row
            .decisions
            .iter()
            .map(|d| {
                Ok(DecisionTrace {
                    prefix: d.prefix.reconstruct(histories)?,
                    decision: d.decision.clone(),
                })
            })
            .collect::<Result<_, Error>>()?,
        metrics: row.metrics.clone(),
        predictions: row.predictions.clone(),
        prediction_beliefs: row.prediction_beliefs.clone(),
        privileged: row.privileged.clone(),
        failure: row
            .failure
            .as_ref()
            .map(|f| -> Result<Failure, Error> {
                Ok(Failure {
                    role: f.role,
                    prefix: f.prefix.reconstruct(histories)?,
                    last_supported: f.last_supported.clone(),
                    spent: f.spent,
                })
            })
            .transpose()?,
        model_projections: row.model_projections.clone(),
        catalog_discoveries: row.catalog_discoveries.clone(),
    })
}

/// All malformed payloads fail validation. Operational arithmetic/physics errors
/// from fresh source execution remain errors, rather than cached false flags.
fn payload_result(result: Result<bool, Error>) -> Result<bool, Error> {
    match result {
        Err(
            Error::InvalidReport(_)
            | Error::InvalidHistory(_)
            | Error::InvalidProtocol(_)
            | Error::UnsupportedHistory { .. },
        ) => Ok(false),
        other => other,
    }
}
#[cfg(test)]
pub(super) fn episode_integrity(
    histories: &[Prefix],
    row: &EpisodeRef,
    protocol: &Protocol,
    environment: Environment,
    policy: &CompiledPolicy,
    replay: &Replay,
) -> Result<bool, Error> {
    let expected = run_episode(protocol, environment, row.sequence, policy, replay)?;
    episode_matches(histories, row, &expected, replay)
}
pub(super) fn episode_matches(
    histories: &[Prefix],
    row: &EpisodeRef,
    expected: &Episode,
    replay: &Replay,
) -> Result<bool, Error> {
    payload_result((|| {
        if reconstruct_episode(histories, row)? != *expected {
            return Ok(false);
        }
        // Require the exact reference ownership/layout as well as its content:
        // a valid but unrelated history cannot substitute for this episode's own.
        let mut library = HistoryLibrary::default();
        let mut compact = library.compact(expected, replay)?;
        for reference in compact
            .decisions
            .iter_mut()
            .map(|d| &mut d.prefix)
            .chain(compact.beliefs.iter_mut().map(|b| &mut b.prefix))
            .chain(compact.failure.iter_mut().map(|f| &mut f.prefix))
        {
            let role = library.histories
                [usize::try_from(reference.history).map_err(|_| Error::ArithmeticOverflow)?]
            .role;
            reference.history = row.histories[role.index()];
        }
        compact.histories = row.histories;
        if compact != *row {
            return Ok(false);
        }
        for belief in &row.beliefs {
            if replay.infer(&belief.prefix.reconstruct(histories)?)? != belief.belief {
                return Ok(false);
            }
        }
        Ok(true)
    })())
}
fn expected_checks() -> Vec<Check> {
    [
        ("frozen_census", "46 ordered settings, 40 primary and 6 secondary, each with unique sequence indices 0..255"),
        ("compact_own_histories", "typed unique complete own histories and checked entry-count/checkpoint reconstruction"),
        ("current_episode_semantics", "fresh host physics, policy alternatives and choices, both-local replay, partial failures, rewards, metrics and privileged origins"),
        ("independent_reference_v2", "46 exact aggregate projections, designated own traces and 86 sequence-paired comparisons; reviewed source/output hashes retained"),
    ].map(|(name, detail)| Check { name: name.into(), passed: true, detail: detail.into() }).to_vec()
}
pub(super) fn empty_report() -> DiagnosticReport {
    DiagnosticReport {
        version: DIAGNOSTIC_VERSION.into(),
        protocol_version: PROTOCOL_VERSION,
        supplied_structure: protocol::supplied_structure(),
        histories: Vec::new(),
        panels: Vec::new(),
        checks: expected_checks(),
        passed: false,
    }
}
fn valid_census(report: &DiagnosticReport) -> bool {
    let settings = frozen_settings();
    report.version == DIAGNOSTIC_VERSION
        && report.protocol_version == PROTOCOL_VERSION
        && report.supplied_structure == protocol::supplied_structure()
        && report.checks == expected_checks()
        && report.panels.len() == settings.len()
        && report
            .panels
            .iter()
            .zip(settings)
            .all(|(panel, (protocol, environment))| {
                panel.protocol == protocol
                    && panel.environment == environment
                    && panel.episodes.len() == 256
                    && panel
                        .episodes
                        .iter()
                        .map(|row| row.sequence)
                        .collect::<BTreeSet<_>>()
                        == (0..256).collect()
            })
}

pub fn diagnose() -> Result<DiagnosticReport, Error> {
    let settings = frozen_settings();
    let reference = references::Reference::read()?;
    let mut library = HistoryLibrary::default();
    let mut panels = vec![None; settings.len()];
    protocol::for_each_engine(&settings, |index, policy, replay| {
        let (protocol, environment) = &settings[index];
        let panel = evaluate_panel(protocol, *environment, policy, replay)?;
        if !reference.validate_panel(&panel)? {
            return Err(Error::InvalidReport(
                "current panel differs from independent reference-v2".into(),
            ));
        }
        let episodes = panel
            .episodes
            .iter()
            .map(|episode| library.compact(episode, replay))
            .collect::<Result<_, _>>()?;
        panels[index] = Some(PanelReport {
            protocol: protocol.clone(),
            environment: *environment,
            episodes,
            aggregate: panel.aggregate,
            search_stats: policy.stats().clone(),
        });
        Ok(())
    })?;
    let mut report = empty_report();
    report.histories = library.histories;
    report.panels = panels
        .into_iter()
        .collect::<Option<_>>()
        .ok_or_else(|| Error::InvalidReport("missing constructed panel".into()))?;
    report.passed = valid_census(&report) && reference.validate_pairs(&report)?;
    Ok(report)
}

/// Recompute from the current payload; neither `passed` nor cached success can
/// bypass source physics, policy selection, inference, census or exact references.
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error> {
    if !valid_census(report) {
        return Ok(false);
    }
    if report.histories.iter().collect::<BTreeSet<_>>().len() != report.histories.len() {
        return Ok(false);
    }
    let used = report
        .panels
        .iter()
        .flat_map(|p| &p.episodes)
        .flat_map(|e| e.histories)
        .collect::<BTreeSet<_>>();
    if used
        != (0..u64::try_from(report.histories.len()).map_err(|_| Error::ArithmeticOverflow)?)
            .collect()
    {
        return Ok(false);
    }
    let reference = references::Reference::read()?;
    let settings = frozen_settings();
    let mut valid = true;
    protocol::for_each_engine(&settings, |index, policy, replay| {
        if !valid {
            return Ok(());
        }
        let panel = &report.panels[index];
        let expected = evaluate_panel(&panel.protocol, panel.environment, policy, replay)?;
        if panel.aggregate != expected.aggregate
            || panel.search_stats != *policy.stats()
            || !reference.validate_panel(&expected)?
        {
            valid = false;
            return Ok(());
        }
        for row in &panel.episodes {
            let expected = &expected.episodes[usize::from(row.sequence)];
            if !episode_matches(&report.histories, row, expected, replay)? {
                valid = false;
                break;
            }
        }
        Ok(())
    })?;
    Ok(valid && reference.validate_pairs(report)?)
}
