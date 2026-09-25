//! Riolo, Cohen & Axelrod 2001's tag-based donation (milestone 12): the
//! paper's tables and figures, Edmonds & Hales' and Roberts & Sherratt's
//! replications, and the presets' descriptions. Each run is 30 000
//! generations, as in the paper; runs that several claims share are
//! memoized per process, keyed by the config and the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::tags::{DonationTest, InitialTolerance, Selection, TagsConfig, TieRule};

use crate::claim::{range, Claim, Outcome, Source};
use crate::runner::model_after;

const RCA: &str = "Riolo, Cohen & Axelrod 2001, Nature 414";
const EH: &str = "Edmonds & Hales 2003, JASSS 6(4)";
const RS: &str = "Roberts & Sherratt 2002, Nature 418";
const GENERATIONS: u32 = 30_000;
/// RCA exclude "the transient period of the first 100 generations".
const TRANSIENT: usize = 100;

/// One run, summarized.
#[derive(Clone, Copy, Debug)]
struct Run {
    /// Mean donation rate over every generation (RCA's average).
    donation: f64,
    /// Generation 0's donation rate.
    first: f64,
    /// The lowest donation rate in generations 1–150.
    dip: f64,
    /// Takeovers over the run.
    takeovers: f64,
    /// Mean cluster share over the generations it is dominant.
    share: f64,
    /// Mean relatedness when a cluster takes over after the transient, and ten generations later.
    related_start: f64,
    related_later: f64,
    /// Mean cluster tolerance when a cluster takes over, and in its last generation.
    tolerance_start: f64,
    tolerance_end: f64,
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

/// A dominant cluster's life: the generation it took over and its last
/// dominant generation before the next takeover (or the run's end).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Whether it was still the dominant cluster `n` generations after taking over.
    fn lasts(&self, n: usize) -> bool {
        self.end >= self.start + n
    }
}

/// The dominant clusters that took over after generation `after`. Every
/// dominant generation between two takeovers belongs to the first cluster
/// (a dominant cluster far from it would itself be a takeover), so a cluster
/// ends at its last generation above half, which may be long before the
/// invader's takeover.
fn spans(share: &[f64], takeovers: &[f64], after: usize) -> Vec<Span> {
    let starts: Vec<usize> = (after + 1..takeovers.len())
        .filter(|&t| takeovers[t] > takeovers[t - 1])
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let next = starts.get(i + 1).copied().unwrap_or(share.len());
            let end = (start..next)
                .rev()
                .find(|&t| share[t] > 0.5)
                .unwrap_or(start);
            Span { start, end }
        })
        .collect()
}

fn summarize(w: &ModelWorld) -> Run {
    let s = |name: &str| w.model().series(name).expect("a tags series");
    let (donation, share, related, tolerance, takeovers) = (
        s("donation_rate"),
        s("cluster_share"),
        s("relatedness"),
        s("cluster_tolerance"),
        s("takeovers"),
    );
    let spans = spans(&share, &takeovers, TRANSIENT);
    let starts: Vec<usize> = spans.iter().map(|s| s.start).collect();
    let at = |v: &[f64], ts: &[usize]| {
        mean(
            &ts.iter()
                .filter_map(|&t| v.get(t).copied())
                .collect::<Vec<_>>(),
        )
    };
    // Only clusters still dominant ten generations on; only clusters that ended
    // (were replaced), at their last dominant generation.
    let later: Vec<usize> = spans
        .iter()
        .filter(|s| s.lasts(10))
        .map(|s| s.start + 10)
        .collect();
    let ends: Vec<usize> = spans.windows(2).map(|p| p[0].end).collect();
    Run {
        donation: mean(&donation),
        first: donation[0],
        dip: donation[1..=150]
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min),
        takeovers: *takeovers.last().expect("a generation"),
        share: mean(
            &share
                .iter()
                .copied()
                .filter(|&x| x > 0.5)
                .collect::<Vec<_>>(),
        ),
        related_start: at(&related, &starts),
        related_later: at(&related, &later),
        tolerance_start: at(&tolerance, &starts),
        tolerance_end: at(&tolerance, &ends),
    }
}

/// The paper's defaults with `edit` applied, run for every seed (memoized).
fn runs(seeds: &[u64], edit: impl FnOnce(&mut TagsConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, Vec<u64>, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = TagsConfig {
        end: 0,
        ..TagsConfig::default()
    };
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, _)| *k == key && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Tags(c),
        seeds,
        GENERATIONS,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds.to_vec(), v.clone()));
    v
}

fn donation(seeds: &[u64], edit: impl FnOnce(&mut TagsConfig)) -> Vec<f64> {
    runs(seeds, edit).iter().map(|r| r.donation).collect()
}

fn ties(t: TieRule) -> impl Fn(&mut TagsConfig) {
    move |c| c.tie_rule = t
}

/// A range check on the mean donation rate that also counts the seeds
/// whose cooperation collapsed (below 10 %).
fn with_collapses(v: &[f64], lo: f64, hi: f64) -> Outcome {
    let collapsed = v.iter().filter(|&&d| d < 0.1).count();
    range(v, lo, hi, false).with(&format!("{collapsed}/{} seeds below 10 %.", v.len()))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "tags.table-1.p1",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 1: with one pairing the donation rate is 2.1 %; with ties to the current agent, 0–5 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.pairings = 1;
                }), 0.0, 0.05, false)
            },
        },
        Claim {
            id: "tags.table-1.p2",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 1: with two pairings, 4.3 %; with ties to the current agent, 0–10 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.pairings = 2;
                }), 0.0, 0.1, false)
            },
        },
        Claim {
            id: "tags.table-1.p3",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 1: with three pairings, 73.6 %; with ties to the current agent, 70–78 %",
            check: |s| range(&donation(s, ties(TieRule::Current)), 0.70, 0.78, false),
        },
        Claim {
            id: "tags.table-1.p10",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 1: with ten pairings, 79.2 %; with ties to the current agent, 76–82 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.pairings = 10;
                }), 0.76, 0.82, false)
            },
        },
        Claim {
            id: "tags.table-2.c05",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 2: at cost 0.5, 24.7 %; with ties to the current agent, 15–35 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.cost = 0.5;
                }), 0.15, 0.35, false)
            },
        },
        Claim {
            id: "tags.table-2.c06",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Table 2: at cost 0.6, 2.2 %; with ties to the current agent, 0–5 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.cost = 0.6;
                }), 0.0, 0.05, false)
            },
        },
        Claim {
            id: "tags.literal.p2",
            item: "rca-literal",
            source: Source::Comment,
            citation: EH,
            text: "Table 7: ties by coin flip give 42.6 % at two pairings, not the paper's 4.3 %: 35–50 %",
            check: |s| {
                range(&donation(s, |c| c.pairings = 2), 0.35, 0.50, false)
            },
        },
        Claim {
            id: "tags.literal.p2-opponent",
            item: "rca-literal",
            source: Source::Comment,
            citation: EH,
            text: "Table 7: ties to the opponent give 49.6 % at two pairings: 42–57 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Other;
                    c.pairings = 2;
                }), 0.42, 0.57, false)
            },
        },
        Claim {
            id: "tags.literal.c05",
            item: "rca-literal",
            source: Source::Comment,
            citation: EH,
            text: "Table 8: ties by coin flip give 45.9 % at cost 0.5, not the paper's 24.7 %: 40–52 %",
            check: |s| range(&donation(s, |c| c.cost = 0.5), 0.40, 0.52, false),
        },
        Claim {
            id: "tags.strict.none",
            item: "rca-strict",
            source: Source::Comment,
            citation: EH,
            text: "Tables 9–10: donating only when |Δtag| < tolerance wipes donation out: 0.0 % (at most 0.5 %)",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.donation_test = DonationTest::Below;
                }), 0.0, 0.005, false)
            },
        },
        Claim {
            id: "tags.strict.collapse",
            item: "rca-strict",
            source: Source::App,
            citation: "spec 2026-09-25-tags-design.md, Finding",
            text: "donating only when |Δtag| < tolerance collapses donation below 6 % under every tie rule",
            check: |s| {
                let mut v = Vec::new();
                for t in [TieRule::Random, TieRule::Current, TieRule::Other] {
                    v.extend(donation(s, |c| {
                        c.tie_rule = t;
                        c.donation_test = DonationTest::Below;
                    }));
                }
                range(&v, 0.0, 0.06, false)
            },
        },
        Claim {
            id: "tags.floor.collapse",
            item: "rs-no-forced-clones",
            source: Source::Comment,
            citation: RS,
            text: "a tolerance floor of −10⁻⁶ cuts donation to 1.48 %: 0.5–3 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.tolerance_floor = -1e-6;
                }), 0.005, 0.03, false)
            },
        },
        Claim {
            id: "tags.zero-tolerance.current",
            item: "eh-clones-only",
            source: Source::Comment,
            citation: EH,
            text: "Table 11: with tolerance fixed at 0 and ties to the current agent, no donation (at most 0.5 %)",
            check: |s| {
                range(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.initial_tolerance = InitialTolerance::Fixed(0.0);
                    c.tolerance_mutation = 0.0;
                }), 0.0, 0.005, false)
            },
        },
        Claim {
            id: "tags.zero-tolerance.coin-flip",
            item: "eh-clones-only",
            source: Source::Comment,
            citation: EH,
            text: "Table 11: with tolerance fixed at 0 and ties by coin flip, 75.3 % — tolerance does no work: 70–80 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.initial_tolerance = InitialTolerance::Fixed(0.0);
                    c.tolerance_mutation = 0.0;
                }), 0.70, 0.80, false)
            },
        },
        Claim {
            id: "tags.noise.collapse",
            item: "eh-no-exact-clones",
            source: Source::Comment,
            citation: EH,
            text: "Tables 13–14: tag noise of 10⁻⁶, so no two tags are equal, gives 1.5–5.1 %: below 6 % under every tie rule",
            check: |s| {
                let mut v = Vec::new();
                for t in [TieRule::Random, TieRule::Current, TieRule::Other] {
                    v.extend(donation(s, |c| {
                        c.tie_rule = t;
                        c.tag_noise = 1e-6;
                    }));
                }
                range(&v, 0.0, 0.06, false)
            },
        },
        Claim {
            id: "tags.population.200",
            item: "rca-published",
            source: Source::Comment,
            citation: EH,
            text: "with 200 agents \"the donations rates vanished\": below 10 % with ties to the current agent",
            check: |s| {
                with_collapses(&donation(s, |c| {
                    c.tie_rule = TieRule::Current;
                    c.agents = 200;
                }), 0.0, 0.1)
            },
        },
        Claim {
            id: "tags.adopt.p1",
            item: "rca-adopt-p1",
            source: Source::Book,
            citation: RCA,
            text: "adopting a better agent's traits in proportion to how much better, one pairing gives 49 % (b + c as the scale): 40–58 %",
            check: |s| {
                range(&donation(s, |c| {
                    c.selection = Selection::Adopt;
                    c.pairings = 1;
                }), 0.40, 0.58, false)
            },
        },
        Claim {
            id: "tags.fig-1.start",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Fig. 1: the initial donation rate is about 67 %",
            check: |s| {
                let v: Vec<f64> = runs(s, ties(TieRule::Current)).iter().map(|r| r.first).collect();
                range(&v, 0.60, 0.74, false)
            },
        },
        Claim {
            id: "tags.fig-1.dip",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "Fig. 1: low-tolerance agents take over within a few generations and donation falls (to 43 % by generation 70 in the run shown): below 50 % in generations 1–150",
            check: |s| {
                let v: Vec<f64> = runs(s, ties(TieRule::Current)).iter().map(|r| r.dip).collect();
                range(&v, 0.0, 0.5, false)
            },
        },
        Claim {
            id: "tags.cycle",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "\"The cycle continues indefinitely\": Fig. 1 shows takeovers at generations 226 and 356, about 120 per 30 000 generations; at least 60",
            check: |s| {
                let v: Vec<f64> = runs(s, ties(TieRule::Current)).iter().map(|r| r.takeovers).collect();
                range(&v, 60.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "tags.cluster.share",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "a dominant cluster holds about 75–80 % of the agents",
            check: |s| {
                let v: Vec<f64> = runs(s, ties(TieRule::Current)).iter().map(|r| r.share).collect();
                range(&v, 0.75, 0.80, false)
            },
        },
        Claim {
            id: "tags.cluster.relatedness-start",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "a cluster's relatedness when it first becomes dominant averages 79 %",
            check: |s| {
                let v: Vec<f64> =
                    runs(s, ties(TieRule::Current)).iter().map(|r| r.related_start).collect();
                range(&v, 0.75, 0.83, false)
            },
        },
        Claim {
            id: "tags.cluster.relatedness-later",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "ten generations later its relatedness is 97 %",
            check: |s| {
                let v: Vec<f64> =
                    runs(s, ties(TieRule::Current)).iter().map(|r| r.related_later).collect();
                range(&v, 0.95, 0.99, false)
            },
        },
        Claim {
            id: "tags.cluster.tolerance-start",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "a dominant cluster's mean tolerance is 0.010 at its beginning (widened by 10 %)",
            check: |s| {
                let v: Vec<f64> =
                    runs(s, ties(TieRule::Current)).iter().map(|r| r.tolerance_start).collect();
                range(&v, 0.008, 0.012, true)
            },
        },
        Claim {
            id: "tags.cluster.tolerance-end",
            item: "rca-published",
            source: Source::Book,
            citation: RCA,
            text: "and 0.027 at its end (widened by 10 %)",
            check: |s| {
                let v: Vec<f64> =
                    runs(s, ties(TieRule::Current)).iter().map(|r| r.tolerance_end).collect();
                range(&v, 0.022, 0.032, true)
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cluster_ends_at_its_last_dominant_generation_not_before_the_next_takeover() {
        // Generations 0–11. Takeovers at 2 and 9; the first cluster is dominant
        // at 2–5, loses dominance at 6–8 while the invader grows, then 9 takes over.
        let share = [0.3, 0.3, 0.6, 0.8, 0.8, 0.7, 0.4, 0.4, 0.45, 0.6, 0.7, 0.7];
        let takeovers = [0., 0., 1., 1., 1., 1., 1., 1., 1., 2., 2., 2.];
        let s = spans(&share, &takeovers, 0);
        assert_eq!(
            s,
            vec![Span { start: 2, end: 5 }, Span { start: 9, end: 11 }]
        );
        assert!(!s[1].lasts(10), "the run ends before generation 19");
        assert!(Span { start: 2, end: 12 }.lasts(10));
        assert!(
            spans(&share, &takeovers, 5).iter().all(|s| s.start > 5),
            "the transient is skipped"
        );
    }
}
