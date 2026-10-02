//! Calvano et al.'s baseline, their code's readings, and the critics' tests.

use super::config::{
    BestResponseTo, CollusionConfig, EquilibriumCheck, Exploration, Grid, Impulse, RngKind, Ties,
    Update,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const CCDP: &str = "Calvano, Calzolari, Denicolò & Pastorello 2020, AER 110(10)";
const AFP: &str = "Asker, Fershtman & Pakes 2021, NBER w28535";
const L24: &str = "Lambin 2024, SSRN 4498926";
const EL: &str = "Epivent & Lambin 2024, Economics Letters 237";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut CollusionConfig),
) -> ModelPreset {
    let mut c = CollusionConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Collusion(c),
    }
}

/// The authors' code's readings where it differs from the paper.
pub fn as_coded(c: &mut CollusionConfig) {
    c.ties = Ties::Random;
    c.rng = RngKind::Calvano;
    c.cap = 1_250_000_000;
    c.equilibrium_check = EquilibriumCheck::OneShot;
    c.best_response_to = BestResponseTo::Code;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "collusion-calvano",
            "Two pricing algorithms",
            CCDP,
            "Calvano, Calzolari, Denicolò and Pastorello's baseline: two firms selling differentiated goods each pick one of 15 prices, using Q-learning on last period's two prices, exploring at random less and less (ε = e^(−βt), β = 4 × 10⁻⁶); a session ends when neither firm's strategy has changed for 100 000 periods. The paper: the algorithms learn to price far above the competitive level (profit gain Δ = 0.849) and punish a rival's price cut. Measured (1 000 sessions, the paper's readings): Δ 0.851 ± 0.004, settled after about 1.76 million periods; 62.5 % settle on one price pair, the rest cycle. After a one-period cut to the static best response the rival cuts too in 92 % of cases and comes back within about 10 periods. But only 0.2 % of sessions are a best response to each other on the path (the code's weaker one-period test passes 49.7 %), and in 91 % of the sessions that punish a cut some other deviation goes unpunished. Watch the strategy maps warm up from blue to orange; Inspect a cell for both firms' Q-values.",
            |_| {},
        ),
        preset(
            "collusion-code",
            "As the authors' code ran it",
            CCDP,
            "The paper's model as its authors' Fortran runs it: ties broken at random, their RAN2 generator seeded by session number (the seed), a cap of 1.25 × 10⁹, an equilibrium test that checks only one-period deviations, and Fig. 4's deviation as the code computes it. Every session is theirs exactly: the first 100 match the code built from their replication package in the same period and all 450 strategy cells. Measured (sessions 1–1 000): Table I to the digit — Δ 0.849, 50.5 % in equilibrium on the path (by the code's test), cycles of 1, 2 and 3+ periods 64.3 / 23.8 / 11.9 %; Table A5's responses — the rival's price change −0.127, deviations unprofitable in 0.936 (the text says 'more than 95 %'), punishment 5.705 periods.",
            as_coded,
        ),
        preset(
            "collusion-no-memory",
            "Algorithms that remember nothing",
            L24,
            "Lambin's 'key robustness test': firms that condition on nothing (one state), so no punishment is possible, with the future still valued (δ = 0.95). The paper's appendix only ran this with δ = 0 — its code sets δ to 0 whenever memory is 0. Lambin: memoryless algorithms 'seem to collude even more effectively'. Measured (1 000 sessions): Δ 0.958 against 0.851 with memory; mean greedy price 1.862 against 1.791 at 2 × 10⁶ periods (Lambin: about 1.85 against 1.76). No session is an equilibrium and none answers any deviation: high prices without any scheme to sustain them. With δ = 0 (the appendix's reading) Δ is 0.255.",
            |c| c.memory = 0,
        ),
        preset(
            "collusion-myopic",
            "Algorithms that ignore the future",
            CCDP,
            "δ = 0: firms value only this period's profit, so no reward–punishment scheme can pay. Measured (1 000 sessions): Δ 0.212 ± 0.003 — the paper's own Fig. 3 shows 0.212 at δ = 0 — about a quarter of the baseline's 0.851. Whatever keeps these prices up, it is not collusion; strategies can account for at most the remaining 0.64.",
            |c| c.delta = 0.0,
        ),
        preset(
            "collusion-two-phase",
            "Explore first, then never",
            L24,
            "Lambin's Theorem 1: after a phase of purely random pricing (here 1 000 periods) and none afterwards, myopic firms (δ = 0) settle where the best symmetric profit they have tried beats what their stale estimates promise — on this grid both at 1.6990 (Δ 0.707) — with or without memory. Measured (1 000 sessions, one period of memory): Δ 0.687, but only 27 % of sessions end with both at 1.6990 (the most common outcome); without memory 13 %; at δ = 0.95 the theorem's 1.7377 is reached in under 4 %. The theorem is a mean-field limit; with α = 0.15 the Q-values stay noisy.",
            |c| {
                c.exploration = Exploration::TwoPhase;
                c.delta = 0.0;
            },
        ),
        preset(
            "collusion-synchronous",
            "Learning from every price",
            AFP,
            "Asker, Fershtman and Pakes: each period a firm updates the value of every price toward what it would have earned against the rival's actual price, not only the price it charged. Measured (1 000 sessions): Δ 0.345 against 0.851 — less than half. Three sessions in four still settle on one price pair; 73 % are a best response on the path.",
            |c| c.update = Update::Synchronous,
        ),
        preset(
            "collusion-explore-more",
            "Algorithms that keep experimenting",
            CCDP,
            "β ten times smaller (4 × 10⁻⁷): the firms experiment for ten times longer before settling (about 15 million periods). Abada and Lambin argue more exploration restores competition. Measured (200 sessions): Δ 0.727 against 0.851 — lower, but not by half. A constant ε = 0.05 never settles (no session in 10⁸ periods); read at 10⁷ periods, Δ 0.559 (100 sessions).",
            |c| c.beta = 4e-7,
        ),
        preset(
            "collusion-every-price",
            "Deviations up as well as down",
            EL,
            "Epivent and Lambin's test (their Table 1): one-period deviations to every other price, up as well as down. If the price cuts that follow were punishment, a rival's price rise — an invitation to collude — should not draw one. Measured (1 000 sessions, sessions settled on a symmetric price): in all 25 (price, higher price) cells with at least 30 sessions the rival cuts its price the next period, by 9.9 % on average against 13.0 % after a cut; responses that look like punishment follow 83 % of increases and 93 % of cuts.",
            |c| c.impulse = Impulse::EveryPrice,
        ),
        preset(
            "collusion-invitation",
            "An invitation to raise prices",
            EL,
            "Epivent and Lambin's invitation (their Fig. 2): one firm raises its price a step and the rival is made to match it for the next periods; after 5 periods the first firm decides again. Measured (617 sessions settled on one price): it cuts back below where it started in 84 % of them, by 5.0 grid steps on average.",
            |c| c.impulse = Impulse::Invitation,
        ),
        preset(
            "collusion-below-nash",
            "A grid below the Nash price",
            EL,
            "Epivent and Lambin's appendix grid: 15 prices from 1.25 up to 1.47, the Nash price (1.47293). Measured (10 000 sessions): 72 % end with both firms at the top price, 5 % cycle, 22 % settle on other points (they report 53, 9 and 38 %); Δ is near 0. In the sessions below the top, price increases draw punishment-like responses more often than cuts (96 % against 85 %).",
            |c| {
                c.grid = Grid::BelowNash;
                c.impulse = Impulse::EveryPrice;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, CollusionConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Collusion(c) => (p.id, c),
                _ => panic!("{} is not a collusion preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("collusion-calvano"), CollusionConfig::default());
        let code = find("collusion-code");
        assert_eq!(
            (code.ties, code.rng, code.cap, code.equilibrium_check),
            (
                Ties::Random,
                RngKind::Calvano,
                1_250_000_000,
                EquilibriumCheck::OneShot
            )
        );
        let none = find("collusion-no-memory");
        assert_eq!((none.memory, none.delta), (0, 0.95));
        assert_eq!(find("collusion-myopic").delta, 0.0);
        assert_eq!(find("collusion-synchronous").update, Update::Synchronous);
        assert_eq!(find("collusion-below-nash").grid, Grid::BelowNash);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
        assert!(presets().iter().all(|p| !p.description.trim().is_empty()));
    }
}
