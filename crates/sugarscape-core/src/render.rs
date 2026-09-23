//! Rasterizes the world into an RGBA buffer: the landscape layer as a
//! background, agents drawn over their sites in the chosen color mode.

use std::str::FromStr;

use crate::agent::{Agent, Sex, Tribe};
use crate::network::CreditRole;
use crate::world::World;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Tribe,
    Wealth,
    Sex,
    Age,
    Vision,
    Credit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Sugar,
    Capacity,
    Pollution,
    Spice,
    SpiceCapacity,
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
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

impl FromStr for Layer {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "sugar" => Self::Sugar,
            "capacity" => Self::Capacity,
            "pollution" => Self::Pollution,
            "spice" => Self::Spice,
            "spice_capacity" => Self::SpiceCapacity,
            _ => return Err(format!("unknown layer {s:?}")),
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
        ColorMode::Wealth => lerp(COOL, HOT, a.sugar.max(0.0).ln_1p() / s.log_max_wealth),
        ColorMode::Age => lerp(COOL, HOT, f64::from(a.age) / f64::from(a.max_age.max(1))),
        ColorMode::Vision => lerp(
            COOL,
            HOT,
            (f64::from(a.vision) - s.vision_min) / s.vision_span,
        ),
        ColorMode::Credit => NEUTRAL,
    }
}

pub fn render(world: &World, mode: ColorMode, layer: Layer, buf: &mut Vec<u8>) {
    buf.resize(world.sites.len() * 4, 0);
    let max_capacity = world
        .sites
        .iter()
        .map(|s| s.capacity)
        .fold(0.0, f64::max)
        .max(1.0);
    let max_pollution = world
        .sites
        .iter()
        .map(|s| s.pollution)
        .fold(0.0, f64::max)
        .max(1e-9);
    let max_spice_capacity = world
        .sites
        .iter()
        .map(|s| s.spice_capacity)
        .fold(0.0, f64::max)
        .max(1.0);
    for (i, site) in world.sites.iter().enumerate() {
        let rgb = match layer {
            Layer::Sugar => lerp(BACKGROUND, SUGAR, site.sugar / max_capacity),
            Layer::Capacity => lerp(BACKGROUND, SUGAR, site.capacity / max_capacity),
            Layer::Pollution => lerp(BACKGROUND, POLLUTION, site.pollution / max_pollution),
            Layer::Spice => lerp(BACKGROUND, SPICE, site.spice / max_spice_capacity),
            Layer::SpiceCapacity => {
                lerp(BACKGROUND, SPICE, site.spice_capacity / max_spice_capacity)
            }
        };
        buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
    }
    let v = world.config.vision;
    let scales = Scales {
        log_max_wealth: world
            .agents()
            .map(|a| a.sugar)
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
    fn landscape_shades_from_background_to_sugar() {
        let mut w = blank_world(10, 10);
        set_sugar(&mut w, 1, 1, 4.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Sugar, &mut buf);
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
        w.site_mut(Pos::new(3, 3)).pollution = 2.0;
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Pollution, &mut buf);
        assert_eq!(pixel(&buf, &w, 3, 3)[..3], POLLUTION);
    }

    #[test]
    fn agents_are_drawn_in_their_mode_color() {
        let mut w = blank_world(10, 10);
        let id = spawn(&mut w, 4, 4);
        let mut buf = Vec::new();
        render(&w, ColorMode::Tribe, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], BLUE);
        render(&w, ColorMode::Sex, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], FEMALE);
        w.agent_mut(id).unwrap().sex = Sex::Male;
        render(&w, ColorMode::Sex, Layer::Sugar, &mut buf);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], MALE);
    }

    #[test]
    fn modes_and_layers_parse_from_names() {
        assert_eq!("wealth".parse::<ColorMode>().unwrap(), ColorMode::Wealth);
        assert_eq!("capacity".parse::<Layer>().unwrap(), Layer::Capacity);
        assert!("plaid".parse::<ColorMode>().is_err());
    }

    #[test]
    fn spice_layer_and_credit_colors() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(1, 1)).spice = 4.0;
        w.site_mut(Pos::new(1, 1)).spice_capacity = 4.0;
        let a = spawn(&mut w, 4, 4);
        let b = spawn(&mut w, 5, 4);
        let c = spawn(&mut w, 6, 4);
        w.originate_loan(a, b, 1.0);
        let mut buf = Vec::new();
        render(&w, ColorMode::Credit, Layer::Spice, &mut buf);
        assert_eq!(pixel(&buf, &w, 1, 1)[..3], SPICE);
        assert_eq!(pixel(&buf, &w, 4, 4)[..3], LENDER);
        assert_eq!(pixel(&buf, &w, 5, 4)[..3], BORROWER);
        assert_eq!(pixel(&buf, &w, 6, 4)[..3], NEUTRAL);
        let _ = c;
        assert_eq!(
            "spice_capacity".parse::<Layer>().unwrap(),
            Layer::SpiceCapacity
        );
        assert_eq!("credit".parse::<ColorMode>().unwrap(), ColorMode::Credit);
    }
}
