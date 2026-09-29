//! Coordination levels and Janssen's (2007) search for their plans: groups of
//! subaks share one plan and start month; each group in turn takes the option
//! (of 21 plans × 12 months) that most raises the watershed's harvest, until
//! none does. Exhaustive for one and two groups, as Janssen's were.

use super::data::watershed;
use super::engine::{area_mean, steady_year, Network, Params};
use rand::Rng;

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
            // The two river systems, by source dam (Oos: dams 0, 1, 5–8): our
            // reading, since Janssen's highlands and lowlands are not in the data.
            2 => usize::from(!matches!(net.source[i], 0 | 1 | 5 | 6 | 7 | 8)),
            // Pairs of masceti temples in the data's order (a stated choice).
            7 => (s.masceti as usize - 1) / 2,
            14 => s.masceti as usize - 1,
            // Each masceti split by the data's second temple column: Lansing
            // and Kremer's 28, though only 22 of the pairs occur.
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

/// The score of `plans` on the draws of `seed` (the draws matter only under
/// Janssen's code's routing, which balances one random dam a month).
pub fn score_at(net: &Network, p: &Params, plans: &[(u8, u8)], seed: u64) -> f64 {
    score(net, p, plans, &mut crate::rng::seeded(seed))
}

/// Plans for every subak, one per group, found by Janssen's hill-climbing
/// from `start` (one option per group), every option scored on the same
/// draws (one seed from `rng`).
pub fn search(
    net: &Network,
    p: &Params,
    group: &[usize],
    start: Vec<(u8, u8)>,
    rng: &mut SimRng,
) -> Vec<(u8, u8)> {
    let seed: u64 = rng.gen();
    search_with(net, p, group, start, seed)
}

/// `search` on the draws of `seed`. With more than two groups the climb
/// starts from the better of `start` and the best single plan for everyone
/// (Janssen used several starting points; this one guarantees a finer level
/// never ends below one group).
pub fn search_with(
    net: &Network,
    p: &Params,
    group: &[usize],
    start: Vec<(u8, u8)>,
    seed: u64,
) -> Vec<(u8, u8)> {
    let k = start.len();
    let opts = options();
    let expand = |g: &[(u8, u8)]| group.iter().map(|&x| g[x]).collect::<Vec<_>>();
    let eval = |g: &[(u8, u8)]| score_at(net, p, &expand(g), seed);
    let exhaustive = |combos: Vec<Vec<(u8, u8)>>| {
        let mut best = (f64::MIN, combos[0].clone());
        for c in combos {
            let s = eval(&c);
            if s > best.0 {
                best = (s, c);
            }
        }
        best
    };
    if k == 1 {
        return expand(&exhaustive(opts.iter().map(|&o| vec![o]).collect()).1);
    }
    if k == 2 {
        let combos = opts
            .iter()
            .flat_map(|&a| opts.iter().map(move |&b| vec![a, b]))
            .collect();
        return expand(&exhaustive(combos).1);
    }
    let (one, one_score) = {
        let (s, c) = exhaustive(opts.iter().map(|&o| vec![o; k]).collect());
        (c, s)
    };
    let mut current = start;
    let mut best = eval(&current);
    if one_score > best {
        current = one;
        best = one_score;
    }
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
                let s = eval(&current);
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
        assert_eq!(count(28), 22, "the data give 22 of the 28");
    }

    #[test]
    fn finer_levels_never_score_below_one_plan_for_all() {
        // Janssen: a coarser level's solution is one of a finer level's, so
        // the finer can do no worse (his multiple starts; ours starts from the
        // better of a random start and the best plan for everyone).
        let mut r = rng::seeded(1);
        let c = BaliConfig::default();
        let net = Network::new(&c, &mut r);
        let p = Params::of(&c, 1, 1);
        let at = |level: u32, r: &mut SimRng| {
            let g = groups(level, &net);
            let k = g.iter().max().unwrap() + 1;
            let start: Vec<(u8, u8)> = (0..k).map(|i| ((i % 21) as u8, (i % 12) as u8)).collect();
            let found = search(&net, &p, &g, start, r);
            score(&net, &p, &found, r)
        };
        let one = at(1, &mut r);
        let fine = at(28, &mut r);
        assert!(fine >= one, "28 groups {fine} < one group {one}");
    }

    #[test]
    fn under_janssens_routing_the_search_compares_options_on_the_same_draws() {
        // Janssen's code balances one random dam a month: every option is
        // scored on the same draws, so the search cannot pick a lucky one.
        let mut r = rng::seeded(1);
        let c = BaliConfig {
            routing: crate::bali::config::Routing::JanssenCode,
            ..BaliConfig::default()
        };
        let net = Network::new(&c, &mut r);
        let p = Params::of(&c, 1, 1);
        let g = groups(14, &net);
        let start = vec![(0u8, 0u8); 14];
        let found = search_with(&net, &p, &g, start.clone(), 7);
        let expand = |s: &[(u8, u8)]| g.iter().map(|&x| s[x]).collect::<Vec<_>>();
        assert!(score_at(&net, &p, &found, 7) >= score_at(&net, &p, &expand(&start), 7));
        assert_eq!(
            found,
            search_with(&net, &p, &g, start, 7),
            "the same seed, the same plans"
        );
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
