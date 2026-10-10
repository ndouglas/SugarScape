//! Validated inputs for the independent stationary engagement kernel.

use crate::config::FieldError;
use serde::{Deserialize, Serialize};

pub const RULES_ID: &str = "war1-engagement-v1";
pub const CHECKPOINT_SCHEMA: &str = "war1-checkpoint-v1";
pub const RECORD_SCHEMA: &str = "war1-records-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Geometry {
    AimedFire,
    DuelContact,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngagementConfig {
    pub blue: u32,
    pub red: u32,
    pub blue_rate: f64,
    pub red_rate: f64,
    pub dt: f64,
    pub max_steps: u64,
    pub geometry: Geometry,
}

impl EngagementConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        for (field, count) in [("blue", self.blue), ("red", self.red)] {
            if count > 4096 {
                errors.push(FieldError::new(field, "must be between 0 and 4096"));
            }
        }
        let valid_dt = self.dt.is_finite() && self.dt > 0.0;
        if !valid_dt {
            errors.push(FieldError::new(
                "dt",
                "must be finite and strictly positive",
            ));
        }
        for (field, rate) in [("blue_rate", self.blue_rate), ("red_rate", self.red_rate)] {
            if !rate.is_finite() || rate < 0.0 {
                errors.push(FieldError::new(field, "must be finite and nonnegative"));
            } else if valid_dt && rate > 0.0 {
                let dose = rate * self.dt;
                if !dose.is_finite() || dose > 0.1 {
                    errors.push(FieldError::new(
                        field,
                        "rate * dt must be finite and at most 0.1",
                    ));
                }
            }
        }
        if !(1..=1_000_000).contains(&self.max_steps) {
            errors.push(FieldError::new(
                "max_steps",
                "must be between 1 and 1000000",
            ));
        }
        if valid_dt && !(self.max_steps as f64 * self.dt).is_finite() {
            errors.push(FieldError::new("dt", "max_steps * dt must remain finite"));
        }
        // Widen before addition even when individual counts are invalid. Checked
        // multiplication must handle arbitrary deserialized max_steps safely.
        let total = u64::from(self.blue) + u64::from(self.red);
        if !matches!(total.checked_mul(self.max_steps), Some(work) if work <= 64_000_000) {
            errors.push(FieldError::new(
                "max_steps",
                "total_initial * max_steps must be at most 64000000 without overflow",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
