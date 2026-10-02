//! Rasterizes the world into an RGBA buffer: the landscape layer as a
//! background, agents drawn over their sites in the chosen color mode.

use std::str::FromStr;

use crate::agent::{Agent, Sex};
use crate::config::{CachingRule, Group};
use crate::network::CreditRole;
use crate::social::Lineage;
use crate::world::World;

pub use crate::config::parse_color;

pub type Rgb = [u8; 3];

pub const BACKGROUND: Rgb = [0x16, 0x15, 0x12];
pub const SUGAR: Rgb = [0xf2, 0xc1, 0x4e];
pub const POLLUTION: Rgb = [0x9b, 0x6b, 0xd6];
pub const BLUE: Rgb = [0x3d, 0x7e, 0xff];
pub const RED: Rgb = [0xff, 0x4d, 0x4d];
pub const FEMALE: Rgb = [0xff, 0x7a, 0xc6];
pub const MALE: Rgb = [0x36, 0xd6, 0xc3];
/// Ends of the ramp used for wealth, age and vision.
pub const COOL: Rgb = [0x4f, 0x9d, 0xff];
pub const HOT: Rgb = [0xff, 0x3d, 0x8b];
pub const SPICE: Rgb = [0xe0, 0x7a, 0x3f];
pub const LENDER: Rgb = [0x3d, 0xd6, 0x6b];
pub const BORROWER: Rgb = [0xff, 0x4d, 0x4d];
pub const BOTH: Rgb = [0xff, 0xe0, 0x4d];
pub const NEUTRAL: Rgb = [0x8a, 0x86, 0x7a];
pub const SICK: Rgb = [0xff, 0x4d, 0x4d];
pub const HEALTHY: Rgb = [0x3d, 0x7e, 0xff];
/// Animation III-5's lineage colors. The book's founders are black; the grid's
/// background is always dark, so they are drawn dark gray.
pub const FOUNDER: Rgb = [0x5a, 0x5a, 0x5a];
pub const FOUNDER_PARENT: Rgb = [0xff, 0x4d, 0x4d];
pub const BORN: Rgb = [0x3d, 0xd6, 0x6b];
pub const BORN_PARENT: Rgb = [0xff, 0xe0, 0x4d];
/// A wall site (stone): opaque, blocks sight.
pub const WALL: Rgb = [0x5a, 0x55, 0x4c];
/// A fence site (wood): passable to sight, not to agents.
pub const FENCE: Rgb = [0x8a, 0x6d, 0x3b];
/// Minds 6's strategies (`ColorMode::Strategy`): a hoarder buries, a
/// cheater never does and pilfers what it finds.
pub const HOARDER: Rgb = [0x3d, 0x7e, 0xff];
pub const CHEATER: Rgb = [0xff, 0x4d, 0x4d];
/// Minds 8's watching (`ColorMode::Watching`): a watcher who also buries
/// (blue, as a hoarder), a scrounger who watches and never buries (red, as
/// a cheater); everyone else is neutral.
pub const WATCHER: Rgb = HOARDER;
pub const SCROUNGER: Rgb = CHEATER;
/// Minds 5's caching rules (`ColorMode::CachingRule`), one color each; a
/// Minds 6 cheater follows `none`.
pub const RULE_NONE: Rgb = NEUTRAL;
pub const RULE_EVEN: Rgb = [0x3d, 0x7e, 0xff];
pub const RULE_COMPENSATE: Rgb = [0x36, 0xd6, 0xc3];
pub const RULE_PLAN: Rgb = [0xff, 0x3d, 0x8b];
/// Minds 3's memory (`ColorMode::Memory`): an agent that remembers, and one
/// that doesn't.
pub const REMEMBERS: Rgb = [0x36, 0xd6, 0xc3];
pub const FORGETS: Rgb = NEUTRAL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Tribe,
    Wealth,
    Sex,
    Age,
    Vision,
    Credit,
    Disease,
    Lineage,
    /// Axelrod's culture (milestone 14): a color per culture.
    Culture,
    /// Minds 6: hoarder or cheater (`Agent.cheater`).
    Strategy,
    /// Minds 5: the caching rule each agent follows (`rules::rule_of`).
    CachingRule,
    /// Minds 3: whether the agent remembers (`Agent.remembers`).
    Memory,
    /// Minds 8: watchers who bury, scroungers (watch and never bury), others.
    Watching,
}

/// A landscape layer: a good's level or capacity, or a pollutant's level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Resource(usize),
    Capacity(usize),
    Pollution(usize),
}

impl FromStr for ColorMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "tribe" => Self::Tribe,
            "wealth" => Self::Wealth,
            "sex" => Self::Sex,
            "age" => Self::Age,
            "vision" => Self::Vision,
            "credit" => Self::Credit,
            "disease" => Self::Disease,
            "culture" => Self::Culture,
            "lineage" => Self::Lineage,
            "strategy" => Self::Strategy,
            "caching_rule" => Self::CachingRule,
            "memory" => Self::Memory,
            "watching" => Self::Watching,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

impl FromStr for Layer {
    type Err = String;
    /// `resource:I`, `capacity:I`, `pollution:K`, or the pre-N-goods names
    /// `sugar`, `spice`, `capacity`, `spice_capacity`, `pollution`.
    fn from_str(s: &str) -> Result<Self, String> {
        let index = |prefix: &str| s.strip_prefix(prefix)?.parse::<usize>().ok();
        Ok(match s {
            "sugar" => Self::Resource(0),
            "spice" => Self::Resource(1),
            "capacity" => Self::Capacity(0),
            "spice_capacity" => Self::Capacity(1),
            "pollution" => Self::Pollution(0),
            _ => {
                if let Some(i) = index("resource:") {
                    Self::Resource(i)
                } else if let Some(i) = index("capacity:") {
                    Self::Capacity(i)
                } else if let Some(k) = index("pollution:") {
                    Self::Pollution(k)
                } else {
                    return Err(format!("unknown layer {s:?}"));
                }
            }
        })
    }
}

pub fn lerp(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    };
    std::array::from_fn(|i| {
        (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * t).round() as u8
    })
}

struct Scales<'a> {
    log_max_wealth: f64,
    vision_min: f64,
    vision_span: f64,
    /// The config's groups and their parsed colors (Tribe mode).
    groups: &'a [Group],
    group_colors: Vec<Rgb>,
}

fn agent_color(a: &Agent, mode: ColorMode, s: &Scales) -> Rgb {
    match mode {
        ColorMode::Tribe => s
            .group_colors
            .get(a.group(s.groups))
            .copied()
            .unwrap_or(NEUTRAL),
        ColorMode::Sex => match a.sex {
            Sex::Female => FEMALE,
            Sex::Male => MALE,
        },
        ColorMode::Wealth => lerp(COOL, HOT, a.holdings[0].max(0.0).ln_1p() / s.log_max_wealth),
        ColorMode::Age => lerp(COOL, HOT, f64::from(a.age) / f64::from(a.max_age.max(1))),
        ColorMode::Vision => lerp(
            COOL,
            HOT,
            (f64::from(a.vision) - s.vision_min) / s.vision_span,
        ),
        // Drawn by `render`, which has the world these need.
        ColorMode::Credit | ColorMode::CachingRule => NEUTRAL,
        ColorMode::Memory => {
            if a.remembers {
                REMEMBERS
            } else {
                FORGETS
            }
        }
        ColorMode::Watching => match (a.watches, a.cheater) {
            (true, false) => WATCHER,
            (true, true) => SCROUNGER,
            (false, _) => NEUTRAL,
        },
        ColorMode::Strategy => {
            if a.cheater {
                CHEATER
            } else {
                HOARDER
            }
        }
        ColorMode::Disease => {
            if a.diseases.is_empty() {
                HEALTHY
            } else {
                SICK
            }
        }
        ColorMode::Culture if a.culture.is_empty() => NEUTRAL,
        ColorMode::Culture => crate::culture::culture_color(&a.culture),
        ColorMode::Lineage => match Lineage::of(a) {
            Lineage::Founder => FOUNDER,
            Lineage::FounderParent => FOUNDER_PARENT,
            Lineage::Born => BORN,
            Lineage::BornParent => BORN_PARENT,
        },
    }
}

/// A caching rule's color (`ColorMode::CachingRule`).
pub fn rule_color(rule: CachingRule) -> Rgb {
    match rule {
        CachingRule::None => RULE_NONE,
        CachingRule::Even => RULE_EVEN,
        CachingRule::Compensate => RULE_COMPENSATE,
        CachingRule::Plan => RULE_PLAN,
    }
}

pub fn render(
    world: &World,
    mode: ColorMode,
    layer: Layer,
    buf: &mut Vec<u8>,
) -> Result<(), String> {
    let (n, m) = (
        world.config.goods.len(),
        world.config.pollution.pollutants.len(),
    );
    let present = match layer {
        Layer::Resource(i) | Layer::Capacity(i) => i < n,
        Layer::Pollution(k) => k < m,
    };
    if !present {
        return Err(format!(
            "{layer:?} is not in this world ({n} goods, {m} pollutants)"
        ));
    }
    buf.resize(world.sites.len() * 4, 0);
    let (color, max) = match layer {
        Layer::Resource(i) | Layer::Capacity(i) => (
            parse_color(&world.config.goods[i].color).unwrap_or(NEUTRAL),
            world
                .sites
                .iter()
                .map(|s| s.capacity[i])
                .fold(0.0, f64::max)
                .max(1.0),
        ),
        Layer::Pollution(k) => (
            POLLUTION,
            world
                .sites
                .iter()
                .map(|s| s.pollution[k])
                .fold(0.0, f64::max)
                .max(1e-9),
        ),
    };
    let has_walls = world.has_walls();
    for (i, site) in world.sites.iter().enumerate() {
        let rgb = if has_walls && world.is_opaque(world.torus.pos(i)) {
            WALL
        } else if has_walls && world.is_wall(world.torus.pos(i)) {
            FENCE
        } else {
            let level = match layer {
                Layer::Resource(g) => site.resource[g],
                Layer::Capacity(g) => site.capacity[g],
                Layer::Pollution(k) => site.pollution[k],
            };
            lerp(BACKGROUND, color, level / max)
        };
        buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    let v = world.config.vision;
    let groups = &world.config.culture.groups;
    let scales = Scales {
        log_max_wealth: world
            .agents()
            .map(|a| a.holdings[0])
            .fold(0.0, f64::max)
            .ln_1p()
            .max(1e-9),
        vision_min: f64::from(v.min),
        vision_span: f64::from(v.max.saturating_sub(v.min).max(1)),
        groups,
        group_colors: groups
            .iter()
            .map(|g| parse_color(&g.color).unwrap_or(NEUTRAL))
            .collect(),
    };
    let roles = if mode == ColorMode::Credit {
        crate::network::credit_roles(world)
    } else {
        std::collections::BTreeMap::new()
    };
    for a in world.agents() {
        let i = world.torus.index(a.pos) * 4;
        let rgb = if mode == ColorMode::Credit {
            match roles.get(&a.id).copied().unwrap_or(CreditRole::None) {
                CreditRole::Lender => LENDER,
                CreditRole::Borrower => BORROWER,
                CreditRole::Both => BOTH,
                CreditRole::None => NEUTRAL,
            }
        } else if mode == ColorMode::CachingRule {
            rule_color(crate::minds::caching::rules::rule_of(world, a.id))
        } else {
            agent_color(a, mode, &scales)
        };
        buf[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn pixel(buf: &[u8], w: &World, x: u32, y: u32) -> [u8; 4] {
        let i = w.torus.index(Pos::new(x, y)) * 4;
        [buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]
    }

    #[test]
    fn wall_and_fence_sites_draw_their_own_colors() {
        let mut c = crate::testkit::blank_config(10, 10);
        c.walls = vec![
            crate::config::Wall {
                x: 2,
                y: 2,
                width: 1,
                height: 1,
                opaque: true,
            },
            crate::config::Wall {
                x: 4,
                y: 4,
                width: 1,
                height: 1,
                opaque: false,
            },
        ];
        let w = World::new(c, 1).unwrap();
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], WALL);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], FENCE);
        assert_eq!(
            pixel(&buf, &w, 0, 0)[..3],
            BACKGROUND,
            "an open, empty site"
        );
    }

    #[test]
    fn layers_parse_per_good_names_and_legacy_aliases() {
        for (name, layer) in [
            ("sugar", Layer::Resource(0)),
            ("spice", Layer::Resource(1)),
            ("capacity", Layer::Capacity(0)),
            ("spice_capacity", Layer::Capacity(1)),
            ("pollution", Layer::Pollution(0)),
            ("resource:3", Layer::Resource(3)),
            ("capacity:7", Layer::Capacity(7)),
            ("pollution:2", Layer::Pollution(2)),
        ] {
            assert_eq!(name.parse::<Layer>().unwrap(), layer, "{name}");
        }
        for bad in ["resource:", "resource:x", "salt", "capacity:-1"] {
            assert!(bad.parse::<Layer>().is_err(), "{bad}");
        }
        assert_eq!(parse_color(crate::config::SUGAR_COLOR), Some(SUGAR));
        assert_eq!(parse_color(crate::config::SPICE_COLOR), Some(SPICE));
    }

    #[test]
    fn good_layers_use_the_goods_color_and_missing_layers_are_errors() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 3);
        w.config.goods[2].color = "#102030".into();
        set_resource(&mut w, 1, 1, 2, 4.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(2), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], [0x10, 0x20, 0x30]);
        render(&w, ColorMode::Tribe, Layer::Capacity(2), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], [0x10, 0x20, 0x30]);
        assert!(render(&w, ColorMode::Tribe, Layer::Resource(3), &mut buf).is_err());
        assert!(render(&w, ColorMode::Tribe, Layer::Pollution(1), &mut buf).is_err());
        w.config
            .pollution
            .pollutants
            .push(crate::config::Pollutant::book(3));
        w.site_mut(Pos::new(2, 2)).pollution[1] = 5.0;
        render(&w, ColorMode::Tribe, Layer::Pollution(1), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], POLLUTION);
    }

    #[test]
    fn landscape_shades_from_background_to_sugar() {
        let mut w = blank_world(10, 10);
        set_sugar(&mut w, 1, 1, 4.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(buf.len(), 10 * 10 * 4);
        assert_eq!(pixel(&buf, &w, 1, 1), [SUGAR[0], SUGAR[1], SUGAR[2], 255]);
        assert_eq!(
            pixel(&buf, &w, 2, 2),
            [BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]
        );
    }

    #[test]
    fn pollution_layer_uses_pollution_color() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(3, 3)).pollution[0] = 2.0;
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Pollution(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], POLLUTION);
    }

    #[test]
    fn agents_are_drawn_in_their_mode_color() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 4, 4);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], BLUE);
        render(&w, ColorMode::Sex, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], FEMALE);
        w.agent_mut(id).unwrap().sex = Sex::Male;
        render(&w, ColorMode::Sex, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], MALE);
    }

    #[test]
    fn modes_and_layers_parse_from_names() {
        assert_eq!("wealth".parse::<ColorMode>().unwrap(), ColorMode::Wealth);
        assert_eq!("capacity".parse::<Layer>().unwrap(), Layer::Capacity(0));
        assert!("plaid".parse::<ColorMode>().is_err());
    }

    #[test]
    fn spice_layer_and_credit_colors() {
        let mut w = blank_world(10, 10);
        add_goods(&mut w.config, 2);
        w.site_mut(Pos::new(1, 1)).resource[1] = 4.0;
        w.site_mut(Pos::new(1, 1)).capacity[1] = 4.0;
        let a = spawn(&mut w, 4, 4);
        let b = spawn(&mut w, 5, 4);
        let c = spawn(&mut w, 6, 4);
        w.originate_loan(a, b, 0, 1.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Credit, Layer::Resource(1), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], SPICE);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], LENDER);
        assert_eq!(pixel(&buf, &w, 5, 4)[..3], BORROWER);
        assert_eq!(pixel(&buf, &w, 6, 4)[..3], NEUTRAL);
        let _ = c;
        assert_eq!(
            "spice_capacity".parse::<Layer>().unwrap(),
            Layer::Capacity(1)
        );
        assert_eq!("credit".parse::<ColorMode>().unwrap(), ColorMode::Credit);
    }

    #[test]
    fn tribe_mode_uses_each_groups_color() {
        let mut w = blank_world(10, 10);
        let red = spawn(&mut w, 4, 4);
        w.agent_mut(red).unwrap().tags = crate::agent::Tags::new(u64::MAX, 11);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(
            pixel(&buf, &w, 4, 4)[..3],
            RED,
            "the default groups keep the book's colors"
        );
        w.config.culture.groups = crate::config::three_tribes(11);
        w.config.culture.groups[0].color = "#102030".into();
        render(&w, ColorMode::Tribe, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(
            pixel(&buf, &w, 4, 4)[..3],
            [0x10, 0x20, 0x30],
            "no zeros: group 0"
        );
    }

    #[test]
    fn disease_mode_colors_sick_and_healthy_agents() {
        let mut w = blank_world(10, 10);
        let sick = spawn(&mut w, 1, 1);
        spawn(&mut w, 2, 2);
        w.agent_mut(sick).unwrap().diseases = vec![0];
        let mut buf = Vec::new();
        render(&w, ColorMode::Disease, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], SICK);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], HEALTHY);
        assert_eq!("disease".parse::<ColorMode>().unwrap(), ColorMode::Disease);
    }

    #[test]
    fn lineage_mode_colors_founders_parents_and_children() {
        let mut w = blank_world(10, 10);
        let founder = spawn(&mut w, 1, 1);
        let parent = spawn(&mut w, 2, 2);
        let child = spawn(&mut w, 3, 3);
        let both = spawn(&mut w, 4, 4);
        w.agent_mut(parent).unwrap().children = vec![child];
        w.agent_mut(child).unwrap().parents = Some([parent, founder]);
        w.agent_mut(both).unwrap().parents = Some([parent, founder]);
        w.agent_mut(both).unwrap().children = vec![999];
        let mut buf = Vec::new();
        render(&w, ColorMode::Lineage, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], FOUNDER);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], FOUNDER_PARENT);
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], BORN);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], BORN_PARENT);
        assert_eq!("lineage".parse::<ColorMode>().unwrap(), ColorMode::Lineage);
    }

    #[test]
    fn strategy_mode_colors_hoarders_and_cheaters() {
        let mut w = blank_world(10, 10);
        let cheat = spawn(&mut w, 1, 1);
        spawn(&mut w, 2, 2);
        w.agent_mut(cheat).unwrap().cheater = true;
        let mut buf = Vec::new();
        render(&w, ColorMode::Strategy, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], CHEATER);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], HOARDER);
        assert_eq!(
            "strategy".parse::<ColorMode>().unwrap(),
            ColorMode::Strategy
        );
    }

    #[test]
    fn caching_rule_mode_colors_each_agents_rule() {
        let mut w = blank_world(10, 10);
        w.config.caching.mixed = true;
        let rules = [
            CachingRule::None,
            CachingRule::Even,
            CachingRule::Compensate,
            CachingRule::Plan,
        ];
        let ids: Vec<_> = (0..4).map(|x| spawn(&mut w, x, 0)).collect();
        for (&id, &rule) in ids.iter().zip(&rules) {
            w.agent_mut(id).unwrap().caching_rule = rule;
        }
        // A cheater follows `none` whatever its own rule says.
        let cheat = spawn(&mut w, 5, 5);
        w.agent_mut(cheat).unwrap().caching_rule = CachingRule::Plan;
        w.agent_mut(cheat).unwrap().cheater = true;
        let mut buf = Vec::new();
        render(&w, ColorMode::CachingRule, Layer::Resource(0), &mut buf).unwrap();
        for (x, &rule) in rules.iter().enumerate() {
            assert_eq!(
                pixel(&buf, &w, x as u32, 0)[..3],
                rule_color(rule),
                "{rule:?}"
            );
        }
        assert_eq!(pixel(&buf, &w, 5, 5)[..3], RULE_NONE);
        // Without `mixed`, every agent follows `caching.rule`.
        w.config.caching.mixed = false;
        w.config.caching.rule = CachingRule::Even;
        render(&w, ColorMode::CachingRule, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 3, 0)[..3], RULE_EVEN);
        assert_eq!(
            "caching_rule".parse::<ColorMode>().unwrap(),
            ColorMode::CachingRule
        );
        let distinct: std::collections::BTreeSet<Rgb> =
            rules.iter().map(|&r| rule_color(r)).collect();
        assert_eq!(distinct.len(), 4, "one color per rule");
    }

    #[test]
    fn memory_mode_colors_rememberers() {
        let mut w = blank_world(10, 10);
        let keeps = spawn(&mut w, 1, 1);
        let forgets = spawn(&mut w, 2, 2);
        w.agent_mut(keeps).unwrap().remembers = true;
        w.agent_mut(forgets).unwrap().remembers = false;
        let mut buf = Vec::new();
        render(&w, ColorMode::Memory, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], REMEMBERS);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], FORGETS);
        assert_eq!("memory".parse::<ColorMode>().unwrap(), ColorMode::Memory);
    }

    #[test]
    fn watching_mode_colors_buriers_scroungers_and_others() {
        let mut w = blank_world(10, 10);
        let (b, s, o) = (
            spawn(&mut w, 1, 1),
            spawn(&mut w, 2, 2),
            spawn(&mut w, 3, 3),
        );
        w.agent_mut(b).unwrap().watches = true;
        w.agent_mut(o).unwrap().watches = false;
        let a = w.agent_mut(s).unwrap();
        a.watches = true;
        a.cheater = true;
        let mut buf = Vec::new();
        render(&w, ColorMode::Watching, Layer::Resource(0), &mut buf).unwrap();
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], WATCHER);
        assert_eq!(pixel(&buf, &w, 2, 2)[..3], SCROUNGER);
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], NEUTRAL);
        assert_eq!(
            "watching".parse::<ColorMode>().unwrap(),
            ColorMode::Watching
        );
    }
}
