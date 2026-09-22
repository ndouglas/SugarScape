//! G_α (growback) and S_{α,β,γ} (seasonal growback).

use crate::config::Config;
use crate::world::World;

/// Growback rate for row `y` during tick `tick`. With seasons on, the north
/// has summer while `tick mod 2γ < γ` (the book's footnote 33) and the other
/// half has winter, growing at α/β per tick.
#[allow(dead_code)] // Used in Task 5 (World::step)
pub(crate) fn rate_at(config: &Config, tick: u64, y: u32) -> f64 {
    let base = config.growback.rate;
    let s = &config.seasons;
    if !s.enabled {
        return base;
    }
    let period = u64::from(s.period);
    let north = y < config.height / 2;
    let north_summer = tick % (2 * period) < period;
    if north == north_summer {
        base
    } else {
        base / f64::from(s.winter_divisor)
    }
}

#[allow(dead_code)] // Used in Task 5 (World::step)
pub(crate) fn apply(world: &mut World) {
    let instant = world.config.growback.instant;
    for i in 0..world.sites.len() {
        let rate = rate_at(&world.config, world.tick, world.torus.pos(i).y);
        let site = &mut world.sites[i];
        site.sugar = if instant {
            site.capacity
        } else {
            (site.sugar + rate).min(site.capacity)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    fn world_with_empty_site(capacity: f64) -> World {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(3, 3)).capacity = capacity;
        w
    }

    #[test]
    fn grows_by_rate_up_to_capacity() {
        let mut w = world_with_empty_site(3.0);
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 1.0);
        for _ in 0..5 {
            apply(&mut w);
        }
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 3.0);
    }

    #[test]
    fn instant_growback_refills_immediately() {
        let mut w = world_with_empty_site(4.0);
        w.config.growback.instant = true;
        apply(&mut w);
        assert_eq!(w.site(Pos::new(3, 3)).sugar, 4.0);
    }

    #[test]
    fn seasons_start_with_summer_in_the_north_and_flip_every_period() {
        let mut c = blank_config(10, 10);
        c.seasons.enabled = true;
        c.seasons.winter_divisor = 8;
        c.seasons.period = 50;
        assert_eq!(rate_at(&c, 0, 0), 1.0, "north summer");
        assert_eq!(rate_at(&c, 0, 9), 0.125, "south winter");
        assert_eq!(rate_at(&c, 49, 4), 1.0);
        assert_eq!(rate_at(&c, 50, 4), 0.125, "north winter after γ ticks");
        assert_eq!(rate_at(&c, 50, 5), 1.0, "south summer after γ ticks");
        assert_eq!(rate_at(&c, 100, 0), 1.0, "back to north summer");
    }

    #[test]
    fn without_seasons_rate_is_uniform() {
        let c = blank_config(10, 10);
        assert_eq!(rate_at(&c, 75, 9), 1.0);
    }
}
