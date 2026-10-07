//! Comparisons against independently authored, precollection exact JSON references.
//! This projects Rust outputs into the reference schema; it contains no independent
//! world simulator and never obtains expected numbers from production replay.
use super::*;
use serde_json::{json, Value};
use std::sync::OnceLock;

fn reference() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../tests/fixtures/diagnostic-reference.json"))
            .expect("reviewed reference JSON")
    })
}
fn trace_reference() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../tests/fixtures/diagnostic-traces-reference.json"
        ))
        .expect("reviewed designated trace reference JSON")
    })
}
fn model(m: Mechanism) -> &'static str {
    match m {
        Mechanism::SharedPersistent => "SP",
        Mechanism::SharedResetting => "SR",
        Mechanism::PrivatePersistent => "PP",
        Mechanism::Inert => "IN",
    }
}
fn environment(e: Environment) -> &'static str {
    match e {
        Environment::InFamily(m) => model(m),
        Environment::DataFlip => "DataFlip",
    }
}
fn knowledge(k: Knowledge) -> &'static str {
    match k {
        Knowledge::Unknown => "unknown",
        Knowledge::Known => "known",
        Knowledge::NoCommunication => "none",
    }
}
fn kind(k: &PanelKind) -> &'static str {
    match k {
        PanelKind::Primary => "primary",
        PanelKind::Asymmetric => "asymmetric",
        PanelKind::Restart => "restart",
        PanelKind::Stale => "stale",
        PanelKind::DataFlip => "data_flip",
    }
}
fn prior(mode: PriorMode) -> &'static str {
    match mode {
        PriorMode::Treatment => "treatment",
        PriorMode::RestartUniform => "restart",
        PriorMode::Stale(m) => model(m),
    }
}
fn own_prior(p: OwnPrior) -> &'static str {
    match p {
        OwnPrior::Uniform => "uniform",
        OwnPrior::PointMass(m) => model(m),
    }
}
fn subset_eq(actual: &Value, expected: &Value) -> bool {
    match actual {
        Value::Object(fields) => fields
            .iter()
            .all(|(k, v)| expected.get(k).is_some_and(|e| subset_eq(v, e))),
        Value::Array(rows) => expected.as_array().is_some_and(|values| {
            rows.len() == values.len() && rows.iter().zip(values).all(|(a, b)| subset_eq(a, b))
        }),
        _ => actual == expected,
    }
}
fn position(p: Position) -> Value {
    match p.phase {
        Phase::Calibration => json!(["cal", -1, p.round, p.slot]),
        Phase::Live { trial } => json!(["live", trial, p.round, p.slot]),
    }
}
fn discovery(c: &Certainty) -> Value {
    match c {
        Certainty::Censored => json!("censored"),
        Certainty::Supplied => json!({"checkpoint":"episode_start","credits":0,"supplied":true}),
        Certainty::Acquired {
            checkpoint,
            credits,
        } => {
            let at = match checkpoint {
                Checkpoint::Action(p) | Checkpoint::AfterSlot(p) => position(*p),
                Checkpoint::EpisodeStart => json!("episode_start"),
                Checkpoint::Prediction { trial } => json!(["prediction", trial]),
            };
            json!({"checkpoint":at,"credits":credits,"supplied":false})
        }
    }
}
fn property(p: Property) -> &'static str {
    match p {
        Property::Visibility => "visibility",
        Property::Retention => "retention",
        Property::UsefulChannel => "useful_channel",
        Property::TrueMechanism => "true_mechanism",
    }
}
fn distribution(values: impl Iterator<Item = Value>, mass: bool) -> Result<Vec<Value>, Error> {
    let mut counts: BTreeMap<String, (Value, u64)> = BTreeMap::new();
    for v in values {
        let key = serde_json::to_string(&v).map_err(|e| invalid(&e.to_string()))?;
        let row = counts.entry(key).or_insert((v, 0));
        row.1 = row.1.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    }
    counts
        .into_values()
        .map(|(v, n)| {
            if mass {
                Ok(json!({"value":v,"mass":Probability::new(n,256)?}))
            } else {
                Ok(json!({"failure":v,"count":n}))
            }
        })
        .collect()
}
fn unordered_eq(actual: &[Value], expected: &Value) -> bool {
    expected
        .as_array()
        .is_some_and(|rows| rows.len() == actual.len() && actual.iter().all(|a| rows.contains(a)))
}
fn average(values: impl Iterator<Item = ScoreFraction>) -> Result<ScoreFraction, Error> {
    let mut total = ScoreFraction::new(0, 1)?;
    for value in values {
        total = total.checked_add(value)?;
    }
    total.checked_mul(ScoreFraction::new(1, 256)?)
}
fn counted(values: impl Iterator<Item = Option<bool>>) -> Value {
    let mut map: BTreeMap<&str, u64> = BTreeMap::new();
    for v in values {
        *map.entry(match v {
            None => "unavailable",
            Some(true) => "true",
            Some(false) => "false",
        })
        .or_default() += 1;
    }
    json!(map)
}
fn reference_failure(f: &FailureTrace) -> Value {
    let at = match f.checkpoint {
        Checkpoint::Action(p) | Checkpoint::AfterSlot(p) => position(p),
        Checkpoint::EpisodeStart => json!("episode_start"),
        Checkpoint::Prediction { trial } => json!(["prediction", trial]),
    };
    json!({"last_supported":[f.last_supported_belief.models,f.last_supported_belief.target],"observed_at":at,"prior":own_prior(f.prior),"role":f.role.index(),"spent":f.spent})
}

pub(in crate::shared_surface) fn matches_panel(
    panel: &Panel,
    panel_kind: PanelKind,
    old: Option<Mechanism>,
) -> Result<bool, Error> {
    if panel.episodes.len() != 256 || panel.sensitivity.len() != 2048 {
        return Ok(false);
    }
    let rows = reference()["panels"]
        .as_array()
        .ok_or_else(|| invalid("missing independent panels"))?;
    let pair = panel.protocol.pair.roles.map(knowledge);
    let expected = rows
        .iter()
        .find(|row| {
            row["kind"] == kind(&panel_kind)
                && row["environment"] == environment(panel.environment)
                && row["calibration_rounds"] == panel.protocol.calibration_rounds
                && row["pair"] == json!(pair)
                && row["prior_mode"] == prior(panel.protocol.prior_mode)
                && row["old_mechanism"] == json!(old.map(model))
        })
        .ok_or_else(|| invalid("setting absent from independent reference"))?;
    let expected = &expected["aggregate"];
    if !subset_eq(
        &serde_json::to_value(&panel.aggregate).map_err(|e| invalid(&e.to_string()))?,
        expected,
    ) {
        return Ok(false);
    }
    for role in 0..2 {
        let metrics: Vec<_> = panel.episodes.iter().map(|e| &e.metrics[role]).collect();
        let expected = &expected["agents"][role];
        let mut counts = serde_json::Map::new();
        for (name, get) in [
            (
                "attempted_sends",
                (|m: &AgentMetrics| u64::from(m.attempted_sends)) as fn(&AgentMetrics) -> u64,
            ),
            ("calibration_reads", |m| u64::from(m.calibration_reads)),
            ("causal_transfers", |m| {
                u64::from(m.other_agent_task_lineage_reads)
            }),
            ("decoded_data", |m| u64::from(m.decoded_data)),
            ("inspect", |m| u64::from(m.inspections)),
            ("live_reads", |m| u64::from(m.live_reads)),
            ("read", |m| u64::from(m.reads)),
            ("wait", |m| u64::from(m.waits)),
            ("write", |m| {
                u64::from(m.calibration_writes) + u64::from(m.attempted_sends)
            }),
        ] {
            let total = metrics.iter().try_fold(0u64, |sum, m| {
                sum.checked_add(get(m)).ok_or(Error::ArithmeticOverflow)
            })?;
            if total != 0 {
                counts.insert(
                    name.into(),
                    json!(ScoreFraction::new(
                        i64::try_from(total).map_err(|_| Error::ArithmeticOverflow)?,
                        256
                    )?),
                );
            }
        }
        if Value::Object(counts) != expected["expected_counts"] {
            return Ok(false);
        }
        for prop in [
            Property::Visibility,
            Property::Retention,
            Property::UsefulChannel,
            Property::TrueMechanism,
        ] {
            if metrics
                .iter()
                .any(|m| m.discoveries.iter().filter(|d| d.property == prop).count() != 1)
            {
                return Ok(false);
            }
            let actual = distribution(
                metrics.iter().map(|m| {
                    discovery(
                        &m.discoveries
                            .iter()
                            .find(|d| d.property == prop)
                            .expect("checked discovery")
                            .certainty,
                    )
                }),
                true,
            )?;
            if !unordered_eq(&actual, &expected["discoveries"][property(prop)]) {
                return Ok(false);
            }
        }
        let model_means = if metrics.iter().any(|m| m.final_belief.is_none()) {
            Value::Null
        } else {
            let mut means = Vec::with_capacity(4);
            for model in 0..4 {
                let mut probabilities = Vec::with_capacity(256);
                for m in &metrics {
                    let p = m.final_belief.as_ref().expect("checked belief").models[model];
                    probabilities.push(ScoreFraction::new(
                        i64::try_from(p.numerator).map_err(|_| Error::ArithmeticOverflow)?,
                        p.denominator,
                    )?);
                }
                means.push(average(probabilities.into_iter())?);
            }
            json!(means)
        };
        if model_means != expected["final_posterior"] {
            return Ok(false);
        }
        let ungraded = panel.environment == Environment::DataFlip
            || panel.protocol.pair.roles[role] == Knowledge::NoCommunication;
        if ungraded {
            if metrics.iter().any(|m| {
                m.true_model_probability.is_some() || m.unique_true_identification.is_some()
            }) {
                return Ok(false);
            }
        } else if let Environment::InFamily(actual_model) = panel.environment {
            for m in &metrics {
                let expected = m
                    .final_belief
                    .as_ref()
                    .map(|b| b.models[actual_model.index()]);
                if m.true_model_probability != expected {
                    return Ok(false);
                }
            }
        }
        if panel.aggregate.failed == 0 && !ungraded {
            let Environment::InFamily(actual_model) = panel.environment else {
                return Ok(false);
            };
            let mut scores = Vec::with_capacity(256);
            for m in &metrics {
                let Some(p) = m.true_model_probability else {
                    return Ok(false);
                };
                scores.push(ScoreFraction::new(
                    i64::try_from(p.numerator).map_err(|_| Error::ArithmeticOverflow)?,
                    p.denominator,
                )?);
            }
            if json!(average(scores.into_iter())?)
                != expected["final_posterior"][actual_model.index()]
            {
                return Ok(false);
            }
        }
        if panel.protocol.pair.roles[role] == Knowledge::NoCommunication {
            let uniform = [Probability::new(1, 4)?; 4];
            if metrics
                .iter()
                .any(|m| m.final_belief.as_ref().is_none_or(|b| b.models != uniform))
            {
                return Ok(false);
            }
        }
        let unique = if metrics
            .iter()
            .any(|m| m.unique_true_identification.is_none())
        {
            Value::Null
        } else {
            json!(Probability::new(
                metrics
                    .iter()
                    .filter(|m| m.unique_true_identification == Some(true))
                    .count() as u64,
                256
            )?)
        };
        // The independent oracle's descriptive zero uniqueness under uniform
        // NoCommunication is not a graded model score. Preserve its full uniform
        // catalog posterior above; the approved report semantics require None.
        let expected_unique = if panel.protocol.pair.roles[role] == Knowledge::NoCommunication {
            let descriptive = expected
                .get("unique_true_identification")
                .cloned()
                .unwrap_or(Value::Null);
            let expected_descriptive = if matches!(panel.environment, Environment::InFamily(_)) {
                json!(Probability::new(0, 1)?)
            } else {
                Value::Null
            };
            if descriptive != expected_descriptive {
                return Ok(false);
            }
            Value::Null
        } else {
            expected
                .get("unique_true_identification")
                .cloned()
                .unwrap_or(Value::Null)
        };
        if unique != expected_unique {
            return Ok(false);
        }
    }
    let failures = distribution(
        panel
            .episodes
            .iter()
            .filter_map(|e| e.failure.as_ref().map(reference_failure)),
        false,
    )?;
    if !unordered_eq(&failures, &expected["failure_locations"]) {
        return Ok(false);
    }
    let mut sensitivity = Vec::with_capacity(8);
    for sender in [Role::A, Role::B] {
        for trial in 0..4 {
            let rows: Vec<_> = panel
                .sensitivity
                .iter()
                .filter(|s| s.sender == sender && s.trial == trial)
                .collect();
            if rows.len() != 256 {
                return Ok(false);
            }
            sensitivity.push(json!({"sender":sender.index(),"trial":trial,
            "read":counted(rows.iter().map(|s|s.read_changed)),
            "posterior":counted(rows.iter().map(|s|s.posterior_changed)),
            "prediction":counted(rows.iter().map(|s|s.prediction_changed))}));
        }
    }
    Ok(
        json!(sensitivity) == expected["paired_sensitivity"]
            && matches_designated_histories(panel)?,
    )
}

fn symbol(s: Symbol) -> &'static str {
    match s {
        Symbol::Blank => "blank",
        Symbol::Probe0 => "probe0",
        Symbol::Ack0 => "ack0",
        Symbol::Probe1 => "probe1",
        Symbol::Ack1 => "ack1",
        Symbol::Data0 => "data0",
        Symbol::Data1 => "data1",
    }
}
fn action(a: &Action) -> Value {
    match a {
        Action::Write(s) => json!(["write", symbol(*s)]),
        Action::Read => json!(["read", null]),
        Action::Wait => json!(["wait", null]),
        Action::InspectOwnTarget => json!(["inspect", null]),
    }
}
fn phase(p: Phase) -> Value {
    match p {
        Phase::Calibration => json!(["cal", -1]),
        Phase::Live { trial } => json!(["live", trial]),
    }
}
fn entries(prefix: &LocalPrefix) -> Result<Value, Error> {
    let mut values = Vec::with_capacity(prefix.entries.len());
    for entry in &prefix.entries {
        values.push(match entry {
            LocalEntry::Reset { phase: p } => json!(["reset", phase(*p)]),
            LocalEntry::PrivateBit { trial, bit } => json!(["private", trial, u8::from(*bit)]),
            LocalEntry::Action(e) => {
                let outcome = match e.outcome {
                    Outcome::Read(s) => json!(symbol(s)),
                    Outcome::Accepted => json!("accepted"),
                    Outcome::Waited => json!("waited"),
                    Outcome::Inspected(bit) => json!(u8::from(bit)),
                };
                json!([
                    "event",
                    position(e.position),
                    action(&e.action),
                    outcome,
                    INITIAL_CREDITS
                        .checked_sub(e.credits_after)
                        .ok_or(Error::ArithmeticOverflow)?
                ])
            }
        });
    }
    Ok(json!(values))
}
fn checkpoint(c: Checkpoint) -> Value {
    match c {
        Checkpoint::EpisodeStart => json!(["episode_start"]),
        Checkpoint::Prediction { trial } => json!(["prediction", trial]),
        Checkpoint::Action(p) | Checkpoint::AfterSlot(p) => {
            let mut values = vec![json!(if matches!(c, Checkpoint::Action(_)) {
                "before"
            } else {
                "after"
            })];
            values.extend(
                position(p)
                    .as_array()
                    .expect("position array")
                    .iter()
                    .cloned(),
            );
            json!(values)
        }
    }
}
fn local(trace: &AgentTrace) -> Result<Value, Error> {
    let mut snapshots = Vec::new();
    for step in &trace.steps {
        if let Checkpoint::Action(p) | Checkpoint::AfterSlot(p) = step.prefix.checkpoint {
            if role_at(p)? != trace.role {
                continue;
            }
        }
        let private_bit = step.prefix.entries.iter().rev().find_map(|e| match e {
            LocalEntry::PrivateBit { bit, .. } => Some(u8::from(*bit)),
            _ => None,
        });
        snapshots.push(json!({"checkpoint":checkpoint(step.prefix.checkpoint),"entries":entries(&step.prefix)?,"credits":step.credits,"private_bit":private_bit,"belief":step.belief,"decision":step.decision.as_ref().map(action),"prediction":step.prediction.map(u8::from)}));
    }
    Ok(
        json!({"role":if trace.role==Role::A {"A"} else {"B"},"own_prior":own_prior(trace.own_prior),"snapshots":snapshots}),
    )
}
pub(in crate::shared_surface) fn matches_designated_histories(
    panel: &Panel,
) -> Result<bool, Error> {
    let reference = trace_reference();
    if reference["trace_reference_version"] != 2
        || reference["cases"]
            .as_array()
            .is_none_or(|cases| cases.len() != 9)
    {
        return Err(invalid("designated trace reference schema mismatch"));
    }
    matches_local_cases(panel, reference)
}
fn matches_local_cases(panel: &Panel, cases: &Value) -> Result<bool, Error> {
    let cases = cases["cases"]
        .as_array()
        .ok_or_else(|| invalid("missing independent trace cases"))?;
    for case in cases {
        let metadata = &case["researcher_metadata"];
        if metadata["environment"] != environment(panel.environment)
            || metadata["calibration_rounds"] != panel.protocol.calibration_rounds
            || metadata["pair"] != json!(panel.protocol.pair.roles.map(knowledge))
            || metadata["prior_mode"] != prior(panel.protocol.prior_mode)
        {
            continue;
        }
        let sequence = metadata["sequence"]
            .as_u64()
            .ok_or_else(|| invalid("invalid trace reference sequence"))?;
        let sequence = usize::try_from(sequence).map_err(|_| Error::ArithmeticOverflow)?;
        let episode = panel
            .episodes
            .get(sequence)
            .ok_or_else(|| invalid("missing designated sequence"))?;
        if json!([local(&episode.local[0])?, local(&episode.local[1])?]) != case["local"] {
            return Ok(false);
        }
        if json!(episode.metrics.each_ref().map(|m| m.spent)) != case["spent"] {
            return Ok(false);
        }
        let failure=episode.failure.as_ref().map(|f| Ok::<_,Error>(json!({"role":if f.role==Role::A {"A"} else {"B"},"checkpoint":checkpoint(f.checkpoint),"entries":entries(&f.prefix)?,"prior":own_prior(f.prior),"last_supported_belief":f.last_supported_belief,"spent":f.spent}))).transpose()?;
        if json!(failure) != case["failure"] {
            return Ok(false);
        }
        let terminal = if episode.failure.is_some() {
            Value::Null
        } else {
            let mut agents = Vec::with_capacity(2);
            for metric in &episode.metrics {
                let terminal = metric
                    .terminal
                    .as_ref()
                    .ok_or_else(|| invalid("missing successful terminal"))?;
                agents.push(json!({"predictions":terminal.predictions.map(u8::from),"correct":terminal.correct.map(u8::from),"gross_reward":terminal.gross_reward,"net_utility":terminal.net_utility}));
            }
            json!({"agents":agents,"group_net_utility":episode.group_net_utility})
        };
        if terminal != case["terminal"] {
            return Ok(false);
        }
    }
    Ok(true)
}
