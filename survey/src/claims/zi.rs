//! Zero-intelligence traders (milestone 28): Gode and Sunder (1993), with
//! Cliff's (1997) critique and ZIP traders. Gode and Sunder's markets run six
//! periods of 2 000 shouts (their "30 seconds" is never translated into
//! shouts); Cliff's run his simulator as coded (his mechanism, side-first
//! turns, days of 11 sessions for ZI-C and 9 for ZIP, each ending in a trade
//! or 100 failures; NYSE rules for ZI-C, not for ZIP) for ten days.

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::zi::{
    Market, Mechanism, Momentum, PeriodEnd, Shift, Strategy, Turns, ZiConfig, ZiWorld,
};

use crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const GS: &str = "Gode & Sunder 1993, JPE 101: 119";
const CLIFF: &str = "Cliff 1997, HP Labs HPL-97-91";

const GS_MARKETS: [Market; 5] = [
    Market::Gs1,
    Market::Gs2,
    Market::Gs3,
    Market::Gs4,
    Market::Gs5,
];
const CLIFF_MARKETS: [Market; 4] = [
    Market::Symmetric,
    Market::FlatSupply,
    Market::ExcessDemand,
    Market::ExcessSupply,
];
/// Table 2 (efficiency, %) and Table 3 (profit dispersion).
const T2_U: [f64; 5] = [90.0, 90.0, 76.7, 48.8, 86.0];
const T2_C: [f64; 5] = [99.9, 99.2, 99.0, 98.2, 97.1];
const T3_U: [f64; 5] = [225.48, 253.12, 90.54, 363.80, 156.28];
const T3_C: [f64; 5] = [28.53, 49.81, 15.90, 60.47, 19.07];

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

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn gs(market: Market, strategy: Strategy) -> ZiConfig {
    ZiConfig {
        market,
        strategy,
        ..ZiConfig::default()
    }
}

/// Cliff's simulator in `market`.
fn cliff(market: Market, strategy: Strategy) -> ZiConfig {
    ZiConfig {
        market,
        strategy,
        price_max: 400,
        mechanism: Mechanism::Cliff,
        nyse: strategy != Strategy::Zip,
        turns: Turns::Side,
        period_end: PeriodEnd::Sessions,
        sessions: if strategy == Strategy::Zip { 9 } else { 11 },
        stop_at: 10,
        ..ZiConfig::default()
    }
}

/// Each seed's finished world.
fn worlds(c: ZiConfig, n: u64) -> Vec<ZiWorld> {
    model_after(&ModelConfig::Zi(c), &seeds(n), 1_000_000, |w| match w {
        ModelWorld::Zi(z) => z.as_ref().clone(),
        _ => unreachable!(),
    })
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    f.iter().sum::<f64>() / f.len().max(1) as f64
}

/// Each seed's mean over its periods of `f`.
fn per_seed(ws: &[ZiWorld], f: fn(&sugarscape_core::zi::Period) -> f64) -> Vec<f64> {
    ws.iter()
        .map(|w| mean(&w.periods().iter().map(f).collect::<Vec<_>>()))
        .collect()
}

/// The mean price on each day, across seeds.
fn daily(ws: &[ZiWorld], days: usize) -> Vec<f64> {
    (0..days)
        .map(|d| {
            mean(
                &ws.iter()
                    .filter_map(|w| w.periods().get(d).map(|p| p.mean_price))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn show(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|x| format!("{x:.1}")).collect();
    format!("[{}]", parts.join(", "))
}

/// The Spearman correlation between each period's trade order and the order
/// of the trades' surplus (value − cost, largest first), averaged over periods.
fn rank_correlation(w: &ZiWorld) -> f64 {
    let mut rs = Vec::new();
    for p in 1..=w.periods().len() as u64 {
        let t: Vec<_> = w.trades().filter(|t| t.period == p).collect();
        let n = t.len();
        if n < 3 {
            continue;
        }
        // Ranks of surplus, largest first (ties by average rank).
        let s: Vec<f64> = t
            .iter()
            .map(|t| f64::from(t.value) - f64::from(t.cost))
            .collect();
        let mut rank = vec![0.0; n];
        for i in 0..n {
            let above = s.iter().filter(|&&x| x > s[i]).count() as f64;
            let same = s.iter().filter(|&&x| x == s[i]).count() as f64;
            rank[i] = above + (same + 1.0) / 2.0;
        }
        let order: Vec<f64> = (1..=n).map(|k| k as f64).collect();
        let (mr, mo) = (mean(&rank), mean(&order));
        let cov: f64 = (0..n).map(|i| (rank[i] - mr) * (order[i] - mo)).sum();
        let (vr, vo): (f64, f64) = (
            rank.iter().map(|r| (r - mr) * (r - mr)).sum(),
            order.iter().map(|o| (o - mo) * (o - mo)).sum(),
        );
        if vr > 0.0 {
            rs.push(cov / (vr * vo).sqrt());
        }
    }
    mean(&rs)
}

/// The slope of the RMS deviation from P₀ against the transaction number
/// (Gode and Sunder's Fig. 6 and Table 1), pooled over periods and seeds, over
/// the transaction numbers reached in at least half the periods.
fn rms_slope(ws: &[ZiWorld]) -> f64 {
    let mut sq: Vec<(f64, u32)> = Vec::new();
    let mut periods = 0;
    for w in ws {
        let p0 = w.equilibrium().price;
        for p in 1..=w.periods().len() as u64 {
            periods += 1;
            for (k, t) in w.trades().filter(|t| t.period == p).enumerate() {
                if sq.len() <= k {
                    sq.push((0.0, 0));
                }
                sq[k].0 += (f64::from(t.price) - p0).powi(2);
                sq[k].1 += 1;
            }
        }
    }
    let pts: Vec<(f64, f64)> = sq
        .iter()
        .enumerate()
        .filter(|(_, (_, n))| 2 * n >= periods)
        .map(|(k, (s, n))| ((k + 1) as f64, (s / f64::from(*n)).sqrt()))
        .collect();
    let (mx, my) = (
        mean(&pts.iter().map(|p| p.0).collect::<Vec<_>>()),
        mean(&pts.iter().map(|p| p.1).collect::<Vec<_>>()),
    );
    let num: f64 = pts.iter().map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = pts.iter().map(|(x, _)| (x - mx) * (x - mx)).sum();
    num / den
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "zi.gs.table2-u",
            item: "gs-efficiency",
            source: Source::Comment,
            citation: GS,
            text: "A calibration check, not a finding: markets 1–4 were read from the figures with Table 2's ZI-U efficiencies (90.0, 90.0, 76.7, 48.8), which follow from the schedules because every unit trades; they must come out exactly (within 0.05), and market 5, read as well as the scan allows, within 1 of 86.0 (10 seeds × 6 periods)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiU), 10), |p| p.efficiency))).collect();
                let ok = v.iter().zip(T2_U).enumerate().all(|(k, (a, b))| (a - b).abs() <= if k < 4 { 0.05 } else { 1.0 });
                outcome(ok, format!("{} against {}", show(&v), show(&T2_U)))
                    .with("Market 4's second cost reads 141 by pixel; 142 is the value that gives 48.8.")
            },
        },
        Claim {
            id: "zi.gs.table2-c",
            item: "gs-efficiency",
            source: Source::Book,
            citation: GS,
            text: "Table 2, ZI-C: 'imposing a budget constraint … is sufficient to raise the allocative efficiency of these auctions close to 100 percent' — 99.9, 99.2, 99.0, 98.2, 97.1 (within 1.5 points in every market, 10 seeds × 6 periods of 2 000 shouts)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiC), 10), |p| p.efficiency))).collect();
                let ok = v.iter().zip(T2_C).all(|(a, b)| (a - b).abs() <= 1.5);
                outcome(ok, format!("{} against {}", show(&v), show(&T2_C)))
            },
        },
        Claim {
            id: "zi.gs.table3",
            item: "gs-dispersion",
            source: Source::Book,
            citation: GS,
            text: "Table 3: profit dispersion — ZI-C 28.5, 49.8, 15.9, 60.5, 19.1 (within 25 % in every market) and ZI-U above ZI-C in every market",
            check: |_| {
                let c: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiC), 10), |p| p.dispersion))).collect();
                let u: Vec<f64> = GS_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(gs(m, Strategy::ZiU), 10), |p| p.dispersion))).collect();
                let close = c.iter().zip(T3_C).all(|(a, b)| (a - b).abs() <= 0.25 * b);
                let above = u.iter().zip(&c).all(|(a, b)| a > b);
                all_of(vec![
                    ("ZI-C within 25 %".into(), outcome(close, format!("{} against {}", show(&c), show(&T3_C)))),
                    ("ZI-U above".into(), outcome(above, format!("ZI-U {} (Table 3: {})", show(&u), show(&T3_U)))),
                ])
            },
        },
        Claim {
            id: "zi.gs.table1",
            item: "gs-1",
            source: Source::Book,
            citation: GS,
            text: "Fig. 6 and Table 1: ZI-C prices 'converge slowly toward equilibrium within each period' — the RMS deviation from P₀ falls with the transaction number (a negative slope in every market, 10 seeds × 6 periods)",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS.iter().map(|&m| rms_slope(&worlds(gs(m, Strategy::ZiC), 10))).collect();
                outcome(v.iter().all(|&s| s < 0.0), format!("slopes {} (Table 1: −0.64, −0.61, −1.23, −3.59, −0.83)", show(&v)))
            },
        },
        Claim {
            id: "zi.gs.order",
            item: "gs-1",
            source: Source::Book,
            citation: GS,
            text: "Footnote 5: 'The Spearman rank correlation between the actual and the efficient order of surplus extracted is, on average, highest for ZI-C traders (.74), lowest for ZI-U traders (.42)' (ZI-C greater in market 1, 10 seeds)",
            check: |_| {
                let c: Vec<f64> = worlds(gs(Market::Gs1, Strategy::ZiC), 10).iter().map(rank_correlation).collect();
                let u: Vec<f64> = worlds(gs(Market::Gs1, Strategy::ZiU), 10).iter().map(rank_correlation).collect();
                greater(&c, &u, "ZI-C", "ZI-U").with(&format!("means: ZI-C {:.2}, ZI-U {:.2} (the footnote: .74, .42)", mean(&c), mean(&u)))
            },
        },
        Claim {
            id: "zi.gs.period",
            item: "gs-shouts",
            source: Source::Comment,
            citation: GS,
            text: "Ours: the period's unstated length decides the efficiency — 'six periods of specified duration (… 30 seconds for machine traders)' is never given in shouts; at 100 shouts a period ZI-C efficiency is below 90 in every market",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS
                    .iter()
                    .map(|&m| mean(&per_seed(&worlds(ZiConfig { shouts: 100, ..gs(m, Strategy::ZiC) }, 10), |p| p.efficiency)))
                    .collect();
                outcome(v.iter().all(|&e| e < 90.0), format!("{} at 100 shouts", show(&v)))
            },
        },
        Claim {
            id: "zi.gs.mechanism",
            item: "gs-mechanism",
            source: Source::Comment,
            citation: GS,
            text: "Ours: Table 2's ZI-C efficiencies survive Cliff's mechanism (a random willing trader at the shout's price, instead of the standing quote at the earlier price) — within 1.5 points in every market",
            check: |_| {
                let v: Vec<f64> = GS_MARKETS
                    .iter()
                    .map(|&m| mean(&per_seed(&worlds(ZiConfig { mechanism: Mechanism::Cliff, nyse: false, ..gs(m, Strategy::ZiC) }, 10), |p| p.efficiency)))
                    .collect();
                outcome(v.iter().zip(T2_C).all(|(a, b)| (a - b).abs() <= 1.5), format!("{} against {}", show(&v), show(&T2_C)))
            },
        },
        Claim {
            id: "zi.cliff.predictions",
            item: "cliff-prices",
            source: Source::Comment,
            citation: CLIFF,
            text: "Cliff's E(P) for ZI-C: 'The mean transaction price in ZI-C markets can be predicted from the expected value E(P) of the pdf given by the intersection of the sellers' offer-price pdf and the buyers' bid-price pdf' — 200 (symmetric), 233⅓ (flat supply), 125 (excess demand), 260 (excess supply); each within 5 in his simulator (50 seeds × 10 days)",
            check: |_| {
                let want = [200.0, 233.3, 125.0, 260.0];
                let v: Vec<f64> = CLIFF_MARKETS.iter().map(|&m| mean(&per_seed(&worlds(cliff(m, Strategy::ZiC), 50), |p| p.mean_price))).collect();
                let parts = CLIFF_MARKETS
                    .iter()
                    .zip(v.iter().zip(want))
                    .map(|(m, (&a, b))| (format!("{m:?}"), outcome((a - b).abs() <= 5.0, format!("{a:.1} against {b:.1}"))))
                    .collect();
                all_of(parts).with("His 233⅓ is not his own formula's: Eq. 5 gives 200 + 125/3 = 241⅔ (summing his discrete pdf, 245⅓).")
            },
        },
        Claim {
            id: "zi.cliff.book",
            item: "cliff-prices",
            source: Source::Comment,
            citation: CLIFF,
            text: "Ours: Cliff's critique holds in Gode and Sunder's own mechanism — with the standing quote and the earlier price, ZI-C mean prices in the flat and box markets also sit more than 10 from P₀ = 200 (50 seeds × 10 periods of 2 000 shouts)",
            check: |_| {
                let v: Vec<f64> = CLIFF_MARKETS[1..]
                    .iter()
                    .map(|&m| {
                        let c = ZiConfig { market: m, price_max: 400, stop_at: 10, ..ZiConfig::default() };
                        mean(&per_seed(&worlds(c, 50), |p| p.mean_price))
                    })
                    .collect();
                outcome(v.iter().all(|&p| (p - 200.0).abs() > 10.0), format!("flat, excess demand, excess supply: {} (Cliff's mechanism: see zi.cliff.predictions)", show(&v)))
            },
        },
        Claim {
            id: "zi.zip.converge",
            item: "zip-days",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in the symmetric and flat markets: prices converge to $2.00 'typically within the first four trading days' (the mean price on day 4 within 5 of 200 in both, 50 seeds)",
            check: |_| {
                let s = daily(&worlds(cliff(Market::Symmetric, Strategy::Zip), 50), 10);
                let f = daily(&worlds(cliff(Market::FlatSupply, Strategy::Zip), 50), 10);
                all_of(vec![
                    ("symmetric".into(), outcome((s[3] - 200.0).abs() <= 5.0, format!("days 1–10 {}", show(&s)))),
                    ("flat supply".into(), outcome((f[3] - 200.0).abs() <= 5.0, format!("days 1–10 {}", show(&f)))),
                ])
            },
        },
        Claim {
            id: "zi.zip.below",
            item: "zip-days",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in the excess-demand market: a 'comparatively slow (yet steady) approach … from below' (the daily mean below 200 on each of days 1–10, and higher on day 10 than day 1)",
            check: |_| {
                let d = daily(&worlds(cliff(Market::ExcessDemand, Strategy::Zip), 50), 10);
                outcome(d.iter().all(|&p| p < 200.0) && d[9] > d[0], format!("days 1–10 {}", show(&d)))
            },
        },
        Claim {
            id: "zi.zip.efficiency",
            item: "zip-symmetric",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP efficiency is 'typically very high (often averaging 100%)' (at least 99 % over days 2–10 in all four markets)",
            check: |_| {
                let v: Vec<f64> = CLIFF_MARKETS
                    .iter()
                    .map(|&m| {
                        let ws = worlds(cliff(m, Strategy::Zip), 50);
                        mean(&ws.iter().flat_map(|w| w.periods()[1..].iter().map(|p| p.efficiency)).collect::<Vec<_>>())
                    })
                    .collect();
                outcome(v.iter().all(|&e| e >= 99.0), format!("{} (symmetric, flat, excess demand, excess supply)", show(&v)))
            },
        },
        Claim {
            id: "zi.zip.dispersion",
            item: "zip-symmetric",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP's profit dispersion is 'in some cases approximately a factor of ten less' than ZI-C's (a tenth or less in at least one of the four markets, days 5–10)",
            check: |_| {
                let ratio: Vec<f64> = CLIFF_MARKETS
                    .iter()
                    .map(|&m| {
                        let late = |ws: Vec<ZiWorld>| mean(&ws.iter().flat_map(|w| w.periods()[4..].iter().map(|p| p.dispersion)).collect::<Vec<_>>());
                        late(worlds(cliff(m, Strategy::Zip), 50)) / late(worlds(cliff(m, Strategy::ZiC), 50))
                    })
                    .collect();
                outcome(ratio.iter().any(|&r| r <= 0.1), format!("ZIP over ZI-C: {}", show(&ratio)))
            },
        },
        Claim {
            id: "zi.zip.shift",
            item: "zip-shift",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP re-converges after a shift: demand up $0.50 (P₀ 225) or supply down $0.50 (P₀ 175) after day 10 — the day-20 mean within 10 of the new P₀ (50 seeds)",
            check: |_| {
                let run = |shift| {
                    let c = ZiConfig { shift, stop_at: 20, ..cliff(Market::Symmetric, Strategy::Zip) };
                    daily(&worlds(c, 50), 20)
                };
                let (d, s) = (run(Shift::Demand), run(Shift::Supply));
                all_of(vec![
                    ("demand".into(), outcome((d[19] - 225.0).abs() <= 10.0, format!("days 11–20 {}", show(&d[10..])))),
                    ("supply".into(), outcome((s[19] - 175.0).abs() <= 10.0, format!("days 11–20 {}", show(&s[10..])))),
                ])
            },
        },
        Claim {
            id: "zi.zip.retail",
            item: "zip-retail",
            source: Source::Comment,
            citation: CLIFF,
            text: "ZIP in Smith's retail market (only sellers shout): 'the average transaction prices are typically less than $2.00 (significantly below the theoretical equilibrium price of $2.25)' (the mean over days 5–10 at least 10 below 225)",
            check: |_| {
                let ws = worlds(ZiConfig { sellers_only: true, ..cliff(Market::Retail, Strategy::Zip) }, 50);
                let d = daily(&ws, 10);
                let late = mean(&d[4..]);
                outcome(late <= 215.0, format!("days 1–10 {} (days 5–10: {late:.1})", show(&d)))
            },
        },
        Claim {
            id: "zi.zip.momentum",
            item: "zip-momentum",
            source: Source::Comment,
            citation: CLIFF,
            text: "Ours: Cliff's text gives ZIP momentum γ ~ U[0.2, 0.8], his code U[0, 0.1]; the text's reading converges faster in the box markets (day-10 distance from P₀ smaller in both, Mann–Whitney)",
            check: |_| {
                let dist = |m, momentum| {
                    worlds(ZiConfig { momentum, ..cliff(m, Strategy::Zip) }, 50)
                        .iter()
                        .map(|w| (w.periods()[9].mean_price - 200.0).abs())
                        .collect::<Vec<f64>>()
                };
                all_of(vec![
                    ("excess demand".into(), greater(&dist(Market::ExcessDemand, Momentum::Code), &dist(Market::ExcessDemand, Momentum::Text), "code", "text")),
                    ("excess supply".into(), greater(&dist(Market::ExcessSupply, Momentum::Code), &dist(Market::ExcessSupply, Momentum::Text), "code", "text")),
                ])
            },
        },
    ]
}
