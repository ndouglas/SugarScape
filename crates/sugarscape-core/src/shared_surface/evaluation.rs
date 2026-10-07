use super::*;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalStep {
    pub prefix: LocalPrefix,
    pub belief: Option<Belief>,
    pub decision: Option<Action>,
    pub prediction: Option<bool>,
    pub credits: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentTrace {
    pub role: Role,
    pub own_prior: OwnPrior,
    pub steps: Vec<LocalStep>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivilegedStep {
    pub role: Role,
    pub event: Event,
    pub read_lineage: Option<Lineage>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureTrace {
    pub role: Role,
    pub checkpoint: Checkpoint,
    pub prefix: LocalPrefix,
    pub prior: OwnPrior,
    pub last_supported_belief: Belief,
    pub spent: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentTerminal {
    pub predictions: [bool; 4],
    pub correct: [bool; 4],
    pub gross_reward: i64,
    pub net_utility: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Property {
    Visibility,
    Retention,
    UsefulChannel,
    TrueMechanism,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Certainty {
    Supplied,
    Acquired { checkpoint: Checkpoint, credits: u8 },
    Censored,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    pub property: Property,
    pub certainty: Certainty,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PriorOrigin {
    Uniform,
    KnownControl,
    RetainedOldAcquisition { old_mechanism: Mechanism },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMetrics {
    pub spent: u8,
    pub attempted_sends: u16,
    pub reads: u16,
    pub decoded_data: u16,
    pub other_agent_task_lineage_reads: u16,
    pub inspections: u16,
    pub calibration_writes: u16,
    pub calibration_reads: u16,
    pub live_reads: u16,
    pub waits: u16,
    pub discoveries: Vec<Discovery>,
    pub terminal: Option<AgentTerminal>,
    pub true_model_probability: Option<Probability>,
    pub unique_true_identification: Option<bool>,
    pub final_belief: Option<Belief>,
    pub prior_origin: PriorOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    #[serde(deserialize_with = "deserialize_sequence")]
    pub sequence: u16,
    pub local: [AgentTrace; 2],
    pub privileged: Vec<PrivilegedStep>,
    pub metrics: [AgentMetrics; 2],
    pub failure: Option<FailureTrace>,
    pub group_net_utility: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentAggregate {
    pub expected_credits: ScoreFraction,
    pub expected_accuracy: Option<Probability>,
    pub expected_correct_count: Option<ScoreFraction>,
    pub expected_net: Option<ScoreFraction>,
    pub expected_gross: Option<ScoreFraction>,
    pub benefit_gross: Option<ScoreFraction>,
    pub benefit_net: Option<ScoreFraction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aggregate {
    pub total: u16,
    pub valid: u16,
    pub failed: u16,
    pub failure_mass: Probability,
    pub agents: [AgentAggregate; 2],
    pub expected_group_net: Option<ScoreFraction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sensitivity {
    #[serde(deserialize_with = "deserialize_sequence")]
    pub sequence: u16,
    #[serde(deserialize_with = "deserialize_sequence")]
    pub paired_sequence: u16,
    #[serde(deserialize_with = "deserialize_trial")]
    pub trial: u8,
    pub sender: Role,
    pub read_changed: Option<bool>,
    pub posterior_changed: Option<bool>,
    pub prediction_changed: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub protocol: Protocol,
    pub environment: Environment,
    pub episodes: Vec<Episode>,
    pub aggregate: Aggregate,
    pub sensitivity: Vec<Sensitivity>,
}
impl Panel {
    pub fn episode_count(&self) -> usize {
        self.episodes.len()
    }
    pub fn agent_expected_credits(&self, role: Role) -> ScoreFraction {
        self.aggregate.agents[role.index()].expected_credits
    }
    pub fn agent_expected_accuracy(&self, role: Role) -> Option<Probability> {
        self.aggregate.agents[role.index()].expected_accuracy
    }
    pub fn agent_expected_correct_count(&self, role: Role) -> Option<ScoreFraction> {
        self.aggregate.agents[role.index()].expected_correct_count
    }
    pub fn agent_expected_net(&self, role: Role) -> Option<ScoreFraction> {
        self.aggregate.agents[role.index()].expected_net
    }
}
/// Evaluate one actual trajectory against an immutable, independently queried ensemble.
pub fn run_episode(
    protocol: &Protocol,
    environment: Environment,
    bits: &EpisodeBits,
    ensemble: &Ensemble,
) -> Result<Episode, Error> {
    protocol.validate()?;
    let mut evaluator = Evaluator::new(protocol, environment, bits);
    if !evaluator.checkpoint(Checkpoint::EpisodeStart, ensemble)? {
        return Ok(evaluator.episode);
    }
    for (role, position) in protocol.schedule()? {
        if !evaluator.slot(role, position, ensemble)? {
            return Ok(evaluator.episode);
        }
        if position.slot == 4 && position.round == 3 {
            if let Phase::Live { trial } = position.phase {
                if !evaluator.checkpoint(Checkpoint::Prediction { trial }, ensemble)? {
                    return Ok(evaluator.episode);
                }
                for role in [Role::A, Role::B] {
                    let prediction = predict(&evaluator.view(role)?)?;
                    evaluator.predictions[role.index()][usize::from(trial)] = prediction;
                    evaluator.episode.local[role.index()]
                        .steps
                        .last_mut()
                        .ok_or_else(|| {
                            Error::InvalidReport("missing prediction checkpoint".into())
                        })?
                        .prediction = Some(prediction);
                }
            }
        }
    }
    evaluator.finish()?;
    Ok(evaluator.episode)
}

struct Evaluator<'a> {
    protocol: &'a Protocol,
    environment: Environment,
    bits: &'a EpisodeBits,
    world: World,
    prefixes: [LocalPrefix; 2],
    last_supported: [Option<Belief>; 2],
    private_bits: [Option<bool>; 2],
    phase: Option<Phase>,
    predictions: [[bool; 4]; 2],
    episode: Episode,
}
impl<'a> Evaluator<'a> {
    fn new(protocol: &'a Protocol, environment: Environment, bits: &'a EpisodeBits) -> Self {
        let priors = [Role::A, Role::B].map(|role| own_prior(protocol, environment, role));
        let prefixes = std::array::from_fn(|i| LocalPrefix {
            role: if i == 0 { Role::A } else { Role::B },
            ids: protocol.ids.clone(),
            checkpoint: Checkpoint::EpisodeStart,
            own_prior: priors[i],
            entries: Vec::new(),
        });
        let local = std::array::from_fn(|i| AgentTrace {
            role: prefixes[i].role,
            own_prior: priors[i],
            steps: Vec::new(),
        });
        let metrics = std::array::from_fn(|i| AgentMetrics {
            spent: 0,
            attempted_sends: 0,
            reads: 0,
            decoded_data: 0,
            other_agent_task_lineage_reads: 0,
            inspections: 0,
            calibration_writes: 0,
            calibration_reads: 0,
            live_reads: 0,
            waits: 0,
            discoveries: [
                Property::Visibility,
                Property::Retention,
                Property::UsefulChannel,
                Property::TrueMechanism,
            ]
            .map(|property| Discovery {
                property,
                certainty: Certainty::Censored,
            })
            .to_vec(),
            terminal: None,
            true_model_probability: None,
            unique_true_identification: None,
            final_belief: None,
            prior_origin: match protocol.prior_mode {
                PriorMode::Stale(old_mechanism) => {
                    PriorOrigin::RetainedOldAcquisition { old_mechanism }
                }
                _ if protocol.pair.roles[i] == Knowledge::Known => PriorOrigin::KnownControl,
                _ => PriorOrigin::Uniform,
            },
        });
        Self {
            protocol,
            environment,
            bits,
            world: World::new(environment, protocol.ids.clone()),
            prefixes,
            last_supported: [None, None],
            private_bits: [None, None],
            phase: None,
            predictions: [[false; 4]; 2],
            episode: Episode {
                sequence: bits.index(),
                local,
                privileged: Vec::new(),
                metrics,
                failure: None,
                group_net_utility: None,
            },
        }
    }

    fn checkpoint(&mut self, checkpoint: Checkpoint, ensemble: &Ensemble) -> Result<bool, Error> {
        // Advance both public clocks before any inference; actual private histories remain separate.
        for prefix in &mut self.prefixes {
            prefix.checkpoint = checkpoint;
        }
        for role in [Role::A, Role::B] {
            let i = role.index();
            let prefix = self.prefixes[i].clone();
            let credits = INITIAL_CREDITS
                .checked_sub(self.episode.metrics[i].spent)
                .ok_or(Error::ArithmeticOverflow)?;
            match ensemble.infer(&prefix) {
                Ok(belief) => {
                    self.record_discoveries(role, &belief)?;
                    self.last_supported[i] = Some(belief.clone());
                    self.episode.local[i].steps.push(LocalStep {
                        prefix,
                        belief: Some(belief),
                        decision: None,
                        prediction: None,
                        credits,
                    });
                }
                Err(Error::UnsupportedHistory { prefix }) => {
                    self.episode.local[i].steps.push(LocalStep {
                        prefix: prefix.clone(),
                        belief: None,
                        decision: None,
                        prediction: None,
                        credits,
                    });
                    self.failed_episode(role, prefix)?;
                    return Ok(false);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }

    fn view(&self, role: Role) -> Result<LocalView, Error> {
        let i = role.index();
        Ok(LocalView {
            prefix: self.prefixes[i].clone(),
            credits: INITIAL_CREDITS
                .checked_sub(self.episode.metrics[i].spent)
                .ok_or(Error::ArithmeticOverflow)?,
            private_bit: self.private_bits[i],
            belief: self.last_supported[i]
                .clone()
                .ok_or_else(|| Error::InvalidReport("missing supported local belief".into()))?,
        })
    }

    fn slot(&mut self, role: Role, position: Position, ensemble: &Ensemble) -> Result<bool, Error> {
        if self.phase != Some(position.phase) {
            self.world.reset(position.phase);
            self.phase = Some(position.phase);
            for i in 0..2 {
                self.prefixes[i].entries.push(LocalEntry::Reset {
                    phase: position.phase,
                });
                self.private_bits[i] = match position.phase {
                    Phase::Calibration => None,
                    Phase::Live { trial } => {
                        let (x, y) = self.bits.0[usize::from(trial)];
                        let bit = if i == 0 { x } else { y };
                        self.prefixes[i]
                            .entries
                            .push(LocalEntry::PrivateBit { trial, bit });
                        Some(bit)
                    }
                };
            }
        }
        if !self.checkpoint(Checkpoint::Action(position), ensemble)? {
            return Ok(false);
        }
        let action = self.advance_local_controller(role)?;
        let target = match position.phase {
            Phase::Calibration => None,
            Phase::Live { trial } => {
                let (x, y) = self.bits.0[usize::from(trial)];
                Some(if role == Role::A { y } else { x })
            }
        };
        let read_lineage = if action == Action::Read {
            self.world.read_lineage(role)
        } else {
            None
        };
        let event = self
            .world
            .apply(role, position, &action, target, self.view(role)?.credits)?;
        self.prefixes[role.index()]
            .entries
            .push(LocalEntry::Action(event.clone()));
        self.record_paid(role, &event, read_lineage.as_ref())?;
        self.episode.privileged.push(PrivilegedStep {
            role,
            event,
            read_lineage,
        });
        if !self.checkpoint(Checkpoint::AfterSlot(position), ensemble)? {
            return Ok(false);
        }
        if position.slot == 4 {
            self.world.finish_round();
        }
        Ok(true)
    }

    fn advance_local_controller(&mut self, role: Role) -> Result<Action, Error> {
        let action = decide(&self.view(role)?, self.protocol.pair.roles[role.index()])?.action;
        self.episode.local[role.index()]
            .steps
            .last_mut()
            .ok_or_else(|| Error::InvalidReport("missing decision checkpoint".into()))?
            .decision = Some(action);
        Ok(action)
    }

    fn failed_episode(&mut self, role: Role, prefix: LocalPrefix) -> Result<(), Error> {
        self.episode.failure = Some(FailureTrace {
            role,
            checkpoint: prefix.checkpoint,
            prior: prefix.own_prior,
            prefix,
            last_supported_belief: self.last_supported[role.index()].clone().ok_or_else(|| {
                Error::InvalidReport(
                    "unsupported initial prior has no previous supported belief".into(),
                )
            })?,
            spent: self.episode.metrics.each_ref().map(|m| m.spent),
        });
        // Earlier trial predictions remain in traces, but incomplete episode terminals stay unavailable.
        Ok(())
    }

    fn record_paid(
        &mut self,
        role: Role,
        event: &Event,
        lineage: Option<&Lineage>,
    ) -> Result<(), Error> {
        let m = &mut self.episode.metrics[role.index()];
        m.spent = INITIAL_CREDITS
            .checked_sub(event.credits_after)
            .ok_or(Error::ArithmeticOverflow)?;
        fn increment(count: &mut u16) -> Result<(), Error> {
            *count = count.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
            Ok(())
        }
        match event.action {
            Action::Write(_) if event.position.phase == Phase::Calibration => {
                increment(&mut m.calibration_writes)?
            }
            Action::Write(_) => increment(&mut m.attempted_sends)?,
            Action::Wait => increment(&mut m.waits)?,
            Action::InspectOwnTarget => increment(&mut m.inspections)?,
            Action::Read => {
                increment(&mut m.reads)?;
                match event.position.phase {
                    Phase::Calibration => increment(&mut m.calibration_reads)?,
                    Phase::Live { trial } => {
                        increment(&mut m.live_reads)?;
                        if matches!(event.outcome, Outcome::Read(Symbol::Data0 | Symbol::Data1)) {
                            increment(&mut m.decoded_data)?;
                            if lineage.is_some_and(|origin| {
                                origin.writer != role && origin.task_trial == Some(trial)
                            }) {
                                increment(&mut m.other_agent_task_lineage_reads)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn record_discoveries(&mut self, role: Role, belief: &Belief) -> Result<(), Error> {
        if self.protocol.pair.roles[role.index()] == Knowledge::NoCommunication {
            return Ok(());
        }
        let m = &mut self.episode.metrics[role.index()];
        let sp = belief.models[0];
        let probabilities = [
            Some(sp.checked_add(belief.models[1])?),
            Some(sp.checked_add(belief.models[2])?),
            Some(sp),
            match self.environment {
                Environment::InFamily(model) => Some(belief.models[model.index()]),
                Environment::DataFlip => None,
            },
        ];
        for (discovery, probability) in m.discoveries.iter_mut().zip(probabilities) {
            if discovery.certainty == Certainty::Censored
                && probability == Some(Probability::new(1, 1)?)
            {
                discovery.certainty =
                    if self.prefixes[role.index()].checkpoint == Checkpoint::EpisodeStart {
                        Certainty::Supplied
                    } else {
                        Certainty::Acquired {
                            checkpoint: self.prefixes[role.index()].checkpoint,
                            credits: m.spent,
                        }
                    };
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        let mut group = 0i64;
        for role in [Role::A, Role::B] {
            let i = role.index();
            let predictions = self.predictions[i];
            let correct = std::array::from_fn(|t| {
                predictions[t]
                    == if role == Role::A {
                        self.bits.0[t].1
                    } else {
                        self.bits.0[t].0
                    }
            });
            let count = i64::try_from(correct.iter().filter(|&&c| c).count())
                .map_err(|_| Error::ArithmeticOverflow)?;
            let gross_reward = count.checked_mul(12).ok_or(Error::ArithmeticOverflow)?;
            let net_utility = gross_reward
                .checked_sub(i64::from(self.episode.metrics[i].spent))
                .ok_or(Error::ArithmeticOverflow)?;
            group = group
                .checked_add(net_utility)
                .ok_or(Error::ArithmeticOverflow)?;
            let m = &mut self.episode.metrics[i];
            m.terminal = Some(AgentTerminal {
                predictions,
                correct,
                gross_reward,
                net_utility,
            });
            let belief = self.last_supported[i]
                .clone()
                .ok_or_else(|| Error::InvalidReport("missing terminal belief".into()))?;
            if self.protocol.pair.roles[i] != Knowledge::NoCommunication {
                if let Environment::InFamily(model) = self.environment {
                    m.true_model_probability = Some(belief.models[model.index()]);
                    m.unique_true_identification =
                        Some(belief.models[model.index()] == Probability::new(1, 1)?);
                }
            }
            m.final_belief = Some(belief);
        }
        self.episode.group_net_utility = Some(group);
        Ok(())
    }
}

fn own_prior(protocol: &Protocol, environment: Environment, role: Role) -> OwnPrior {
    match protocol.prior_mode {
        PriorMode::Stale(model) => OwnPrior::PointMass(model),
        PriorMode::RestartUniform => OwnPrior::Uniform,
        PriorMode::Treatment if protocol.pair.roles[role.index()] == Knowledge::Known => {
            OwnPrior::PointMass(match environment {
                Environment::InFamily(model) => model,
                Environment::DataFlip => Mechanism::SharedPersistent,
            })
        }
        PriorMode::Treatment => OwnPrior::Uniform,
    }
}

/// All 256 sequences have equal unconditional mass; no unsuccessful row is removed.
pub fn evaluate_panel(protocol: &Protocol, environment: Environment) -> Result<Panel, Error> {
    protocol.validate()?;
    let ensemble = Ensemble::build(protocol)?;
    evaluate_panel_with_ensemble(protocol, environment, &ensemble)
}

/// Evaluate the complete 256-row distribution using a caller-owned immutable ensemble.
/// The ensemble must have been built for this exact protocol; inference validates
/// every queried local prefix, including identifiers and supplied prior shape.
/// This permits diagnostics to reuse one ensemble across actual environments.
pub fn evaluate_panel_with_ensemble(
    protocol: &Protocol,
    environment: Environment,
    ensemble: &Ensemble,
) -> Result<Panel, Error> {
    protocol.validate()?;
    let mut episodes = Vec::with_capacity(256);
    for sequence in 0..256 {
        episodes.push(run_episode(
            protocol,
            environment,
            &EpisodeBits::from_index(sequence)?,
            ensemble,
        )?);
    }
    let aggregate = aggregate_episodes(protocol, &episodes)?;
    let sensitivity = paired_sensitivity(&episodes)?;
    Ok(Panel {
        protocol: protocol.clone(),
        environment,
        episodes,
        aggregate,
        sensitivity,
    })
}

fn aggregate_episodes(protocol: &Protocol, episodes: &[Episode]) -> Result<Aggregate, Error> {
    let total = u16::try_from(episodes.len()).map_err(|_| Error::ArithmeticOverflow)?;
    if total != 256 {
        return Err(Error::InvalidReport(
            "panel requires all 256 bit sequences".into(),
        ));
    }
    let failed = u16::try_from(episodes.iter().filter(|e| e.failure.is_some()).count())
        .map_err(|_| Error::ArithmeticOverflow)?;
    let baseline_credits = i64::from(protocol.calibration_rounds)
        .checked_mul(2)
        .and_then(|c| c.checked_add(36))
        .ok_or(Error::ArithmeticOverflow)?;
    let baseline_net = 48i64
        .checked_sub(baseline_credits)
        .ok_or(Error::ArithmeticOverflow)?;
    let mut agents = Vec::with_capacity(2);
    for i in 0..2 {
        let mut spent = 0i64;
        let mut correct = 0i64;
        let mut gross = 0i64;
        let mut net = 0i64;
        for episode in episodes {
            spent = spent
                .checked_add(i64::from(episode.metrics[i].spent))
                .ok_or(Error::ArithmeticOverflow)?;
            if let Some(terminal) = &episode.metrics[i].terminal {
                correct = correct
                    .checked_add(
                        i64::try_from(terminal.correct.iter().filter(|&&b| b).count())
                            .map_err(|_| Error::ArithmeticOverflow)?,
                    )
                    .ok_or(Error::ArithmeticOverflow)?;
                gross = gross
                    .checked_add(terminal.gross_reward)
                    .ok_or(Error::ArithmeticOverflow)?;
                net = net
                    .checked_add(terminal.net_utility)
                    .ok_or(Error::ArithmeticOverflow)?;
            } else if episode.failure.is_none() {
                return Err(Error::InvalidReport(
                    "successful episode is missing terminal metrics".into(),
                ));
            }
        }
        let denominator = u64::from(total);
        let mean = |value| ScoreFraction::new(value, denominator);
        agents.push(AgentAggregate {
            expected_credits: mean(spent)?,
            expected_accuracy: if failed == 0 {
                Some(Probability::new(
                    u64::try_from(correct).map_err(|_| Error::ArithmeticOverflow)?,
                    denominator
                        .checked_mul(4)
                        .ok_or(Error::ArithmeticOverflow)?,
                )?)
            } else {
                None
            },
            expected_correct_count: if failed == 0 {
                Some(mean(correct)?)
            } else {
                None
            },
            expected_net: if failed == 0 { Some(mean(net)?) } else { None },
            expected_gross: if failed == 0 {
                Some(mean(gross)?)
            } else {
                None
            },
            benefit_gross: if failed == 0 {
                Some(mean(
                    gross
                        .checked_sub(
                            48i64
                                .checked_mul(i64::from(total))
                                .ok_or(Error::ArithmeticOverflow)?,
                        )
                        .ok_or(Error::ArithmeticOverflow)?,
                )?)
            } else {
                None
            },
            benefit_net: if failed == 0 {
                Some(mean(
                    net.checked_sub(
                        baseline_net
                            .checked_mul(i64::from(total))
                            .ok_or(Error::ArithmeticOverflow)?,
                    )
                    .ok_or(Error::ArithmeticOverflow)?,
                )?)
            } else {
                None
            },
        });
    }
    let agents: [AgentAggregate; 2] = agents
        .try_into()
        .map_err(|_| Error::InvalidReport("expected two Agent aggregates".into()))?;
    let expected_group_net = if failed == 0 {
        Some(
            agents[0]
                .expected_net
                .ok_or_else(|| Error::InvalidReport("missing Agent net".into()))?
                .checked_add(
                    agents[1]
                        .expected_net
                        .ok_or_else(|| Error::InvalidReport("missing Agent net".into()))?,
                )?,
        )
    } else {
        None
    };
    Ok(Aggregate {
        total,
        valid: total.checked_sub(failed).ok_or(Error::ArithmeticOverflow)?,
        failed,
        failure_mass: Probability::new(u64::from(failed), u64::from(total))?,
        agents,
        expected_group_net,
    })
}

fn paired_sensitivity(episodes: &[Episode]) -> Result<Vec<Sensitivity>, Error> {
    if episodes.len() != 256
        || episodes
            .iter()
            .enumerate()
            .any(|(i, e)| usize::from(e.sequence) != i)
    {
        return Err(Error::InvalidReport(
            "sensitivity requires canonical exhaustive sequence order".into(),
        ));
    }
    let mut sensitivity = Vec::with_capacity(256 * 4 * 2);
    for episode in episodes {
        for sender in [Role::A, Role::B] {
            let recipient = if sender == Role::A { Role::B } else { Role::A };
            for trial in 0..TRIALS {
                let paired_sequence =
                    episode.sequence ^ (1 << (2 * usize::from(trial) + sender.index()));
                let paired = &episodes[usize::from(paired_sequence)];
                let read_position = Position {
                    phase: Phase::Live { trial },
                    round: if recipient == Role::A { 3 } else { 2 },
                    slot: if recipient == Role::A { 1 } else { 2 },
                };
                let read = |e: &Episode| {
                    e.privileged.iter().find_map(|s| {
                        if s.role == recipient && s.event.position == read_position {
                            if let Outcome::Read(symbol) = s.event.outcome {
                                Some(symbol)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                };
                let posterior = |e: &Episode| {
                    e.local[recipient.index()]
                        .steps
                        .iter()
                        .find(|s| s.prefix.checkpoint == Checkpoint::AfterSlot(read_position))
                        .and_then(|s| s.belief.as_ref())
                        .cloned()
                };
                let prediction = |e: &Episode| {
                    e.local[recipient.index()]
                        .steps
                        .iter()
                        .find(|s| s.prefix.checkpoint == Checkpoint::Prediction { trial })
                        .and_then(|s| s.prediction)
                };
                let read_changed = read(episode).zip(read(paired)).map(|(a, b)| a != b);
                // A posterior comparison refers to the applicable read; inspection-only paths have none.
                let posterior_changed = if read_changed.is_some() {
                    posterior(episode)
                        .zip(posterior(paired))
                        .map(|(a, b)| a != b)
                } else {
                    None
                };
                let prediction_changed = prediction(episode)
                    .zip(prediction(paired))
                    .map(|(a, b)| a != b);
                sensitivity.push(Sensitivity {
                    sequence: episode.sequence,
                    paired_sequence,
                    trial,
                    sender,
                    read_changed,
                    posterior_changed,
                    prediction_changed,
                });
            }
        }
    }
    Ok(sensitivity)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferOrigin {
    pub old_mechanism: Mechanism,
    pub acquisition: [AgentTrace; 2],
    pub posterior: [Belief; 2],
    pub spent: [u8; 2],
}
/// Record only the fixed old three-round calibration, with no old live task bits.
pub fn acquire_transfer_origin(
    old_mechanism: Mechanism,
    ids: &Ids,
) -> Result<TransferOrigin, Error> {
    let protocol = Protocol {
        calibration_rounds: 3,
        pair: Pair {
            roles: [Knowledge::Unknown; 2],
        },
        prior_mode: PriorMode::Treatment,
        ids: ids.clone(),
    };
    protocol.validate()?;
    let ensemble = Ensemble::build(&protocol)?;
    // Calibration never issues or consumes these task bits.
    let bits = EpisodeBits::from_index(0)?;
    let mut evaluator = Evaluator::new(&protocol, Environment::InFamily(old_mechanism), &bits);
    if !evaluator.checkpoint(Checkpoint::EpisodeStart, &ensemble)? {
        return Err(Error::InvalidReport(
            "old acquisition failed at EpisodeStart".into(),
        ));
    }
    for (role, position) in protocol.schedule()?.into_iter().take(12) {
        if !evaluator.slot(role, position, &ensemble)? {
            return Err(Error::InvalidReport(
                "in-family old calibration was unsupported".into(),
            ));
        }
    }
    let [Some(a), Some(b)] = evaluator.last_supported else {
        return Err(Error::InvalidReport(
            "old acquisition is missing beliefs".into(),
        ));
    };
    for belief in [&a, &b] {
        if belief.models[old_mechanism.index()] != Probability::new(1, 1)? {
            return Err(Error::InvalidReport(
                "old full calibration did not identify its mechanism".into(),
            ));
        }
    }
    Ok(TransferOrigin {
        old_mechanism,
        acquisition: evaluator.episode.local,
        posterior: [a, b],
        spent: evaluator.episode.metrics.each_ref().map(|m| m.spent),
    })
}

fn deserialize_sequence<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u16, D::Error> {
    let sequence = u16::deserialize(deserializer)?;
    EpisodeBits::from_index(sequence).map_err(serde::de::Error::custom)?;
    Ok(sequence)
}

fn deserialize_trial<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    let trial = u8::deserialize(deserializer)?;
    Phase::Live { trial }
        .validate()
        .map_err(serde::de::Error::custom)?;
    Ok(trial)
}
