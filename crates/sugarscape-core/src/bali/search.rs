//! Coordination levels and Janssen's (2007) search for their plans: groups of
//! subaks share one plan and start month; each group in turn takes the option
//! (of 21 plans × 12 months) that most raises the watershed's harvest, until
//! none does. Exhaustive for one and two groups, as Janssen's were.

use super::data::watershed;
use super::engine::{area_mean, steady_year, Network, Params};
use crate::rng::SimRng;

/// Each subak's group at `level` (1, 2, 7, 14, 28 or 172), numbered from 0.
pub fn groups(level: u32, net: &Network) -> Vec<usize> {
    let w = watershed();
    let raw: Vec<usize> = w
        .subaks
        .iter()
        .enumerate()
        .map(|(i, s)| match level {
            1 => 0,
            // The two river systems, by source dam (Oos: dams 0, 1, 5–8).
            2 => usize::from(!matches!(net.source[i], 0 | 1 | 5 | 6 | 7 | 8)),
            // Pairs of masceti temples in the data's order (a stated choice).
            7 => (s.masceti as usize - 1) / 2,
            14 => s.masceti as usize - 1,
            // Each masceti split by the data's second temple column.
            28 => (s.masceti as usize - 1) * 2 + (s.ulun as usize - 1),
            _ => i,
        })
        .collect();
    // Renumber densely.
    let mut ids: Vec<usize> = raw.clone();
    ids.sort_unstable();
    ids.dedup();
    raw.iter().map(|g| ids.binary_search(g).unwrap()).collect()
}

/// The 252 options: (plan, start month).
pub fn options() -> Vec<(u8, u8)> {
    (0..21u8)
        .flat_map(|p| (0..12u8).map(move |m| (p, m)))
        .collect()
}

fn score(net: &Network, p: &Params, plans: &[(u8, u8)], rng: &mut SimRng) -> f64 {
    area_mean(&steady_year(net, p, plans, rng))
}

/// Plans for every subak, one per group, found by Janssen's hill-climbing
/// from `start` (one option per group).
pub fn search(
    net: &Network,
    p: &Params,
    group: &[usize],
    start: Vec<(u8, u8)>,
    rng: &mut SimRng,
) -> Vec<(u8, u8)> {
    let k = start.len();
    let opts = options();
    let expand = |g: &[(u8, u8)]| group.iter().map(|&x| g[x]).collect::<Vec<_>>();
    let mut current = start;
    if k <= 2 {
        // Exhaustive.
        let mut best = (f64::MIN, current.clone());
        let combos: Vec<Vec<(u8, u8)>> = if k == 1 {
            opts.iter().map(|&o| vec![o]).collect()
        } else {
            opts.iter()
                .flat_map(|&a| opts.iter().map(move |&b| vec![a, b]))
                .collect()
        };
        for c in combos {
            let s = score(net, p, &expand(&c), rng);
            if s > best.0 {
                best = (s, c);
            }
        }
        return expand(&best.1);
    }
    let mut best = score(net, p, &expand(&current), rng);
    for _pass in 0..5 {
        let mut changed = false;
        for g in 0..k {
            let mine = current[g];
            let mut top = (best, mine);
            for &o in &opts {
                if o == mine {
                    continue;
                }
                current[g] = o;
                let s = score(net, p, &expand(&current), rng);
                if s > top.0 + 1e-12 {
                    top = (s, o);
                }
            }
            current[g] = top.1;
            if top.1 != mine {
                best = top.0;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    expand(&current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bali::config::BaliConfig;
    use crate::rng;

    #[test]
    fn the_levels_have_their_sizes() {
        let mut r = rng::seeded(1);
        let net = Network::new(&BaliConfig::default(), &mut r);
        let count = |l: u32| {
            let g = groups(l, &net);
            g.iter().max().unwrap() + 1
        };
        assert_eq!(count(1), 1);
        assert_eq!(count(2), 2);
        assert_eq!(count(7), 7);
        assert_eq!(count(14), 14);
        assert_eq!(count(172), 172);
        assert!((20..=28).contains(&count(28)), "{}", count(28));
    }

    #[test]
    fn the_search_never_lowers_the_harvest() {
        let mut r = rng::seeded(1);
        let c = BaliConfig::default();
        let net = Network::new(&c, &mut r);
        let p = Params::of(&c, 1, 1);
        let g = groups(14, &net);
        let start = vec![(0u8, 0u8); 14];
        let before = score(
            &net,
            &p,
            &g.iter().map(|&x| start[x]).collect::<Vec<_>>(),
            &mut r,
        );
        let found = search(&net, &p, &g, start, &mut r);
        let after = score(&net, &p, &found, &mut r);
        assert!(after >= before, "{before} → {after}");
        // Every group shares one plan.
        for i in 0..172 {
            for j in 0..172 {
                if g[i] == g[j] {
                    assert_eq!(found[i], found[j]);
                }
            }
        }
    }
}
