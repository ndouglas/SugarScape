//! Builders for tiny hand-crafted worlds in unit tests.

use crate::agent::{Agent, AgentId, Sex, Tags};
use crate::bits::Bits;
use crate::config::{Config, LandscapeKind, Placement, URange};
use crate::geometry::Pos;
use crate::world::World;

/// Every rule off, a flat zero-capacity landscape and no agents.
pub fn blank_config(width: u32, height: u32) -> Config {
    Config {
        width,
        height,
        landscape: LandscapeKind::Flat { capacity: 0.0 },
        population: 0,
        placement: Placement::Random,
        vision: URange::new(1, 1),
        ..Config::default()
    }
}

pub fn blank_world(width: u32, height: u32) -> World {
    World::new(blank_config(width, height), 7).expect("blank config is valid")
}

/// A fertile-aged female with vision 1, metabolism 0, 10 sugar (endowment 10),
/// all-zero tags (Blue) and all-zero immune strings, no diseases. Tweak fields through `world.agent_mut(id)`.
pub fn spawn(world: &mut World, x: u32, y: u32) -> AgentId {
    let agent = Agent {
        id: 0,
        pos: Pos::new(x, y),
        vision: 1,
        metabolism: 0,
        sugar: 10.0,
        initial_sugar: 10.0,
        age: 20,
        max_age: 100,
        sex: Sex::Female,
        fertility_onset: 12,
        fertility_end: 50,
        tags: Tags::new(0, world.config.tag_length),
        parents: None,
        children: Vec::new(),
        born: 0,
        spice: 10.0,
        initial_spice: 10.0,
        spice_metabolism: 0,
        foresight: 0,
        income: 0.0,
        immune_genome: Bits::new(0, world.config.disease.immune_length),
        immune: Bits::new(0, world.config.disease.immune_length),
        diseases: Vec::new(),
        infected_by: None,
    };
    world.insert_agent(agent).expect("test site is empty")
}

/// Puts `sugar` on a site, raising its capacity to match if needed.
pub fn set_sugar(world: &mut World, x: u32, y: u32, sugar: f64) {
    let site = world.site_mut(Pos::new(x, y));
    site.capacity = site.capacity.max(sugar);
    site.sugar = sugar;
}
