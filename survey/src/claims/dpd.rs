//! The demographic Prisoner's Dilemma (milestone 19): Epstein's working
//! paper (1997) and chapter (GSS 2006, ch. 9 and its appendix) and Radax &
//! Rengs' replication (RR, 2009), each claim in its source's words. Every
//! claim runs Epstein's and RR's 30 runs (seeds 1–30, whatever `--seeds`
//! says) and reads each run's counts at cycle 500 unless it says otherwise,
//! so its numbers are the plan's measurements. Runs that several claims
//! share are memoized per process. RR's statistic (a pooled two-sample t
//! against the source's mean and s.d., 30 runs each) is reported in the
//! details; the verdicts use the survey's judges.

use std::sync::{Arc, Mutex};

use sugarscape_core::dpd::{
    DeathTiming, DpdConfig, DpdWorld, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct,
    Pairing, Play, Removal, Shuffle, Updating,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source};
use crate::runner::model_after;

const GSS: &str = "Epstein, Generative Social Science (2006), ch. 9";
const GSS_APPENDIX: &str = "Epstein, Generative Social Science (2006), ch. 9 appendix";
const WP: &str = "Epstein, SFI Working Paper 97-12-094 (1997)";
const RR: &str = "Radax & Rengs 2009, MPRA 14419";
const OURS: &str = "spec 2026-09-26-demographic-pd-design.md; plan Decision 17";

/// Epstein's and RR's 30 runs.
const SEEDS: std::ops::RangeInclusive<u64> = 1..=30;

/// RR's critical |t| (df 58, α = 0.05).
const T_CRIT: f64 = 2.0017;

fn dpd(w: &ModelWorld) -> &DpdWorld {
    match w {
        ModelWorld::Dpd(w) => w,
        _ => unreachable!("a demographic PD world"),
    }
}

/// One run: its cooperators and defectors at every cycle (index t is cycle t).
struct Run {
    c: Vec<f64>,
    d: Vec<f64>,
}

impl Run {
    fn c_at(&self, t: usize) -> f64 {
        self.c[t]
    }
    fn d_at(&self, t: usize) -> f64 {
        self.d[t]
    }
}

/// `c` run `ticks` cycles from seeds 1–30 (memoized by config and length).
fn runs(c: &DpdConfig, ticks: u32) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = serde_json::to_string(c).expect("configs serialize");
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, _)| *k == key && *t == ticks)
    {
        return v.clone();
    }
    let seeds: Vec<u64> = SEEDS.collect();
    let v = Arc::new(model_after(
        &ModelConfig::Dpd(c.clone()),
        &seeds,
        ticks,
        |w| {
            let s = &dpd(w).stats;
            Run {
                c: s.series("cooperators").expect("a dpd series"),
                d: s.series("defectors").expect("a dpd series"),
            }
        },
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, v.clone()));
    v
}

/// Each run's cooperators and defectors at cycle `t` of a `ticks`-cycle run.
fn counts(c: &DpdConfig, ticks: u32, t: usize) -> (Vec<f64>, Vec<f64>) {
    let v = runs(c, ticks);
    (
        v.iter().map(|r| r.c_at(t)).collect(),
        v.iter().map(|r| r.d_at(t)).collect(),
    )
}

/// Mean and sample standard deviation.
fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    (
        m,
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt(),
    )
}

/// RR's t for two samples of 30 given as (mean, s.d.): (source − ours) over
/// the pooled standard error.
fn t_from(source: (f64, f64), ours: (f64, f64)) -> f64 {
    (source.0 - ours.0) / ((source.1 * source.1 + ours.1 * ours.1) / 2.0 * 2.0 / 30.0).sqrt()
}

/// RR's t of our 30 runs against the source's (mean, s.d.).
fn t_stat(source: (f64, f64), ours: &[f64]) -> f64 {
    t_from(source, mean_sd(ours))
}

/// A source's number as printed: whole, or to one decimal.
fn printed(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{x}")
    } else {
        format!("{x:.1}")
    }
}

/// "cooperators 729.2 ± 17.4 against 779 ± 15 (RR's t 11.86)".
fn against(name: &str, v: &[f64], source: (f64, f64)) -> String {
    let (m, sd) = mean_sd(v);
    format!(
        "{name} {m:.1} ± {sd:.1} against {} ± {} (RR's t {:.2})",
        printed(source.0),
        printed(source.1),
        t_stat(source, v)
    )
}

/// Whether both means pass RR's test against a table's (mean, s.d.) pairs.
fn passes(c: &[f64], d: &[f64], table: [(f64, f64); 2]) -> bool {
    t_stat(table[0], c).abs() < T_CRIT && t_stat(table[1], d).abs() < T_CRIT
}

/// Each run's counts inside a table's ranges (cooperators, defectors).
fn in_ranges(c: &[f64], d: &[f64], rc: (f64, f64), rd: (f64, f64)) -> Outcome {
    all_of(vec![
        ("cooperators".into(), range(c, rc.0, rc.1, false)),
        ("defectors".into(), range(d, rd.0, rd.1, false)),
    ])
}

const TABLE_1: [(f64, f64); 2] = [(779.0, 15.0), (121.0, 15.0)];
const TABLE_2: [(f64, f64); 2] = [(784.0, 29.0), (99.0, 25.0)];

fn run_1() -> DpdConfig {
    DpdConfig::default()
}

fn run_2() -> DpdConfig {
    DpdConfig {
        max_age: 100,
        ..DpdConfig::default()
    }
}

fn shifted(c: DpdConfig) -> DpdConfig {
    DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        ..c
    }
}

fn metabolism(m: f64, per: MetabolismPer) -> DpdConfig {
    DpdConfig {
        metabolism: m,
        metabolism_per: per,
        ..shifted(run_2())
    }
}

fn closest(c: DpdConfig) -> DpdConfig {
    DpdConfig {
        play: Play::RandomNeighbor,
        initial_wealth: 0.0,
        newborns_act: NewbornsAct::ThisCycle,
        ..c
    }
}

/// 1 where `f` holds of a run, else 0.
fn indicator(v: &[Run], t: usize, f: impl Fn(f64, f64) -> bool) -> Vec<f64> {
    v.iter()
        .map(|r| f64::from(u8::from(f(r.c_at(t), r.d_at(t)))))
        .collect()
}

/// GSS Table 9.3 as printed: (T, R, cooperators' mean, s.d., 95% CI, range,
/// defectors' mean, s.d., 95% CI, range). Row (4, 2)'s defectors' CI is
/// printed "(254, 376)" (its mean and s.d. give (253, 277)); kept as printed.
type Cell = (u32, u32, [f64; 6], [f64; 6]);
#[rustfmt::skip]
const TABLE_9_3: [Cell; 45] = [
    (10, 9, [809., 19., 802., 816., 772., 845.], [77., 17., 71., 83., 43., 109.]),
    (10, 8, [748., 27., 738., 757., 707., 802.], [132., 23., 124., 140., 82., 169.]),
    (10, 7, [654., 38., 641., 668., 568., 717.], [198., 30., 188., 209., 154., 280.]),
    (10, 6, [469., 60., 447., 490., 312., 604.], [274., 30., 264., 285., 191., 329.]),
    (10, 5, [258., 51., 240., 277., 150., 383.], [270., 26., 261., 279., 202., 319.]),
    (10, 4, [231., 151., 177., 285., 0., 598.], [199., 74., 172., 225., 0., 287.]),
    (10, 3, [0., 0., 0., 0., 0., 1.], [0., 0., 0., 0., 0., 1.]),
    (10, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (10, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (9, 8, [806., 21., 799., 814., 766., 850.], [81., 19., 74., 88., 39., 117.]),
    (9, 7, [728., 34., 716., 740., 669., 793.], [146., 28., 136., 156., 86., 190.]),
    (9, 6, [604., 58., 583., 625., 490., 768.], [225., 38., 212., 239., 102., 303.]),
    (9, 5, [374., 44., 358., 390., 268., 460.], [290., 20., 283., 297., 252., 323.]),
    (9, 4, [203., 78., 175., 231., 98., 442.], [235., 36., 222., 248., 159., 296.]),
    (9, 3, [35., 89., 3., 67., 0., 382.], [38., 84., 8., 68., 0., 284.]),
    (9, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (9, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (8, 7, [807., 30., 796., 818., 744., 879.], [80., 27., 70., 89., 14., 142.]),
    (8, 6, [721., 36., 708., 734., 626., 787.], [153., 30., 142., 163., 98., 231.]),
    (8, 5, [530., 39., 516., 544., 460., 604.], [263., 22., 255., 271., 209., 312.]),
    (8, 4, [259., 57., 239., 279., 128., 387.], [271., 22., 263., 279., 227., 313.]),
    (8, 3, [93., 113., 53., 134., 0., 513.], [113., 99., 77., 148., 0., 279.]),
    (8, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (8, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (7, 6, [797., 28., 787., 807., 739., 852.], [88., 25., 79., 97., 43., 133.]),
    (7, 5, [668., 44., 652., 684., 542., 782.], [187., 33., 175., 199., 101., 271.]),
    (7, 4, [430., 55., 410., 449., 323., 547.], [286., 26., 277., 295., 244., 345.]),
    (7, 3, [126., 101., 90., 162., 0., 370.], [153., 76., 126., 180., 0., 267.]),
    (7, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (7, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (6, 5, [779., 15., 773., 784., 752., 806.], [121., 15., 115., 126., 93., 148.]),
    (6, 4, [587., 33., 576., 599., 524., 658.], [241., 22., 233., 248., 193., 278.]),
    (6, 3, [266., 53., 247., 285., 120., 344.], [280., 29., 270., 291., 199., 320.]),
    (6, 2, [24., 92., 0., 57., 0., 482.], [13., 37., 0., 27., 0., 166.]),
    (6, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (5, 4, [741., 33., 729., 752., 689., 810.], [136., 28., 126., 146., 79., 179.]),
    (5, 3, [473., 46., 456., 489., 379., 574.], [283., 24., 274., 291., 243., 329.]),
    (5, 2, [125., 155., 70., 180., 0., 589.], [114., 89., 82., 145., 0., 260.]),
    (5, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (4, 3, [710., 47., 693., 727., 613., 814.], [159., 36., 146., 172., 76., 231.]),
    (4, 2, [282., 56., 262., 302., 171., 380.], [265., 32., 254., 376., 176., 322.]),
    (4, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (3, 2, [624., 38., 610., 637., 516., 699.], [221., 30., 210., 232., 164., 284.]),
    (3, 1, [4., 22., 0., 12., 0., 123.], [7., 35., 0., 20., 0., 197.]),
    (2, 1, [361., 47., 344., 377., 256., 471.], [280., 21., 273., 288., 234., 337.]),
];

/// Table 9.3's cell (T, R): S = −T, P = −R, Run 1's other settings.
fn cell(t: u32, r: u32) -> DpdConfig {
    DpdConfig {
        t: f64::from(t),
        r: f64::from(r),
        p: -f64::from(r),
        s: -f64::from(t),
        ..DpdConfig::default()
    }
}

/// A mean inside an interval Epstein printed rounded: it rounds into it.
fn rounds_into(m: f64, lo: f64, hi: f64) -> bool {
    m >= lo - 0.5 && m < hi + 0.5
}

/// Per cell: our (cooperators, defectors) means at 500.
fn table_9_3() -> Vec<(f64, f64)> {
    TABLE_9_3
        .iter()
        .map(|&(t, r, ..)| {
            let (c, d) = counts(&cell(t, r), 500, 500);
            (mean_sd(&c).0, mean_sd(&d).0)
        })
        .collect()
}

/// Swings of the cooperator count from above 400 to below 100 (Decision M's
/// cycle count: hysteresis, so noise near one threshold does not count).
fn swings(r: &Run) -> f64 {
    let (mut high, mut n) = (false, 0.0);
    for &x in &r.c {
        if x > 400.0 {
            high = true;
        } else if x < 100.0 && high {
            high = false;
            n += 1.0;
        }
    }
    n
}

/// The coordination game at cycle `ticks`: per run, whether both
/// conventions are present, and the share of occupied von Neumann neighbour
/// pairs that differ over the share if the same counts were mixed at random
/// (2pq).
fn coordination(ticks: u32) -> Vec<(bool, f64)> {
    let c = DpdConfig {
        r: 1.0,
        s: -3.0,
        t: -3.0,
        p: 1.0,
        max_age: 1000,
        ..DpdConfig::default()
    };
    let seeds: Vec<u64> = SEEDS.collect();
    model_after(&ModelConfig::Dpd(c), &seeds, ticks, |w| {
        let w = dpd(w);
        let (mut unlike, mut pairs, mut coop, mut all) = (0u32, 0u32, 0u32, 0u32);
        for s in 0..w.sites() {
            let Some(a) = w.agent_at(s) else { continue };
            all += 1;
            coop += u32::from(a.cooperator);
            for &j in w.geometry.neighbors(s) {
                if let Some(b) = w.agent_at(j as usize) {
                    pairs += 1;
                    unlike += u32::from(a.cooperator != b.cooperator);
                }
            }
        }
        let p = f64::from(coop) / f64::from(all.max(1));
        let mixed = coop > 0 && coop < all;
        let observed = f64::from(unlike) / f64::from(pairs.max(1));
        (
            mixed,
            observed / (2.0 * p * (1.0 - p)).max(f64::MIN_POSITIVE),
        )
    })
}

/// RR's six model switches in their order, TRUE first: remove dead
/// immediately, die immediately, endowment inherited, random birth age,
/// asynchronous updating, Repast list-shuffle (their RNG column dropped).
fn rr_setting(flags: &str, c: DpdConfig) -> DpdConfig {
    let on: Vec<bool> = flags.chars().map(|f| f == 'T').collect();
    DpdConfig {
        removal: if on[0] {
            Removal::Immediate
        } else {
            Removal::EndOfCycle
        },
        death_timing: if on[1] {
            DeathTiming::Immediate
        } else {
            DeathTiming::OwnTurn
        },
        endowment_from: if on[2] {
            EndowmentFrom::Parent
        } else {
            EndowmentFrom::Granted
        },
        newborn_age: if on[3] {
            NewbornAge::Random
        } else {
            NewbornAge::Zero
        },
        updating: if on[4] {
            Updating::Asynchronous
        } else {
            Updating::Synchronous
        },
        shuffle: if on[5] { Shuffle::Full } else { Shuffle::Swaps },
        ..c
    }
}

/// RR's Table 7: the seven settings that reproduce Run 2 in Repast, six once
/// their RNG column is dropped (rows 3 and 4 differ only there).
const RR_RUN_2_FITS: [&str; 6] = ["TTFTTT", "TFTFTT", "FTTTTF", "FTFTTF", "FFTTTT", "FFFTTT"];

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "dpd-run-1.table-1",
            item: "dpd-run-1",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.2 (Run 1, 30 runs, t = 500): cooperators range (752, 806), mean 779, s.d. 15; defectors range (93, 148), mean 121, s.d. 15",
            check: |_| {
                let (c, d) = counts(&run_1(), 500, 500);
                in_ranges(&c, &d, (752.0, 806.0), (93.0, 148.0)).with(&format!(
                    "{}; {}. Cooperation dominates, but with 50 fewer cooperators than Epstein's.",
                    against("cooperators", &c, TABLE_1[0]),
                    against("defectors", &d, TABLE_1[1])
                ))
            },
        },
        Claim {
            id: "dpd-run-2.table-2",
            item: "dpd-run-2",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.4 (Run 2, maximum age 100): cooperators range (708, 846), mean 784, s.d. 29; defectors range (45, 160), mean 99, s.d. 25",
            check: |_| {
                let (c, d) = counts(&run_2(), 500, 500);
                in_ranges(&c, &d, (708.0, 846.0), (45.0, 160.0)).with(&format!(
                    "{}; {}.",
                    against("cooperators", &c, TABLE_2[0]),
                    against("defectors", &d, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-run-1.five-to-one",
            item: "dpd-run-1",
            source: Source::Book,
            citation: GSS,
            text: "Run 1: by t = 50 a stable ratio of cooperators to defectors, approximately 5 to 1 (each run's ratio within 10% of 5)",
            check: |_| {
                let v = runs(&run_1(), 500);
                let ratio = |t: usize| -> Vec<f64> { v.iter().map(|r| r.c_at(t) / r.d_at(t)).collect() };
                let mean = |t: usize| {
                    let c: f64 = v.iter().map(|r| r.c_at(t)).sum();
                    c / v.iter().map(|r| r.d_at(t)).sum::<f64>()
                };
                range(&ratio(50), 5.0, 5.0, true).with(&format!(
                    "Ratio of the means: {:.2} at t = 30, {:.2} at 50, {:.2} at 100, {:.2} at 500 — stable from about t = 30, as Epstein says, but about 4.3 to 1.",
                    mean(30),
                    mean(50),
                    mean(100),
                    mean(500)
                ))
            },
        },
        Claim {
            id: "dpd-payoffs.table-9-3",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3 (45 payoff vectors, S = −T, P = −R, 30 runs each at t = 500): our mean cooperators and defectors in each cell inside Epstein's 95% confidence interval and range (one value per mean: 90 of each)",
            check: |_| {
                let ours = table_9_3();
                let (mut ci, mut rg, mut both, mut t_pass) = (vec![], vec![], 0, 0);
                for (&(t, r, ec, ed), &(c, d)) in TABLE_9_3.iter().zip(&ours) {
                    let (cc, dc) = (rounds_into(c, ec[2], ec[3]), rounds_into(d, ed[2], ed[3]));
                    ci.extend([f64::from(u8::from(cc)), f64::from(u8::from(dc))]);
                    rg.push(f64::from(u8::from(rounds_into(c, ec[4], ec[5]))));
                    rg.push(f64::from(u8::from(rounds_into(d, ed[4], ed[5]))));
                    both += usize::from(cc && dc);
                    let (cs, ds) = counts(&cell(t, r), 500, 500);
                    t_pass += usize::from(passes(&cs, &ds, [(ec[0], ec[1]), (ed[0], ed[1])]));
                }
                all_of(vec![
                    ("means in the 95% CI".into(), range(&ci, 1.0, 1.0, false)),
                    ("means in the range".into(), range(&rg, 1.0, 1.0, false)),
                ])
                .with(&format!(
                    "Cells with both means in the CI: {both} of 45; passing RR's two t-tests: {t_pass} of 45. Our cooperators run 13–92 below Epstein's along the diagonal (R = T − 1) and our defectors 33–106 above. Row (4, 2)'s defectors' CI is counted as printed, (254, 376); its mean and s.d. give (253, 277)."
                ))
            },
        },
        Claim {
            id: "dpd-payoffs.collapse",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3: cooperators die out (mean 0) at R ≤ 3 when T = 10, R ≤ 2 when T = 7–9 and R = 1 when T ≤ 6, and survive in every other cell",
            check: |_| {
                let ours = table_9_3();
                let agree: Vec<f64> = TABLE_9_3
                    .iter()
                    .zip(&ours)
                    .map(|(&(_, _, ec, _), &(c, _))| f64::from(u8::from((ec[0] == 0.0) == (c < 0.5))))
                    .collect();
                range(&agree, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "dpd-payoffs.all-zero",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3: where cooperation collapses, the whole population dies out: 0 cooperators and 0 defectors in every run (ranges (0, 0))",
            check: |_| {
                let mut d = Vec::new();
                for &(t, r, ec, ed) in &TABLE_9_3 {
                    if ec[5] == 0.0 && ed[5] == 0.0 {
                        d.extend(counts(&cell(t, r), 500, 500).1);
                    }
                }
                range(&d, 0.0, 0.0, false).with(&format!(
                    "Defectors at 500 over the {} runs of the 11 all-zero cells: mean {:.1}. Once cooperators are gone the last defectors have nobody to play, and with no maximum age and no metabolism nothing kills them.",
                    d.len(),
                    mean_sd(&d).0
                ))
            },
        },
        Claim {
            id: "dpd-run-4.cycles",
            item: "dpd-run-4",
            source: Source::Book,
            citation: GSS,
            text: "Run 4 (R = 1, maximum age 100): predator–prey cycles — at least three swings of the cooperators (above 400, then below 100) in 2,000 cycles, and populations that live on",
            check: |_| {
                let c = DpdConfig {
                    r: 1.0,
                    ..run_2()
                };
                let v = runs(&c, 2000);
                let s: Vec<f64> = v.iter().map(swings).collect();
                let alive = indicator(&v, 2000, |c, d| c + d > 0.0);
                let at_500 = indicator(&v, 500, |c, d| c + d > 0.0);
                all_of(vec![
                    ("swings".into(), range(&s, 3.0, f64::INFINITY, false)),
                    ("alive at 2,000".into(), range(&alive, 1.0, 1.0, false)),
                ])
                .with(&format!(
                    "Mean {:.1} swings, at most {}; {} of 30 populations alive at 500, none at 2,000 — no run sustains cycles, and no seed ends in cooperator monopoly.",
                    mean_sd(&s).0,
                    s.iter().copied().fold(0.0, f64::max),
                    at_500.iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-run-4.paradox",
            item: "dpd-run-4",
            source: Source::Book,
            citation: GSS,
            text: "\"Cooperators ultimately do better with a low payoff (R = 1) than with a high one (R = 5)!\" — more cooperators at 2,000 cycles with R = 1 than with R = 5 (maximum age 100)",
            check: |_| {
                let low = DpdConfig {
                    r: 1.0,
                    ..run_2()
                };
                let (cl, dl) = counts(&low, 2000, 2000);
                let (ch, dh) = counts(&run_2(), 2000, 2000);
                let monopolies = |c: &[f64], d: &[f64]| c.iter().zip(d).filter(|(c, d)| **c > 0.0 && **d == 0.0).count();
                greater(&cl, &ch, "R = 1", "R = 5").with(&format!(
                    "Cooperator monopolies at 2,000: {} of 30 at R = 1, {} of 30 at R = 5; every R = 1 population dies out.",
                    monopolies(&cl, &dl),
                    monopolies(&ch, &dh)
                ))
            },
        },
        Claim {
            id: "dpd-run-5.persists",
            item: "dpd-run-5",
            source: Source::Book,
            citation: GSS,
            text: "Run 5 (Run 2 with 50% mutation): cooperation persists through 10,000 cycles",
            check: |_| {
                let c = DpdConfig {
                    mutation: 0.5,
                    ..run_2()
                };
                let v = runs(&c, 10_000);
                let (co, de) = counts(&c, 10_000, 10_000);
                range(&co, 1.0, f64::INFINITY, false).with(&format!(
                    "{} of 30 populations die out entirely; at 10,000: {:.0} cooperators, {:.0} defectors (means); the lowest cooperator count in a surviving run: {:.0}.",
                    v.iter().filter(|r| r.c_at(10_000) + r.d_at(10_000) == 0.0).count(),
                    mean_sd(&co).0,
                    mean_sd(&de).0,
                    v.iter()
                        .filter(|r| r.c_at(10_000) > 0.0)
                        .map(|r| r.c.iter().copied().fold(f64::INFINITY, f64::min))
                        .fold(f64::INFINITY, f64::min)
                ))
            },
        },
        Claim {
            id: "dpd-soup.pure-defection",
            item: "dpd-soup",
            source: Source::Book,
            citation: GSS,
            text: "Soup (equiprobable random agent pairings): the population runs to pure defection — no cooperators at t = 500",
            check: |_| {
                let c = DpdConfig {
                    pairing: Pairing::Soup,
                    ..run_1()
                };
                let v = runs(&c, 500);
                let (co, _) = counts(&c, 500, 500);
                let gone: Vec<f64> = v
                    .iter()
                    .filter_map(|r| r.c.iter().position(|&x| x == 0.0))
                    .map(|t| t as f64)
                    .collect();
                range(&co, 0.0, 0.0, false).with(&format!(
                    "The last cooperator dies at cycle {:.1} on average ({} runs); then the defectors starve each other: {} of 30 populations are empty at 500, the rest one lone agent.",
                    mean_sd(&gone).0,
                    gone.len(),
                    indicator(&v, 500, |c, d| c + d == 0.0).iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-shifted.pure-defection",
            item: "dpd-shifted",
            source: Source::Book,
            citation: GSS,
            text: "Payoffs shifted up by 6 (12, 11, 1, 0), maximum age 100, zero mutation: the population converges to pure defection (Fig. 13) — no cooperators at t = 500",
            check: |_| {
                let (c, d) = counts(&shifted(run_2()), 500, 500);
                let (c5, _) = counts(
                    &shifted(DpdConfig {
                        mutation: 0.5,
                        ..run_2()
                    }),
                    500,
                    500,
                );
                let (c1, _) = counts(&shifted(run_1()), 500, 500);
                range(&c, 0.0, 0.0, false).with(&format!(
                    "{:.1} cooperators, {:.1} defectors (means); every run coexists. With Run 5's settings (the working paper's context) {:.1} cooperators, with Run 1's {:.1}. With no negative payoff only old age kills; the full lattice's vacancies go to whichever neighbour moves first.",
                    mean_sd(&c).0,
                    mean_sd(&d).0,
                    mean_sd(&c5).0,
                    mean_sd(&c1).0
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.per-cycle",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: WP,
            text: "The shifted payoffs with a metabolism of 6 \"recover, in effect, our initial payoffs\" (metabolism \"a fixed decrement to accumulated payoff per cycle\", note 29): cooperators at 500 as in Run 2",
            check: |_| {
                let (c, _) = counts(&metabolism(6.0, MetabolismPer::Cycle), 500, 500);
                let (base, _) = counts(&run_2(), 500, 500);
                equivalent(&c, &base, None, "metabolism 6 per cycle", "Run 2").with(&format!(
                    "{}. Cooperative, but a different model: its spread is three times Run 2's.",
                    against("cooperators", &c, mean_sd(&base))
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.per-interaction",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: GSS,
            text: "\"Equivalent mathematically\": the shifted payoffs with a metabolism of 6 \"imposed on all agents after every interaction\" give Run 2",
            check: |_| {
                let per_game = runs(&metabolism(6.0, MetabolismPer::Interaction), 500);
                let base = runs(&run_2(), 500);
                let same = per_game
                    .iter()
                    .zip(base.iter())
                    .filter(|(a, b)| a.c == b.c && a.d == b.d)
                    .count();
                let (c, _) = counts(&metabolism(6.0, MetabolismPer::Interaction), 500, 500);
                let (b, _) = counts(&run_2(), 500, 500);
                equivalent(&c, &b, None, "metabolism 6 per game", "Run 2").with(&format!(
                    "Identical counts at every cycle in {same} of 30 runs: each game's payoff less 6 is the original payoff."
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.necessity",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: GSS,
            text: "\"Necessity is the mother of cooperation\": on the shifted payoffs a higher metabolism (5 per cycle) leaves more cooperators than a low one (1)",
            check: |_| {
                let (hi, _) = counts(&metabolism(5.0, MetabolismPer::Cycle), 500, 500);
                let (lo, _) = counts(&metabolism(1.0, MetabolismPer::Cycle), 500, 500);
                greater(&hi, &lo, "metabolism 5", "metabolism 1")
            },
        },
        Claim {
            id: "dpd-footnote-27.monopoly",
            item: "dpd-footnote-27",
            source: Source::Book,
            citation: GSS,
            text: "Footnote 27: payoffs T 16, R 11, P 5, S 4 and a maximum lifetime of 10 cycles give an evolution to cooperative monopoly (at 2,000 cycles)",
            check: |_| {
                let c = DpdConfig {
                    t: 16.0,
                    r: 11.0,
                    p: 5.0,
                    s: 4.0,
                    max_age: 10,
                    ..DpdConfig::default()
                };
                let v = runs(&c, 2000);
                let mono = indicator(&v, 2000, |c, d| c > 0.0 && d == 0.0);
                range(&mono, 1.0, 1.0, false).with(&format!(
                    "{} of 30 runs reach cooperative monopoly by 2,000, none by 500. \"Hiked by ten\" would make R 15; that gives the same.",
                    mono.iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-coordination.regions",
            item: "dpd-coordination",
            source: Source::Book,
            citation: GSS_APPENDIX,
            text: "The coordination game (payoffs [1, −3, −3, 1], death age 1,000): persistent norm maps — both conventions still present at 2,000 cycles, in regions (unlike neighbours under half as common as if mixed at random)",
            check: |_| {
                let v = coordination(2000);
                let present: Vec<f64> = v.iter().map(|x| f64::from(u8::from(x.0))).collect();
                let clumped: Vec<f64> = v.iter().filter(|x| x.0).map(|x| x.1).collect();
                all_of(vec![
                    ("both present".into(), range(&present, 1.0, 1.0, false)),
                    ("regions".into(), range(&clumped, 0.0, 0.5, false)),
                ])
                .with("Measured at 100, 500, 2,000 and 5,000 cycles: both conventions persist in 18, 17, 16 and 11 of 30 runs — the maps persist over hundreds of cycles and erode over thousands.")
            },
        },
        Claim {
            id: "dpd-rr.run-1",
            item: "dpd-run-1",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs: of their timing settings only one reproduces Run 1, a synchronous one they set aside — no asynchronous setting of the six model switches passes their t-tests against Table 1",
            check: |_| {
                let mut fails = Vec::new();
                for bits in 0..32u32 {
                    // Asynchronous (the fifth switch) and every setting of the other five.
                    let flags: String = [0, 1, 2, 3, 99, 4]
                        .map(|k| if k == 99 || bits & (1 << k) == 0 { 'T' } else { 'F' })
                        .iter()
                        .collect();
                    let (c, d) = counts(&rr_setting(&flags, run_1()), 500, 500);
                    fails.push(f64::from(u8::from(!passes(&c, &d, TABLE_1))));
                }
                range(&fails, 1.0, 1.0, false).with(
                    "Of all 64 settings none passes here, synchronous ones included (the core's radax_and_rengs_factorial_as_measured pins the full factorial).",
                )
            },
        },
        Claim {
            id: "dpd-rr.run-2",
            item: "dpd-run-2",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs' Table 7: seven settings of their switches (six once their RNG column is dropped) reproduce Run 2 — each passes their t-tests against Table 2",
            check: |_| {
                let mut pass = Vec::new();
                let mut notes = Vec::new();
                for flags in RR_RUN_2_FITS {
                    let (c, d) = counts(&rr_setting(flags, run_2()), 500, 500);
                    pass.push(f64::from(u8::from(passes(&c, &d, TABLE_2))));
                    notes.push(format!(
                        "{flags}: {:.0} / {:.0} (t {:.1} / {:.1})",
                        mean_sd(&c).0,
                        mean_sd(&d).0,
                        t_stat(TABLE_2[0], &c),
                        t_stat(TABLE_2[1], &d)
                    ));
                }
                range(&pass, 1.0, 1.0, false).with(&format!(
                    "{}. Our only Run 2 fit of the 64 is TFTTTT (785 / 110), not one of theirs.",
                    notes.join("; ")
                ))
            },
        },
        Claim {
            id: "dpd-rr-best.table-2",
            item: "dpd-rr-best",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs' best Run 2 fit (dead removed at the end of the cycle, death at once, endowment granted, random newborn age, asynchronous, Epstein's swaps): 780 (25) cooperators, 97 (22) defectors, inside Table 2's ranges (708, 846) and (45, 160)",
            check: |_| {
                let (c, d) = counts(&rr_setting("FTFTTF", run_2()), 500, 500);
                in_ranges(&c, &d, (708.0, 846.0), (45.0, 160.0)).with(&format!(
                    "{}; {}. The same six switches in two implementations give different models.",
                    against("cooperators", &c, TABLE_2[0]),
                    against("defectors", &d, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-working-paper.table-1",
            item: "dpd-working-paper",
            source: Source::Book,
            citation: WP,
            text: "Table 1 of the working paper, whose rule plays one random neighbour a turn: cooperators range (752, 806), mean 779; defectors range (93, 148), mean 121",
            check: |_| {
                let wp = |c: DpdConfig| DpdConfig {
                    play: Play::RandomNeighbor,
                    ..c
                };
                let (c, d) = counts(&wp(run_1()), 500, 500);
                let (c2, d2) = counts(&wp(run_2()), 500, 500);
                in_ranges(&c, &d, (752.0, 806.0), (93.0, 148.0)).with(&format!(
                    "{}; {} — nearer Table 1 than the published rule's 729 / 171, but still rejected. Run 2 with this rule: {}; {}.",
                    against("cooperators", &c, TABLE_1[0]),
                    against("defectors", &d, TABLE_1[1]),
                    against("cooperators", &c2, TABLE_2[0]),
                    against("defectors", &d2, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-closest.both-tables",
            item: "dpd-closest",
            source: Source::App,
            citation: OURS,
            text: "dpd-closest (the working paper's rule, no initial wealth, newborns acting at once) reproduces both tables: each run inside Tables 9.2's and 9.4's ranges",
            check: |_| {
                let (c1, d1) = counts(&closest(run_1()), 500, 500);
                let (c2, d2) = counts(&closest(run_2()), 500, 500);
                all_of(vec![
                    ("Table 1".into(), in_ranges(&c1, &d1, (752.0, 806.0), (93.0, 148.0))),
                    ("Table 2".into(), in_ranges(&c2, &d2, (708.0, 846.0), (45.0, 160.0))),
                ])
                .with(&format!(
                    "{}; {}; {}; {}. On seeds 31–60 and 61–90 Run 2's defectors fail RR's test (t −2.6, −3.2).",
                    against("Run 1 cooperators", &c1, TABLE_1[0]),
                    against("Run 1 defectors", &d1, TABLE_1[1]),
                    against("Run 2 cooperators", &c2, TABLE_2[0]),
                    against("Run 2 defectors", &d2, TABLE_2[1])
                ))
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_9_3_rows_are_the_printed_ones() {
        assert_eq!(TABLE_9_3.len(), 45);
        let mut cells = Vec::new();
        for t in (2..=10).rev() {
            for r in (1..t).rev() {
                cells.push((t, r));
            }
        }
        let ours: Vec<(u32, u32)> = TABLE_9_3.iter().map(|&(t, r, ..)| (t, r)).collect();
        assert_eq!(ours, cells);
        // Run 1 is the cell (6, 5).
        let run_1 = TABLE_9_3.iter().find(|c| (c.0, c.1) == (6, 5)).unwrap();
        assert_eq!(
            (run_1.2[0], run_1.2[1], run_1.3[0], run_1.3[1]),
            (779.0, 15.0, 121.0, 15.0)
        );
    }

    #[test]
    fn rr_flags_set_the_switches_in_their_order() {
        let c = rr_setting("FTFTTF", run_2());
        assert_eq!(
            (
                c.removal,
                c.death_timing,
                c.endowment_from,
                c.newborn_age,
                c.updating,
                c.shuffle
            ),
            (
                Removal::EndOfCycle,
                DeathTiming::Immediate,
                EndowmentFrom::Granted,
                NewbornAge::Random,
                Updating::Asynchronous,
                Shuffle::Swaps
            )
        );
        assert_eq!(rr_setting("TTTTTT", run_1()).shuffle, Shuffle::Full);
        assert_eq!(rr_setting("TTTTTF", run_1()), run_1());
    }

    #[test]
    fn rr_t_matches_their_appendix() {
        // RR's appendix row 1: 779 (15) against 755 (16) prints t = 6.01
        // (from unrounded means).
        assert!((t_from((779.0, 15.0), (755.0, 16.0)) - 5.99).abs() < 0.01);
        let v: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 771.0 } else { 739.0 })
            .collect();
        assert_eq!(t_stat((755.0, 15.0), &v), 0.0);
        assert!(passes(&[779.0; 30], &[121.0; 30], TABLE_1));
        assert!(!passes(&[729.0; 30], &[121.0; 30], TABLE_1));
    }

    #[test]
    fn swings_need_both_thresholds() {
        let run = |c: Vec<f64>| Run {
            d: vec![0.0; c.len()],
            c,
        };
        assert_eq!(swings(&run(vec![500.0, 50.0, 500.0, 50.0])), 2.0);
        assert_eq!(swings(&run(vec![500.0, 300.0, 500.0, 50.0])), 1.0);
        assert_eq!(swings(&run(vec![50.0, 500.0, 300.0])), 0.0);
    }
}
