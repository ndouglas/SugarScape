//! Builders for tiny hand-crafted worlds in unit tests.

use crate::agent::{Agent, AgentId, Sex, Tags};
use crate::bits::Bits;
use crate::config::{Config, Good, Map, Placement, URange, MAX_GOODS};
use crate::geometry::Pos;
use crate::social::Social;
use crate::world::World;

/// Every rule off, a flat zero-capacity landscape and no agents.
pub fn blank_config(width: u32, height: u32) -> Config {
    Config {
        width,
        height,
        goods: vec![Good {
            map: Map::Flat { capacity: 0.0 },
            ..Good::sugar()
        }],
        population: 0,
        placement: Placement::Random,
        vision: URange::new(1, 1),
        ..Config::default()
    }
}

/// Appends flat zero-capacity goods until `config` has `n`, bypassing
/// validation (so tests may use more goods than validation allows yet).
/// Good 1 is spice; later goods are "good2", "good3", ….
pub fn add_goods(config: &mut Config, n: usize) {
    while config.goods.len() < n {
        let i = config.goods.len();
        let good = if i == 1 {
            Good::spice()
        } else {
            Good {
                name: format!("good{i}"),
                color: "#7fb3d5".into(),
                ..Good::sugar()
            }
        };
        config.add_good(Good {
            map: Map::Flat { capacity: 0.0 },
            ..good
        });
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
        metabolism: [0; MAX_GOODS],
        holdings: [10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        initial: [10.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        age: 20,
        max_age: 100,
        sex: Sex::Female,
        fertility_onset: 12,
        fertility_end: 50,
        tags: Tags::new(0, world.config.tag_length),
        parents: None,
        children: Vec::new(),
        born: 0,
        foresight: 0,
        income: [0.0; MAX_GOODS],
        immune_genome: Bits::new(0, world.config.disease.immune_length),
        immune: Bits::new(0, world.config.disease.immune_length),
        diseases: Vec::new(),
        infected_by: None,
        culture: Vec::new(),
        social: Social::default(),
    };
    world.insert_agent(agent).expect("test site is empty")
}

/// Puts `amount` of `good` on a site, raising its capacity to match if needed.
pub fn set_resource(world: &mut World, x: u32, y: u32, good: usize, amount: f64) {
    let site = world.site_mut(Pos::new(x, y));
    site.capacity[good] = site.capacity[good].max(amount);
    site.resource[good] = amount;
}

/// Puts `sugar` on a site, raising its capacity to match if needed.
pub fn set_sugar(world: &mut World, x: u32, y: u32, sugar: f64) {
    set_resource(world, x, y, 0, sugar);
}
