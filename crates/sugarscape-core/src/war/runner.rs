//! Bounded, observational dispatch to literal World ticks or graph settlements.

use super::{
    config::{StudyInput, RECORD_SCHEMA},
    records::*,
    reference::reference_at,
    Checkpoint, Engagement,
};
use crate::{
    agent::Tribe,
    config::FieldError,
    stats::Snapshot,
    world::{DeathCause, World},
};
use std::collections::{BTreeMap, BTreeSet};

/// Emit immutable observations without exposing an engine or semantic RNG.
/// Failed writes are never retried; completed and emitted settlements are distinct.
#[allow(clippy::result_large_err)] // The approved failure contract owns its checkpoint.
pub fn run_to(
    input: &StudyInput,
    capture: &Capture,
    emit: &mut dyn FnMut(&RunRecord) -> Result<(), String>,
) -> Result<RunSummary, RunFailure> {
    validate(input, capture).map_err(invalid_input)?;
    let identity = input_identity(input).map_err(invalid_input)?;
    let mut output = Output {
        identity,
        emit,
        completed: 0,
        emitted: 0,
        requested: capture.retain_steps.iter().copied().collect(),
        retained: BTreeMap::new(),
    };
    match input {
        StudyInput::BookC {
            config,
            seed,
            max_steps,
        } => {
            let mut world = World::new(config.clone(), *seed)
                .map_err(|errors| invalid_input(format!("World construction: {errors:?}")))?;
            output
                .send(RunPayload::Header {
                    header: header(input),
                })
                .map_err(|detail| output.failure("output", detail, None, None))?;
            output
                .observe(0, ObservedFrame::Initial { counts: None })
                .map_err(|detail| output.failure("output", detail, Some(0), None))?;
            // World controls always request the supplied tick horizon, including
            // empty populations. Graph extinction rules never schedule World.
            for step in 1..=*max_steps {
                world.step();
                output.completed = world.tick;
                if world.tick != step {
                    return Err(output.failure(
                        "world_halted",
                        format!(
                            "World::step did not complete requested tick {step}; actual tick {}",
                            world.tick
                        ),
                        Some(step),
                        None,
                    ));
                }
                let frame = book_frame(&world).map_err(|detail| {
                    output.failure("invalid_observation", detail, Some(step), None)
                })?;
                output
                    .observe(step, ObservedFrame::Book { frame })
                    .map_err(|detail| output.failure("output", detail, Some(step), None))?;
            }
            let summary = output.summary(capture, None);
            output
                .send(RunPayload::Terminal {
                    summary: summary.clone(),
                })
                .map_err(|detail| output.failure("output", detail, None, None))?;
            Ok(summary)
        }
        StudyInput::ReciprocalGraph { config, seed } => {
            let mut engine = Engagement::new(config.clone(), *seed)
                .map_err(|errors| invalid_input(format!("engagement construction: {errors:?}")))?;
            output
                .send(RunPayload::Header {
                    header: header(input),
                })
                .map_err(|detail| {
                    output.failure("output", detail, None, Some(engine.checkpoint()))
                })?;
            output
                .observe(
                    0,
                    ObservedFrame::Initial {
                        counts: Some(engine.counts()),
                    },
                )
                .map_err(|detail| {
                    output.failure("output", detail, Some(0), Some(engine.checkpoint()))
                })?;
            loop {
                let frame = match engine.step() {
                    Ok(Some(frame)) => frame,
                    Ok(None) => break,
                    Err(failure) => {
                        return Err(output.failure(
                            "invalid_numeric",
                            format!("{}: {}", failure.issue.field, failure.issue.detail),
                            Some(failure.attempted_step),
                            Some(engine.checkpoint()),
                        ))
                    }
                };
                output.completed = frame.step;
                let (reference, reference_error) = match reference_at(config, frame.calendar_time) {
                    Ok(point) => (Some(point), None),
                    Err(error) => (None, Some(error)),
                };
                output
                    .observe(
                        frame.step,
                        ObservedFrame::Graph {
                            frame,
                            reference,
                            reference_error,
                        },
                    )
                    .map_err(|detail| {
                        output.failure(
                            "output",
                            detail,
                            Some(output.completed),
                            Some(engine.checkpoint()),
                        )
                    })?;
            }
            let summary = output.summary(capture, engine.ending().cloned());
            output
                .send(RunPayload::Terminal {
                    summary: summary.clone(),
                })
                .map_err(|detail| {
                    output.failure("output", detail, None, Some(engine.checkpoint()))
                })?;
            Ok(summary)
        }
    }
}

fn invalid_input(detail: String) -> RunFailure {
    RunFailure {
        kind: "invalid_input".into(),
        detail,
        attempted_step: None,
        completed_steps: 0,
        emitted_steps: 0,
        checkpoint: None,
    }
}

/// Validate bounded work and all capture requests before constructing either engine.
fn validate(input: &StudyInput, capture: &Capture) -> Result<(), String> {
    let horizon = match input {
        StudyInput::BookC {
            config, max_steps, ..
        } => {
            if !(1..=1_000_000).contains(max_steps) {
                return Err("max_steps must be between 1 and 1000000".into());
            }
            let sites = u64::from(config.width) * u64::from(config.height);
            if sites > 4096 {
                return Err("Book controls permit at most 4096 sites".into());
            }
            if !matches!(sites.checked_mul(*max_steps), Some(work) if work <= 64_000_000) {
                return Err(
                    "Book controls permit at most 64000000 site-ticks without overflow".into(),
                );
            }
            config.validate().map_err(field_errors)?;
            *max_steps
        }
        StudyInput::ReciprocalGraph { config, .. } => {
            config.validate().map_err(field_errors)?;
            config.max_steps
        }
    };
    if capture.retain_steps.len() > 1024 {
        return Err("at most 1024 capture requests are permitted".into());
    }
    let mut seen = BTreeSet::new();
    for &step in &capture.retain_steps {
        if step > horizon {
            return Err(format!("capture step {step} exceeds horizon {horizon}"));
        }
        if !seen.insert(step) {
            return Err(format!("duplicate capture step {step}"));
        }
    }
    Ok(())
}

fn field_errors(errors: Vec<FieldError>) -> String {
    format!("invalid configuration: {errors:?}")
}

/// Ordered resolved-input serialization, with a versioned FNV-1a64 identifier.
/// This is a reproducibility checksum, never cryptographic authentication.
fn input_identity(input: &StudyInput) -> Result<String, String> {
    let bytes = serde_json::to_vec(input).map_err(|error| error.to_string())?;
    let hash = bytes
        .iter()
        .fold(14_695_981_039_346_656_037_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
        });
    Ok(format!("war1-input-v1:0x{hash:016x}"))
}

fn unavailable(quantity: &str, reason: &str) -> UnavailableObservation {
    UnavailableObservation {
        quantity: quantity.into(),
        reason: reason.into(),
    }
}

fn header(input: &StudyInput) -> RunHeader {
    let (equality, unavailable, force_unit, clock_unit) = match input {
        StudyInput::BookC { .. } => (
            EqualityBasis { counts: None, rates: None, resources: None },
            vec![
                unavailable("equality_basis", "World aggregate snapshots do not establish equal engagement counts, rates or resources"),
                unavailable("active_battle_time", "World has a tick clock, without an engagement-active clock"),
            ],
            "world_agent", "world_tick",
        ),
        StudyInput::ReciprocalGraph { config, .. } => (
            EqualityBasis { counts: Some(config.blue == config.red), rates: Some(config.blue_rate == config.red_rate), resources: None },
            vec![
                unavailable("resources", "stationary graph has no holdings, geography or economy"),
                unavailable("geography", "contact geometry is an abstract exposure relation, not spatial positions"),
                unavailable("killer", "casualty receipts attribute aggregate opposing exposure, not an individual lethal attacker"),
            ],
            "finite_individual (reference: continuous_force)", "dimensionless_model_time",
        ),
    };
    RunHeader {
        input: input.clone(),
        equality,
        unavailable,
        force_unit: force_unit.into(),
        clock_unit: clock_unit.into(),
    }
}

fn book_frame(world: &World) -> Result<BookFrame, String> {
    let goods = world.config.goods.len();
    let frame = BookFrame {
        tick: world.tick,
        fingerprint: format!("0x{:016x}", world.fingerprint()),
        rng_state: crate::rng::state_json(&world.rng),
        snapshot: Snapshot::of(world),
        combat_enabled: world.config.combat.enabled,
        deaths: world
            .events()
            .deaths
            .iter()
            .map(|death| BookDeath {
                id: death.id,
                tribe: match death.tribe {
                    Tribe::Blue => "blue",
                    Tribe::Red => "red",
                }
                .into(),
                cause: match death.cause {
                    DeathCause::Starvation => "starvation",
                    DeathCause::OldAge => "old_age",
                    DeathCause::Combat => "combat",
                }
                .into(),
            })
            .collect(),
        kills: world
            .events()
            .kills
            .iter()
            .map(|kill| BookKill {
                attacker: kill.attacker,
                victim: kill.victim,
                loot: kill.loot,
            })
            .collect(),
        agent_stores: (0..goods)
            .map(|good| world.agents().map(|agent| agent.holdings[good]).sum())
            .collect(),
        site_stores: (0..goods)
            .map(|good| world.sites.iter().map(|site| site.resource[good]).sum())
            .collect(),
        unavailable: vec![
            unavailable(
                "harvest_flow",
                "ordinary tick events do not expose complete harvest flows",
            ),
            unavailable(
                "removed_wealth",
                "ordinary death events do not expose all possessions removed at death",
            ),
            unavailable(
                "death_site",
                "ordinary death events do not expose death-site positions",
            ),
            unavailable(
                "active_battle_time",
                "World tick time does not establish active engagement time",
            ),
        ],
    };
    // Finite individual stores can still overflow their sum. serde_json maps
    // nonfinite floats to null, which would misrepresent a mandatory stock.
    for (quantity, stores) in [
        ("agent_stores", &frame.agent_stores),
        ("site_stores", &frame.site_stores),
    ] {
        for (good, value) in stores.iter().enumerate() {
            if !value.is_finite() {
                return Err(format!(
                    "tick {}: {quantity}[{good}] is nonfinite after summing actual World stores",
                    world.tick
                ));
            }
        }
    }
    Ok(frame)
}

struct Output<'a> {
    identity: String,
    emit: &'a mut dyn FnMut(&RunRecord) -> Result<(), String>,
    completed: u64,
    emitted: u64,
    requested: BTreeSet<u64>,
    retained: BTreeMap<u64, ObservedFrame>,
}

impl Output<'_> {
    fn send(&mut self, payload: RunPayload) -> Result<(), String> {
        (self.emit)(&RunRecord {
            schema: RECORD_SCHEMA.into(),
            input_identity: self.identity.clone(),
            payload,
        })
    }

    fn observe(&mut self, step: u64, frame: ObservedFrame) -> Result<(), String> {
        if self.requested.contains(&step) {
            self.retained.insert(step, frame.clone());
        }
        self.send(RunPayload::Observed { frame })?;
        if step > 0 {
            self.emitted += 1;
        }
        Ok(())
    }

    fn failure(
        &self,
        kind: &str,
        detail: String,
        attempted_step: Option<u64>,
        checkpoint: Option<Checkpoint>,
    ) -> RunFailure {
        RunFailure {
            kind: kind.into(),
            detail,
            attempted_step,
            completed_steps: self.completed,
            emitted_steps: self.emitted,
            checkpoint,
        }
    }

    fn summary(&mut self, capture: &Capture, ending: Option<Ending>) -> RunSummary {
        let reason = match ending.as_ref().map(|ending| &ending.reason) {
            Some(EndReason::DoubleExtinction) => "double_extinction",
            Some(EndReason::OneSideExtinction) => "one_side_extinction",
            Some(EndReason::RateZero) => "rate_zero",
            Some(EndReason::Horizon) => "horizon",
            None => "world_horizon",
        };
        let frames = capture
            .retain_steps
            .iter()
            .map(|&step| match self.retained.remove(&step) {
                Some(frame) => CapturedFrame::Available {
                    input_identity: self.identity.clone(),
                    frame,
                },
                None => CapturedFrame::Unavailable {
                    step,
                    reason: format!(
                        "step {step} unavailable: run ended at step {} ({reason})",
                        self.completed
                    ),
                },
            })
            .collect();
        RunSummary {
            completed_steps: self.completed,
            ending,
            capture: frames,
        }
    }
}
