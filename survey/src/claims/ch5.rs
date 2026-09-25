//! Chapter V: v-1-rid, v-2-endemic, v-mcneill.

use std::collections::BTreeSet;

use sugarscape_core::world::World;

use crate::claim::{greater, range, Claim, Source};
use crate::runner::{after, each_seed, preset, series, window_mean};

/// The infected_fraction series of `id` after `ticks` ticks, per seed.
fn infected(id: &str, ticks: u32, seeds: &[u64]) -> Vec<Vec<f64>> {
    after(&preset(id), seeds, ticks, |w| series(w, "infected_fraction"))
}

/// (agent, disease) pairs where the disease is a substring of the agent's
/// trained immune string.
fn immunities(w: &World) -> BTreeSet<(u64, usize)> {
    w.agents()
        .flat_map(|a| {
            w.diseases
                .iter()
                .enumerate()
                .filter(|(_, d)| a.immune.contains(d))
                .map(move |(i, _)| (a.id, i))
        })
        .collect()
}

/// Per seed: immunities lost over the first `ticks` ticks. An immunity is
/// lost when an agent alive at both t and t + 1 is immune to a disease at t
/// and no longer immune to it at t + 1 (its window was overwritten).
fn lost_immunities(id: &str, ticks: u32, seeds: &[u64]) -> Vec<f64> {
    each_seed(&preset(id), seeds, |mut w| {
        let mut before = immunities(&w);
        let mut lost = 0u64;
        for _ in 0..ticks {
            w.step();
            let now = immunities(&w);
            let alive: BTreeSet<u64> = w.agents().map(|a| a.id).collect();
            lost += before
                .iter()
                .filter(|(a, d)| alive.contains(a) && !now.contains(&(*a, *d)))
                .count() as u64;
            before = now;
        }
        lost as f64
    })
}

/// Per seed, for v-mcneill run to t = 400: agent-to-agent transmissions
/// (`infector: Some(_)`) during the steps starting at ticks 200..300 and
/// 300..400, and the outbreak's own seeded infections (`infector: None`)
/// during the step starting at tick 300. Mirrors
/// `a_novel_disease_spreads_after_the_mcneill_outbreak` in tests/book.rs.
fn mcneill_transmissions(seeds: &[u64]) -> Vec<[f64; 3]> {
    each_seed(&preset("v-mcneill"), seeds, |mut w| {
        let (mut before, mut after, mut seeded) = (0u32, 0u32, 0u32);
        for _ in 0..400 {
            let tick = w.tick;
            w.step();
            let inf = &w.events().infections;
            let transmitted = inf.iter().filter(|i| i.infector.is_some()).count() as u32;
            if (200..300).contains(&tick) {
                before += transmitted;
            } else if (300..400).contains(&tick) {
                after += transmitted;
            }
            if tick == 300 {
                seeded = inf.iter().filter(|i| i.infector.is_none()).count() as u32;
            }
        }
        [f64::from(before), f64::from(after), f64::from(seeded)]
    })
}

fn col(rows: &[[f64; 3]], i: usize) -> Vec<f64> {
    rows.iter().map(|r| r[i]).collect()
}

fn min_over(s: &[f64], from: usize, to: usize) -> f64 {
    s[from..=to].iter().copied().fold(f64::INFINITY, f64::min)
}

pub fn claims() -> Vec<Claim> {
    vec![
        // ---- v-1-rid ----
        Claim {
            id: "v-1.learn",
            item: "v-1-rid",
            source: Source::App,
            citation: "presets.rs v-1-rid description",
            text: "immune systems learn the diseases their agents carry (infected fraction at t = 1000 is below that at t = 0)",
            check: |s| {
                let f = infected("v-1-rid", 1000, s);
                let start: Vec<f64> = f.iter().map(|f| f[0]).collect();
                let end: Vec<f64> = f.iter().map(|f| f[1000]).collect();
                greater(&start, &end, "infected t=0", "infected t=1000")
            },
        },
        Claim {
            id: "v-1.residue",
            item: "v-1-rid",
            source: Source::App,
            citation: "presets.rs v-1-rid description: \"a residue of ~1-3% persists\"; README.md: \"about 1–3%\"",
            text: "a residue of ~1–3% infected persists (mean infected_fraction over t = 500..=1000 in about [0.01, 0.03])",
            check: |s| {
                let m: Vec<f64> = infected("v-1-rid", 1000, s)
                    .iter()
                    .map(|f| window_mean(f, 500, 1000))
                    .collect();
                range(&m, 0.01, 0.03, true)
            },
        },
        Claim {
            id: "v-1.overwrite",
            item: "v-1-rid",
            source: Source::App,
            citation: "presets.rs v-1-rid description",
            text: "learning one disease can overwrite the window that cured another (at least one immunity lost by a surviving agent over t = 0..200)",
            // Metric: count of (agent, disease) immunities (disease a
            // substring of the trained immune string) held at t and gone at
            // t + 1 by an agent alive at both, summed over the first 200 ticks.
            check: |s| range(&lost_immunities("v-1-rid", 200, s), 1.0, f64::INFINITY, false),
        },
        Claim {
            id: "v-1.near-eradication",
            item: "v-1-rid",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md (Animation V-1: \"the immune response drives near-eradication\"); tests/book.rs immune_learning_rids_the_society_of_disease asserts f[1000] < 0.05",
            text: "the immune response drives near-eradication (infected_fraction at t = 1000 below 0.05)",
            check: |s| {
                let f: Vec<f64> = infected("v-1-rid", 1000, s).iter().map(|f| f[1000]).collect();
                range(&f, 0.0, 0.05, false)
            },
        },
        Claim {
            id: "v-1.saturated-start",
            item: "v-1-rid",
            source: Source::Book,
            citation: "tests/book.rs immune_learning_rids_the_society_of_disease: \"drives the population from near-saturation\", asserts f[0] > 0.8",
            text: "infection starts near saturation (infected_fraction at t = 0 above 0.8)",
            check: |s| {
                let f: Vec<f64> = infected("v-1-rid", 0, s).iter().map(|f| f[0]).collect();
                range(&f, 0.8, 1.0, false)
            },
        },
        Claim {
            id: "v-1.never-zero",
            item: "v-1-rid",
            source: Source::Comment,
            citation: "tests/book.rs immune_learning_rids_the_society_of_disease: \"the fraction never once touches 0.0 ... (minimum over t in 500..=5000 is 0.0090-0.0303 depending on seed)\"",
            text: "infection never reaches zero (minimum infected_fraction over t = 500..=5000 above 0)",
            check: |s| {
                let m: Vec<f64> = infected("v-1-rid", 5000, s)
                    .iter()
                    .map(|f| min_over(f, 500, 5000))
                    .collect();
                range(&m, f64::MIN_POSITIVE, 1.0, false)
            },
        },
        // ---- v-2-endemic ----
        Claim {
            id: "v-2.endemic",
            item: "v-2-endemic",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md (Animation V-2: endemic disease; \"v-2-endemic still has infected agents at t = 1000\"); tests/book.rs many_diseases_stay_endemic; presets.rs: \"disease stays endemic\"",
            text: "disease stays endemic (infected_fraction above 0 at every tick of t = 500..=1000)",
            check: |s| {
                let m: Vec<f64> = infected("v-2-endemic", 1000, s)
                    .iter()
                    .map(|f| min_over(f, 500, 1000))
                    .collect();
                range(&m, f64::MIN_POSITIVE, 1.0, false)
            },
        },
        Claim {
            id: "v-2.exceeds-v1",
            item: "v-2-endemic",
            source: Source::App,
            citation: "presets.rs v-2-endemic (\"disease stays endemic\") against v-1-rid (\"near-eradication\"); tests/book.rs many_diseases_stay_endemic asserts mean2 > mean1",
            text: "V-2 carries more disease than V-1 (mean infected_fraction over t = 500..=1000)",
            check: |s| {
                let m = |id| -> Vec<f64> {
                    infected(id, 1000, s).iter().map(|f| window_mean(f, 500, 1000)).collect()
                };
                greater(&m("v-2-endemic"), &m("v-1-rid"), "V-2", "V-1")
            },
        },
        Claim {
            id: "v-2.overwrite",
            item: "v-2-endemic",
            source: Source::App,
            citation: "presets.rs v-2-endemic description",
            text: "learning one immunity can overwrite another (at least one immunity lost by a surviving agent over t = 0..200)",
            // Same metric as v-1.overwrite.
            check: |s| range(&lost_immunities("v-2-endemic", 200, s), 1.0, f64::INFINITY, false),
        },
        Claim {
            id: "v-2.measured-level",
            item: "v-2-endemic",
            source: Source::Comment,
            citation: "tests/book.rs many_diseases_stay_endemic: V-2 means over t in 500..=1000 of 0.045334, 0.042674, 0.078412 (seeds 1-3)",
            text: "mean infected_fraction over t = 500..=1000 within the recorded 0.0427–0.0784",
            check: |s| {
                let m: Vec<f64> = infected("v-2-endemic", 1000, s)
                    .iter()
                    .map(|f| window_mean(f, 500, 1000))
                    .collect();
                range(&m, 0.042674, 0.078412, false)
            },
        },
        // ---- v-mcneill ----
        Claim {
            id: "v-mcneill.reproducing",
            item: "v-mcneill",
            source: Source::App,
            citation: "presets.rs v-mcneill description",
            text: "a reproducing society (at least one birth over t = 1..=300)",
            check: |s| {
                let b = after(&preset("v-mcneill"), s, 300, |w| series(w, "births")[1..=300].iter().sum::<f64>());
                range(&b, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "v-mcneill.familiar",
            item: "v-mcneill",
            source: Source::App,
            citation: "presets.rs v-mcneill description",
            text: "carrying its familiar diseases when the novel one arrives (diseases_in_circulation at t = 300, before the outbreak, at least 1)",
            check: |s| {
                let d = after(&preset("v-mcneill"), s, 300, |w| series(w, "diseases_in_circulation")[300]);
                range(&d, 1.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "v-mcneill.five-agents",
            item: "v-mcneill",
            source: Source::App,
            citation: "presets.rs v-mcneill description",
            text: "a novel disease at t = 300, brought in by 5 agents (exactly 5 outbreak infections, infector None, in the step starting at t = 300)",
            check: |s| range(&col(&mcneill_transmissions(s), 2), 5.0, 5.0, false),
        },
        Claim {
            id: "v-mcneill.spreads",
            item: "v-mcneill",
            source: Source::Book,
            citation: "docs/superpowers/specs/2026-09-23-chapter-v-disease-design.md: \"v-mcneill transmissions (agent-to-agent spread, excluding the outbreak's own seeding) rise after the t = 300 outbreak\"; tests/book.rs a_novel_disease_spreads_after_the_mcneill_outbreak",
            text: "transmissions over steps t = 300..400 exceed those over t = 200..300",
            check: |s| {
                let r = mcneill_transmissions(s);
                greater(&col(&r, 1), &col(&r, 0), "after", "before")
            },
        },
        Claim {
            id: "v-mcneill.before-zero",
            item: "v-mcneill",
            source: Source::Comment,
            citation: "tests/book.rs a_novel_disease_spreads_after_the_mcneill_outbreak: \"A wider 15-seed sweep confirmed every seed's after-count is > 0 (range 1-111) with before always 0\"",
            text: "no transmissions over steps t = 200..300 (the society has learned away what it carries)",
            check: |s| range(&col(&mcneill_transmissions(s), 0), 0.0, 0.0, false),
        },
        Claim {
            id: "v-mcneill.after-count",
            item: "v-mcneill",
            source: Source::Comment,
            citation: "tests/book.rs a_novel_disease_spreads_after_the_mcneill_outbreak: \"every seed's after-count is > 0 (range 1-111)\"",
            text: "transmissions over steps t = 300..400 within the recorded 1–111",
            check: |s| range(&col(&mcneill_transmissions(s), 1), 1.0, 111.0, false),
        },
    ]
}
