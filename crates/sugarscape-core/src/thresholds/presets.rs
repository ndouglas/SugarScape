//! Granovetter's crowds and their extensions, and Watts's cascade window.

use super::config::{
    Ceilings, Clusters, Crowd, Distribution, Friends, Network, Population, Rounding,
    ThresholdsConfig, Trigger, Update, Zero,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const GRANOVETTER: &str = "Granovetter 1978, AJS 83(6)";
const WATTS: &str = "Watts 2002, PNAS 99(9)";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ThresholdsConfig),
) -> ModelPreset {
    let mut c = ThresholdsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Thresholds(c),
    }
}

/// Granovetter's Fig. 2: 100 people, normal thresholds around 25 %.
fn normal(c: &mut ThresholdsConfig, sd: f64) {
    c.distribution = Distribution::Normal;
    c.mean = 0.25;
    c.sd = sd;
    c.rounding = Rounding::Nearest;
}

/// Watts: 10 000 nodes, φ* 0.18, one random seed, random order, repeated.
fn watts(c: &mut ThresholdsConfig, z: f64) {
    c.actors = 10_000;
    c.distribution = Distribution::Fixed;
    c.mean = 0.18;
    c.network = Network::Random;
    c.degree = z;
    c.trigger = Trigger::Random;
    c.update = Update::Asynchronous;
    c.repeat = true;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "gr-uniform",
            "Uniform thresholds, 0 to 99",
            GRANOVETTER,
            "Granovetter's crowd: 100 people milling around a square, each with a threshold — the share of the crowd he must see join before he joins a riot. Here the thresholds are 0, 1, 2, … 99 %. The instigator (threshold 0) breaks a window; that brings in the person at 1 %, the two bring in the one at 2 %, and so on: one more person each step until all 100 riot. The middle panel is his Figure 1: the share whose threshold is at most x (blue) against the 45° line, with the riot climbing the staircase (orange). Measured: 100, as Granovetter says.",
            |_| {},
        ),
        preset(
            "gr-perturbed",
            "The 1 replaced by a 2",
            GRANOVETTER,
            "The same crowd with one change: the person at 1 % is replaced by one at 2 %. By any average the two crowds are the same; but now the instigator riots alone — nobody else's threshold is reached. Granovetter: 'A demented troublemaker broke a window while a group of solid citizens looked on.' Measured: one rioter. Compare it with gr-uniform from the presets menu.",
            |c| {
                c.distribution = Distribution::Perturbed;
            },
        ),
        preset(
            "gr-normal-12",
            "Fig. 2: normal, σ = 12",
            GRANOVETTER,
            "Granovetter's Figure 2: 100 people with normally distributed thresholds, mean 25 %, spread 12 %, thresholds rounded to whole people. Below his critical spread of about 12.2, the equilibrium is a handful of rioters. Measured: 4 rioters (his continuous calculation: 4.0). The Figure 1 panel shows the c.d.f. crossing the 45° line just above the start.",
            |c| normal(c, 0.12),
        ),
        preset(
            "gr-normal-13",
            "Fig. 2: normal, σ = 13",
            GRANOVETTER,
            "The same crowd with a spread of 13 %: past the critical point the c.d.f. never crosses the 45° line low, and nearly everyone riots. Measured: 100 (the continuous calculation: 100.0). A difference of one point in the spread of a crowd's dispositions, and a riot instead of a broken window. The tipping point for a crowd of 100 depends on how the normal thresholds become people: 12.23 rounded to the nearest person (Granovetter's 12.2), 11.89 rounded down, 12.55 kept as fractions (the gr-sd sweep).",
            |c| normal(c, 0.13),
        ),
        preset(
            "gr-normal-sampled",
            "Fig. 2's crowd, sampled",
            GRANOVETTER,
            "Figure 2's crowd with σ 12.2, but each crowd drawn at random from the normal distribution, as real crowds would be, and one crowd after another. Granovetter's jump — 'a wholly discontinuous, striking qualitative effect' — is a property of the idealized distribution: drawn crowds riot past half 15 % of the time at σ 12 and 25 % at 12.5, rising smoothly (the survey, 1 000 crowds each). Measured here (10 seeds, 3 000 steps): 21 % of about 600 crowds riot past a tenth; the mean crowd ends at 21 %. The histogram of outcomes is split: most crowds end with a few rioters, a fifth with nearly everyone.",
            |c| {
                normal(c, 0.122);
                c.crowd = Crowd::Sampled;
                c.rounding = Rounding::Exact;
                c.repeat = true;
            },
        ),
        preset(
            "gr-city",
            "Crowds sampled from a city",
            GRANOVETTER,
            "Granovetter's sampled crowds: a city whose thresholds are uniform from 0 to 99 %, and crowds of 100 drawn from it. The uniform crowd riots to the last person, but a random crowd usually does not: with no instigator (37 %) nobody riots, with an instigator but nobody at 1 % (14 %) one does — 'in over half the cases (.37 + .14 = .51) the equilibrium result is either no rioters or one rioter.' Measured (the survey, 5 000 crowds): 36.9 % + 13.7 % = 50.5 %. And the whole crowd riots in only 2.3 % of them; the mean is 12 rioters. The histogram shows the pile at 0–1 and the long, thin tail.",
            |c| {
                c.population = Population::City;
                c.repeat = true;
            },
        ),
        preset(
            "gr-friends",
            "Friends count twice",
            GRANOVETTER,
            "The uniform crowd, but friends count twice: each pair are friends with probability ¼, and a person weighs what his friends do double, dividing by the whole crowd with friends counted twice (Granovetter's 63/120 example). Now the person at 1 % joins only if the instigator is his friend. Granovetter: the uniform crowd's riot of 100 'is unstable against almost any kind of social structural influence … the modal equilibrium result is one rioter.' Measured (10 seeds, 3 000 steps, about 850 crowds): the mean crowd ends at 1.5 rioters; the mode is one rioter in 8 of 10 weight-and-acquaintance settings (the survey).",
            |c| {
                c.friends = Friends {
                    enabled: true,
                    ..Friends::default()
                };
                c.repeat = true;
            },
        ),
        preset(
            "gr-friends-perturbed",
            "Perturbed crowd, friends count five times",
            GRANOVETTER,
            "The perturbed crowd, whose lone instigator leaves everyone else unmoved, with friends counting five times. When the instigator's friends include the person at 2 %, the riot can spread a little. Granovetter: the change 'rarely exceeds five to 10 rioters'. Measured (the survey, 1 000 crowds): more than one rioter in 43 % of crowds at acquaintance ¼ — against 19 % at 0.05 and 1 % at 0.5, the largest effect at a quarter, as he says — and the 95th percentile at 7 rioters. With friends counting only twice, never.",
            |c| {
                c.distribution = Distribution::Perturbed;
                c.friends = Friends {
                    enabled: true,
                    weight: 5,
                    ..Friends::default()
                };
                c.repeat = true;
            },
        ),
        preset(
            "gr-ceilings",
            "Fig. 3: 10 % leave above 90 %",
            GRANOVETTER,
            "Granovetter's Figure 3: a threshold model needs each person's net benefit to cross zero once. Some 'might join a riot when 50% of the others had but leave when the total passed 90% for fear that so large a riot would bring official reprisals.' Here a random 10 % of the uniform crowd leave once more than 90 of the others riot, and everyone decides together each step. The riot climbs past 90, the cautious leave, those near the top follow them out, and it climbs again: it pulses between 83 and 92 and never settles, in 34 of 40 crowds (the survey); in the rest it rests at 91. Which people hold the ceilings decides it. Deciding one at a time instead, it hovers near 90.",
            |c| {
                c.ceilings = Ceilings {
                    share: 0.1,
                    at: 0.9,
                };
            },
        ),
        preset(
            "gr-clusters",
            "Ten crowds, 5 % moving each step",
            GRANOVETTER,
            "Granovetter's clusters: ten crowds of 100 drawn from the uniform city, with each person moving to another crowd with probability 0.05 each step and reconsidering there — so rioters who wander into a calm crowd can stop. He asks 'what level of movement among clusters would have the most incendiary effect'. Measured (the survey, 20 runs): 39 % rioting at this movement, against 12 % with no movement and 11 % with everyone moving every step — a middling movement spreads instigators without dissolving the riots they start. The time panel shows each crowd's share in its own color.",
            |c| {
                c.population = Population::City;
                c.clusters = Clusters {
                    enabled: true,
                    count: 10,
                    movement: 0.05,
                };
            },
        ),
        preset(
            "watts-lower",
            "Fig. 3: z = 1.05",
            WATTS,
            "Watts's cascades: 10 000 people on a random network with 1.05 links each on average, every threshold 18 % of one's neighbors; one random person switched on, everyone updating in random order, again and again. Just above z = 1 the network barely holds together: most cascades are tiny, and their sizes follow a power law — Watts: cumulative slope ½. Measured (the survey, 3 000 seeds at n 1 000): slope −0.48. Here: 2.6 % of cascades reach a tenth of the network.",
            |c| watts(c, 1.05),
        ),
        preset(
            "watts-middle",
            "Fig. 2: z = 3",
            WATTS,
            "Watts at z = 3, inside his cascade window: nodes with 5 or fewer links are 'vulnerable' — one active neighbor tips them — and they percolate. Most sparks spread everywhere. Measured (10 seeds, about 175 cascades): 87 % reach a tenth of the network, and those that do cover 94 % of it — the size of the whole connected network (S = 0.94), as Watts found.",
            |c| watts(c, 3.0),
        ),
        preset(
            "watts-upper",
            "Fig. 3: z = 6.14",
            WATTS,
            "Watts at z = 6.14, near the window's upper edge: most people have too many neighbors to be tipped by one, so nearly every spark dies — and very rarely one sweeps the whole network. Watts: at this z, at n 1 000, 'a single cascade occurring in 1,000 random trials'. Measured: at n 1 000, 20 % of seeds go global (the survey) — the upper edge moves with the network's size; here, at n 10 000, 2.5 %.",
            |c| watts(c, 6.14),
        ),
        preset(
            "watts-hetero",
            "Fig. 4a: φ normal, σ = 0.1, z = 8",
            WATTS,
            "Watts's Figure 4a: thresholds normal around 18 % with spread 0.1, at z = 8, where uniform thresholds (18 % each) never cascade. With a spread, some people have low thresholds and many links, and cascades return. Measured: 83 % of cascades global (10 seeds). Watts says varied thresholds widen the window in z both ways; at the sparse end they narrow it (the survey: 10 % against 23 % at z 1.2). Thresholds at or below 0 act only once a neighbor does here; read literally (0 ≥ 0), they would all act at once.",
            |c| {
                watts(c, 8.0);
                c.distribution = Distribution::Normal;
                c.sd = 0.1;
                c.crowd = Crowd::Sampled;
                c.zero = Zero::WhenReached;
            },
        ),
        preset(
            "watts-hub",
            "The best-connected seed, z = 1.3",
            WATTS,
            "Watts's targeting: the spark is the best-connected person instead of a random one, at z = 1.3. 'The most connected nodes are far more likely than average nodes to trigger cascades'. Measured: 96 % of cascades global, against 39 % from random sparks (the watts-targeting sweep). Watts says the advantage vanishes in the dense regime; it does not (88 % against 44 % at z 5.5).",
            |c| {
                watts(c, 1.3);
                c.trigger = Trigger::Hub;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, ThresholdsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Thresholds(c) => (p.id, c),
                _ => panic!("{} is not a thresholds preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("gr-uniform"), ThresholdsConfig::default());
        let n = find("gr-normal-13");
        assert_eq!(
            (n.distribution, n.sd, n.rounding),
            (Distribution::Normal, 0.13, Rounding::Nearest)
        );
        let w = find("watts-upper");
        assert_eq!(
            (w.actors, w.degree, w.mean, w.network),
            (10_000, 6.14, 0.18, Network::Random)
        );
        assert_eq!(find("watts-hetero").zero, Zero::WhenReached);
        assert!(find("gr-clusters").clusters.enabled);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
