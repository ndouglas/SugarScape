use super::*;
use crate::{model::ModelConfig, presets::ModelPreset};
pub fn presets() -> Vec<ModelPreset> {
    [
("democratic-peace-printed-2001","Democratic peace: printed2001",DemocraticPeaceConfig::default()),
("democratic-peace-prose-probability","Superiority increases aggression",DemocraticPeaceConfig{probability_direction:ProbabilityDirection::ProseIncreasing,..Default::default()}),
("democratic-peace-tagging","Tagging only",DemocraticPeaceConfig{mechanism:Mechanism::Tagging,..Default::default()}),
("democratic-peace-alliances","Defensive alliances",DemocraticPeaceConfig{mechanism:Mechanism::Alliances,..Default::default()}),
("democratic-peace-collective-security","Collective security",DemocraticPeaceConfig::default()),
("democratic-peace-nondemocratic","Predatory reference",DemocraticPeaceConfig{initial_democratic_share:0.,..Default::default()}),
].into_iter().map(|(id,name,c)|ModelPreset{id,name,source:"Cederman2001 JCR45(4):470–502",description:"Selection through conquest, partner tags, defensive alliances and collective security. Printed probability conflicts with superiority prose; resolved readings are explicit. No learning or historical validation.",config:ModelConfig::DemocraticPeace(c)}).collect()
}
