//! Algorithmic Collusion: Calvano, Calzolari, Denicolò & Pastorello (2020),
//! checked against the authors' own code, and the critics' tests (the spec's
//! A and B tables, docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md).
//! Sessions are memoized per process by config and count; every rule here
//! was fixed in the spec before measuring, except where a claim says so.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sugarscape_core::collusion::analysis::{
    best_response_deviation, equilibrium, every_deviation, invitation, lambin_point, limit_cycle,
    policy_values, repair, rp_complete, Strategies,
};
use sugarscape_core::collusion::demand::Game;
use sugarscape_core::collusion::learner::Space;
use sugarscape_core::collusion::{
    as_coded, BestResponseTo, CollusionConfig, CollusionWorld, EquilibriumCheck, Exploration, Grid,
    Update,
};

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::on_threads;

const CCDP: &str = "Calvano, Calzolari, Denicolò & Pastorello 2020, AER 110(10)";
const L24: &str = "Lambin 2024, SSRN 4498926";
const EL: &str = "Epivent & Lambin 2024, Economics Letters 237 (SSRN 4227229)";
const DBMS: &str = "den Boer, Meylahn & Schinkel 2026, Amsterdam LSRP 2022-25";
const AFP: &str = "Asker, Fershtman & Pakes 2021, NBER w28535";
const EMZ: &str = "Eschenbaum, Mellgren & Zahn 2022, arXiv 2210.10528";

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

/// What the survey keeps of a session.
#[derive(Clone, Debug)]
struct Run {
    gain: f64,
    converged: bool,
    /// Both firms at one price every period: that price's index.
    point: Option<u8>,
    /// A one-price cycle, symmetric or not.
    single: bool,
    eq_best: bool,
    eq_one_shot: bool,
    /// Over every cycle position and deviating firm, the paper's deviation
    /// (the static best response to the rival's price there, or the code's
    /// impulse-response reading): punishment-like shares; the non-deviator's
    /// relative price change in the next period.
    punished: f64,
    punished_code: f64,
    response: f64,
    response_code: f64,
    /// Table A5's rows (the code's DetailedAnalysis: one per cycle position
    /// and deviating firm, the deviation to the static best response at the
    /// state): (IR, IC, punishment length).
    a5: Vec<(f64, f64, f64)>,
    /// Rival prices: in the deviation period and the 11 after (Fig. 4).
    rival_path: Vec<f64>,
    /// Every one-period deviation up and down: punishment-like shares.
    up: f64,
    down: f64,
    /// Firm 1 deviating from a symmetric point: (own, deviation, firm 2's
    /// relative change).
    table: Vec<(u8, u8, f64)>,
    rp: bool,
    invitation: Option<(u8, u8)>,
    window: f64,
    greedy: Vec<f64>,
    strategies: Strategies,
}

type Key = String;
static MEMO: OnceLock<Mutex<HashMap<Key, Arc<Vec<Run>>>>> = OnceLock::new();

/// `n` sessions (seeds 1..=n) of `c`, each read for the mean greedy price
/// at `at` periods (L24's Fig. 1) and then run to its end.
fn runs(c: &CollusionConfig, n: u64, at: &[u64]) -> Arc<Vec<Run>> {
    let key = format!("{}|{n}|{at:?}", serde_json::to_string(c).unwrap());
    let memo = MEMO.get_or_init(Default::default);
    if let Some(r) = memo.lock().unwrap().get(&key) {
        return r.clone();
    }
    let seeds: Vec<u64> = (1..=n).collect();
    let out = Arc::new(on_threads(&seeds, |seed| session(c, seed, at)));
    memo.lock().unwrap().insert(key, out.clone());
    out
}

fn session(c: &CollusionConfig, seed: u64, at: &[u64]) -> Run {
    let mut w = CollusionWorld::new(c.clone(), seed).unwrap();
    let mut greedy = Vec::new();
    for &t in at {
        while w.period() < t && !w.is_finished() {
            let ticks = (t - w.period()).div_ceil(u64::from(c.periods_per_tick));
            w.run(ticks.min(1_000_000) as u32);
        }
        let s = w.current_state();
        greedy.push(
            (0..w.space().firms)
                .map(|i| w.game().grid[i][usize::from(w.firms()[i].greedy[s])])
                .sum::<f64>()
                / w.space().firms as f64,
        );
    }
    while !w.is_finished() {
        w.run(1_000_000);
    }
    let o = w.outcome().unwrap().clone();
    let (g, sp) = (w.game(), w.space());
    let (st, cy) = (&o.strategies, &o.cycle);
    let n = sp.firms;
    let mut values = Vec::new();
    for i in 0..n {
        values.push(policy_values(g, sp, st, i, c.delta));
    }
    let (mut pun, mut pun_code, mut resp, mut resp_code, mut count) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut a5 = Vec::new();
    let mut rival_path = vec![0.0; 12];
    for firm in 0..n {
        let rival = (firm + 1) % n;
        for k in 0..cy.len() {
            let r = best_response_deviation(g, sp, st, cy, k, firm, BestResponseTo::Path);
            let rc = best_response_deviation(g, sp, st, cy, k, firm, BestResponseTo::Code);
            pun += f64::from(u8::from(r.punishment_like()));
            pun_code += f64::from(u8::from(rc.punishment_like()));
            resp += r.change(g, rival);
            resp_code += rc.change(g, rival);
            for (t, slot) in rival_path.iter_mut().enumerate() {
                *slot += g.grid[rival][usize::from(rc.path[t][rival])];
            }
            // Table A5's IC: the deviation's discounted profit (deviate once,
            // then everyone follows the strategies) below the path's; a
            // "deviation" to the price the strategy charges anyway is not one.
            let s0 = cy.states[k];
            let a = &r.path[0];
            let dev = g.profit(a, firm) + c.delta * values[firm][sp.next(s0, a)];
            let ic = r.deviation != r.before[firm] && dev < values[firm][s0];
            a5.push((
                r.change(g, rival),
                f64::from(u8::from(ic)),
                r.settled as f64,
            ));
            count += 1.0;
        }
    }
    for x in rival_path.iter_mut() {
        *x /= count;
    }
    let all = every_deviation(sp, st, cy);
    let (mut up, mut upn, mut down, mut downn) = (0.0, 0.0, 0.0, 0.0);
    let mut table = Vec::new();
    for r in &all {
        let own = r.before[r.firm];
        let p = f64::from(u8::from(r.punishment_like()));
        if r.deviation > own {
            up += p;
            upn += 1.0;
        } else {
            down += p;
            downn += 1.0;
        }
        if cy.is_point() && r.firm == 0 && r.before.iter().all(|&x| x == own) {
            table.push((own, r.deviation, r.change(g, 1)));
        }
    }
    let point = (cy.is_point() && cy.actions[0].iter().all(|&x| x == cy.actions[0][0]))
        .then_some(cy.actions[0][0]);
    Run {
        gain: o.gain,
        converged: o.converged,
        point,
        single: cy.is_point(),
        eq_best: equilibrium(g, sp, st, cy, c.delta, EquilibriumCheck::BestResponse).on_path,
        eq_one_shot: equilibrium(g, sp, st, cy, c.delta, EquilibriumCheck::OneShot).on_path,
        punished: pun / count,
        punished_code: pun_code / count,
        response: resp / count,
        response_code: resp_code / count,
        a5,
        rival_path,
        up: if upn > 0.0 { up / upn } else { f64::NAN },
        down: if downn > 0.0 { down / downn } else { f64::NAN },
        table,
        rp: rp_complete(sp, st, cy),
        invitation: invitation(g, sp, st, cy, 0, c.invitation_hold),
        window: o.window_gain,
        greedy,
        strategies: st.clone(),
    }
}

fn mean(v: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = v.filter(|x| x.is_finite()).collect();
    v.iter().sum::<f64>() / v.len() as f64
}

fn se(v: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = v.filter(|x| x.is_finite()).collect();
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0)).sqrt()
        / (v.len() as f64).sqrt()
}

fn share(r: &[Run], f: impl Fn(&Run) -> bool) -> f64 {
    r.iter().filter(|x| f(x)).count() as f64 / r.len() as f64
}

fn base() -> CollusionConfig {
    CollusionConfig::default()
}

fn code() -> CollusionConfig {
    let mut c = base();
    as_coded(&mut c);
    c
}

fn with(edit: impl FnOnce(&mut CollusionConfig)) -> CollusionConfig {
    let mut c = base();
    edit(&mut c);
    c
}

fn gains(r: &[Run]) -> (f64, f64) {
    (mean(r.iter().map(|x| x.gain)), se(r.iter().map(|x| x.gain)))
}

/// The paper's figures, read from the replication package's vector PDFs
/// (collusion-figures.json): Fig. 1 (Δ) and Fig. 2 (the equilibrium share)
/// at the 10 × 10 subgrid, each cell its color bin's midpoint; Fig. 3, Δ
/// against δ.
#[derive(serde::Deserialize)]
struct Figures {
    fig1: Vec<[f64; 3]>,
    fig2: Vec<[f64; 3]>,
    delta: Vec<[f64; 2]>,
}

fn figures() -> Figures {
    serde_json::from_str(include_str!("collusion-figures.json")).unwrap()
}

/// Mean and largest absolute gap.
fn gaps(pairs: &[(f64, f64)]) -> (f64, f64) {
    let g: Vec<f64> = pairs.iter().map(|(a, b)| (a - b).abs()).collect();
    (
        mean(g.iter().copied()),
        g.iter().copied().fold(0.0, f64::max),
    )
}

#[derive(serde::Deserialize)]
struct Fixture {
    sessions: Vec<FixtureSession>,
}

#[derive(serde::Deserialize)]
struct FixtureSession {
    session: u64,
    periods: u64,
    strategies: Vec<u8>,
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "collusion.ccdp.table-i",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Table I (1 000 sessions, α = 0.15, β = 4 × 10⁻⁶): Δ = 0.849 and 50.5 % of sessions in equilibrium on path — under the code's readings, mean Δ within 2 SE of 0.849 and the equilibrium share (the code's test) within 5 points of 50.5 %",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let (m, s) = gains(&r);
                let eq = share(&r, |x| x.eq_one_shot);
                let lit = runs(&base(), 1000, &[]);
                let (lm, ls) = gains(&lit);
                outcome(
                    (m - 0.849).abs() <= 2.0 * s && (eq - 0.505).abs() <= 0.05,
                    format!("Δ {m:.4} ± {s:.4}, equilibrium {:.1} %; under the paper's readings Δ {lm:.4} ± {ls:.4}, equilibrium by the code's test {:.1} %", 100.0 * eq, 100.0 * share(&lit, |x| x.eq_one_shot)),
                )
                .with("Seeds 1–1 000 under the code's readings are the authors' own sessions 1–1 000 (collusion.ccdp.docking).")
            },
        },
        Claim {
            id: "collusion.ccdp.docking",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "The authors' code (replication package, built with gfortran): with their RAN2 seeded −session, random ties and their cap, our sessions 1–10 reach the same strategies in the same period — 10 of 10 (all 100 in the fixture reported)",
            check: |_| {
                let f: Fixture = serde_json::from_str(include_str!(
                    "../../../crates/sugarscape-core/tests/fixtures/calvano-sessions.json"
                ))
                .unwrap();
                let seeds: Vec<u64> = f.sessions.iter().map(|s| s.session).collect();
                let same = on_threads(&seeds, |seed| {
                    let s = &f.sessions[(seed - 1) as usize];
                    let mut w = CollusionWorld::new(code(), seed).unwrap();
                    while !w.is_finished() {
                        w.run(1_000_000);
                    }
                    let o = w.outcome().unwrap();
                    let ours: Vec<u8> = (0..225)
                        .flat_map(|st| [o.strategies[0][st], o.strategies[1][st]])
                        .collect();
                    o.periods == s.periods && ours == s.strategies
                });
                let first = same.iter().take(10).filter(|&&x| x).count();
                let all = same.iter().filter(|&&x| x).count();
                outcome(first == 10, format!("{first} of the first 10 identical; {all} of {}", same.len()))
            },
        },
        Claim {
            id: "collusion.ccdp.heat-map",
            item: "collusion-alpha-beta",
            source: Source::Book,
            citation: CCDP,
            text: "Figs. 1–2: Δ over α ∈ [0.025, 0.25] and β ∈ [0.02, 2] × 10⁻⁵ — on a 10 × 10 subgrid, 100 sessions a cell under the paper's readings, the mean absolute gap to the figure's cells at most 0.03 and the worst at most 0.08; Fig. 2's equilibrium share (the code's test) reported",
            check: |_| {
                let f = figures();
                let mut pairs = Vec::new();
                let mut eq = Vec::new();
                for (k, cell) in f.fig1.iter().enumerate() {
                    let c = with(|c| {
                        c.alpha = cell[0];
                        c.beta = cell[1] * 1e-5;
                    });
                    let r = runs(&c, 100, &[]);
                    pairs.push((gains(&r).0, cell[2]));
                    eq.push((share(&r, |x| x.eq_one_shot), f.fig2[k][2]));
                }
                let (m, w) = gaps(&pairs);
                let (em, ew) = gaps(&eq);
                outcome(m <= 0.03 && w <= 0.08, format!("Δ: mean gap {m:.4}, worst {w:.4}; equilibrium share: mean gap {em:.3}, worst {ew:.3}"))
            },
        },
        Claim {
            id: "collusion.ccdp.delta",
            item: "collusion-delta",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 3: Δ against δ falls to a minimum of 0.156 at δ = 0.34 and rises to 0.85 at 0.95 — at 13 values of δ (100 sessions each), the mean absolute gap to the curve at most 0.03 and the worst at most 0.08",
            check: |_| {
                let f = figures();
                let pairs: Vec<(f64, f64)> = f
                    .delta
                    .iter()
                    .map(|p| (gains(&runs(&with(|c| c.delta = p[0]), 100, &[])).0, p[1]))
                    .collect();
                let (m, w) = gaps(&pairs);
                let low = f.delta.iter().zip(&pairs).min_by(|a, b| a.1 .0.total_cmp(&b.1 .0)).unwrap();
                outcome(m <= 0.03 && w <= 0.08, format!("mean gap {m:.4}, worst {w:.4}; our lowest Δ {:.4} at δ = {}", low.1 .0, low.0[0]))
                    .with("The paper's own curve gives Δ = 0.212 at δ = 0, where no reward–punishment scheme can pay (collusion.critics.myopic).")
            },
        },
        Claim {
            id: "collusion.ccdp.impulse",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 4: after a one-period deviation to the static best response the rival's price falls from 1.795 to 1.551 in the next period and is back near 1.79 by period 10; Table A5 (n = 2, 1 000 sessions): the rival's mean relative change −0.127, deviations unprofitable in 0.936 ('in more than 95 % of the cases', p. 3282), punishment 5.705 periods — each within 2 SE, under the code's readings (Table A5's rows pooled as its script pools them)",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let path = |t: usize| mean(r.iter().map(|x| x.rival_path[t]));
                let path_se = |t: usize| se(r.iter().map(|x| x.rival_path[t]));
                let rows: Vec<(f64, f64, f64)> = r.iter().flat_map(|x| x.a5.iter().copied()).collect();
                let pooled = |f: fn(&(f64, f64, f64)) -> f64| (mean(rows.iter().map(f)), se(rows.iter().map(f)));
                let (ir, ic, len) = (pooled(|x| x.0), pooled(|x| x.1), pooled(|x| x.2));
                let within = |v: (f64, f64), want: f64| (v.0 - want).abs() <= 2.0 * v.1;
                all_of(vec![
                    ("rival at τ = 2 (Fig. 4)".into(), outcome(within((path(1), path_se(1)), 1.551), format!("{:.3} → {:.3} (± {:.3}), {:.3} at τ = 10", path(0), path(1), path_se(1), path(9)))),
                    ("IR".into(), outcome(within(ir, -0.127), format!("{:.4} ± {:.4} over {} rows", ir.0, ir.1, rows.len()))),
                    ("IC".into(), outcome(within(ic, 0.936), format!("{:.4} ± {:.4} (the text: more than 0.95)", ic.0, ic.1))),
                    ("punishment length".into(), outcome(within(len, 5.705), format!("{:.3} ± {:.3} periods", len.0, len.1))),
                ])
                .with("Fig. 4 comes from the code's impulse-response routine, which answers the state numbered by the cycle position (collusion.ccdp.figure-4-reading); Table A5 from its detailed analysis, which answers the state itself. On the authors' first 100 sessions ours equal their Fortran's to 10⁻⁹ (IR −0.12236, IC 0.92628, length 5.58013).")
            },
        },
        Claim {
            id: "collusion.ccdp.equilibrium",
            item: "collusion-calvano",
            source: Source::Book,
            citation: CCDP,
            text: "p. 3278: half the sessions are subgame-perfect-like on path — the paper describes solving for the true Q (eq. 3) and checking best responses; the code checks only one-period deviations. Under the paper's description, at least 40 % of sessions in equilibrium on path (the code's test gives 50.5 %; 40 % is ours)",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let b = share(&r, |x| x.eq_best);
                let o = share(&r, |x| x.eq_one_shot);
                let contained = r.iter().all(|x| !x.eq_best || x.eq_one_shot);
                outcome(b >= 0.4 && contained, format!("{:.1} % a best response on path; {:.1} % pass the one-period test; every best-response session also passes it: {contained}", 100.0 * b, 100.0 * o))
                    .with("Re-optimizing against the rival's learned strategy gains 5–41 % of a firm's value on path (20 sessions measured in planning): the strategies punish one-period deviations but can be exploited over longer ones.")
            },
        },
        Claim {
            id: "collusion.ccdp.figure-4-reading",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 4's deviation is 'the static best response' to the rival's price — but the code's ImpulseResponse.f90 passes the cycle position where it means the state; reported: the share of deviations whose price differs between the two readings (at least 1 % counts as a difference, ours)",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let differ = r.iter().filter(|x| (x.response - x.response_code).abs() > 1e-12).count() as f64 / r.len() as f64;
                outcome(differ < 0.01, format!("sessions whose responses differ between the readings: {:.1} %; punishment-like after the paper's deviation {:.3}, the code's {:.3}", 100.0 * differ, mean(r.iter().map(|x| x.punished)), mean(r.iter().map(|x| x.punished_code))))
            },
        },
        Claim {
            id: "collusion.l24.memoryless-higher",
            item: "collusion-no-memory",
            source: Source::Book,
            citation: L24,
            text: "Fig. 1: memoryless algorithms (δ = 0.95) price at least as high as algorithms with one period of memory — the mean greedy price at 1.5 × 10⁶ and 2 × 10⁶ periods, and the converged Δ, each at least as high at k = 0 (difference ≥ −2 SE)",
            check: |_| {
                let at = [1_500_000, 2_000_000];
                let one = runs(&base(), 1000, &at);
                let none = runs(&with(|c| c.memory = 0), 1000, &at);
                let cmp = |f: &dyn Fn(&Run) -> f64| {
                    let (a, b) = (mean(none.iter().map(f)), mean(one.iter().map(f)));
                    let s = (se(none.iter().map(f)).powi(2) + se(one.iter().map(f)).powi(2)).sqrt();
                    (a - b >= -2.0 * s, a, b)
                };
                let (h1, a1, b1) = cmp(&|x| x.greedy[0]);
                let (h2, a2, b2) = cmp(&|x| x.greedy[1]);
                let (h3, a3, b3) = cmp(&|x| x.gain);
                outcome(h1 && h2 && h3, format!("greedy price at 1.5M: {a1:.4} against {b1:.4}; at 2M: {a2:.4} against {b2:.4}; Δ {a3:.4} against {b3:.4} (k = 0 against k = 1)"))
                    .with("L24 Fig. 1: about 1.85 against 1.76 at 2 × 10⁶. No memoryless session is an equilibrium, and none answers any deviation.")
            },
        },
        Claim {
            id: "collusion.critics.memoryless-half",
            item: "collusion-no-memory",
            source: Source::Comment,
            citation: "the spec's B1 (secondary)",
            text: "B1's weaker critique, fixed before reading L24's own criterion: Δ(k = 0) ≥ ½ Δ(k = 1), at δ = 0.95; CCDP-A's memoryless reading (δ = 0) reported",
            check: |_| {
                let one = gains(&runs(&base(), 1000, &[])).0;
                let none = gains(&runs(&with(|c| c.memory = 0), 1000, &[])).0;
                let ccdp = gains(&runs(&with(|c| {
                    c.memory = 0;
                    c.delta = 0.0;
                }), 1000, &[]))
                .0;
                outcome(none >= 0.5 * one, format!("Δ {none:.4} against {one:.4}; memory 0 with δ = 0 (CCDP-A): {ccdp:.4}"))
                    .with("The code sets δ = 0 whenever memory is 0 (globals.f90), so it cannot run L24's test.")
            },
        },
        Claim {
            id: "collusion.critics.myopic",
            item: "collusion-myopic",
            source: Source::Comment,
            citation: "the spec's B2 (Schildknecht 2026's δ = 0)",
            text: "B2: with δ = 0 there is no future to protect, so no reward–punishment scheme can pay; the critique holds if Δ(δ = 0) > 0.1, and Δ(0.95) − Δ(0) is then the part strategies can explain",
            check: |_| {
                let (z, zs) = gains(&runs(&with(|c| c.delta = 0.0), 1000, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(z > 0.1, format!("Δ(δ = 0) {z:.4} ± {zs:.4}; Δ(0.95) − Δ(0) = {:.4} ({:.0} % of the baseline)", b - z, 100.0 * (b - z) / b))
            },
        },
        Claim {
            id: "collusion.l24.theorem-1",
            item: "collusion-two-phase",
            source: Source::Book,
            citation: L24,
            text: "Theorem 1: after exploring every price at random (here 1 000 periods), the firms settle at I — 1.6990 at δ = 0 and 1.7377 at δ = 0.95 on CCDP's grid — with or without memory; holds if at least 80 % of sessions end with both firms at I in each of the four cases (80 % is ours)",
            check: |_| {
                let mut parts = Vec::new();
                for (memory, delta) in [(1, 0.0), (0, 0.0), (1, 0.95), (0, 0.95)] {
                    let c = with(|c| {
                        c.exploration = Exploration::TwoPhase;
                        c.memory = memory;
                        c.delta = delta;
                    });
                    let i = lambin_point(&Game::new(&c), delta);
                    let r = runs(&c, 1000, &[]);
                    let at = share(&r, |x| x.point == Some(i));
                    let mut modal: HashMap<u8, usize> = HashMap::new();
                    for x in r.iter() {
                        if let Some(p) = x.point {
                            *modal.entry(p).or_default() += 1;
                        }
                    }
                    let top = modal.iter().max_by_key(|(_, &n)| n).map(|(&p, &n)| (p, n));
                    parts.push((
                        format!("k = {memory}, δ = {delta}"),
                        outcome(at >= 0.8, format!("{:.1} % at I = index {i}; most common point {top:?}; Δ {:.4}", 100.0 * at, gains(&r).0)),
                    ));
                }
                // Reported beside the rule (k = 0, δ = 0.95): his figures' apparent
                // start (zero Q-values), and a hundred times more exploring.
                let extra = |edit: fn(&mut CollusionConfig)| {
                    let c = with(|c| {
                        c.exploration = Exploration::TwoPhase;
                        c.memory = 0;
                        edit(c);
                    });
                    let r = runs(&c, 1000, &[]);
                    (100.0 * share(&r, |x| x.point == Some(8)), gains(&r).0)
                };
                let (z, zg) = extra(|c| c.q_init = sugarscape_core::collusion::QInit::Zero);
                let (l, lg) = extra(|c| c.explore_for = 100_000);
                all_of(parts)
                    .with("Theorem 1 is a mean-field limit (every Q-value at its mean after exploring); with α = 0.15 the Q-values stay noisy.")
                    .with(&format!("k = 0, δ = 0.95 from zero Q-values: {z:.1} % at I, Δ {zg:.4}; exploring 10⁵ periods: {l:.1} % at I, Δ {lg:.4}."))
            },
        },
        Claim {
            id: "collusion.el.table-1",
            item: "collusion-every-price",
            source: Source::Book,
            citation: EL,
            text: "Table 1: price increases are 'also followed by aggressive price wars' — in every (pre-deviation price, upward deviation) cell with at least 30 sessions the non-deviator's mean relative change at τ + 1 is negative, and the mean over upward cells is at least half the mean over downward cells (30 and half are ours); the first rule (punishment-like responses after upward deviations at least half as often as after downward ones) reported",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let mut cells: HashMap<(u8, u8), (f64, usize)> = HashMap::new();
                for x in r.iter() {
                    for &(own, dev, ch) in &x.table {
                        let e = cells.entry((own, dev)).or_default();
                        e.0 += ch;
                        e.1 += 1;
                    }
                }
                let (mut ups, mut downs) = (Vec::new(), Vec::new());
                for (&(own, dev), &(sum, n)) in &cells {
                    if n >= 30 {
                        let m = sum / n as f64;
                        if dev > own { ups.push(m) } else { downs.push(m) }
                    }
                }
                let negative = ups.iter().filter(|&&m| m < 0.0).count();
                let (mu, md) = (mean(ups.iter().copied()), mean(downs.iter().copied()));
                let (pu, pd) = (mean(r.iter().map(|x| x.up)), mean(r.iter().map(|x| x.down)));
                outcome(negative == ups.len() && !ups.is_empty() && mu <= 0.5 * md, format!("{negative} of {} upward cells negative; mean change after increases {mu:.4}, after cuts {md:.4}; punishment-like after increases {pu:.3}, after cuts {pd:.3}", ups.len()))
            },
        },
        Claim {
            id: "collusion.el.invitation",
            item: "collusion-invitation",
            source: Source::Book,
            citation: EL,
            text: "Fig. 2: an 'invitation to collude' — one firm a step up, the rival made to follow — is answered by a price cut: the deviator's mean price when it regains control is at least one grid step below its price before",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let v: Vec<(u8, u8)> = r.iter().filter_map(|x| x.invitation).collect();
                let drop = mean(v.iter().map(|&(b, a)| f64::from(a) - f64::from(b)));
                outcome(drop >= 1.0, format!("{} point sessions; mean drop {drop:.2} grid steps; {:.0} % cut", v.len(), 100.0 * v.iter().filter(|(b, a)| b < a).count() as f64 / v.len() as f64))
            },
        },
        Claim {
            id: "collusion.el.below-nash",
            item: "collusion-below-nash",
            source: Source::Book,
            citation: EL,
            text: "App. C (15 prices from 1.25 to 1.47, 10 000 sessions): 53 % converge to the top price, 9 % to cycles, 38 % to points below it (reported); in the below-top points, upward deviations draw punishment-like responses at least half as often as downward ones",
            check: |_| {
                let mut parts = Vec::new();
                for (label, top) in [("1.47", 1.47), ("p^N = 1.47293", 1.47293)] {
                    let c = with(|c| {
                        c.grid = Grid::BelowNash;
                        c.below_top = top;
                    });
                    let r = runs(&c, 10_000, &[]);
                    let at_top = share(&r, |x| x.point == Some(14));
                    let cycles = share(&r, |x| !x.single);
                    let below: Vec<&Run> = r.iter().filter(|x| x.single && x.point != Some(14)).collect();
                    let (u, d) = (mean(below.iter().map(|x| x.up)), mean(below.iter().map(|x| x.down)));
                    parts.push((format!("top {label}"), outcome(u >= 0.5 * d, format!("top price {:.1} %, cycles {:.1} %, other points {:.1} %; there punishment-like after increases {u:.3}, after cuts {d:.3}", 100.0 * at_top, 100.0 * cycles, 100.0 * below.len() as f64 / r.len() as f64))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "collusion.afp.synchronous",
            item: "collusion-synchronous",
            source: Source::Book,
            citation: AFP,
            text: "Updating every price toward what it would have earned (synchronous learning) removes the supra-competitive prices: Δ falls by more than half",
            check: |_| {
                let s = gains(&runs(&with(|c| c.update = Update::Synchronous), 1000, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}", s.0, s.1))
            },
        },
        Claim {
            id: "collusion.critics.slower-decay",
            item: "collusion-explore-more",
            source: Source::Comment,
            citation: "the spec's B5 (Abada & Lambin 2023; Calvano et al. 2023)",
            text: "B5: with ten times slower exploration decay (β = 4 × 10⁻⁷) Δ falls by more than half (200 sessions: a session takes about ten times longer)",
            check: |_| {
                let s = gains(&runs(&with(|c| c.beta = 4e-7), 200, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}", s.0, s.1))
            },
        },
        Claim {
            id: "collusion.critics.constant-exploration",
            item: "collusion-explore-more",
            source: Source::Comment,
            citation: "the spec's B5",
            text: "B5: with a constant ε = 0.05 Δ falls by more than half — read at 10⁷ periods, 100 sessions (set after measuring: no session settled in 10⁸ periods, so the 10⁹ cap would take hours)",
            check: |_| {
                let c = with(|c| {
                    c.exploration = Exploration::Constant;
                    c.epsilon = 0.05;
                    c.cap = 10_000_000;
                });
                let r = runs(&c, 100, &[]);
                let s = gains(&r);
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}; {:.0} % converged; realized Δ over the last window {:.4}", s.0, s.1, 100.0 * share(&r, |x| x.converged), mean(r.iter().map(|x| x.window))))
            },
        },
        Claim {
            id: "collusion.emz.repair",
            item: "collusion-calvano",
            source: Source::Book,
            citation: EMZ,
            text: "Collusion does not transfer: firm 1 trained in session s against firm 2 trained in session s + 1 (greedy play from a fixed state) earns less than half the original pairs' Δ",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let c = base();
                let (g, sp) = (Game::new(&c), Space::of(&c));
                let n = r.len();
                let start = |i: usize| (i * 7919) % sp.states;
                let cross = mean((0..n).map(|i| repair(&g, &sp, &r[i].strategies, &r[(i + 1) % n].strategies, start(i)).gain(&g)));
                let own = mean((0..n).map(|i| limit_cycle(&g, &sp, &r[i].strategies, start(i)).gain(&g)));
                outcome(cross < 0.5 * own, format!("cross pairs Δ {cross:.4}; the same pairs from the same states {own:.4}"))
            },
        },
        Claim {
            id: "collusion.dbms.timescale",
            item: "collusion-calvano",
            source: Source::Book,
            citation: DBMS,
            text: "§3: within the effective horizon T_δ = 165 periods, Q-learning prices like uniform random play: Δ̃ within 2 SE of 0.497 (and of −0.510 on the grid centered on Nash, ξ = 0)",
            check: |_| {
                let mut parts = Vec::new();
                for (label, want, c) in [
                    ("CCDP's grid", 0.497, base()),
                    ("centered on Nash", -0.510, with(|c| {
                        c.grid = Grid::Symmetric;
                        c.xi = 0.0;
                    })),
                ] {
                    let seeds: Vec<u64> = (1..=1000).collect();
                    let d = on_threads(&seeds, |seed| {
                        let mut w = CollusionWorld::new(c.clone(), seed).unwrap();
                        while w.period() < u64::from(w.horizon()) {
                            w.run(1);
                        }
                        w.discounted_gain()
                    });
                    let (m, s) = (mean(d.iter().copied()), se(d.iter().copied()));
                    parts.push((label.to_string(), outcome((m - want).abs() <= 2.0 * s, format!("Δ̃ {m:.4} ± {s:.4}"))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "collusion.dbms.rp-complete",
            item: "collusion-calvano",
            source: Source::Book,
            citation: DBMS,
            text: "§5: the pattern is not a scheme — among sessions where the paper's deviation draws a punishment-like response, at least a quarter are not RP-complete (some one-period deviation goes unpunished; a quarter is ours)",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let pass: Vec<&Run> = r.iter().filter(|x| x.punished > 0.0).collect();
                let not = pass.iter().filter(|x| !x.rp).count();
                outcome(not as f64 >= 0.25 * pass.len() as f64, format!("{} sessions pass CCDP's test; {not} of them not RP-complete; {:.1} % of all sessions RP-complete", pass.len(), 100.0 * share(&r, |x| x.rp)))
            },
        },
    ]
}
