//! Schelling's bounded neighborhood (milestone 31; 1971, pp. 167–186; 1969,
//! pp. 491–493). With exact tolerance schedules the model is deterministic,
//! so each claim runs once (the first seed); each rule takes its numbers from
//! Schelling's text and was written before the run.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::tipping::{Schedule, Start, TippingConfig};

use crate::claim::{Claim, Outcome, Source, Verdict};
use crate::runner::model_preset;

const SCHELLING: &str = "Schelling 1971, J. Math. Sociol. 1";
const SCHELLING69: &str = "Schelling 1969, AER 59";

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn preset(id: &str) -> TippingConfig {
    match model_preset(id) {
        ModelConfig::Tipping(c) => c,
        _ => panic!("{id} is not a tipping preset"),
    }
}

/// Red and Blue inside once nobody moves (or after 20 000 steps).
fn rest(c: &TippingConfig, seed: u64) -> (u32, u32) {
    let mut w = ModelWorld::new(ModelConfig::Tipping(c.clone()), seed).expect("a valid config");
    for _ in 0..20_000 {
        w.model_mut().run(1);
        if w.model().latest_value("still") == Some(1.0) {
            break;
        }
    }
    let m = w.model();
    (
        m.latest_value("red_in").unwrap() as u32,
        m.latest_value("blue_in").unwrap() as u32,
    )
}

fn from(c: &TippingConfig, red: u32, blue: u32) -> TippingConfig {
    TippingConfig {
        start: Start::Given { red, blue },
        ..c.clone()
    }
}

fn lines(c: &TippingConfig, red: f64, blue: f64) -> TippingConfig {
    TippingConfig {
        red_schedule: Schedule::Line { intercept: red },
        blue_schedule: Schedule::Line { intercept: blue },
        ..c.clone()
    }
}

fn mixed((r, b): (u32, u32)) -> bool {
    r > 0 && b > 0
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "tipping.fig18.two-stable",
            item: "tipping-fig18",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 18: \"There are only two stable equilibria. One consists of all the blacks and no whites, the other all the whites and no blacks.\" Holds if every start on a grid (Red 0–100 and Blue 0–50 inside, in steps of 10, someone inside) ends with one color gone",
            check: |seeds| {
                let c = preset("tipping-fig18");
                let mut ends = Vec::new();
                for r in (0..=100).step_by(10) {
                    for b in (0..=50).step_by(10) {
                        if r + b > 0 {
                            ends.push(rest(&from(&c, r, b), seeds[0]));
                        }
                    }
                }
                let kept = ends.iter().filter(|&&e| mixed(e)).count();
                outcome(kept == 0, format!("{} starts; {kept} end mixed", ends.len()))
            },
        },
        Claim {
            id: "tipping.fig19.eighty",
            item: "tipping-fig19",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 19: \"there is a stable mixture at 80 blacks and 80 whites\". Holds if half of each inside ends within 2 of 80 and 80",
            check: |seeds| {
                let (r, b) = rest(&preset("tipping-fig19"), seeds[0]);
                outcome(r.abs_diff(80) <= 2 && b.abs_diff(80) <= 2, format!("ends at {r} Red, {b} Blue"))
            },
        },
        Claim {
            id: "tipping.fig19.forty-percent",
            item: "tipping-start",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 19: \"As long as half or more of both colors are present—actually, slightly over 40% of both colors—the dynamics of entry and departure will lead to the stable mixture.\" Holds if every start with 41 to 100 of each inside (steps of 3, not only equal) ends within 2 of 80 and 80",
            check: |seeds| {
                let c = preset("tipping-fig19");
                let (mut n, mut missed) = (0, Vec::new());
                for r in (41..=100).step_by(3) {
                    for b in (41..=100).step_by(3) {
                        n += 1;
                        let (er, eb) = rest(&from(&c, r, b), seeds[0]);
                        if er.abs_diff(80) > 2 || eb.abs_diff(80) > 2 {
                            missed.push((r, b, er, eb));
                        }
                    }
                }
                outcome(missed.is_empty(), format!("{n} starts; {} miss the mixture {:?}", missed.len(), missed.first()))
            },
        },
        Claim {
            id: "tipping.fig19.concerted-entry",
            item: "tipping-entry",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 19, from all of one color: \"it would require the concerted entry of more than 25% of the other color\". Holds if the fewest Blue who, entering an all-Red area together, reach the mixture number 26 to 30",
            check: |seeds| {
                let c = preset("tipping-fig19");
                let k = (0..=100).find(|&k| mixed(rest(&from(&c, 100, k), seeds[0])));
                outcome(matches!(k, Some(26..=30)), format!("the fewest: {k:?}"))
            },
        },
        Claim {
            id: "tipping.fig20.lost",
            item: "tipping-fig20",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 20: \"The stable equilibrium generated in Figure 19 disappears if … whites exceed blacks by, say, two to one.\" Holds if, with 200 Red and 100 Blue, every start with both colors inside (steps of 20) ends with one color gone",
            check: |seeds| {
                let c = preset("tipping-fig20");
                let mut kept = 0;
                let mut n = 0;
                for r in (20..=200).step_by(20) {
                    for b in (20..=100).step_by(20) {
                        n += 1;
                        kept += usize::from(mixed(rest(&from(&c, r, b), seeds[0])));
                    }
                }
                outcome(kept == 0, format!("{n} starts; {kept} end mixed"))
            },
        },
        Claim {
            id: "tipping.fig21.threshold",
            item: "tipping-intercept",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 21: \"For straight-line tolerance schedules and equal numbers of the two colors, there is no stable intersection of the two parabolas unless the tolerance schedules have vertical intercepts of 3.0\". Holds if, from 55 Red and 45 Blue, intercepts of 2.5, 2.8 and 2.9 end one-colored and 3.0 and 3.5 end mixed",
            check: |seeds| {
                let c = preset("tipping-fig21");
                let at = |a: f64| rest(&from(&lines(&c, a, a), 55, 45), seeds[0]);
                let below = [2.5, 2.8, 2.9].map(at);
                let above = [3.0, 3.5].map(at);
                outcome(
                    below.iter().all(|&e| !mixed(e)) && above.iter().all(|&e| mixed(e)),
                    format!("below 3: {below:?}; at 3.0 and 3.5: {above:?}"),
                )
            },
        },
        Claim {
            id: "tipping.fig22.limit",
            item: "tipping-fig22",
            source: Source::Book,
            citation: SCHELLING,
            text: "Fig. 22: with whites limited to 40, \"a stable mixture at 40 whites and a comparable number of blacks\". Holds if Red limited to 40 ends mixed with 40 Red (his \"comparable number\" is not quantified: the Blue count is reported)",
            check: |seeds| {
                let (r, b) = rest(&preset("tipping-fig22"), seeds[0]);
                outcome(r == 40 && b > 0, format!("ends at {r} Red, {b} Blue"))
            },
        },
        Claim {
            id: "tipping.1969.forty-apiece",
            item: "tipping-intolerant",
            source: Source::Book,
            citation: SCHELLING69,
            text: "1969: \"Make the least tolerant 60 percent of blacks and whites absolutely intolerant … and a stable equilibrium will occur at forty apiece.\" Holds if it ends at exactly 40 and 40",
            check: |seeds| {
                let (r, b) = rest(&preset("tipping-intolerant"), seeds[0]);
                outcome((r, b) == (40, 40), format!("ends at {r} Red, {b} Blue"))
            },
        },
        Claim {
            id: "tipping.minority.more-tolerant",
            item: "tipping-minority",
            source: Source::Book,
            citation: SCHELLING,
            text: "\"for a stable mixture, the minority must be the more tolerant of the two groups\" (p. 179). Holds if a 5:1 minority with the majority's own schedule (Fig. 19's) ends pushed out from every start with both inside (steps of 20)",
            check: |seeds| {
                let c = preset("tipping-minority");
                let mut kept = 0;
                let mut n = 0;
                for r in (20..=500).step_by(60) {
                    for b in (20..=100).step_by(20) {
                        n += 1;
                        kept += usize::from(mixed(rest(&from(&c, r, b), seeds[0])));
                    }
                }
                outcome(kept == 0, format!("{n} starts; {kept} end mixed"))
            },
        },
        Claim {
            id: "tipping.less-tolerant.paradox",
            item: "tipping-less-tolerant",
            source: Source::Book,
            citation: SCHELLING,
            text: "\"replacing the two-thirds least tolerant whites … by even less tolerant whites keeps the whites from overwhelming the blacks by their numbers. This would not happen if we made all whites less tolerant.\" (p. 174). Holds if Fig. 20's numbers from 40 and 40 end mixed with the least tolerant two-thirds of Red intolerant, but one-colored as they are and with all Red less tolerant (the intercept cut to a third, 5/3)",
            check: |seeds| {
                let less = preset("tipping-less-tolerant");
                let plain = TippingConfig {
                    intolerant_red: 0.0,
                    ..less.clone()
                };
                let all_less = TippingConfig {
                    red_schedule: Schedule::Line { intercept: 5.0 / 3.0 },
                    ..plain.clone()
                };
                let (a, b, c) = (rest(&less, seeds[0]), rest(&plain, seeds[0]), rest(&all_less, seeds[0]));
                outcome(
                    mixed(a) && !mixed(b) && !mixed(c),
                    format!("least tolerant two-thirds intolerant: {a:?}; as they are: {b:?}; all less tolerant: {c:?}"),
                )
            },
        },
    ]
}
