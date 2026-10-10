//! Reproducibility validation of checkpoint fields, not historical authentication.

use super::{
    config::{EngagementConfig, CHECKPOINT_SCHEMA},
    records::{ending_for, Ending},
};
use crate::{config::FieldError, rng::SimRng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub schema: String,
    pub config: EngagementConfig,
    pub seed: u64,
    pub step: u64,
    pub active_steps: u64,
    pub blue_alive: Vec<u32>,
    pub red_alive: Vec<u32>,
    pub contact_rng: String,
    pub casualty_rng: String,
    pub ending: Option<Ending>,
}

pub(super) struct ValidatedCheckpoint {
    pub(super) wire: Checkpoint,
    pub(super) contact_rng: SimRng,
    pub(super) casualty_rng: SimRng,
}

fn same_config(a: &EngagementConfig, b: &EngagementConfig) -> bool {
    a.blue == b.blue
        && a.red == b.red
        && a.geometry == b.geometry
        && a.max_steps == b.max_steps
        && a.blue_rate.to_bits() == b.blue_rate.to_bits()
        && a.red_rate.to_bits() == b.red_rate.to_bits()
        && a.dt.to_bits() == b.dt.to_bits()
}

fn parse_rng(text: &str) -> Result<SimRng, String> {
    // Validate the locked generator's one-field schema directly as u128. Value
    // would lose precision for the state, and the upstream derive permits extras.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct State {
        state: u128,
    }
    if text.len() > 128 {
        return Err("RNG state text exceeds 128 bytes".into());
    }
    let state: State = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if state.state & 1 == 0 {
        return Err("PCG multiplicative state must be odd".into());
    }
    serde_json::from_str(text).map_err(|e| e.to_string())
}

pub(super) fn validate_for(
    saved: &Checkpoint,
    expected: &EngagementConfig,
    seed: u64,
) -> Result<ValidatedCheckpoint, Vec<FieldError>> {
    let mut errors = Vec::new();
    if saved.schema != CHECKPOINT_SCHEMA {
        errors.push(FieldError::new("schema", "unknown checkpoint schema"));
    }
    if !same_config(&saved.config, expected) {
        errors.push(FieldError::new(
            "config",
            "must match engine configuration bit patterns",
        ));
    }
    if saved.seed != seed {
        errors.push(FieldError::new("seed", "must match engine seed"));
    }
    if saved.step > expected.max_steps {
        errors.push(FieldError::new("step", "exceeds configured horizon"));
    }
    if saved.active_steps > saved.step {
        errors.push(FieldError::new("active_steps", "exceeds completed steps"));
    }
    // Every completed W1 step has positive exposure. Terminal initial states
    // cannot have completed steps, and there are no inactive/no-contact steps.
    if saved.active_steps != saved.step {
        errors.push(FieldError::new(
            "active_steps",
            "must equal completed W1 steps",
        ));
    }
    for (field, ids, lo, hi) in [
        ("blue_alive", &saved.blue_alive, 0, expected.blue),
        (
            "red_alive",
            &saved.red_alive,
            expected.blue,
            expected.blue + expected.red,
        ),
    ] {
        if ids.iter().any(|id| *id < lo || *id >= hi) || ids.windows(2).any(|p| p[0] >= p[1]) {
            errors.push(FieldError::new(
                field,
                "IDs must be sorted, unique and within their initial side range",
            ));
        }
    }
    let counts = [saved.blue_alive.len() as u32, saved.red_alive.len() as u32];
    let rates = [expected.blue_rate, expected.red_rate];
    if saved.ending != ending_for(counts, rates, saved.step, expected.max_steps) {
        errors.push(FieldError::new(
            "ending",
            "inconsistent with population, rates or horizon precedence",
        ));
    }
    if saved.step == 0 && counts != [expected.blue, expected.red] {
        errors.push(FieldError::new(
            "step",
            "initial checkpoint must retain all initial actors",
        ));
    }
    if saved.step > 0
        && ending_for([expected.blue, expected.red], rates, 0, expected.max_steps).is_some()
    {
        errors.push(FieldError::new(
            "step",
            "initially ended engagement cannot complete steps",
        ));
    }
    let contact = parse_rng(&saved.contact_rng)
        .map_err(|message| errors.push(FieldError::new("contact_rng", message)));
    let casualty = parse_rng(&saved.casualty_rng)
        .map_err(|message| errors.push(FieldError::new("casualty_rng", message)));
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(ValidatedCheckpoint {
        wire: saved.clone(),
        contact_rng: contact.expect("validated contact stream"),
        casualty_rng: casualty.expect("validated casualty stream"),
    })
}
