//! Pollution diffusion rule D_α: every α ticks, each site's pollution becomes
//! the mean of its four von Neumann neighbors' pollution.

use crate::world::World;

pub(crate) fn diffuse(world: &mut World) {
    let d = world.config.diffusion;
    if !d.enabled || !(world.tick + 1).is_multiple_of(u64::from(d.every)) {
        return;
    }
    let has_walls = world.has_walls();
    for k in 0..world.config.pollution.pollutants.len() {
        let next: Vec<f64> = (0..world.sites.len())
            .map(|i| {
                let p = world.torus.pos(i);
                if !has_walls {
                    // Today's exact expression: bit-identical without walls.
                    return world
                        .torus
                        .neighbors(p)
                        .iter()
                        .map(|&q| world.site(q).pollution[k])
                        .sum::<f64>()
                        / 4.0;
                }
                if world.is_wall(p) {
                    return 0.0;
                }
                let mut sum = 0.0;
                let mut count = 0u32;
                for q in world.torus.neighbors(p) {
                    if !world.is_wall(q) {
                        sum += world.site(q).pollution[k];
                        count += 1;
                    }
                }
                if count == 0 {
                    0.0
                } else {
                    sum / f64::from(count)
                }
            })
            .collect();
        for (site, p) in world.sites.iter_mut().zip(next) {
            site.pollution[k] = p;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::testkit::*;

    #[test]
    fn diffusion_spreads_to_neighbors() {
        let mut w = blank_world(10, 10);
        w.config.diffusion.enabled = true;
        w.site_mut(Pos::new(4, 4)).pollution[0] = 4.0;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution[0], 0.0);
        for p in [
            Pos::new(4, 3),
            Pos::new(4, 5),
            Pos::new(3, 4),
            Pos::new(5, 4),
        ] {
            assert_eq!(w.site(p).pollution[0], 1.0);
        }
        let total: f64 = w.sites.iter().map(|s| s.pollution[0]).sum();
        assert_eq!(total, 4.0, "diffusion conserves pollution");
    }

    #[test]
    fn diffusion_runs_every_alpha_ticks() {
        let mut w = blank_world(10, 10);
        w.config.diffusion.enabled = true;
        w.config.diffusion.every = 2;
        w.site_mut(Pos::new(4, 4)).pollution[0] = 4.0;
        diffuse(&mut w); // tick 0: (0 + 1) % 2 != 0
        assert_eq!(w.site(Pos::new(4, 4)).pollution[0], 4.0);
        w.tick = 1;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution[0], 0.0);
    }

    #[test]
    fn diffusion_averages_over_non_wall_neighbors_and_wall_sites_stay_zero() {
        let mut c = crate::testkit::blank_config(10, 10);
        c.walls = vec![crate::config::Wall {
            x: 4,
            y: 3,
            width: 1,
            height: 1,
            opaque: true,
        }];
        c.diffusion.enabled = true;
        let mut w = World::new(c, 1).unwrap();
        // (4,4)'s neighbors: N (4,3) is the wall; S (4,5), E (5,4), W (3,4) are open.
        for p in [Pos::new(4, 5), Pos::new(5, 4), Pos::new(3, 4)] {
            w.site_mut(p).pollution[0] = 3.0;
        }
        diffuse(&mut w);
        assert_eq!(
            w.site(Pos::new(4, 4)).pollution[0],
            3.0,
            "the sum of its three open neighbors over 3"
        );
        assert_eq!(
            w.site(Pos::new(4, 3)).pollution[0],
            0.0,
            "a wall site stays at 0"
        );
    }

    #[test]
    fn no_diffusion_when_disabled() {
        let mut w = blank_world(10, 10);
        w.site_mut(Pos::new(4, 4)).pollution[0] = 4.0;
        diffuse(&mut w);
        assert_eq!(w.site(Pos::new(4, 4)).pollution[0], 4.0);
    }
}
