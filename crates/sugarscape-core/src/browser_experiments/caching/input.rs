//! Required browser fields; original checked rigs remain the domain authority.
use crate::{
    browser_experiments::{spatial::IdText, FieldError},
    minds::{deception, protection},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProtectionFixture {
    Single {
        initial_observed: bool,
        redeposit_observed: bool,
    },
    Mixed {
        observed_first: bool,
    },
    CueVisibleNonwatcher {},
    CueUnseenWatcher {},
    Stumble {
        initial_observed: bool,
    },
}
impl ProtectionFixture {
    fn to_core(&self) -> protection::state::Fixture {
        use protection::state::Fixture;
        match *self {
            Self::Single {
                initial_observed,
                redeposit_observed,
            } => Fixture::Single {
                initial_observed,
                redeposit_observed,
            },
            Self::Mixed { observed_first } => Fixture::Mixed { observed_first },
            Self::CueVisibleNonwatcher {} => Fixture::CueVisibleNonwatcher,
            Self::CueUnseenWatcher {} => Fixture::CueUnseenWatcher,
            Self::Stumble { initial_observed } => Fixture::Stumble { initial_observed },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectionInput {
    pub policy: protection::state::Policy,
    pub fixture: ProtectionFixture,
    pub mirrored: bool,
    pub reburial_cost: f64,
    pub discovery: f64,
    pub exposure_span: IdText,
    pub observer_span: u32,
}
impl ProtectionInput {
    pub fn to_core(&self) -> Result<protection::state::LabConfig, Vec<FieldError>> {
        let lab = protection::state::LabConfig {
            policy: self.policy.clone(),
            fixture: self.fixture.to_core(),
            mirrored: self.mirrored,
            reburial_cost: self.reburial_cost,
            discovery: self.discovery,
            exposure_span: self.exposure_span.value(),
            observer_span: self.observer_span,
        };
        protection::lab::rig_config(lab.clone()).validate()?;
        Ok(lab)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeceptionInput {
    pub sender: deception::state::SenderPolicy,
    pub view: deception::state::View,
    pub display_seen: bool,
    pub layout: deception::state::Layout,
    pub effort_cost: f64,
    pub mirrored: bool,
}
impl DeceptionInput {
    pub fn to_core(&self) -> Result<deception::state::LabConfig, Vec<FieldError>> {
        let lab = deception::state::LabConfig {
            sender: self.sender,
            view: self.view,
            display_seen: self.display_seen,
            layout: self.layout,
            effort_cost: self.effort_cost,
            mirrored: self.mirrored,
        };
        deception::lab::rig_config(lab.clone()).validate()?;
        Ok(lab)
    }
}
