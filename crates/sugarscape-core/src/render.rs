//! Rasterizes the world into an RGBA buffer: the landscape layer as a
//! background, agents drawn over their sites in the chosen color mode.

use std::str::FromStr;

use crate::agent::{Agent, Sex, Tribe};
use crate::network::CreditRole;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Tribe,
    Wealth,
    Sex,
    Age,
    Vision,
    Credit,
    Disease,
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

struct Scales {
    log_max_wealth: f64,
    vision_min: f64,
    vision_span: f64,
}

fn agent_color(a: &Agent, mode: ColorMode, s: &Scales) -> Rgb {
    match mode {
        ColorMode::Tribe => match a.tribe() {
            Tribe::Blue => BLUE,
            Tribe::Red => RED,
        },
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
        ColorMode::Credit => NEUTRAL,
        ColorMode::Disease => {
            if a.diseases.is_empty() {
                HEALTHY
            } else {
                SICK
            }
        }
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
    for (i, site) in world.sites.iter().enumerate() {
        let level = match layer {
            Layer::Resource(g) => site.resource[g],
            Layer::Capacity(g) => site.capacity[g],
            Layer::Pollution(k) => site.pollution[k],
        };
        let rgb = lerp(BACKGROUND, color, level / max);
        buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    let v = world.config.vision;
    let scales = Scales {
        log_max_wealth: world
            .agents()
            .map(|a| a.holdings[0])
            .fold(0.0, f64::max)
            .ln_1p()
            .max(1e-9),
        vision_min: f64::from(v.min),
        vision_span: f64::from(v.max.saturating_sub(v.min).max(1)),
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
}
