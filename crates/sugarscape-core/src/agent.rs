//! Agents and their genetic and cultural attributes.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::geometry::Pos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Female,
    Male,
}

/// Group membership (Chapter III): Blue when zeros outnumber ones on the tag string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tribe {
    Blue,
    Red,
}

pub type AgentId = u64;

/// A cultural tag string of `len` bits (1..=64); bit `i` is tag position `i`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tags {
    bits: u64,
    len: u32,
}

impl Tags {
    pub fn new(bits: u64, len: u32) -> Self {
        assert!((1..=64).contains(&len), "tag length must be 1..=64");
        Self {
            bits: bits & Self::mask(len),
            len,
        }
    }

    fn mask(len: u32) -> u64 {
        if len == 64 {
            u64::MAX
        } else {
            (1u64 << len) - 1
        }
    }

    pub fn random(len: u32, rng: &mut impl Rng) -> Self {
        Self::new(rng.gen(), len)
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn get(&self, i: u32) -> bool {
        assert!(
            i < self.len,
            "tag index {} out of range [0, {})",
            i,
            self.len
        );
        (self.bits >> i) & 1 == 1
    }

    pub fn set(&mut self, i: u32, value: bool) {
        assert!(
            i < self.len,
            "tag index {} out of range [0, {})",
            i,
            self.len
        );
        if value {
            self.bits |= 1 << i;
        } else {
            self.bits &= !(1 << i);
        }
    }

    pub fn ones(&self) -> u32 {
        self.bits.count_ones()
    }

    pub fn zeros(&self) -> u32 {
        self.len - self.ones()
    }

    pub fn tribe(&self) -> Tribe {
        if self.zeros() > self.ones() {
            Tribe::Blue
        } else {
            Tribe::Red
        }
    }

    /// These tags if they already belong to `tribe`, otherwise every bit
    /// inverted. Inversion swaps the zero and one counts, so it changes the
    /// tribe except on an even-length tie, which stays Red.
    pub fn forced_to(self, tribe: Tribe) -> Self {
        if self.tribe() == tribe {
            self
        } else {
            Self::new(!self.bits, self.len)
        }
    }

    /// Tag position 0 first.
    pub fn to_bit_string(&self) -> String {
        (0..self.len)
            .map(|i| if self.get(i) { '1' } else { '0' })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: AgentId,
    pub pos: Pos,
    pub vision: u32,
    pub metabolism: u32,
    pub sugar: f64,
    /// Endowment at birth; the fertility threshold and the basis of a parent's
    /// contribution to a child (half of it).
    pub initial_sugar: f64,
    pub age: u32,
    /// Drawn from `lifespan.max_age` at birth; enforced only while lifespan is on.
    pub max_age: u32,
    pub sex: Sex,
    pub fertility_onset: u32,
    pub fertility_end: u32,
    pub tags: Tags,
    pub parents: Option<[AgentId; 2]>,
    pub children: Vec<AgentId>,
    /// Tick of birth.
    pub born: u64,
    /// Spice holdings and birth endowment (0 while spice is off).
    pub spice: f64,
    pub initial_spice: f64,
    pub spice_metabolism: u32,
    /// Book eq. 6's φ (0 while foresight is off).
    pub foresight: u32,
    /// Sugar gathered minus sugar metabolism minus per-tick loan obligations,
    /// this turn (credit's creditworthiness input).
    pub income: f64,
}

impl Agent {
    /// A first-generation agent with random genetics, endowment and tags.
    /// `id` is assigned by `World::insert_agent`.
    pub fn random(config: &Config, pos: Pos, born: u64, rng: &mut impl Rng) -> Self {
        let sex = if rng.gen_bool(0.5) {
            Sex::Female
        } else {
            Sex::Male
        };
        let endowment = f64::from(config.endowment.sample(rng));
        let mut agent = Self {
            id: 0,
            pos,
            vision: config.vision.sample(rng),
            metabolism: config.metabolism.sample(rng),
            sugar: endowment,
            initial_sugar: endowment,
            age: 0,
            max_age: config.lifespan.max_age.sample(rng),
            sex,
            fertility_onset: config.sex.fertility_onset.sample(rng),
            fertility_end: config.sex.end_for(sex).sample(rng),
            tags: Tags::random(config.tag_length, rng),
            parents: None,
            children: Vec::new(),
            born,
            spice: 0.0,
            initial_spice: 0.0,
            spice_metabolism: 0,
            foresight: 0,
            income: 0.0,
        };
        if config.spice.enabled {
            let spice = f64::from(config.spice.endowment.sample(rng));
            agent.spice = spice;
            agent.initial_spice = spice;
            agent.spice_metabolism = config.spice.metabolism.sample(rng);
        }
        if config.foresight.enabled {
            agent.foresight = config.foresight.range.sample(rng);
        }
        agent
    }

    pub fn tribe(&self) -> Tribe {
        self.tags.tribe()
    }

    /// Of childbearing age and holding at least the endowment it was born with
    /// (of spice too, when it was born with spice traits).
    pub fn is_fertile(&self) -> bool {
        (self.fertility_onset..=self.fertility_end).contains(&self.age)
            && self.sugar >= self.initial_sugar
            && (self.initial_spice <= 0.0 || self.spice >= self.initial_spice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tribe_is_blue_when_zeros_outnumber_ones() {
        assert_eq!(Tags::new(0b00000000011, 11).tribe(), Tribe::Blue);
        assert_eq!(Tags::new(0b00000111111, 11).tribe(), Tribe::Red);
        // Ties are Red (zeros do not outnumber ones).
        assert_eq!(Tags::new(0b0011, 4).tribe(), Tribe::Red);
    }

    #[test]
    fn set_get_and_bit_string() {
        let mut t = Tags::new(0, 5);
        t.set(1, true);
        t.set(4, true);
        assert!(t.get(1) && !t.get(0));
        assert_eq!(t.to_bit_string(), "01001");
        assert_eq!((t.ones(), t.zeros()), (2, 3));
    }

    #[test]
    fn new_masks_bits_beyond_length() {
        assert_eq!(Tags::new(u64::MAX, 3).ones(), 3);
    }

    #[test]
    fn forced_to_flips_tribe_by_inversion() {
        let blue = Tags::new(0b00001, 5);
        assert_eq!(blue.forced_to(Tribe::Blue), blue);
        let red = blue.forced_to(Tribe::Red);
        assert_eq!(red.tribe(), Tribe::Red);
        assert_eq!(red.to_bit_string(), "01111");
    }

    #[test]
    #[should_panic(expected = "tag index")]
    fn set_panics_on_out_of_range_index() {
        let mut t = Tags::new(0, 5);
        t.set(5, true); // index 5 is out of range for len=5
    }

    #[test]
    fn spice_and_foresight_are_drawn_only_when_enabled() {
        use crate::config::{Config, URange};
        use crate::rng::seeded;
        let mut off = Config::default();
        off.spice.metabolism = URange::new(3, 3);
        let a = Agent::random(&off, Pos::new(0, 0), 0, &mut seeded(4));
        let b = Agent::random(&Config::default(), Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(a, b, "spice parameters don't matter while spice is off");
        assert_eq!((a.spice, a.spice_metabolism, a.foresight), (0.0, 0, 0));
        let mut on = Config::default();
        on.spice.enabled = true;
        on.foresight.enabled = true;
        let c = Agent::random(&on, Pos::new(0, 0), 0, &mut seeded(4));
        assert!((1..=4).contains(&c.spice_metabolism));
        assert!((5.0..=25.0).contains(&c.spice) && c.spice == c.initial_spice);
        assert!(c.foresight <= 10);
        assert_eq!(
            (c.vision, c.metabolism, c.sugar),
            (b.vision, b.metabolism, b.sugar),
            "new draws come after the existing ones"
        );
    }

    #[test]
    fn spice_counts_for_fertility_only_for_agents_with_spice_traits() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let a = w.agent_mut(id).unwrap();
        a.spice = 9.0;
        assert!(!a.is_fertile(), "below its spice endowment");
        a.initial_spice = 0.0;
        a.spice = -1.0;
        assert!(a.is_fertile(), "no spice endowment, no spice requirement");
    }
}
