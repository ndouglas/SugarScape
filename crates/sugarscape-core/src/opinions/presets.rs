//! The paper's runs and its two unfigured claims.

use super::config::{Confidence, Interaction, OpinionsConfig, Start, Updating};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut OpinionsConfig),
) -> ModelPreset {
    let mut c = OpinionsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: HK,
        description,
        config: ModelConfig::Opinions(c),
    }
}

const HK: &str = "Hegselmann & Krause 2002";

fn regular(c: &mut OpinionsConfig, n: u32) {
    c.agents = n;
    c.start = Start::Regular;
}

fn asymmetric(c: &mut OpinionsConfig, left: f64, right: f64) {
    c.confidence = Confidence::Asymmetric;
    c.epsilon_left = left;
    c.epsilon_right = right;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("hk-plurality", "Little confidence: plurality", "Hegselmann and Krause's bounded confidence: 625 agents hold opinions between 0 and 1 and each period move, all at once, to the mean of every opinion within ε of their own (their own included). Lines are agents, colored by where they started (red at 0, magenta at 1); gray fills the gaps between neighbors still within reach. With ε = 0.01 (Fig. 2a) the paper's single run ends with 'exactly 38 different opinions' in under 15 periods. Measured (20 seeds): a median of 37.5 surviving opinions (34 to 43), stable by period 8 to 13.", |c| c.epsilon = 0.01),
        preset("hk-polarisation", "Middling confidence: two camps", "The same society with ε = 0.15 (Fig. 2b): the paper's run ends in two camps. Measured (20 seeds): two camps in only 6 runs; in 14 a third camp holds the middle, in 12 of them about as big as the other two (in 2 just 1 or 10 agents). Stable by period 7 to 11 in all but one run (88). Over 50 seeds (the hk-diagonal sweep) two camps are the rule only from ε = 0.16 to 0.21.", |_| {}),
        preset("hk-consensus", "Much confidence: consensus", "The same society with ε = 0.25 (Fig. 2c): consensus. Measured (20 seeds): consensus in every run, stable by period 10 (median), but some runs linger for up to 168 periods while two nearly merged camps close — the paper's 'less than 15 periods' holds for most runs, not all.", |c| c.epsilon = 0.25),
        preset("hk-regular-50", "Fifty evenly spaced, ε = 0.2", "Fifty evenly spaced opinions, ε = 0.2 (Figs. 4–5). The extremes move in first, condense, and the profile splits in two in period 6; 'from period 7 to 8 onwards nothing changes anymore'. Measured: exactly that — the split in period 6, two camps, stable at period 8. An evenly spaced start has no randomness, so every seed runs the same.", |c| {
            regular(c, 50);
            c.epsilon = 0.2;
        }),
        preset("hk-regular-plurality", "A hundred evenly spaced, ε = 0.05", "A hundred evenly spaced opinions, ε = 0.05 (Fig. 7): the paper counts 8 splits. Measured: 8 splits, 9 surviving opinions, stable at period 20; the splits open two at a time, from the ends inward.", |c| {
            regular(c, 100);
            c.epsilon = 0.05;
        }),
        preset("hk-regular-consensus", "A hundred evenly spaced, ε = 0.25", "A hundred evenly spaced opinions, ε = 0.25 (Fig. 8): 'no split, total consensus'. Measured: no split, consensus at period 10.", |c| {
            regular(c, 100);
            c.epsilon = 0.25;
        }),
        preset("hk-asym-a", "Asymmetric: 0.02 left, 0.04 right", "Asymmetric confidence the same for everyone (§4.2.1, Fig. 10a): agents listen 0.02 to the left and 0.04 to the right. The profile drifts right. Measured (20 seeds): 12 surviving opinions (median), a mean opinion of 0.54 instead of 0.50.", |c| asymmetric(c, 0.02, 0.04)),
        preset("hk-asym-b", "Asymmetric: 0.03 left, 0.15 right", "Asymmetric confidence, 0.03 left and 0.15 right (Fig. 10b): a big camp near the right border and a smaller one left of it. Measured (20 seeds): two camps of a fifth or more in 11 runs, consensus in 2, a mean opinion of 0.84; one-sided splits (a gap one side reaches across and the other does not) open and close in every run.", |c| asymmetric(c, 0.03, 0.15)),
        preset("hk-asym-c", "Asymmetric: 0.10 left, 0.25 right", "Asymmetric confidence, 0.10 left and 0.25 right (Fig. 10c). Measured (20 seeds): two camps of a fifth or more in 16 runs, consensus in 4, a mean opinion of 0.75.", |c| asymmetric(c, 0.10, 0.25)),
        preset("hk-one-sided", "One-sided splits", "A hundred evenly spaced opinions reaching 0.08 left and 0.24 right, to watch one-sided splits (Fig. 13). The caption says εl = 0.8, which would reach every opinion below and could not split anything; 0.08 is read here. Measured: a one-sided split opens in period 5 and closes in period 9 — 'contrary to two-sided splits one-sided splits can close again' — and the profile ends in consensus at 0.85.", |c| {
            regular(c, 100);
            asymmetric(c, 0.08, 0.24);
        }),
        preset("hk-bias", "Confidence leaning with opinion", "Confidence that leans with one's opinion (§4.2.2): a total reach of 0.6, split so that agents left of center look further left and those right of it further right, with bias m = 0.5 (Fig. 18c, fifty evenly spaced opinions). The paper: 'in period 4 the profile splits finally and two polarised opinion camps remain'. Measured: the split in period 4, two camps 0.50 apart, stable at period 6.", |c| {
            regular(c, 50);
            c.confidence = Confidence::OpinionDependent;
            c.epsilon = 0.6;
            c.bias = 0.5;
        }),
        preset("hk-serial", "Two camps, updated one at a time", "The two-camp society (ε = 0.15) updated one agent at a time in a fresh random order each period, each seeing the opinions already moved this period. HK §4.3: 'Random serial updating gives extreme opinions a slightly better chance to survive. But none of the results … depends crucially on simultaneous updating' — a claim with no figure, and no word on which serial order. Measured (50 seeds, hk-updating and the survey): the phases keep their places; serial updating leaves slightly more opinions at small ε.", |c| {
            c.updating = Updating::SerialShuffled;
        }),
        preset("hk-lattice", "Two camps on a lattice?", "The two-camp society on a 25 × 25 torus where each agent hears only itself and its eight neighbors — HK §4.3's 'first simulations': with small, overlapping neighborhoods 'polarization … disappears'. The torus is drawn right of the diagram. Measured (20 seeds, run to stability): a second camp holding a fifth of the agents in 7 runs at ε = 0.15 and in none at any other ε from 0.05 to 0.6; instead one big camp and dozens of stranded local minorities, settling only after thousands of periods. The claim largely holds. Its Largest camps chart under-reads for the first thousands of periods: a lattice camp closes slowly, and opinions count as one only within 10⁻⁶.", |c| {
            c.interaction = Interaction::Lattice;
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, OpinionsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Opinions(c) => (p.id, c),
                _ => panic!("{} is not a bounded-confidence preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("hk-polarisation"), OpinionsConfig::default());
        assert_eq!(find("hk-plurality").epsilon, 0.01);
        let r = find("hk-regular-50");
        assert_eq!((r.agents, r.start, r.epsilon), (50, Start::Regular, 0.2));
        let o = find("hk-one-sided");
        assert_eq!(
            (o.epsilon_left, o.epsilon_right, o.agents),
            (0.08, 0.24, 100)
        );
        let b = find("hk-bias");
        assert_eq!(
            (b.confidence, b.epsilon, b.bias),
            (Confidence::OpinionDependent, 0.6, 0.5)
        );
        assert_eq!(find("hk-serial").updating, Updating::SerialShuffled);
        assert_eq!(find("hk-lattice").interaction, Interaction::Lattice);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
