//! Strict, normalized episode selection. Payload transport has separate rules.
use super::{record, spatial, FieldError, StudyId, MAX_INPUT_BYTES};
use crate::deduction::PolicyKind;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A u64 that crosses JSON boundaries only as canonical decimal text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedText(u64);
impl SeedText {
    pub fn value(self) -> u64 {
        self.0
    }
}
impl Serialize for SeedText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for SeedText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(serde::de::Error::custom(
                "seed must contain decimal digits only",
            ));
        }
        text.parse::<u64>()
            .map(Self)
            .map_err(|_| serde::de::Error::custom("seed exceeds u64"))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WinkMode {
    Ordinary,
    Diagnostic,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "study", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    ProtectionRecaching {
        lab: super::caching::ProtectionInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
    },
    DeceptionGestures {
        lab: super::caching::DeceptionInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
    },
    BurrowExcavation {
        config: spatial::input::BurrowConfigInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
        ticks: u32,
        sample_every: u32,
    },
    BurrowAccess {
        config: spatial::input::BurrowAccessConfigInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
        ticks: u32,
        sample_every: u32,
    },
    ForagingFixed {
        setup: spatial::input::FixedSetupInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
        ticks: u32,
        sample_every: u32,
    },
    ForagingPassage {
        setup: spatial::input::PassageSetupInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
        ticks: u32,
        sample_every: u32,
    },
    ForagingConstruction {
        setup: spatial::input::ConstructionSetupInput,
        #[serde(deserialize_with = "spatial::input::deserialize_seed")]
        seed: SeedText,
        ticks: u32,
        sample_every: u32,
    },
    SharedSurface {
        protocol: crate::shared_surface::Protocol,
        environment: crate::shared_surface::Environment,
        sequence: u16,
    },
    ActiveSurface {
        protocol: crate::active_surface::Protocol,
        environment: crate::shared_surface::Environment,
        sequence: u16,
    },
    Wink {
        seed: SeedText,
        policy: PolicyKind,
        mode: WinkMode,
    },
    Testimony {
        fixture: String,
    },
    StrategicReporting {
        environment: String,
        history: u8,
        policy: String,
        listener: String,
    },
    StrategyInference {
        environment: String,
        history: u8,
        catalog: String,
    },
    AdversarialAudit {
        environment: String,
        history: u8,
        controller: String,
        witness: String,
    },
    TestimonyGame {
        environment: String,
        history: u8,
        listener: String,
    },
}
impl Input {
    pub fn study(&self) -> StudyId {
        match self {
            Self::ProtectionRecaching { .. } => StudyId::ProtectionRecaching,
            Self::DeceptionGestures { .. } => StudyId::DeceptionGestures,
            Self::BurrowExcavation { .. } => StudyId::BurrowExcavation,
            Self::BurrowAccess { .. } => StudyId::BurrowAccess,
            Self::ForagingFixed { .. } => StudyId::ForagingFixed,
            Self::ForagingPassage { .. } => StudyId::ForagingPassage,
            Self::ForagingConstruction { .. } => StudyId::ForagingConstruction,
            Self::SharedSurface { .. } => StudyId::SharedSurface,
            Self::ActiveSurface { .. } => StudyId::ActiveSurface,
            Self::Wink { .. } => StudyId::Wink,
            Self::Testimony { .. } => StudyId::Testimony,
            Self::TestimonyGame { .. } => StudyId::TestimonyGame,
            Self::StrategicReporting { .. } => StudyId::StrategicReporting,
            Self::StrategyInference { .. } => StudyId::StrategyInference,
            Self::AdversarialAudit { .. } => StudyId::AdversarialAudit,
        }
    }
}

pub fn normalize_input(json: &str) -> Result<Input, Vec<FieldError>> {
    if json.len() > MAX_INPUT_BYTES {
        return Err(super::error("input", "input exceeds 64 KiB"));
    }
    let input: Input =
        serde_json::from_str(json).map_err(|e| super::error("input", e.to_string()))?;
    record::serialized_size(&input, MAX_INPUT_BYTES, "input")?;
    match &input {
        Input::ProtectionRecaching { .. } | Input::DeceptionGestures { .. } => {
            super::caching::validate_input(&input)?
        }
        Input::BurrowExcavation { .. }
        | Input::BurrowAccess { .. }
        | Input::ForagingFixed { .. }
        | Input::ForagingPassage { .. }
        | Input::ForagingConstruction { .. } => spatial::validate_input(&input)?,
        Input::Wink { .. } => {}
        Input::SharedSurface { .. } | Input::ActiveSurface { .. } => {
            super::surfaces::validate_input(&input)?
        }
        Input::Testimony { .. } | Input::TestimonyGame { .. } => {
            super::testimony::validate_input(&input)?
        }
        _ => super::reporting::validate_input(&input)?,
    }
    Ok(input)
}
