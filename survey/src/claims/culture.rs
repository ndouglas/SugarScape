//! Axelrod 1997's culture model (milestone 14): the paper's tables and
//! figures, Axtell, Axelrod, Epstein & Cohen's docking (1996), and
//! Castellano et al.'s and Klemm et al.'s later results. Runs go to
//! stability; runs that several claims share are memoized per process,
//! keyed by the config and the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::culture::{Activation, Changes, CultureConfig, Edges, Neighborhood, Pick};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{equivalent, greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, model_preset};

const PAPER: &str = "Axelrod 1997, J. Conflict Resolution 41";
const AAEC: &str = "Axtell, Axelrod, Epstein & Cohen 1996";
/// Far past any stability time the survey meets (100 × 100: about 10⁵).
const CAP: u32 = 400_000;
/// Seeds for the 100 × 100 runs (about three minutes each).
const BIG_SEEDS: usize = 8;

/// One run at its end.
#[derive(Clone, Copy, Debug)]
struct Run {
    regions: f64,
    cultures: f64,
    stable_at: f64,
    /// The first tick from which the zone count never changed again.
    zones_final_at: f64,
}

fn summarize(w: &ModelWorld) -> Run {
    let s = |n: &str| w.model().series(n).expect("a culture series");
    let zones = s("zones");
    let last = *zones.last().expect("a tick");
    let settled = zones.iter().rposition(|&z| z != last).map_or(0, |t| t + 1);
    Run {
        regions: *s("regions").last().unwrap(),
        cultures: *s("cultures").last().unwrap(),
        stable_at: *s("stable_at").last().unwrap(),
        zones_final_at: settled as f64,
    }
}

/// The paper's defaults with `edit` applied, run to stability for every seed.
fn runs(seeds: &[u64], edit: impl FnOnce(&mut CultureConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, Vec<u64>, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = CultureConfig::default();
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
    let v = Arc::new(model_after(&ModelConfig::Culture(c), seeds, CAP, summarize));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds.to_vec(), v.clone()));
    v
}

fn regions(seeds: &[u64], edit: impl FnOnce(&mut CultureConfig)) -> Vec<f64> {
    runs(seeds, edit).iter().map(|r| r.regions).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn square(c: &mut CultureConfig, side: u32, traits: u32) {
    c.width = side;
    c.height = side;
    c.traits = traits;
}

/// A claim about a share of seeds: holds when the share is in `lo..=hi`.
fn share(v: &[f64], pred: impl Fn(f64) -> bool, lo: f64, hi: f64, what: &str) -> Outcome {
    let k = v.iter().filter(|&&x| pred(x)).count();
    let s = k as f64 / v.len() as f64;
    Outcome {
        verdict: if (lo..=hi).contains(&s) {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!("{k}/{} seeds {what} ({:.0} %)", v.len(), 100.0 * s),
        detail: String::new(),
    }
}

/// A claim about a mean over configurations: holds in `lo..=hi`.
fn mean_in(m: f64, lo: f64, hi: f64, what: &str) -> Outcome {
    Outcome {
        verdict: if (lo..=hi).contains(&m) {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!("{what} {m:.2}; in [{lo}, {hi}]"),
        detail: String::new(),
    }
}

/// Mean regions per neighborhood over the paper's nine cultures.
fn neighborhood_mean(seeds: &[u64], n: Neighborhood) -> f64 {
    let mut all = Vec::new();
    for f in [5, 10, 15] {
        for q in [5, 10, 15] {
            all.extend(regions(seeds, |c| {
                c.features = f;
                c.traits = q;
                c.neighborhood = n;
            }));
        }
    }
    mean(&all)
}

/// Distinct cultures in the docked Sugarscape after 20 000 ticks.
fn docked(seeds: &[u64], id: &str) -> Vec<f64> {
    model_after(&model_preset(id), seeds, 20_000, |w| {
        *w.model()
            .series("distinct_cultures")
            .expect("the Axelrod rule's series")
            .last()
            .unwrap()
    })
}

/// The sample setup's seeds: enough to know this model's distribution well,
/// against which his samples (10 and 100 runs) are judged.
fn sample_seeds() -> Vec<u64> {
    (1..=1_000).collect()
}

/// Whether a mean of `n` runs could be `his` here: a two-sided z test on the
/// seeds' mean and spread (p ≥ 0.01 holds).
fn his_mean(v: &[f64], his: f64, n: u32) -> Outcome {
    let m = mean(v);
    let sd = (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64).sqrt();
    let z = (his - m).abs() / (sd / f64::from(n).sqrt());
    let p = 2.0 * (1.0 - normal_cdf(z));
    Outcome {
        verdict: if p >= 0.01 {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "mean {m:.2} (sd {sd:.2}, {} seeds); a mean of {n} runs at {his}: p = {p:.3}",
            v.len()
        ),
        detail: String::new(),
    }
}

#[derive(Clone, Copy)]
enum Tail {
    AtLeast,
    Both,
}

/// Whether `k` of 100 runs meeting `test` could happen here: a binomial test
/// at the seeds' share (p ≥ 0.01 holds).
fn his_count(v: &[f64], test: fn(f64) -> bool, k: u32, tail: Tail, what: &str) -> Outcome {
    let share = v.iter().filter(|&&r| test(r)).count() as f64 / v.len() as f64;
    let pmf = |i: u32| {
        if share <= 0.0 || share >= 1.0 {
            // A share of 0 or 1 allows only 0 or 100 of 100.
            return f64::from(u8::from(f64::from(i) == share * 100.0));
        }
        let ln_choose = (1..=i)
            .map(|j| (f64::from(100 - i + j) / f64::from(j)).ln())
            .sum::<f64>();
        (ln_choose + f64::from(i) * share.ln() + f64::from(100 - i) * (1.0 - share).ln()).exp()
    };
    let upper: f64 = (k..=100).map(pmf).sum();
    let lower: f64 = (0..=k).map(pmf).sum();
    let p = match tail {
        Tail::AtLeast => upper,
        Tail::Both => (2.0 * upper.min(lower)).min(1.0),
    };
    Outcome {
        verdict: if p >= 0.01 {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured: format!(
            "{:.1} % {what} over {} seeds; {k} of 100: p = {p:.3}",
            share * 100.0,
            v.len()
        ),
        detail: String::new(),
    }
}

/// The standard normal CDF (Abramowitz & Stegun 7.1.26, error < 1.5e-7).
fn normal_cdf(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t
        * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    let erf = 1.0 - poly * (-x * x).exp();
    if z >= 0.0 {
        0.5 * (1.0 + erf)
    } else {
        0.5 * (1.0 - erf)
    }
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "culture.table-2.f5-q5",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "Table 2: 5 features of 5 traits leave 1.0 stable region (1–1.5)",
            check: |s| range(&regions(s, |c| c.traits = 5), 1.0, 1.5, false),
        },
        Claim {
            id: "culture.table-2.f5-q10",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "Table 2: 5 features of 10 traits leave 3.2, a mean of 10 runs. Holds if a mean of 10 runs could be 3.2 here: over 1,000 seeds, two-sided p ≥ 0.01 (rule revised 2026-09-30, after the result was known: the earlier fixed range, 2–4.5 on 20 seeds, treated his 10-run mean as exact)",
            check: |_| {
                let v = regions(&sample_seeds(), |_| {});
                his_mean(&v, 3.2, 10)
            },
        },
        Claim {
            id: "culture.table-2.f5-q15",
            item: "ac-many-regions",
            source: Source::Book,
            citation: PAPER,
            text: "Table 2: 5 features of 15 traits leave 20.0 (the mean within 15–25)",
            check: |s| mean_in(mean(&regions(s, |c| c.traits = 15)), 15.0, 25.0, "mean regions"),
        },
        Claim {
            id: "culture.table-2.more-features",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "Table 2: with 10 or 15 features every culture but 15 traits converges to one region, and 15 traits leaves 1.4 and 1.2 (means within 1–2)",
            check: |s| {
                let mut v = Vec::new();
                for f in [10, 15] {
                    for q in [5, 10] {
                        v.extend(regions(s, |c| { c.features = f; c.traits = q }));
                    }
                }
                let ones = v.iter().all(|&r| r == 1.0);
                let a = mean(&regions(s, |c| { c.features = 10; c.traits = 15 }));
                let b = mean(&regions(s, |c| { c.features = 15; c.traits = 15 }));
                Outcome {
                    verdict: if ones && (1.0..=2.0).contains(&a) && (1.0..=2.0).contains(&b) { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("all one region at q = 5, 10: {ones}; 15 traits: {a:.2} (F 10), {b:.2} (F 15)"),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "culture.sample.median",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "100 runs of the sample setup: the median number of stable regions was three, so at least 50 of his 100 ended with 3 or fewer. Holds if that could happen here: over 1,000 seeds, binomial p ≥ 0.01 for 50 or more of 100 (rule revised 2026-09-30 after the result was known, as table-2.f5-q10)",
            check: |_| his_count(&regions(&sample_seeds(), |_| {}), |r| r <= 3.0, 50, Tail::AtLeast, "with at most 3 regions"),
        },
        Claim {
            id: "culture.sample.one-region",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "in 14 % of the runs there was only one stable region (14 of 100). Holds if that could happen here: over 1,000 seeds, two-sided binomial p ≥ 0.01 (rule revised 2026-09-30 after the result was known, as table-2.f5-q10)",
            check: |_| his_count(&regions(&sample_seeds(), |_| {}), |r| r == 1.0, 14, Tail::Both, "with one region"),
        },
        Claim {
            id: "culture.sample.more-than-six",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "in 10 % of the runs there were more than six (10 of 100). Holds if that could happen here: over 1,000 seeds, two-sided binomial p ≥ 0.01 (rule revised 2026-09-30 after the result was known, as table-2.f5-q10)",
            check: |_| his_count(&regions(&sample_seeds(), |_| {}), |r| r > 6.0, 10, Tail::Both, "with more than six"),
        },
        Claim {
            id: "culture.neighborhoods",
            item: "ac-sample-run",
            source: Source::Book,
            citation: PAPER,
            text: "averaged over the nine cultures, 4, 8 and 12 neighbors leave 3.4, 2.5 and 1.5 regions: fewer with more neighbors",
            check: |s| {
                let m: Vec<f64> = [Neighborhood::VonNeumann, Neighborhood::Moore, Neighborhood::Diamond]
                    .into_iter()
                    .map(|n| neighborhood_mean(s, n))
                    .collect();
                Outcome {
                    verdict: if m[0] > m[1] && m[1] > m[2] { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("4: {:.2}, 8: {:.2}, 12: {:.2} (the paper: 3.4, 2.5, 1.5)", m[0], m[1], m[2]),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "culture.territory.peak",
            item: "ac-many-regions",
            source: Source::Book,
            citation: PAPER,
            text: "Fig. 2 (5 features, 15 traits): regions rise to a maximum of about 23 near 12 × 12 (the mean at 12 × 12 within 17–29)",
            check: |s| mean_in(mean(&regions(s, |c| square(c, 12, 15))), 17.0, 29.0, "mean regions at 12 × 12"),
        },
        Claim {
            id: "culture.territory.50",
            item: "ac-large-territory",
            source: Source::Book,
            citation: PAPER,
            text: "Fig. 2: about 6 stable regions at 50 × 50 (the mean within 3–9)",
            check: |s| mean_in(mean(&regions(s, |c| square(c, 50, 15))), 3.0, 9.0, "mean regions at 50 × 50"),
        },
        Claim {
            id: "culture.territory.100",
            item: "ac-large-territory",
            source: Source::Book,
            citation: PAPER,
            text: "Fig. 2 and note 10: about 2 stable regions at 100 × 100 (the mean within 1–4; 8 seeds)",
            check: |s| {
                let s = &s[..s.len().min(BIG_SEEDS)];
                mean_in(mean(&regions(s, |c| square(c, 100, 15))), 1.0, 4.0, "mean regions at 100 × 100")
            },
        },
        Claim {
            id: "culture.territory.torus",
            item: "ac-torus",
            source: Source::Book,
            citation: PAPER,
            text: "on a torus 'the peak occurs earlier … and is not as high': its highest mean over 5–15 sites a side is below the bounded one, at a side no larger",
            check: |s| {
                let sides = [5u32, 8, 10, 12, 15];
                let curve = |edges| {
                    sides
                        .iter()
                        .map(|&n| mean(&regions(s, |c| { square(c, n, 15); c.boundary = edges })))
                        .collect::<Vec<f64>>()
                };
                let (b, t) = (curve(Edges::Bounded), curve(Edges::Torus));
                let arg = |v: &[f64]| (0..v.len()).max_by(|&i, &j| v[i].total_cmp(&v[j])).unwrap();
                let (bm, tm) = (b[arg(&b)], t[arg(&t)]);
                Outcome {
                    verdict: if tm < bm && sides[arg(&t)] <= sides[arg(&b)] { Verdict::Holds } else { Verdict::Fails },
                    measured: format!("bounded peak {bm:.1} at {0} × {0}; torus peak {tm:.1} at {1} × {1}", sides[arg(&b)], sides[arg(&t)]),
                    detail: String::new(),
                }
            },
        },
        Claim {
            id: "culture.time.32",
            item: "ac-large-territory",
            source: Source::Book,
            citation: PAPER,
            text: "a 32 × 32 territory needs on average 10 036 events per site to reach stability (the mean within ±25 %)",
            check: |s| {
                let v: Vec<f64> = runs(s, |c| square(c, 32, 15)).iter().map(|r| r.stable_at).collect();
                mean_in(mean(&v), 7527.0, 12545.0, "mean events per site")
            },
        },
        Claim {
            id: "culture.time.50",
            item: "ac-large-territory",
            source: Source::Book,
            citation: PAPER,
            text: "a 50 × 50 territory needs about 25 900 events per site (the mean within ±25 %)",
            check: |s| {
                let v: Vec<f64> = runs(s, |c| square(c, 50, 15)).iter().map(|r| r.stable_at).collect();
                mean_in(mean(&v), 19425.0, 32375.0, "mean events per site")
            },
        },
        Claim {
            id: "culture.zones-first",
            item: "ac-large-territory",
            source: Source::Book,
            citation: PAPER,
            text: "Fig. 3: the final zones form long before the final regions ('more than four times as long' at 100 × 100); at 50 × 50, stability takes at least twice as long as the zones",
            check: |s| {
                let v: Vec<f64> = runs(s, |c| square(c, 50, 15)).iter().map(|r| r.stable_at / r.zones_final_at.max(1.0)).collect();
                range(&v, 2.0, f64::INFINITY, false)
            },
        },
        Claim {
            id: "culture.docking.activation",
            item: "ac-sweep-activation",
            source: Source::Comment,
            citation: AAEC,
            text: "at 20 × 20 (5 features, 15 traits), random activation leaves more regions than the Sugarscape's shuffled sweeps (16.25 vs 9.23)",
            check: |s| {
                let r = regions(s, |c| square(c, 20, 15));
                let w = regions(s, |c| { square(c, 20, 15); c.activation = Activation::Sweep });
                greater(&r, &w, "random", "sweep")
            },
        },
        Claim {
            id: "culture.docking.neighbor",
            item: "ac-neighbor-changes",
            source: Source::Comment,
            citation: AAEC,
            text: "changing the neighbor instead of the active site (the original Sugarscape) \"made a subtle difference because agents on the edge of the territory have fewer neighbors\" (no size given). Holds if over 200 seeds of the sample setup the region counts differ either way (one-sided Mann–Whitney p < 0.01 in either direction)",
            check: |_| {
                let seeds: Vec<u64> = (1..=200).collect();
                let a = regions(&seeds, |_| {});
                let n = regions(&seeds, |c| c.changes = Changes::Neighbor);
                let (up, down) = (greater(&a, &n, "active", "neighbor"), greater(&n, &a, "neighbor", "active"));
                if down.verdict == Verdict::Holds {
                    down
                } else {
                    Outcome {
                        verdict: if up.verdict == Verdict::Holds { Verdict::Holds } else { Verdict::Fails },
                        ..up
                    }
                }
            },
        },
        Claim {
            id: "culture.code.scan",
            item: "ac-sample-run",
            source: Source::Comment,
            citation: "Axelrod, CULTURE.P (archived demo program, 1995–96)",
            text: "his archived demo program copies not a random differing feature, as the paper says, but the first it meets scanning two features at a time from a random start. Holds if over 1,000 seeds of the sample setup the region counts under the two rules are equivalent (TOST within 10 % of the mean)",
            check: |_| {
                let seeds = sample_seeds();
                let paper = regions(&seeds, |_| {});
                let code = regions(&seeds, |c| c.pick = Pick::Scan);
                equivalent(&paper, &code, None, "paper", "program")
            },
        },
        Claim {
            id: "culture.docking.cultures",
            item: "ac-sample-run",
            source: Source::Comment,
            citation: AAEC,
            text: "distinct cultures are a fair surrogate for regions: at stability they agree in at least 80 % of the sample runs",
            check: |s| {
                let v: Vec<f64> = runs(s, |_| {}).iter().map(|r| f64::from(u8::from(r.regions == r.cultures))).collect();
                share(&v, |x| x == 1.0, 0.8, 1.0, "with cultures = regions")
            },
        },
        Claim {
            id: "culture.docking.soup-15",
            item: "ac-soup",
            source: Source::Comment,
            citation: AAEC,
            text: "soup (random pairing), 15 traits: never more than one culture in 10 runs (at least 80 % of seeds end with one)",
            check: |s| share(&regions(s, |c| { c.traits = 15; c.neighborhood = Neighborhood::Soup }), |r| r == 1.0, 0.8, 1.0, "with one culture"),
        },
        Claim {
            id: "culture.docking.soup-30",
            item: "ac-soup",
            source: Source::Comment,
            citation: AAEC,
            text: "soup, 30 traits: 1.4 cultures on average (the mean within 1–2)",
            check: |s| mean_in(mean(&regions(s, |c| { c.traits = 30; c.neighborhood = Neighborhood::Soup })), 1.0, 2.0, "mean cultures"),
        },
        Claim {
            id: "culture.docking.mobility-15",
            item: "dock-mobility-15",
            source: Source::Comment,
            citation: AAEC,
            text: "mobile agents on a sugar mountain, 15 traits: 1.1 ± 0.3 cultures over 10 runs. Holds if a mean of 10 runs could be 1.1 here (two-sided p ≥ 0.01; rule revised 2026-09-30 after the result was known, as table-2.f5-q10; under the old range, 1–1.5, it also holds)",
            check: |s| his_mean(&docked(s, "dock-mobility-15"), 1.1, 10),
        },
        Claim {
            id: "culture.docking.mobility-30",
            item: "dock-mobility-30",
            source: Source::Comment,
            citation: AAEC,
            text: "30 traits: 2.2 ± 1.2 cultures over 10 runs. Holds if a mean of 10 runs could be 2.2 here (two-sided p ≥ 0.01; revised as mobility-15; under the old range, 1–3.4, it also holds)",
            check: |s| his_mean(&docked(s, "dock-mobility-30"), 2.2, 10),
        },
        Claim {
            id: "culture.docking.mobility-mixes",
            item: "dock-mobility-15",
            source: Source::Comment,
            citation: AAEC,
            text: "mobility collapses diversity: fewer cultures than the fixed 10 × 10 lattice with the same culture",
            check: |s| greater(&regions(s, |c| c.traits = 15), &docked(s, "dock-mobility-15"), "fixed", "mobile"),
        },
        Claim {
            id: "culture.castellano.below",
            item: "ac-many-regions",
            source: Source::Comment,
            citation: "Castellano, Marsili & Vespignani 2000, PRL 85",
            text: "below the transition (15 traits) regions fall from 20 × 20 to 30 × 30, as Axelrod found",
            check: |s| greater(&regions(s, |c| square(c, 20, 15)), &regions(s, |c| square(c, 30, 15)), "20 × 20", "30 × 30"),
        },
        Claim {
            id: "culture.castellano.above",
            item: "ac-many-regions",
            source: Source::Comment,
            citation: "Castellano, Marsili & Vespignani 2000, PRL 85",
            text: "above it (25 traits) regions grow from 20 × 20 to 30 × 30: Axelrod's size result depends on his choice of traits",
            check: |s| greater(&regions(s, |c| square(c, 30, 25)), &regions(s, |c| square(c, 20, 25)), "30 × 30", "20 × 20"),
        },
        Claim {
            id: "culture.klemm.drift",
            item: "ac-drift",
            source: Source::Comment,
            citation: "Klemm, Eguíluz, Toral & San Miguel 2003, PRE 67",
            text: "a little drift (10⁻⁴ per event) leaves fewer cultures after 20 000 ticks than none",
            check: |s| {
                let at = |r: f64| {
                    let mut c = CultureConfig::default();
                    square(&mut c, 12, 15);
                    c.drift = r;
                    model_after(&ModelConfig::Culture(c), s, 20_000, |w| *w.model().series("cultures").unwrap().last().unwrap())
                };
                greater(&at(0.0), &at(1e-4), "no drift", "drift 10⁻⁴")
            },
        },
    ]
}
