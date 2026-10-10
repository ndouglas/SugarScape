//! Finite stationary engagements with transactional, simultaneous settlement.

use super::{
    checkpoint::{validate_for, Checkpoint},
    config::{EngagementConfig, Geometry, Side, CHECKPOINT_SCHEMA},
    math::{self, NumericIssue},
    records::{self, Casualty, Ending, Exposure, Frame, StepFailure},
};
use crate::{
    config::FieldError,
    rng::{self, SimRng},
};
use rand::{seq::SliceRandom, RngCore};
use std::collections::HashSet;

#[derive(Clone)]
pub struct Engagement {
    config: EngagementConfig,
    seed: u64,
    step: u64,
    active_steps: u64,
    blue_alive: Vec<u32>,
    red_alive: Vec<u32>,
    contact_rng: SimRng,
    casualty_rng: SimRng,
    ending: Option<Ending>,
}

impl Engagement {
    pub fn new(config: EngagementConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let ending = records::ending_for(
            [config.blue, config.red],
            [config.blue_rate, config.red_rate],
            0,
            config.max_steps,
        );
        Ok(Self {
            blue_alive: (0..config.blue).collect(),
            red_alive: (config.blue..config.blue + config.red).collect(),
            config,
            seed,
            step: 0,
            active_steps: 0,
            ending,
            contact_rng: rng::seeded(math::derive_seed(seed, b"war1-contact-v1")),
            casualty_rng: rng::seeded(math::derive_seed(seed, b"war1-casualty-v1")),
        })
    }

    pub fn counts(&self) -> [u32; 2] {
        [self.blue_alive.len() as u32, self.red_alive.len() as u32]
    }
    pub fn ending(&self) -> Option<&Ending> {
        self.ending.as_ref()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            schema: CHECKPOINT_SCHEMA.into(),
            config: self.config.clone(),
            seed: self.seed,
            step: self.step,
            active_steps: self.active_steps,
            blue_alive: self.blue_alive.clone(),
            red_alive: self.red_alive.clone(),
            contact_rng: rng::state_json(&self.contact_rng),
            casualty_rng: rng::state_json(&self.casualty_rng),
            ending: self.ending.clone(),
        }
    }

    pub fn restore(&mut self, saved: &Checkpoint) -> Result<(), Vec<FieldError>> {
        let candidate = validate_for(saved, &self.config, self.seed)?;
        self.step = candidate.wire.step;
        self.active_steps = candidate.wire.active_steps;
        self.blue_alive = candidate.wire.blue_alive;
        self.red_alive = candidate.wire.red_alive;
        self.contact_rng = candidate.contact_rng;
        self.casualty_rng = candidate.casualty_rng;
        self.ending = candidate.wire.ending;
        Ok(())
    }

    pub fn step(&mut self) -> Result<Option<Frame>, StepFailure> {
        self.attempt(None, None)
    }

    #[cfg(test)]
    pub(super) fn step_supplied(
        &mut self,
        matching: &[(u32, u32)],
        words: &[u64],
    ) -> Result<Option<Frame>, StepFailure> {
        self.attempt(Some(matching), Some(words))
    }

    fn attempt(
        &mut self,
        matching: Option<&[(u32, u32)]>,
        words: Option<&[u64]>,
    ) -> Result<Option<Frame>, StepFailure> {
        if self.ending.is_some() {
            return Ok(None);
        }
        let mut candidate = self.clone();
        let frame = candidate
            .settle(matching, words)
            .map_err(|issue| StepFailure {
                attempted_step: self.step + 1,
                issue,
            })?;
        *self = candidate;
        Ok(Some(frame))
    }

    fn settle(
        &mut self,
        supplied_matching: Option<&[(u32, u32)]>,
        supplied_words: Option<&[u64]>,
    ) -> Result<Frame, NumericIssue> {
        let start = self.counts();
        let duel = self.config.geometry == Geometry::DuelContact;
        let mut matched_blue = HashSet::new();
        let mut matched_red = HashSet::new();
        if duel {
            let pairs = if let Some(pairs) = supplied_matching {
                pairs.to_vec()
            } else {
                let mut blue = self.blue_alive.clone();
                let mut red = self.red_alive.clone();
                blue.shuffle(&mut self.contact_rng);
                red.shuffle(&mut self.contact_rng);
                blue.into_iter().zip(red).collect()
            };
            let living_blue: HashSet<_> = self.blue_alive.iter().copied().collect();
            let living_red: HashSet<_> = self.red_alive.iter().copied().collect();
            if pairs.len() != start[0].min(start[1]) as usize {
                return Err(issue("matching", "must contain min(B,R) disjoint pairs"));
            }
            for (blue, red) in pairs {
                if !living_blue.contains(&blue)
                    || !living_red.contains(&red)
                    || !matched_blue.insert(blue)
                    || !matched_red.insert(red)
                {
                    return Err(issue(
                        "matching",
                        "pair has nonliving, mis-sided or duplicate IDs",
                    ));
                }
            }
        }
        let sources = if duel {
            [start[0].min(start[1]); 2]
        } else {
            start
        };
        let contributed_rate = [
            self.config.blue_rate * f64::from(sources[0]),
            self.config.red_rate * f64::from(sources[1]),
        ];
        let integrated = [
            contributed_rate[0] * self.config.dt,
            contributed_rate[1] * self.config.dt,
        ];
        for side in 0..2 {
            if !contributed_rate[side].is_finite() || !integrated[side].is_finite() {
                return Err(issue(
                    "exposure",
                    "contributed rate and integrated dose must remain finite",
                ));
            }
            if contributed_rate[side] > 0.0 && integrated[side] == 0.0 {
                return Err(issue("exposure", "positive integrated exposure underflow"));
            }
        }
        let hazards = if duel {
            [self.config.red_rate, self.config.blue_rate]
        } else {
            [
                contributed_rate[1] / f64::from(start[0]),
                contributed_rate[0] / f64::from(start[1]),
            ]
        };
        let mut target_probability = [None, None];
        let mut exposed = [0, 0];
        for side in 0..2 {
            if hazards[side] > 0.0 {
                target_probability[side] = Some(
                    math::probability(hazards[side], self.config.dt).map_err(|mut error| {
                        error.field = format!("target_probability[{side}].{}", error.field);
                        let (ids, matched) = if side == 0 {
                            (&self.blue_alive, &matched_blue)
                        } else {
                            (&self.red_alive, &matched_red)
                        };
                        let target = ids
                            .iter()
                            .find(|id| !duel || matched.contains(id))
                            .expect("validated nonempty exposure");
                        error.detail = format!("target_id={target}; {}", error.detail);
                        error
                    })?,
                );
                exposed[side] = if duel { sources[side] } else { start[side] };
            } else if !hazards[side].is_finite() || hazards[side] < 0.0 {
                return Err(issue(
                    "hazard",
                    "target hazard must be finite and nonnegative",
                ));
            } else if contributed_rate[1 - side] > 0.0 {
                return Err(issue("hazard", "positive target hazard underflow"));
            }
        }
        let next_step = self.step + 1;
        let active_steps = self.active_steps + u64::from(exposed != [0, 0]);
        let calendar_time = next_step as f64 * self.config.dt;
        let active_time = active_steps as f64 * self.config.dt;
        if !calendar_time.is_finite() || !active_time.is_finite() {
            return Err(issue("clock", "candidate times must remain finite"));
        }
        if let Some(words) = supplied_words {
            if words.len() != (exposed[0] + exposed[1]) as usize {
                return Err(issue(
                    "words",
                    "must supply exactly one word per positive-hazard target",
                ));
            }
        }
        let mut casualties = Vec::new();
        let mut word_index = 0;
        for (side, ids, matched) in [
            (0, &self.blue_alive, &matched_blue),
            (1, &self.red_alive, &matched_red),
        ] {
            if let Some(p) = &target_probability[side] {
                for id in ids {
                    if duel && !matched.contains(id) {
                        continue;
                    }
                    let word = if let Some(words) = supplied_words {
                        words[word_index]
                    } else {
                        self.casualty_rng.next_u64()
                    };
                    word_index += 1;
                    if math::chance(word, p) {
                        casualties.push(Casualty {
                            id: *id,
                            side: if side == 0 { Side::Blue } else { Side::Red },
                        });
                    }
                }
            }
        }
        let dead: HashSet<_> = casualties.iter().map(|c| c.id).collect();
        if dead.len() != casualties.len() || word_index != (exposed[0] + exposed[1]) as usize {
            return Err(issue(
                "settlement",
                "casualty uniqueness or exposure reconciliation failed",
            ));
        }
        self.blue_alive.retain(|id| !dead.contains(id));
        self.red_alive.retain(|id| !dead.contains(id));
        let survivors = self.counts();
        for (side, identity) in [(0, Side::Blue), (1, Side::Red)] {
            let deaths = casualties.iter().filter(|c| c.side == identity).count() as u32;
            if deaths > exposed[side] || survivors[side] + deaths != start[side] {
                return Err(issue(
                    "settlement",
                    "population and exposure counts do not reconcile",
                ));
            }
        }
        self.step = next_step;
        self.active_steps = active_steps;
        self.ending = records::ending_for(
            survivors,
            [self.config.blue_rate, self.config.red_rate],
            next_step,
            self.config.max_steps,
        );
        Ok(Frame {
            step: next_step,
            start,
            survivors,
            active_steps,
            calendar_time,
            active_time,
            exposure: Exposure {
                contact_pairs: if duel {
                    u64::from(sources[0])
                } else {
                    u64::from(start[0]) * u64::from(start[1])
                },
                exposed,
                contributed_rate,
                integrated,
                target_probability,
            },
            casualties,
            ending: self.ending.clone(),
        })
    }
}

fn issue(field: &str, detail: &str) -> NumericIssue {
    NumericIssue {
        field: field.into(),
        detail: detail.into(),
    }
}
