//! Chronological display projections composed with the frozen surface evaluators.
use super::{
    error, record, wire::lossless_value, Checkpoint, EpisodeKind, EpisodeRecord, FieldError, Input,
    Semantics, StudyDescriptor, StudyFamily, StudyId, EPISODE_VERSION,
};
use crate::{active_surface as a, shared_surface as s};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

type Result<T> = std::result::Result<T, Vec<FieldError>>;
fn checked<T, E: std::fmt::Display>(v: std::result::Result<T, E>) -> Result<T> {
    v.map_err(|e| error("surface", e.to_string()))
}
fn fixture() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw = include_str!("fixtures/surface-examples.json");
        assert!(
            raw.len() <= 128 * 1024,
            "surface definitions exceed 128 KiB"
        );
        serde_json::from_str(raw).expect("retained surface definitions")
    })
}
fn name(study: StudyId) -> &'static str {
    if study == StudyId::SharedSurface {
        "shared_surface"
    } else {
        "active_surface"
    }
}
pub(crate) fn rules_identity(study: StudyId) -> Result<String> {
    let data = &fixture()["studies"][name(study)];
    Ok(format!(
        "{}:protocol-1:browser-surface-examples-v1:engine-sha256:{}:report-sha256:{}",
        name(study),
        data["engine_sha256"]
            .as_str()
            .ok_or_else(|| error("rules_identity", "missing engine receipt"))?,
        data["source"]["sha256"]
            .as_str()
            .ok_or_else(|| error("rules_identity", "missing report receipt"))?
    ))
}
pub(crate) fn validate_input(input: &Input) -> Result<()> {
    let (protocol, environment, sequence) = match input {
        Input::SharedSurface {
            protocol,
            environment,
            sequence,
        } => {
            checked(protocol.validate())?;
            (
                checked(serde_json::to_value(protocol))?,
                environment,
                sequence,
            )
        }
        Input::ActiveSurface {
            protocol,
            environment,
            sequence,
        } => {
            checked(protocol.validate())?;
            (
                checked(serde_json::to_value(protocol))?,
                environment,
                sequence,
            )
        }
        _ => return Err(error("study", "expected a surface study")),
    };
    checked(s::EpisodeBits::from_index(*sequence))?;
    let mut logical = protocol;
    logical
        .as_object_mut()
        .expect("protocol object")
        .remove("ids");
    let declared = fixture()["studies"][name(input.study())]["settings"]
        .as_array()
        .expect("settings")
        .iter()
        .any(|setting| {
            let mut p = setting["protocol"].clone();
            p.as_object_mut().expect("protocol object").remove("ids");
            p == logical && setting["environment"] == json!(environment)
        });
    if !declared {
        return Err(error(
            "protocol",
            "protocol/environment is not a declared original control",
        ));
    }
    Ok(())
}
pub(crate) fn descriptors() -> Vec<StudyDescriptor> {
    [StudyId::SharedSurface,StudyId::ActiveSurface].into_iter().map(|id| {
        let data=&fixture()["studies"][name(id)];
        StudyDescriptor {id,family:StudyFamily::Surface,title:if id==StudyId::SharedSurface {"Shared surface"} else {"Active surface experiments"}.into(),supplied:"Original role schedule, four-model catalog, opaque IDs, exact priors, and paid physical operations.".into(),question:"What can each Agent infer from its own chronological observations?".into(),default_input:json!({"study":name(id),"protocol":data["settings"][0]["protocol"],"environment":data["settings"][0]["environment"],"sequence":0}),controls:json!({"settings":data["settings"],"sequence":{"min":0,"max":255},"ids":"Opaque identifiers may be consistently renamed"})}
    }).collect()
}
pub fn run(input: &Input) -> Result<EpisodeRecord> {
    record::serialized_size(input, super::MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    let (checkpoints, payload) = match input {
        Input::SharedSurface {
            protocol,
            environment,
            sequence,
        } => shared(protocol, *environment, *sequence)?,
        Input::ActiveSurface {
            protocol,
            environment,
            sequence,
        } => active(protocol, *environment, *sequence)?,
        _ => return Err(error("study", "expected a surface study")),
    };
    let result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: input.study(),
        rules_identity: rules_identity(input.study())?,
        input: checked(serde_json::to_value(input))?,
        semantics: Semantics::Trajectory,
        checkpoints,
        payload,
    };
    record::check_bounds(&result)?;
    Ok(result)
}

/// The last actual paid Read is an observation, never a claim about current storage.
fn opaque_surface<'a>(entries: impl Iterator<Item = &'a s::LocalEntry>) -> Result<Value> {
    let mut last = None;
    for entry in entries {
        if let s::LocalEntry::Action(event) = entry {
            if let s::Outcome::Read(symbol) = event.outcome {
                last = Some(
                    json!({"symbol":lossless_value(&symbol)?,"observed_at":lossless_value(&s::Checkpoint::AfterSlot(event.position))?}),
                );
            }
        }
    }
    Ok(json!({"kind":"opaque_surface","last_observation":last,"current_state":"unobserved"}))
}
fn certainty(
    belief: Option<&s::Belief>,
    prior: s::OwnPrior,
    empty: bool,
    clock: &Value,
    spent: u8,
) -> Result<Option<Value>> {
    let Some(belief) = belief else {
        return Ok(None);
    };
    let model = s::Mechanism::ALL.into_iter().find(|m| {
        let p = belief.models[m.index()];
        p.numerator == p.denominator
    });
    Ok(model.map(|m|json!({"model":m,"source":if empty && matches!(prior,s::OwnPrior::PointMass(_)) {"Supplied"} else {"Observed"},"checkpoint":clock,"spent":spent.to_string()})))
}

/// This world only projects already-authorized original events. Reads on discarded
/// clones expose physical fields to Researcher without adding costs or history.
struct PhysicalDisplay {
    world: s::World,
    environment: s::Environment,
    phase: Option<s::Phase>,
    credits: [u8; 2],
    last_after: Option<s::Position>,
}
impl PhysicalDisplay {
    fn new(environment: s::Environment, ids: &s::Ids) -> Self {
        Self {
            world: s::World::new(environment, ids.clone()),
            environment,
            phase: None,
            credits: [48; 2],
            last_after: None,
        }
    }
    fn leave_after(&mut self) {
        if self.last_after.take().is_some_and(|p| p.slot == 4) {
            self.world.finish_round();
        }
    }
    fn reset(&mut self, phase: s::Phase) {
        if self.phase != Some(phase) {
            self.world.reset(phase);
            self.phase = Some(phase);
        }
    }
    fn event(&mut self, role: s::Role, event: &s::Event, bits: &s::EpisodeBits) -> Result<()> {
        let target = match event.position.phase {
            s::Phase::Calibration => None,
            s::Phase::Live { trial } => Some(if role == s::Role::A {
                bits.0[usize::from(trial)].1
            } else {
                bits.0[usize::from(trial)].0
            }),
        };
        let actual = checked(self.world.apply(
            role,
            event.position,
            &event.action,
            target,
            self.credits[role.index()],
        ))?;
        if actual != *event {
            return Err(error(
                "surface",
                "display replay event differs from original evaluator",
            ));
        }
        self.credits[role.index()] = event.credits_after;
        self.last_after = Some(event.position);
        Ok(())
    }
    fn snapshot(&self) -> Result<Value> {
        let mut fields = Vec::new();
        for role in [s::Role::A, s::Role::B] {
            let position = s::Position {
                phase: self.phase.unwrap_or(s::Phase::Calibration),
                round: 1,
                slot: if role == s::Role::A { 1 } else { 2 },
            };
            let probe =
                checked(
                    self.world
                        .clone()
                        .apply(role, position, &s::Action::Read, None, 48),
                )?;
            let s::Outcome::Read(symbol) = probe.outcome else {
                return Err(error("surface", "display probe was not a read"));
            };
            fields.push(json!({"visible_to":role,"symbol":symbol,"lineage":lossless_value(&self.world.read_lineage(role))?}));
        }
        Ok(
            json!({"label":"Researcher","environment":self.environment,"topology":if self.environment==s::Environment::InFamily(s::Mechanism::PrivatePersistent) {"private_fields"} else {"shared_field"},"fields":fields,"credits":self.credits.map(|v|v.to_string())}),
        )
    }
}
fn append<T: Serialize>(
    out: &mut Vec<Checkpoint>,
    clock: &T,
    kind: &str,
    public: Value,
    local: BTreeMap<String, Value>,
    researcher: Value,
) -> Result<()> {
    record::push_checkpoint(
        out,
        Checkpoint {
            index: 0,
            clock: lossless_value(clock)?,
            kind: kind.into(),
            public,
            local,
            researcher: Some(researcher),
        },
    )
}
fn shared(
    protocol: &s::Protocol,
    environment: s::Environment,
    sequence: u16,
) -> Result<(Vec<Checkpoint>, Value)> {
    let ensemble = checked(s::Ensemble::build(protocol))?;
    let bits = checked(s::EpisodeBits::from_index(sequence))?;
    let episode = checked(s::run_episode(protocol, environment, &bits, &ensemble))?;
    let mut clocks = vec![s::Checkpoint::EpisodeStart];
    for (_, p) in checked(protocol.schedule())? {
        clocks.extend([s::Checkpoint::Action(p), s::Checkpoint::AfterSlot(p)]);
        if let s::Phase::Live { trial } = p.phase {
            if p.round == 3 && p.slot == 4 {
                clocks.push(s::Checkpoint::Prediction { trial });
            }
        }
    }
    let mut out = Vec::new();
    let mut physical = PhysicalDisplay::new(environment, &protocol.ids);
    let mut discoveries: [Option<Value>; 2] = [None, None];
    for clock in clocks {
        if let s::Checkpoint::Action(p) = clock {
            physical.leave_after();
            physical.reset(p.phase);
        }
        if matches!(clock, s::Checkpoint::Prediction { .. }) {
            physical.leave_after();
        }
        let event = if let s::Checkpoint::AfterSlot(p) = clock {
            episode.privileged.iter().find(|e| e.event.position == p)
        } else {
            None
        };
        if let Some(event) = event {
            physical.event(event.role, &event.event, &bits)?;
        }
        let mut local = BTreeMap::new();
        for role in [s::Role::A, s::Role::B] {
            // A short trace is possible when the evaluator fails first for the other role.
            let step = episode.local[role.index()]
                .steps
                .iter()
                .find(|step| step.prefix.checkpoint == clock);
            if let Some(step) = step {
                let inferred = match ensemble.infer(&step.prefix) {
                    Ok(b) => Some(b),
                    Err(s::Error::UnsupportedHistory { .. }) => None,
                    Err(e) => return Err(error("surface", e.to_string())),
                };
                if inferred != step.belief {
                    return Err(error(
                        "surface",
                        "original shared local belief disagrees with full-prefix inference",
                    ));
                }
                if discoveries[role.index()].is_none() {
                    discoveries[role.index()] = certainty(
                        step.belief.as_ref(),
                        step.prefix.own_prior,
                        step.prefix.entries.is_empty(),
                        &lossless_value(&clock)?,
                        48 - step.credits,
                    )?;
                }
                let bit = step.prefix.entries.iter().rev().find_map(|e| {
                    if let s::LocalEntry::PrivateBit { bit, .. } = e {
                        Some(*bit)
                    } else {
                        None
                    }
                });
                local.insert(protocol.ids.agents[role.index()].clone(),json!({"prefix":lossless_value(&step.prefix)?,"belief":lossless_value(&step.belief)?,"credits":step.credits.to_string(),"private_bit":bit,"surface":opaque_surface(step.prefix.entries.iter())?,"decision":lossless_value(&step.decision)?,"prediction":step.prediction,"catalog_discovery":discoveries[role.index()],"inference_status":if step.belief.is_some() {"supported"} else {"unsupported"}}));
            } else if let Some(previous) = out
                .last()
                .and_then(|c: &Checkpoint| c.local.get(&protocol.ids.agents[role.index()]))
            {
                local.insert(protocol.ids.agents[role.index()].clone(), previous.clone());
            }
        }
        let failed = episode
            .failure
            .as_ref()
            .is_some_and(|f| f.checkpoint == clock);
        let terminal = failed || clock == s::Checkpoint::Prediction { trial: 3 };
        let mut research = physical.snapshot()?;
        research["event"] = lossless_value(&event)?;
        append(
            &mut out,
            &clock,
            match clock {
                s::Checkpoint::EpisodeStart => "episode_start",
                s::Checkpoint::Action(_) => "before_slot",
                s::Checkpoint::AfterSlot(_) => "after_slot",
                s::Checkpoint::Prediction { .. } => "prediction",
            },
            json!({"reset_phase":lossless_value(&physical.phase)?,"status":if terminal {"stopped"} else {"running"}}),
            local,
            research,
        )?;
        if terminal {
            break;
        }
    }
    Ok((out, lossless_value(&episode)?))
}

fn active(
    protocol: &a::Protocol,
    environment: s::Environment,
    sequence: u16,
) -> Result<(Vec<Checkpoint>, Value)> {
    let nominal = match environment {
        s::Environment::InFamily(m) => m,
        s::Environment::DataFlip => s::Mechanism::SharedPersistent,
    };
    let prior = if protocol.policy == a::PolicyKind::Known {
        s::OwnPrior::PointMass(nominal)
    } else {
        s::OwnPrior::Uniform
    };
    let policy = checked(a::CompiledPolicy::build(protocol, prior))?;
    let replay = checked(a::Replay::build(protocol, &policy))?;
    let episode = checked(a::run_episode(
        protocol,
        environment,
        sequence,
        &policy,
        &replay,
    ))?;
    let bits = checked(s::EpisodeBits::from_index(sequence))?;
    let mut priors = [s::OwnPrior::PointMass(nominal); 2];
    priors[protocol.experimenter.index()] = prior;
    let mut state = checked(a::EpisodeState::new(
        protocol,
        environment,
        bits.clone(),
        priors,
    ))?;
    let mut physical = PhysicalDisplay::new(environment, &protocol.ids);
    let mut out = Vec::new();
    let mut selected = false;
    let mut decision_index = 0;
    let mut event_index = 0;
    let mut discoveries: [Option<Value>; 2] = [None, None];
    let mut last_supported: [Option<s::Belief>; 2] = [None, None];
    let mut last_decision: Option<&a::Decision> = None;
    loop {
        let own = state.prefix(protocol.experimenter);
        let clock = own.checkpoint;
        // Trial routine choice is private; the public clock is the first owned slot.
        let public_clock = match clock {
            a::Checkpoint::Boundary(b @ a::Boundary::TrialChoice { .. }) => {
                a::Checkpoint::BeforeSlot(checked(b.trial_position(protocol.experimenter))?)
            }
            c => c,
        };
        let mut local = BTreeMap::new();
        let mut unsupported = false;
        for role in [s::Role::A, s::Role::B] {
            let prefix = state.prefix(role);
            let belief = match replay.infer(prefix) {
                Ok(b) => {
                    last_supported[role.index()] = Some(b.clone());
                    Some(b)
                }
                Err(a::Error::UnsupportedHistory { .. }) => {
                    unsupported = true;
                    None
                }
                Err(e) => return Err(error("surface", e.to_string())),
            };
            if discoveries[role.index()].is_none() {
                discoveries[role.index()] = certainty(
                    belief.as_ref(),
                    prefix.own_prior,
                    prefix.entries.is_empty(),
                    &lossless_value(&prefix.checkpoint)?,
                    48 - state.credits(role),
                )?;
            }
            let bit = prefix.entries.iter().rev().find_map(|entry| {
                if let a::Entry::Physical(s::LocalEntry::PrivateBit { bit, .. }) = entry {
                    Some(*bit)
                } else {
                    None
                }
            });
            let decision = if role == protocol.experimenter && selected {
                last_decision
            } else {
                None
            };
            let prediction = if let a::Checkpoint::Prediction { trial } = clock {
                episode.predictions[role.index()]
                    .get(usize::from(trial))
                    .copied()
            } else {
                None
            };
            local.insert(protocol.ids.agents[role.index()].clone(),json!({"prefix":lossless_value(prefix)?,"belief":lossless_value(&belief)?,"last_supported_belief":if belief.is_none() {lossless_value(&last_supported[role.index()])?} else {Value::Null},"credits":state.credits(role).to_string(),"private_bit":bit,"surface":opaque_surface(prefix.entries.iter().filter_map(|e|if let a::Entry::Physical(e)=e {Some(e)} else {None}))?,"decision":lossless_value(&decision)?,"decision_label":decision.map(|_|"Expected remaining net utility under the actual continuation policy"),"prediction":prediction,"catalog_discovery":discoveries[role.index()],"inference_status":if belief.is_some() {"supported"} else {"unsupported"}}));
        }
        let stopped = unsupported || clock == a::Checkpoint::Finished;
        let probe_stop = own.entries.iter().find_map(|e| {
            if let a::Entry::PublicProbeStop { completed } = e {
                Some(completed.to_string())
            } else {
                None
            }
        });
        let kind = if selected {
            "after_choice"
        } else {
            match clock {
                a::Checkpoint::Boundary(_) => "before_choice",
                a::Checkpoint::BeforeSlot(_) => "before_slot",
                a::Checkpoint::AfterSlot(_) => "after_slot",
                a::Checkpoint::Prediction { .. } => "prediction",
                a::Checkpoint::Finished => "finished",
            }
        };
        let mut research = physical.snapshot()?;
        research["event"] = if let a::Checkpoint::AfterSlot(p) = clock {
            lossless_value(&episode.privileged.iter().find(|e| e.event.position == p))?
        } else {
            Value::Null
        };
        append(
            &mut out,
            &public_clock,
            kind,
            json!({"reset_phase":lossless_value(&physical.phase)?,"probe_stop":probe_stop,"status":if stopped {"stopped"} else {"running"}}),
            local,
            research,
        )?;
        if stopped {
            if [state.prefix(s::Role::A), state.prefix(s::Role::B)] != episode.histories.each_ref()
                || event_index != episode.privileged.len()
                || decision_index != episode.decisions.len()
            {
                return Err(error(
                    "surface",
                    "display replay differs from original terminal history",
                ));
            }
            break;
        }
        if matches!(clock, a::Checkpoint::Boundary(_)) && !selected {
            let trace = episode
                .decisions
                .get(decision_index)
                .ok_or_else(|| error("surface", "missing original decision"))?;
            if trace.prefix != *own {
                return Err(error("surface", "decision prefix differs from original"));
            }
            checked(a::select_choice(&mut state, trace.decision.choice))?;
            decision_index += 1;
            selected = true;
            last_decision = Some(&trace.decision);
            continue;
        }
        physical.leave_after();
        let step = checked(a::advance_one(&mut state))?;
        let phase = state.prefix(s::Role::A).entries.iter().rev().find_map(|e| {
            if let a::Entry::Physical(s::LocalEntry::Reset { phase }) = e {
                Some(*phase)
            } else {
                None
            }
        });
        if let Some(phase) = phase {
            physical.reset(phase);
        }
        for (role, event) in step.events {
            let original = episode.privileged.get(event_index).ok_or_else(|| {
                error("surface", "display advanced past original charged failure")
            })?;
            if original.role != role || original.event != event {
                return Err(error("surface", "display event differs from original"));
            }
            physical.event(role, &event, &bits)?;
            event_index += 1;
        }
        selected = false;
        last_decision = None;
    }
    Ok((out, lossless_value(&episode)?))
}
