//! Checked parameters for the standalone excavation lab.

use crate::config::FieldError;
use serde::{Deserialize, Serialize};

pub(super) const MAX_CELLS: u32 = 262_144;
pub(super) const MAX_WORKERS: u32 = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    Direct,
    Relay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cue {
    Blind,
    Responsive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pile {
    FreshAccumulation,
    OldAccumulation,
    SingleFresh,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Fixture {
    Growing {
        width: u32,
        height: u32,
        workers: u32,
    },
    Choice {
        side: Side,
        pile: Pile,
    },
    Corridor {
        length: u32,
        workers: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LabConfig {
    pub fixture: Fixture,
    pub transport: Transport,
    pub cue: Cue,
    pub freshness_window: u64,
    pub relay_distance: u32,
    pub response_weight: u32,
    pub minimum_recent_units: u32,
}

impl Default for LabConfig {
    fn default() -> Self {
        Self {
            fixture: Fixture::Growing {
                width: 41,
                height: 25,
                workers: 8,
            },
            transport: Transport::Direct,
            cue: Cue::Blind,
            freshness_window: 32,
            relay_distance: 3,
            response_weight: 3,
            minimum_recent_units: 2,
        }
    }
}

pub(super) fn checked_cells(width: u32, height: u32, field: &str) -> Result<usize, FieldError> {
    if width == 0 || height == 0 {
        return Err(FieldError::new(field, "dimensions must be positive"));
    }
    let cells = width
        .checked_mul(height)
        .ok_or_else(|| FieldError::new(field, "cell product overflow"))?;
    if cells > MAX_CELLS {
        return Err(FieldError::new(field, "lab supports at most 262144 cells"));
    }
    Ok(cells as usize)
}

pub(super) fn checked_workers(workers: u32, capacity: u32) -> Result<(), FieldError> {
    if workers == 0 || workers > MAX_WORKERS {
        return Err(FieldError::new(
            "fixture.workers",
            "must be between 1 and 4096 workers",
        ));
    }
    if workers > capacity {
        return Err(FieldError::new(
            "fixture.workers",
            "exceeds spawn capacity of two workers per cell",
        ));
    }
    Ok(())
}

impl LabConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        for (field, positive) in [
            ("freshness_window", self.freshness_window > 0),
            ("relay_distance", self.relay_distance > 0),
            ("response_weight", self.response_weight > 0),
            ("minimum_recent_units", self.minimum_recent_units > 0),
        ] {
            if !positive {
                errors.push(FieldError::new(field, "must be positive"));
            }
        }
        let dimensions = match self.fixture {
            Fixture::Growing {
                width,
                height,
                workers,
            } => {
                if width < 3 || height < 15 {
                    errors.push(FieldError::new(
                        "fixture",
                        "growing dimensions must cover staging x=0..2, y=10..14 and exit (0,12)",
                    ));
                }
                if let Err(e) = checked_workers(workers, 28) {
                    errors.push(e);
                }
                Some((width, height))
            }
            Fixture::Choice { .. } => {
                if self
                    .freshness_window
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(1))
                    .is_none()
                {
                    errors.push(FieldError::new(
                        "freshness_window",
                        "choice starting timestamp overflow",
                    ));
                }
                Some((9, 7))
            }
            Fixture::Corridor { length, workers } => {
                if length == 0 {
                    errors.push(FieldError::new("fixture.length", "must be positive"));
                }
                if let Err(e) = checked_workers(workers, length.saturating_mul(2)) {
                    errors.push(e);
                }
                match length.checked_add(2) {
                    Some(width) => Some((width, 3)),
                    None => {
                        errors.push(FieldError::new("fixture.length", "corridor width overflow"));
                        None
                    }
                }
            }
        };
        if let Some((width, height)) = dimensions {
            match checked_cells(width, height, "fixture") {
                Ok(cells) => {
                    if self.response_weight.checked_mul(cells as u32).is_none() {
                        errors.push(FieldError::new(
                            "response_weight",
                            "frontier weight sum overflow",
                        ));
                    }
                }
                Err(e) => errors.push(e),
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
