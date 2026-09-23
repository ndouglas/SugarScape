//! Agents and their genetic and cultural attributes.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::bits::Bits;
use crate::config::{Config, MAX_GOODS};
use crate::geometry::Pos;

/// An array holding `x` for good 0 and zero for every other good.
pub(crate) fn in_slot_0<T: Copy + Default>(x: T) -> [T; MAX_GOODS] {
    let mut out = [T::default(); MAX_GOODS];
    out[0] = x;
    out
}

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

/// An index into `World::diseases`.
pub type DiseaseId = u32;

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
    /// Per-tick burn of each good (slots ≥ n are 0).
    pub metabolism: [u32; MAX_GOODS],
    /// Holdings of each good.
    pub holdings: [f64; MAX_GOODS],
    /// Endowment of each good at birth: the fertility threshold and the basis
    /// of a parent's contribution to a child (half of it).
    pub initial: [f64; MAX_GOODS],
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
    /// Book eq. 6's φ (0 while foresight is off).
    pub foresight: u32,
    /// Sugar gathered minus sugar metabolism minus per-tick loan obligations,
    /// this turn (credit's creditworthiness input).
    pub income: f64,
    /// Chapter V: the inherited, untrained immune string (empty while disease
    /// is off).
    pub immune_genome: Bits,
    /// The trained immune string; starts as a copy of the genome.
    pub immune: Bits,
    /// Diseases currently carried: distinct indices into `World::diseases`.
    pub diseases: Vec<DiseaseId>,
    /// The agent that most recently infected this one.
    pub infected_by: Option<AgentId>,
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
        let endowment = f64::from(config.goods[0].endowment.sample(rng));
        let mut agent = Self {
            id: 0,
            pos,
            vision: config.vision.sample(rng),
            metabolism: in_slot_0(config.goods[0].metabolism.sample(rng)),
            holdings: in_slot_0(endowment),
            initial: in_slot_0(endowment),
            age: 0,
            max_age: config.lifespan.max_age.sample(rng),
            sex,
            fertility_onset: config.sex.fertility_onset.sample(rng),
            fertility_end: config.sex.end_for(sex).sample(rng),
            tags: Tags::random(config.tag_length, rng),
            parents: None,
            children: Vec::new(),
            born,
            foresight: 0,
            income: 0.0,
            immune_genome: Bits::default(),
            immune: Bits::default(),
            diseases: Vec::new(),
            infected_by: None,
        };
        // Goods 1..n draw where Chapter IV drew spice: after the tags,
        // endowment then metabolism, in good order.
        for (i, good) in config.goods.iter().enumerate().skip(1) {
            let e = f64::from(good.endowment.sample(rng));
            agent.holdings[i] = e;
            agent.initial[i] = e;
            agent.metabolism[i] = good.metabolism.sample(rng);
        }
        if config.foresight.enabled {
            agent.foresight = config.foresight.range.sample(rng);
        }
        if config.disease.enabled {
            let genome = Bits::random(config.disease.immune_length, rng);
            agent.immune_genome = genome;
            agent.immune = genome;
        }
        agent
    }

    pub fn tribe(&self) -> Tribe {
        self.tags.tribe()
    }

    /// Of childbearing age and holding at least the endowment it was born
    /// with of every good (goods it was born without are excepted).
    pub fn is_fertile(&self) -> bool {
        (self.fertility_onset..=self.fertility_end).contains(&self.age)
            && self
                .holdings
                .iter()
                .zip(&self.initial)
                .all(|(&have, &born_with)| born_with <= 0.0 || have >= born_with)
    }

    /// Units of `good` burned per tick: its metabolism plus `fee` per carried
    /// disease.
    pub fn effective_metabolism(&self, good: usize, fee: f64) -> f64 {
        f64::from(self.metabolism[good]) + fee * self.diseases.len() as f64
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
    fn a_second_good_and_foresight_are_drawn_only_when_configured() {
        use crate::config::{Config, Good};
        use crate::rng::seeded;
        let one = Agent::random(&Config::default(), Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(
            (one.holdings[1], one.metabolism[1], one.foresight),
            (0.0, 0, 0)
        );
        let mut on = Config::default();
        on.add_good(Good::spice());
        on.foresight.enabled = true;
        let c = Agent::random(&on, Pos::new(0, 0), 0, &mut seeded(4));
        assert!((1..=4).contains(&c.metabolism[1]));
        assert!((5.0..=25.0).contains(&c.holdings[1]) && c.holdings[1] == c.initial[1]);
        assert!(c.foresight <= 10);
        assert_eq!(
            (c.vision, c.metabolism[0], c.holdings[0]),
            (one.vision, one.metabolism[0], one.holdings[0]),
            "new draws come after the existing ones"
        );
    }

    #[test]
    fn spice_counts_for_fertility_only_for_agents_with_spice_traits() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let a = w.agent_mut(id).unwrap();
        a.holdings[1] = 9.0;
        assert!(!a.is_fertile(), "below its spice endowment");
        a.initial[1] = 0.0;
        a.holdings[1] = -1.0;
        assert!(a.is_fertile(), "no spice endowment, no spice requirement");
    }

    #[test]
    fn immune_genomes_are_drawn_only_when_disease_is_on() {
        use crate::config::Config;
        use crate::rng::seeded;
        let off = Agent::random(&Config::default(), Pos::new(0, 0), 0, &mut seeded(4));
        assert!(off.immune.is_empty() && off.immune_genome.is_empty());
        assert!(off.diseases.is_empty() && off.infected_by.is_none());
        let mut on = Config::default();
        on.disease.enabled = true;
        let a = Agent::random(&on, Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(a.immune.len(), 50);
        assert_eq!(a.immune, a.immune_genome, "the phenotype starts untrained");
        assert_eq!(
            (a.vision, a.metabolism, a.holdings[0], a.tags),
            (off.vision, off.metabolism, off.holdings[0], off.tags),
            "the genome is drawn after the existing traits"
        );
        assert!(a.diseases.is_empty(), "diseases come from the world's list");
    }

    #[test]
    fn every_good_is_drawn_after_the_tags_in_good_order() {
        use crate::config::{Config, Good, Map, URange};
        use crate::rng::seeded;
        let mut two = Config::default();
        two.add_good(Good::spice());
        let mut three = two.clone();
        three.add_good(Good {
            name: "salt".into(),
            color: "#7fb3d5".into(),
            map: Map::Flat { capacity: 1.0 },
            metabolism: URange::new(7, 7),
            endowment: URange::new(9, 9),
        });
        let a = Agent::random(&two, Pos::new(0, 0), 0, &mut seeded(4));
        let b = Agent::random(&three, Pos::new(0, 0), 0, &mut seeded(4));
        assert_eq!(
            (b.metabolism[2], b.holdings[2], b.initial[2]),
            (7, 9.0, 9.0)
        );
        assert_eq!((b.vision, b.tags, b.max_age), (a.vision, a.tags, a.max_age));
        assert_eq!(
            b.holdings[..2],
            a.holdings[..2],
            "good 2 draws after goods 0 and 1"
        );
        assert_eq!(b.metabolism[..2], a.metabolism[..2]);
    }

    #[test]
    fn fertility_needs_every_good_the_agent_was_born_with() {
        let mut w = crate::testkit::blank_world(5, 5);
        let id = crate::testkit::spawn(&mut w, 1, 1);
        let a = w.agent_mut(id).unwrap();
        a.initial[2] = 4.0;
        a.holdings[2] = 3.0;
        assert!(!a.is_fertile(), "short of good 2");
        a.holdings[2] = 4.0;
        assert!(a.is_fertile());
    }
}
