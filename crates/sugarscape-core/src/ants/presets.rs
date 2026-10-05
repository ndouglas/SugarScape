//! Kirman's Figures I and II, the extensions he names, and Alfarano and
//! Milaković's networks.

use super::config::{AntsConfig, Network, Rule};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const KIRMAN: &str = "Kirman 1993, QJE 108(1)";
const AM: &str = "Alfarano & Milaković 2007 (JEDC 2009)";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut AntsConfig),
) -> ModelPreset {
    let mut c = AntsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Ants(c),
    }
}

/// Kirman's ε and δ.
fn kirman(c: &mut AntsConfig, epsilon: f64, delta: f64) {
    c.epsilon = epsilon;
    c.delta = delta;
}

/// Alfarano and Milaković's rule on `network`, a and λ 1.
fn alfarano(c: &mut AntsConfig, ants: u32, network: Network, a: f64) {
    c.rule = Rule::Alfarano;
    c.ants = ants;
    c.network = network;
    c.a = a;
    c.lambda = 1.0;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ants-1a",
            "Fig. Ia: ε 0.005, δ 0.01",
            KIRMAN,
            "Kirman's ants: 100 ants feed at two identical food sources. Each step, 50 times, an ant meets another: with probability ε it switches source on its own, and otherwise an ant from the other source recruits it with probability 1 − δ. With ε 0.005 and δ 0.01 (Figure Ia) self-conversion is below Kirman's threshold (1 − δ)/(N − 1), so the colony spends most of its time crowding one source. The histogram's dots are the exact long-run distribution — the beta-binomial with α = ε(N − 1)/(1 − δ) = 0.5. Measured (20 seeds, 20 000 steps): Var of the share 0.124 against the exact 0.126; a source holds 80 % or more 61 % of the time; 45 flips from one source to the other per run.",
            |c| kirman(c, 0.005, 0.01),
        ),
        preset(
            "ants-1b",
            "Fig. Ib: ε 0.01, δ 0.02",
            KIRMAN,
            "Figure Ib: ε 0.01, δ 0.02, just above Kirman's threshold ε = (1 − δ)/(N − 1), where every split is exactly equally likely. The histogram fills out flat. Measured (20 seeds, 20 000 steps): Var of the share 0.085 against the exact 0.084 (a uniform split gives 0.085).",
            |c| kirman(c, 0.01, 0.02),
        ),
        preset(
            "ants-1c",
            "Fig. Ic: ε 0.15, δ 0.3",
            KIRMAN,
            "Figure Ic: ε 0.15, δ 0.3 — ants change source on their own often and recruit weakly, so the colony stays near half and half. Measured (20 seeds, 20 000 steps): 75 % of steps between 40 % and 60 %; Var of the share 0.0081 against the exact 0.0082.",
            |c| kirman(c, 0.15, 0.3),
        ),
        preset(
            "ants-2a",
            "Fig. IIa: 100 000 meetings",
            KIRMAN,
            "Figure IIa: Figure Ic's ε 0.15 and δ 0.3 over Kirman's 100 000 meetings (2 000 steps of 50). 'The state k/N of the system fluctuates around one-half.' Measured (20 seeds): time means 0.49–0.51; 75 % of steps between 40 % and 60 %. Reproduced.",
            |c| {
                kirman(c, 0.15, 0.3);
                c.stop_at = 2000;
            },
        ),
        preset(
            "ants-2b",
            "Fig. IIb: ε 0.002, δ 0.01",
            KIRMAN,
            "Figure IIb: ε 0.002, δ 0.01 — strong recruiting. Nearly all the ants crowd one source, then, with no outside cause, flip to the other. Kirman: 'the system spends little time around the value of one-half and a great deal of time in the extremes … Although the average value of the system over the period is about one-half.' Measured (20 seeds, 2 000 steps, the figure's 100 000 meetings): 8 % of steps between 40 % and 60 %; a source holds 80 % or more 77 % of the time; 0–4 flips. Figure IIb illustrates one realization, not a claim about all records: time means in 0.4–0.6 occur in 4 of these 20 runs (means range from 0.15 to 0.95). Longer records average near half more reliably. The base chain's stationary modes are at 0 and 100 %, rather than at 20 and 80 %; this does not rule out transient 80–20 imbalances.",
            |_| {},
        ),
        preset(
            "ants-crowd",
            "Fig. IIb with 1 000 ants",
            AM,
            "Figure IIb's ε and δ with ten times the ants (and ten times the meetings per step, so each ant meets as often). α = ε(N − 1)/(1 − δ) passes 1 near N 500, and the herding fades: Alfarano and Milaković's N-dependence. Measured (20 seeds, 20 000 steps): a source holds 80 % or more 21 % of the time, against 79 % with 100 ants; Var of the share 0.048 (exact 0.050) against 0.18. Compare it with ants-2b from the presets menu.",
            |c| {
                c.ants = 1000;
                c.meetings = 500;
            },
        ),
        preset(
            "ants-becker",
            "Becker's pull, Fig. Ic",
            KIRMAN,
            "Kirman suggests, but does not run, Becker's externality: 'having the probability, 1 − δ, of conversion to the majority increase with the size of the majority. This would make the process more extreme.' Our chosen formula scales recruiting by 1 + (the recruiter's share − the recruit's), with capped switching probabilities, at Figure Ic's weak recruiting. The exact long-run distribution now has modes at 18 % and 82 %. Kirman supplies neither this formula nor a numerical split. The base chain has no preferred stationary 80–20 split, although it can pass through that imbalance. Measured (20 seeds, 20 000 steps): a source holds 80 % or more 52 % of the time; 17 flips per run.",
            |c| {
                kirman(c, 0.15, 0.3);
                c.pull = 1.0;
            },
        ),
        preset(
            "ants-lock",
            "Becker's pull, Fig. IIb",
            KIRMAN,
            "Our Becker-style pull at Figure IIb's strong recruiting, pull 0.5: one source holds nearly the whole colony for long stretches. Measured (20 seeds, 20 000 steps, 10⁶ meetings each): no flips observed; the largest source holds 99.5 % on average. Positive ε permits a path between every split, so this is finite-horizon persistence, not permanent lock-in.",
            |c| {
                c.pull = 0.5;
            },
        ),
        preset("ants-three", "Three sources", KIRMAN, "Figure IIb with three sources: an ant that changes source on its own picks either of the others. Kirman: 'Generalizing to a larger number of sources would not change the analysis.' Measured (20 seeds, 20 000 steps): one source holds 80 % or more 77 % of the time (79 % with two); the colony moves from one holder to another 24 times per run (26 with two). The measured occupancy is similar under our uniform-other-source self-conversion rule; this does not imply every statistic is unchanged.", |c| {
            c.sources = 3;
        }),
        preset("am-ring", "A ring, D = 10", AM, "Alfarano and Milaković's version of the ants: each ant in turn switches with probability (a + λ × its neighbors at the other source)/(a + λN) — here on a ring, each ant knowing its 10 nearest. Their mean-field theory gives a Beta distribution with α = aN/λD = 0.5, the histogram's dots, 'irrespective of the underlying network structure'. At our N 100 and fixed-index sequential updates, the ring's measured variance is narrower than the nominal Beta prediction; the paper's Figure 3 omits N, so this is a conditional reconstruction, not an exact published-figure verdict. Neighbor correlations can matter. Measured (10 seeds, 100 000 sweeps): Var of the share 0.088 against the theory's 0.126.", |c| {
            alfarano(c, 100, Network::Ring, 0.05)
        }),
        preset(
            "am-random",
            "A random network, p = 0.1",
            AM,
            "Alfarano and Milaković's rule on a random network (each pair linked with probability 0.1), α = 0.5. At our N 100 the measured variance is near their mean-field prediction; Figure 3 omits N. Measured (10 seeds, 100 000 sweeps): Var of the share 0.116 against the theory's 0.123. The random network is also the one whose herding survives as N grows: the number of neighbors grows with N (see the ants-n sweep).",
            |c| alfarano(c, 100, Network::Random, 0.05),
        ),
        preset(
            "am-scale-free",
            "A scale-free network, 2m = 10",
            AM,
            "Alfarano and Milaković's rule on a Barabási–Albert network with hubs (2m = 10), α about 0.5. Measured (10 seeds, 100 000 sweeps): Var of the share 0.117 against the theory's 0.124 — close.",
            |c| alfarano(c, 100, Network::ScaleFree, 0.05),
        ),
        preset(
            "am-independent",
            "Fig. 6: q = 0.05",
            AM,
            "Alfarano and Milaković's Figure 6: 1 000 ants on a random network, a 0.5, λ 1, and 5 % who never herd, placed on the network. Everyone else hears from them, and the colony's swings shrink far more than removing them would suggest. Measured (3 seeds): Var of the share 0.0063 against 0.019 for the same ants off the network, and 0.021 with everyone herding (the survey, 30 000 sweeps).",
            |c| {
                alfarano(c, 1000, Network::Random, 0.5);
                c.independent = 0.05;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, AntsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Ants(c) => (p.id, c),
                _ => panic!("{} is not an ants preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("ants-2b"), AntsConfig::default());
        let a = find("ants-1a");
        assert_eq!((a.epsilon, a.delta, a.ants), (0.005, 0.01, 100));
        assert_eq!(find("ants-2a").stop_at, 2000);
        let crowd = find("ants-crowd");
        assert_eq!(
            (crowd.ants, crowd.meetings),
            (1000, 500),
            "the same meetings per ant"
        );
        let r = find("am-random");
        assert_eq!(
            (r.rule, r.network, r.link),
            (Rule::Alfarano, Network::Random, 0.1)
        );
        // α = aN/λD ≈ 0.5 on each of Alfarano and Milaković's networks.
        assert!((r.a * 100.0 / (0.1 * 99.0) - 0.5).abs() < 0.01);
        assert!((find("am-ring").a * 100.0 / 10.0 - 0.5).abs() < 1e-12);
        assert_eq!(find("am-independent").independent, 0.05);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
