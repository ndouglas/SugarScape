//! Schelling's own models (milestone 30): his checkerboard and his line (1971,
//! "Dynamic Models of Segregation", J. Math. Sociol. 1: 143–186). Each
//! decision rule takes its numbers from Schelling's text and figures, and was
//! written before the run. His 2-D numbers come from hand-worked examples ("My
//! samples have been too small, so far, to allow serious generalizations",
//! p. 158), so a rule tests whether they are typical, not whether one board
//! matches.

use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, model_preset};

const SCHELLING: &str = "Schelling 1971, J. Math. Sociol. 1";
const BOARD_TICKS: u32 = 60;
const LINE_TICKS: u32 = 50;

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

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The median over seeds of `series`' last value, `ticks` into preset `id`
/// (edited by `edit` when it is a Schelling board).
fn last(id: &str, seeds: &[u64], ticks: u32, series: &str, edit: fn(&mut ModelConfig)) -> f64 {
    let mut c = model_preset(id);
    edit(&mut c);
    median(model_after(&c, seeds, ticks, |w: &ModelWorld| {
        w.model().latest_value(series).unwrap()
    }))
}

fn same(_: &mut ModelConfig) {}

/// Total moves over the run, median over seeds.
fn moves(id: &str, seeds: &[u64]) -> f64 {
    median(model_after(
        &model_preset(id),
        seeds,
        BOARD_TICKS,
        |w: &ModelWorld| w.model().series("moves").unwrap().iter().sum(),
    ))
}

fn with_preference(c: &mut ModelConfig, p: f64) {
    if let ModelConfig::Schelling(s) = c {
        s.preference.min = p;
        s.preference.max = p;
    }
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "s71.board.exaggerated",
            item: "s71-board",
            source: Source::Book,
            citation: SCHELLING,
            text: "The checkerboard with \"no fewer than half\" alike wanted: Fig. 9 ends with neighbors four-fifths to five-sixths alike and 40 % with no neighbor of the other color (Fig. 8: 90 % and two-thirds). Holds if the median like share is at least 0.80 and the median share with no opposite neighbor at least 0.40",
            check: |seeds| {
                let like = last("s71-board", seeds, BOARD_TICKS, "segregation", same);
                let none = last("s71-board", seeds, BOARD_TICKS, "no_unlike", same);
                outcome(
                    like >= 0.8 && none >= 0.4,
                    format!("like share {like:.3}; no opposite neighbor {none:.3} (medians)"),
                )
            },
        },
        Claim {
            id: "s71.board.four-to-one",
            item: "s71-board",
            source: Source::Book,
            citation: SCHELLING,
            text: "\"the resulting ratios of like to opposite neighbors is upwards of four to one for demands of one-half or more\" (p. 158). Holds if the median ratio is at least 4",
            check: |seeds| {
                let r = last("s71-board", seeds, BOARD_TICKS, "like_ratio", same);
                outcome(r >= 4.0, format!("like to unlike ratio {r:.2} (median)"))
            },
        },
        Claim {
            id: "s71.third.slight",
            item: "s71-third",
            source: Source::Book,
            citation: SCHELLING,
            text: "Demands of about one-third (Fig. 11) give \"slight\" segregation: ratios of like to opposite neighbors \"less than 1.5\" (p. 158). Holds if the median ratio is below 1.5",
            check: |seeds| {
                let r = last("s71-third", seeds, BOARD_TICKS, "like_ratio", same);
                outcome(r < 1.5, format!("like to unlike ratio {r:.2} (median)"))
            },
        },
        Claim {
            id: "s71.demand.steep",
            item: "s71-demand",
            source: Source::Book,
            citation: SCHELLING,
            text: "Segregation is \"a rapidly rising function of demands in the range from about 35% to 50%\" (p. 159). Holds if the median like share rises more from a 35 % to a 50 % demand than from 20 % to 35 %",
            check: |seeds| {
                let at = |p: f64| {
                    let mut c = model_preset("s71-board");
                    with_preference(&mut c, p);
                    median(model_after(&c, seeds, BOARD_TICKS, |w: &ModelWorld| {
                        w.model().latest_value("segregation").unwrap()
                    }))
                };
                let (a, b, c) = (at(0.2), at(0.35), at(0.5));
                outcome(
                    c - b > b - a,
                    format!("like share {a:.3} at 20 %, {b:.3} at 35 %, {c:.3} at 50 % (medians)"),
                )
            },
        },
        Claim {
            id: "s71.order.character",
            item: "s71-order",
            source: Source::Book,
            citation: SCHELLING,
            text: "\"The particular outcome will depend very much on the order in which discontented stars and zeros are moved, the character of the outcome not very much\" (p. 156). Holds if the median like shares of rounds from the upper left, rounds from the center out and a random order all lie within 0.03",
            check: |seeds| {
                let reading = last("s71-board", seeds, BOARD_TICKS, "segregation", same);
                let center = last("s71-center-out", seeds, BOARD_TICKS, "segregation", same);
                let random = last("s71-board", seeds, BOARD_TICKS, "segregation", |c| {
                    if let ModelConfig::Schelling(s) = c {
                        s.order = sugarscape_core::schelling::Order::Random;
                    }
                });
                let (lo, hi) = (reading.min(center).min(random), reading.max(center).max(random));
                outcome(
                    hi - lo <= 0.03,
                    format!("like share {reading:.3}, {center:.3} and {random:.3} (medians)"),
                )
            },
        },
        Claim {
            id: "s71.unequal.denser",
            item: "s71-unequal-demands",
            source: Source::Book,
            citation: SCHELLING,
            text: "Unequal demands (Fig. 12): \"the more demanding end up with a higher proportion of like neighbors, but not much higher\", and \"in more densely populated neighborhoods\" (pp. 159, 163). Holds if the stars' (Red) median like share is at least the zeros' but within 0.1, and the stars' median neighbors exceed the zeros'",
            check: |seeds| {
                let id = "s71-unequal-demands";
                let (lr, lb) = (
                    last(id, seeds, BOARD_TICKS, "like_red", same),
                    last(id, seeds, BOARD_TICKS, "like_blue", same),
                );
                let (nr, nb) = (
                    last(id, seeds, BOARD_TICKS, "neighbors_red", same),
                    last(id, seeds, BOARD_TICKS, "neighbors_blue", same),
                );
                outcome(
                    lr >= lb && lr - lb <= 0.1 && nr > nb,
                    format!("like share stars {lr:.3}, zeros {lb:.3}; neighbors {nr:.2} and {nb:.2} (medians)"),
                )
            },
        },
        Claim {
            id: "s71.minority.two-to-one",
            item: "s71-minority",
            source: Source::Book,
            citation: SCHELLING,
            text: "A minority half the size of the majority, each wanting two like neighbors (Fig. 13): the minority goes from about 1:2 like to unlike \"to 2:1\", and is \"denser\" (pp. 160–161, 164). Holds if the minority's (Blue) median like share is at least 0.6 (a ratio of 1.5: our tolerance around his 2:1) and its median neighbors exceed the majority's",
            check: |seeds| {
                let id = "s71-minority";
                let like = last(id, seeds, BOARD_TICKS, "like_blue", same);
                let (nb, nr) = (
                    last(id, seeds, BOARD_TICKS, "neighbors_blue", same),
                    last(id, seeds, BOARD_TICKS, "neighbors_red", same),
                );
                outcome(
                    like >= 0.6 && nb > nr,
                    format!("minority like share {like:.3}; neighbors {nb:.2} against {nr:.2} (medians)"),
                )
            },
        },
        Claim {
            id: "s71.wide.attenuates",
            item: "s71-wide",
            source: Source::Book,
            citation: SCHELLING,
            text: "\"Enlarging the area within which a person counts his neighbors attenuates the tendency to segregate, at least for moderate demands and near-equal numbers\" (p. 164; no figure). Measured on one ruler, the like share among the eight surrounding squares, at a moderate demand of one-third (Fig. 11's table and its 24-square equivalent, a third alike). Holds if that median is lower counting 24 neighbors than counting eight",
            check: |seeds| {
                let near = |radius: u32, third: bool| {
                    let mut c = model_preset("s71-board");
                    if let ModelConfig::Schelling(s) = &mut c {
                        s.radius = radius;
                        if third {
                            s.preference.min = 1.0 / 3.0;
                            s.preference.max = 1.0 / 3.0;
                        }
                    }
                    median(model_after(&c, seeds, BOARD_TICKS, |w: &ModelWorld| {
                        w.model().latest_value("like_near").unwrap()
                    }))
                };
                let (e3, w3, e2, w2) = (near(1, true), near(2, true), near(1, false), near(2, false));
                outcome(
                    w3 < e3,
                    format!(
                        "like share among the eight around: at a third, {e3:.3} counting 8, {w3:.3} counting 24; at half, {e2:.3} and {w2:.3} (medians)"
                    ),
                )
            },
        },
        Claim {
            id: "s71.congregate.separate",
            item: "s71-congregate",
            source: Source::Book,
            citation: SCHELLING,
            text: "Congregationists wanting three alike of eight and indifferent to the other color (Fig. 16): \"like neighbors are just over 75%\", and \"he separates from the others just as if he had demanded majority status\" (p. 165). Holds if the median like share is at least 0.75",
            check: |seeds| {
                let like = last("s71-congregate", seeds, BOARD_TICKS, "segregation", same);
                outcome(like >= 0.75, format!("like share {like:.3} (median)"))
            },
        },
        Claim {
            id: "s71.integrate.harder",
            item: "s71-integrate",
            source: Source::Book,
            citation: SCHELLING,
            text: "Integrationist bands (Fig. 17): \"equilibrium is achieved only with a much larger number of moves … More individuals may be incapable of being satisfied\" (p. 166). Holds if the median total moves exceed those of the same population (two-thirds Red) wanting half alike, and the median share still unsatisfied at the end is above zero",
            check: |seeds| {
                let mi = moves("s71-integrate", seeds);
                let mut half = model_preset("s71-board");
                if let ModelConfig::Schelling(s) = &mut half {
                    s.red_share = 2.0 / 3.0;
                }
                let mb = median(model_after(&half, seeds, BOARD_TICKS, |w: &ModelWorld| {
                    w.model().series("moves").unwrap().iter().sum()
                }));
                let left = last("s71-integrate", seeds, BOARD_TICKS, "unsatisfied", same);
                outcome(
                    mi > mb && left > 0.0,
                    format!("moves {mi:.0} against {mb:.0}; unsatisfied at the end {left:.3} (medians)"),
                )
            },
        },
        Claim {
            id: "s71-line.groups",
            item: "s71-line",
            source: Source::Book,
            citation: SCHELLING,
            text: "The line (70, four neighbors each side, half alike): \"different random sequences yield from about five groupings with an average of 14 members to seven or eight groupings with an average of 9 or 10\" (p. 151). Holds if the median number of groups is 5 to 8 and the median mean group 9 to 14",
            check: |seeds| {
                let g = last("s71-line", seeds, LINE_TICKS, "groups", same);
                let m = last("s71-line", seeds, LINE_TICKS, "mean_group", same);
                outcome(
                    (5.0..=8.0).contains(&g) && (9.0..=14.0).contains(&m),
                    format!("{g:.1} groups of {m:.1} (medians)"),
                )
            },
        },
        Claim {
            id: "s71-line-3.groups",
            item: "s71-line-3",
            source: Source::Book,
            citation: SCHELLING,
            text: "Three neighbors each side: \"a mean of 7 or 8 per cluster … with the average person's neighborhood 75% to 80% his own color\" (p. 152). Holds if the median mean group is 7 to 8 (rounded to the nearest whole person) and the median like share 0.75 to 0.80",
            check: |seeds| {
                let m = last("s71-line-3", seeds, LINE_TICKS, "mean_group", same);
                let like = last("s71-line-3", seeds, LINE_TICKS, "like_share", same);
                outcome(
                    (6.5..8.5).contains(&m) && (0.75..=0.8).contains(&like),
                    format!("mean group {m:.2}; like share {like:.3} (medians)"),
                )
            },
        },
        Claim {
            id: "s71-line-minority.clusters",
            item: "s71-line-minority",
            source: Source::Book,
            citation: SCHELLING,
            text: "Halving the zeros (p. 152): \"the minority itself tends to become more segregated from the majority, as its relative size diminishes\" (p. 153). Holds if the zeros' (Blue) median like share is higher on the halved line than on the even one",
            check: |seeds| {
                let even = last("s71-line", seeds, LINE_TICKS, "like_blue", same);
                let minority = last("s71-line-minority", seeds, LINE_TICKS, "like_blue", same);
                outcome(
                    minority > even,
                    format!("the zeros' like share {minority:.3} when halved, {even:.3} on the even line (medians)"),
                )
            },
        },
        Claim {
            id: "s71-line-reach.everybody",
            item: "s71-line-reach",
            source: Source::Book,
            citation: SCHELLING,
            text: "Restricted movement with the fallback of three alike of eight: \"everybody achieves his desired neighborhood, half or more his own color, without traveling as far\" (p. 154; the radius is not given: here 10 people). Holds if the median share unsatisfied at the end is zero",
            check: |seeds| {
                let left = last("s71-line-reach", seeds, LINE_TICKS, "unsatisfied", same);
                outcome(left == 0.0, format!("unsatisfied at the end {left:.3} (median)"))
            },
        },
    ]
}
