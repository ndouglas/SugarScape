//! Strict browser DTOs; original lab validators remain the setup authority.
use crate::browser_experiments::{error, FieldError, SeedText};
use crate::{burrow, foraging};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Exact canonical decimal u64, shared by identities and unbounded counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdText(u64);
impl IdText {
    pub fn value(self) -> u64 {
        self.0
    }
}
impl Serialize for IdText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for IdText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.is_empty()
            || !text.bytes().all(|b| b.is_ascii_digit())
            || (text.len() > 1 && text.starts_with('0'))
        {
            return Err(serde::de::Error::custom(
                "must be canonical decimal u64 text",
            ));
        }
        text.parse()
            .map(Self)
            .map_err(|_| serde::de::Error::custom("decimal text exceeds u64"))
    }
}
pub(crate) fn deserialize_seed<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SeedText, D::Error> {
    let value = IdText::deserialize(deserializer)?;
    SeedText::deserialize(serde::de::value::StringDeserializer::<D::Error>::new(
        value.value().to_string(),
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionInput {
    pub x: u32,
    pub y: u32,
}
impl PositionInput {
    fn fixed(self) -> foraging::fixed::Pos {
        foraging::fixed::Pos {
            x: self.x,
            y: self.y,
        }
    }
    fn passage(self) -> foraging::passage::Pos {
        foraging::passage::Pos {
            x: self.x,
            y: self.y,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceInput {
    pub id: IdText,
    pub pos: PositionInput,
}
impl ResourceInput {
    fn fixed(&self) -> foraging::fixed::Resource {
        foraging::fixed::Resource {
            id: self.id.value(),
            pos: self.pos.fixed(),
        }
    }
    fn passage(&self) -> foraging::passage::Resource {
        foraging::passage::Resource {
            id: self.id.value(),
            pos: self.pos.passage(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BurrowConfigInput {
    pub fixture: burrow::Fixture,
    pub transport: burrow::Transport,
    pub cue: burrow::Cue,
    pub freshness_window: IdText,
    pub relay_distance: u32,
    pub response_weight: u32,
    pub minimum_recent_units: u32,
}
impl BurrowConfigInput {
    pub fn to_core(&self) -> Result<burrow::LabConfig, Vec<FieldError>> {
        let config = burrow::LabConfig {
            fixture: self.fixture.clone(),
            transport: self.transport,
            cue: self.cue,
            freshness_window: self.freshness_window.value(),
            relay_distance: self.relay_distance,
            response_weight: self.response_weight,
            minimum_recent_units: self.minimum_recent_units,
        };
        config.validate()?;
        Ok(config)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BurrowAccessConfigInput {
    pub lab: BurrowConfigInput,
    pub task: burrow::AccessTask,
}
impl BurrowAccessConfigInput {
    /// Mirrors only the supplied Growing fixture's private native task checks.
    /// The original access runner validates again when its single World exists.
    pub fn to_core(&self) -> Result<burrow::AccessConfig, Vec<FieldError>> {
        let lab = self.lab.to_core()?;
        let burrow::Fixture::Growing { width, height, .. } = lab.fixture else {
            return Err(error(
                "lab.fixture",
                "resource access requires a growing fixture",
            ));
        };
        let goal = self.task.goal;
        let mut errors = Vec::new();
        if goal.x >= width || goal.y >= height {
            errors.push(FieldError::new("task.goal", "must be in bounds"));
        } else if goal.x <= 2 && (10..=14).contains(&goal.y) {
            errors.push(FieldError::new("task.goal", "must initially be solid"));
        }
        // Every in-bounds Growing cell belongs to the supplied diggable mask.
        if self.task.goal_weight == 0 {
            errors.push(FieldError::new("task.goal_weight", "must be positive"));
        }
        if lab
            .response_weight
            .checked_mul(self.task.goal_weight)
            .and_then(|w| u64::from(w).checked_mul(u64::from(width) * u64::from(height)))
            .is_none()
        {
            errors.push(FieldError::new(
                "task.goal_weight",
                "cue times goal weight or frontier ticket sum overflow",
            ));
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(burrow::AccessConfig {
            lab,
            task: self.task.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedParametersInput {
    pub p_search: f64,
    pub p_return: f64,
    pub omega: f64,
    pub lambda_informed: f64,
    pub lambda_fidelity: f64,
    pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}
impl FixedParametersInput {
    pub fn to_core(&self) -> Result<foraging::CpfaParameters, Vec<FieldError>> {
        let parameters = foraging::CpfaParameters {
            p_search: self.p_search,
            p_return: self.p_return,
            omega: self.omega,
            lambda_informed: self.lambda_informed,
            lambda_fidelity: self.lambda_fidelity,
            lambda_publish: self.lambda_publish,
            lambda_waypoint: self.lambda_waypoint,
        };
        parameters.validate()?;
        Ok(parameters)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassageParametersInput {
    pub p_search: f64,
    pub p_return: f64,
    pub lambda_fidelity: f64,
    pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}
impl PassageParametersInput {
    pub fn to_core(&self) -> Result<foraging::passage::Parameters, Vec<FieldError>> {
        let parameters = foraging::passage::Parameters {
            p_search: self.p_search,
            p_return: self.p_return,
            lambda_fidelity: self.lambda_fidelity,
            lambda_publish: self.lambda_publish,
            lambda_waypoint: self.lambda_waypoint,
        };
        parameters.validate()?;
        Ok(parameters)
    }
}
fn parameter_errors(errors: Vec<FieldError>) -> Vec<FieldError> {
    errors
        .into_iter()
        .map(|e| FieldError::new(format!("parameters.{}", e.field), e.message))
        .collect()
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedSetupInput {
    pub width: u32,
    pub height: u32,
    pub nest: PositionInput,
    pub agents: u32,
    pub resources: Vec<ResourceInput>,
    pub parameters: FixedParametersInput,
}
impl FixedSetupInput {
    pub fn to_core(&self) -> Result<foraging::fixed::Setup, Vec<FieldError>> {
        let setup = foraging::fixed::Setup {
            width: self.width,
            height: self.height,
            nest: self.nest.fixed(),
            agents: self.agents,
            resources: self.resources.iter().map(ResourceInput::fixed).collect(),
            parameters: self.parameters.to_core().map_err(parameter_errors)?,
        };
        setup.validate()?;
        Ok(setup)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassageSetupInput {
    pub width: u32,
    pub height: u32,
    pub open: Vec<PositionInput>,
    pub nest: Vec<PositionInput>,
    pub workers: Vec<PositionInput>,
    pub resources: Vec<ResourceInput>,
    pub parameters: PassageParametersInput,
}
impl PassageSetupInput {
    pub fn to_core(&self) -> Result<foraging::passage::Setup, Vec<FieldError>> {
        let setup = foraging::passage::Setup {
            width: self.width,
            height: self.height,
            open: self.open.iter().map(|p| p.passage()).collect(),
            nest: self.nest.iter().map(|p| p.passage()).collect(),
            workers: self.workers.iter().map(|p| p.passage()).collect(),
            resources: self.resources.iter().map(ResourceInput::passage).collect(),
            parameters: self.parameters.to_core().map_err(parameter_errors)?,
        };
        setup.validate()?;
        Ok(setup)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionSetupInput {
    pub width: u32,
    pub height: u32,
    pub open: Vec<PositionInput>,
    pub diggable: Vec<PositionInput>,
    pub waste: PositionInput,
    pub nest: Vec<PositionInput>,
    pub workers: Vec<PositionInput>,
    pub food: Vec<ResourceInput>,
    pub parameters: PassageParametersInput,
}
impl ConstructionSetupInput {
    pub fn to_core(&self) -> Result<foraging::construction::Setup, Vec<FieldError>> {
        let setup = foraging::construction::Setup {
            width: self.width,
            height: self.height,
            open: self.open.iter().map(|p| p.passage()).collect(),
            diggable: self.diggable.iter().map(|p| p.passage()).collect(),
            waste: self.waste.passage(),
            nest: self.nest.iter().map(|p| p.passage()).collect(),
            workers: self.workers.iter().map(|p| p.passage()).collect(),
            food: self.food.iter().map(ResourceInput::passage).collect(),
            parameters: self.parameters.to_core().map_err(parameter_errors)?,
        };
        setup.validate()?;
        Ok(setup)
    }
}
