use super::{
    lab::rig_config,
    state::{LabConfig, Policy},
};
use crate::world::World;
pub(crate) fn rig(policy: Policy) -> World {
    World::new(
        rig_config(LabConfig {
            policy,
            ..LabConfig::default()
        }),
        7,
    )
    .unwrap()
}
pub(crate) fn source_site(world: &World) -> u32 {
    world.torus.index(world.agent(1).unwrap().pos) as u32
}
