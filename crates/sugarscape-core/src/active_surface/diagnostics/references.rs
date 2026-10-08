//! Small independently authored numeric/trace projections. The 202 MB oracle
//! and its source remain evidence, not production assets or actor information.
use super::*;
use serde_json::{json, Value};

pub(in crate::active_surface) struct Reference {
    value: Value,
}
fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidReport(message.into())
}
fn json_value(value: &impl Serialize) -> Result<Value, Error> {
    serde_json::to_value(value).map_err(|e| invalid(format!("reference projection: {e}")))
}
pub(in crate::active_surface) fn environment_name(environment: Environment) -> &'static str {
    match environment {
        Environment::InFamily(Mechanism::SharedPersistent) => "SP",
        Environment::InFamily(Mechanism::SharedResetting) => "SR",
        Environment::InFamily(Mechanism::PrivatePersistent) => "PP",
        Environment::InFamily(Mechanism::Inert) => "I",
        Environment::DataFlip => "DF",
    }
}
pub(in crate::active_surface) fn phase_value(phase: Phase) -> Value {
    match phase {
        Phase::Calibration => json!(-1),
        Phase::Live { trial } => json!(trial),
    }
}
pub(in crate::active_surface) fn choice_name(choice: Choice) -> &'static str {
    match choice {
        Choice::ContinueProbe => "Continue",
        Choice::StopProbing => "Stop",
        Choice::Inspect => "Inspect",
        Choice::AttemptCommunication => "Attempt",
    }
}
pub(in crate::active_surface) fn boundary_value(boundary: Boundary) -> (&'static str, u8) {
    match boundary {
        Boundary::ProbeChoice { completed } => ("probe", completed),
        Boundary::TrialChoice { trial } => ("trial", trial),
    }
}
pub(in crate::active_surface) fn entries_value(entries: &[Entry]) -> Value {
    use crate::shared_surface::Outcome;
    json!(entries
        .iter()
        .map(|entry| match entry {
            Entry::OwnChoice { boundary, choice } => {
                let (kind, n) = boundary_value(*boundary);
                json!(["Choice", kind, n, choice_name(*choice)])
            }
            Entry::PublicProbeStop { completed } => json!(["Stop", completed]),
            Entry::Physical(LocalEntry::Reset { phase }) => json!(["Reset", phase_value(*phase)]),
            Entry::Physical(LocalEntry::PrivateBit { trial, bit }) =>
                json!(["Bit", trial, u8::from(*bit)]),
            Entry::Physical(LocalEntry::Action(event)) => {
                let action = match event.action {
                    Action::Write(symbol) => json!(["Write", symbol]),
                    Action::Read => json!(["Read"]),
                    Action::Wait => json!(["Wait"]),
                    Action::InspectOwnTarget => json!(["Inspect"]),
                };
                let outcome = match event.outcome {
                    Outcome::Read(symbol) => json!(["Read", symbol]),
                    Outcome::Accepted => json!(["Accepted"]),
                    Outcome::Waited => json!(["Waited"]),
                    Outcome::Inspected(bit) => json!(["Inspected", u8::from(bit)]),
                };
                json!([
                    "Act",
                    phase_value(event.position.phase),
                    event.position.round,
                    event.position.slot,
                    action,
                    outcome,
                    event.credits_after
                ])
            }
        })
        .collect::<Vec<_>>())
}
pub(in crate::active_surface) fn checkpoint_value(checkpoint: Checkpoint) -> Value {
    match checkpoint {
        Checkpoint::Boundary(boundary) => {
            let (kind, n) = boundary_value(boundary);
            json!(["boundary", kind, n])
        }
        Checkpoint::BeforeSlot(p) => json!(["before", phase_value(p.phase), p.round, p.slot]),
        Checkpoint::AfterSlot(p) => json!(["slot", phase_value(p.phase), p.round, p.slot]),
        Checkpoint::Prediction { trial } => json!(["prediction", trial]),
        Checkpoint::Finished => json!(["finished"]),
    }
}
fn pair(value: &Value) -> Value {
    if value.is_null() {
        json!([null, null])
    } else {
        value.clone()
    }
}
impl Reference {
    pub(in crate::active_surface) fn read() -> Result<Self, Error> {
        let value: Value =
            serde_json::from_str(include_str!("../tests/fixtures/evaluation-reference.json"))
                .map_err(|e| invalid(format!("independent reference fixture: {e}")))?;
        if value["version"] != "independent-active-evaluation-projections-v2"
            || value["source_sha256"]
                != "6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675"
            || value["output_sha256"]
                != "e39b6bb0f76a626892223343f7c41a9843a8622363bb421fa54d0aedfe2e9f83"
            || value["panels"].as_array().map(Vec::len) != Some(46)
            || value["paired_comparisons"].as_array().map(Vec::len) != Some(86)
        {
            return Err(invalid(
                "independent reference provenance or declared coverage mismatch",
            ));
        }
        Ok(Self { value })
    }
    pub(in crate::active_surface) fn validate_panel(&self, panel: &Panel) -> Result<bool, Error> {
        let role = json_value(&panel.protocol.experimenter)?;
        let kind = json_value(&panel.protocol.policy)?;
        let rows = self.value["panels"]
            .as_array()
            .ok_or_else(|| invalid("missing reference panels"))?;
        let rows = rows
            .iter()
            .filter(|r| {
                r["experimenter"] == role
                    && r["policy"] == kind
                    && r["environment"] == environment_name(panel.environment)
            })
            .collect::<Vec<_>>();
        if rows.len() != 1 {
            return Err(invalid("reference setting missing or duplicated"));
        }
        Self::validate_row(panel, rows[0])
    }
    pub(in crate::active_surface) fn validate_row(
        panel: &Panel,
        row: &Value,
    ) -> Result<bool, Error> {
        let want = &row["aggregate"];
        let complete = panel
            .episodes
            .iter()
            .filter(|e| e.failure.is_none())
            .count();
        if panel.episodes.len() != 256
            || want["episodes"] != 256
            || want["complete"] != complete
            || want["failure"] != 256 - complete
            || json_value(&panel.aggregate.spent)? != want["spent"]
            || json_value(&panel.aggregate.reward)? != pair(&want["reward"])
            || json_value(&panel.aggregate.net)? != pair(&want["net"])
            || json_value(&panel.aggregate.group_net)? != want["group_net"]
            || panel.aggregate.valid_mass
                != Probability::new(
                    u64::try_from(complete).map_err(|_| Error::ArithmeticOverflow)?,
                    256,
                )?
            || panel.aggregate.failed_mass
                != Probability::new(
                    u64::try_from(256 - complete).map_err(|_| Error::ArithmeticOverflow)?,
                    256,
                )?
        {
            return Ok(false);
        }
        let mut stops = BTreeMap::<String, usize>::new();
        for episode in &panel.episodes {
            let stop = episode.histories[0].entries.iter().find_map(|e| {
                if let Entry::PublicProbeStop { completed } = e {
                    Some(*completed)
                } else {
                    None
                }
            });
            let Some(stop) = stop else {
                return Ok(false);
            };
            *stops.entry(stop.to_string()).or_default() += 1;
        }
        if json!(stops) != want["stops"] {
            return Ok(false);
        }
        for i in 0..2 {
            let identified = panel
                .episodes
                .iter()
                .filter(|e| e.catalog_discoveries[i].is_some())
                .count();
            if want["identified"][i] != identified
                || want["identification_censored"][i] != 256 - identified
            {
                return Ok(false);
            }
            for (field, metric) in [
                (
                    "attempted_messages",
                    (|m: &AgentMetrics| m.attempted_sends) as fn(&AgentMetrics) -> u8,
                ),
                ("received_peer_task_writes", |m: &AgentMetrics| {
                    m.lineage_reads
                }),
                ("inspections", |m: &AgentMetrics| m.inspections),
            ] {
                let sum: i64 = panel
                    .episodes
                    .iter()
                    .map(|e| i64::from(metric(&e.metrics[i])))
                    .sum();
                if json_value(&ScoreFraction::new(sum, 256)?)? != want[field][i] {
                    return Ok(false);
                }
            }
            if complete == 256 {
                let sum: i64 = panel
                    .episodes
                    .iter()
                    .map(|e| i64::from(e.metrics[i].correct.unwrap_or(0)))
                    .sum();
                if json_value(&ScoreFraction::new(sum, 256)?)? != want["correct"][i] {
                    return Ok(false);
                }
            } else if !want["correct"].is_null() {
                return Ok(false);
            }
        }
        if let Some(traces) = row["traces"].as_array() {
            for trace in traces {
                let sequence = trace["sequence"]
                    .as_u64()
                    .ok_or_else(|| invalid("reference trace sequence"))?;
                let Some(episode) = panel
                    .episodes
                    .iter()
                    .find(|e| u64::from(e.sequence) == sequence)
                else {
                    return Ok(false);
                };
                if json_value(&episode.predictions)? != trace["predictions"] {
                    return Ok(false);
                }
                for i in 0..2 {
                    if entries_value(&episode.histories[i].entries) != trace["histories"][i] {
                        return Ok(false);
                    }
                    let Some(beliefs) = trace["prediction_beliefs"][i].as_array() else {
                        return Err(invalid("reference prediction beliefs"));
                    };
                    if beliefs.len() != episode.prediction_beliefs[i].len() {
                        return Ok(false);
                    }
                    for (belief, expected) in episode.prediction_beliefs[i].iter().zip(beliefs) {
                        if json_value(&belief.models)? != expected["models"]
                            || json_value(&belief.target)? != expected["target"]
                        {
                            return Ok(false);
                        }
                    }
                }
                match (&episode.failure, &trace["failure"]) {
                    (None, expected) if expected.is_null() => {}
                    (Some(failure), expected) => {
                        if expected["role"] != failure.role.index()
                            || checkpoint_value(failure.prefix.checkpoint) != expected["checkpoint"]
                            || json_value(&failure.spent)? != expected["spent"]
                        {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
            }
        }
        Ok(true)
    }
    pub(in crate::active_surface) fn validate_pairs(
        &self,
        report: &DiagnosticReport,
    ) -> Result<bool, Error> {
        for pair in self.value["paired_comparisons"]
            .as_array()
            .ok_or_else(|| invalid("missing reference paired comparisons"))?
        {
            let find = |field: &str| -> Result<Panel, Error> {
                let panel = report
                    .panels
                    .iter()
                    .find(|p| {
                        json!(p.protocol.experimenter) == pair["experimenter"]
                            && environment_name(p.environment) == pair["environment"]
                            && json!(p.protocol.policy) == pair[field]
                    })
                    .ok_or_else(|| invalid("paired report setting missing"))?;
                Ok(Panel {
                    protocol: panel.protocol.clone(),
                    environment: panel.environment,
                    episodes: panel
                        .episodes
                        .iter()
                        .map(|e| reconstruct_episode(&report.histories, e))
                        .collect::<Result<_, _>>()?,
                    aggregate: panel.aggregate.clone(),
                })
            };
            if json_value(
                &compare_panels(&find("left_policy")?, &find("right_policy")?)?.difference,
            )? != pair["difference"]
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
