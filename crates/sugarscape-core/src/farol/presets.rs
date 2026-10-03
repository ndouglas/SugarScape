//! Arthur's bar, Challet and Zhang's minority game and its variants, Savit,
//! Manuca and Riolo's three phases, and Challet, Marsili and Ottino's critique.

use super::config::{Behavior, Evolution, FarolConfig, Game, MixedMemory, Payoff, Scoring};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const ARTHUR: &str = "Arthur 1994, AER 84(2)";
const CZ: &str = "Challet & Zhang 1997, Physica A 246";
const SMR: &str = "Savit, Manuca & Riolo 1999, PRL 82";
const CMO: &str = "Challet, Marsili & Ottino 2004, Physica A 332";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut FarolConfig),
) -> ModelPreset {
    let mut c = FarolConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Farol(c),
    }
}

/// The plain minority game: N agents, S strategies, memory M, capacity (N − 1)/2.
fn minority(c: &mut FarolConfig, n: u32, s: u32, m: u32) {
    c.game = Game::Minority;
    c.agents = n;
    c.strategies = s;
    c.memory = m;
    c.capacity = None;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ef-arthur",
            "The bar problem, k = 12",
            ARTHUR,
            "Arthur-inspired bar problem (48 predictors, absolute-error decay 0.9, random reties each round): 100 people decide each week whether to go to a bar that is fun only if fewer than 60 come. Each holds 12 of 48 simple forecasts of next week's attendance from past weeks (the same as some week ago, a mirror image, an average, a trend) and acts on the one that has lately been most accurate, going if it forecasts fewer than 60. Arthur: attendance 'converges always to 60', 'no persistent cycles', and about 40 % of the forecasts in use are above 60. Measured (20 seeds, rounds 401–2 000): mean attendance 59.1 — but its attendance variance is 29 times the coin-flippers' (σ²/N 6.9 against 0.24); a lag-1 autocorrelation of −0.43, antipersistence, which alone does not establish persistent cycles; 32 % of forecasts in use above 60; forecasts exactly at 60 explain the gap from the stay-home share. The time panel shows the swings; the histogram, how far they reach.",
            |_| {},
        ),
        preset(
            "ef-payoff",
            "Predictors rated by payoff",
            CMO,
            "The same bar with CMO-inspired payoff scoring (our equality convention differs from their Θ(0) = 1): by whether their advice (go or stay) was right, not by how close their number came — 'if A(t) = 59, a prediction of 5 is better than a prediction of 61'. Measured (20 seeds, rounds 401–2 000): mean attendance 59.9; lag-1 autocorrelation is small (−0.06); attendance variance shrinks to σ²/N 2.7 — still 11 times the coin-flippers'. Compare it with ef-arthur from the presets menu.",
            |c| {
                c.scoring = Scoring::Payoff;
            },
        ),
        preset(
            "ef-random",
            "Zero-intelligence agents",
            CMO,
            "Challet, Marsili and Ottino's point: agents that simply go with probability 0.6, with no forecasting at all, also average 60 — Arthur's convergence is 'trivial'. What matters is how far attendance strays from 60. Measured (20 seeds): mean 60.0, σ²/N 0.239, the binomial's 0.24. Every other El Farol preset is measured against this.",
            |c| {
                c.behavior = Behavior::Random;
            },
        ),
        preset(
            "ef-shared",
            "Everyone shares the library",
            ARTHUR,
            "Arthur: 'The reader might ponder what would happen if all agents shared the same set of predictors.' Here everyone holds all 48, so a unique best predictor synchronizes choices; random reties can select different advice. Measured (20 seeds): mean attendance 49.8, alternating near 0 and near 100 (σ²/N 26), and success is very low in these runs. Sharing this library greatly increases attendance variance.",
            |c| {
                c.shared = true;
            },
        ),
        preset("mg-m6", "Fig. 1a: M = 6", CZ, "Challet and Zhang's minority game: 1001 players each choose side A or B, and those on the smaller side win a point. Each holds 5 strategies — tables from the last M winning sides to a choice — and plays the one that would have won most so far. With memories of 6 (their Fig. 1a) the crowd herds. Measured (10 seeds, rounds 1 001–5 000): σ²/N 6.9 against coin-flippers' 0.25; players win 44 % of rounds.", |c| {
            minority(c, 1001, 5, 6)
        }),
        preset("mg-m8", "Fig. 1b: M = 8", CZ, "The same game with memories of 8 (Fig. 1b). Measured (10 seeds, rounds 1 001–5 000): σ²/N 2.1; players win 46 %. Longer memories, smaller swings — Challet and Zhang's 'decreasing order for ever increasingly intelligent players'.", |c| {
            minority(c, 1001, 5, 8)
        }),
        preset("mg-m10", "Fig. 1c: M = 10", CZ, "Memories of 10 (Fig. 1c). Measured (10 seeds, rounds 1 001–5 000): σ²/N 0.38, near coin-flippers' 0.25; players win 48 %. With 5 strategies each the fluctuations keep falling through M = 10, as the paper says; with 2 they would already be rising again (the mg-fig-1 sweep).", |c| {
            minority(c, 1001, 5, 10)
        }),
        preset(
            "mg-mixed",
            "Fig. 2: mixed memories",
            CZ,
            "Players with memories from 1 to 10 in one game (Fig. 2). Challet and Zhang: longer memories do better, and 'above a certain size (M ≈ 6) the average performance … appears to saturate'. Measured (10 seeds, 5 000 rounds): wins per round 0.35, 0.39, 0.45, 0.48, 0.49, 0.50, 0.50, 0.50, 0.50, 0.50 for memories 1 to 10 — exactly that. Color the agents by Memory, then by Gain.",
            |c| {
                minority(c, 1001, 5, 10);
                c.mixed_memory = MixedMemory {
                    enabled: true,
                    min: 1,
                    max: 10,
                };
            },
        ),
        preset(
            "mg-inverse",
            "Fig. 4: payoff N/x − 2",
            CZ,
            "Winners get N/x − 2 points for x winners, rounded as the paper says ('these many (nearest integer values) points'): a small minority pays well, a near-even one pays nothing. Challet and Zhang (Fig. 4): attendance splits into two peaks. Measured (10 seeds, random reties each round): a narrow central distribution (σ²/N 0.25): with an even split paying round(2.0) − 2 = 0, strategies almost never score, their points stay tied, and players choose at random. The unrounded experiment has different fluctuations. Central mass alone does not establish histogram shape or a failure of Fig. 4 under its unspecified tie choices.",
            |c| {
                minority(c, 1001, 5, 4);
                c.payoff = Payoff::Inverse;
            },
        ),
        preset(
            "mg-evolution",
            "Fig. 9: Darwinian selection",
            CZ,
            "Challet and Zhang's Darwinism (Fig. 9): every 10 rounds the player with the least winnings since the last replacement is replaced by a copy of the best (its strategies, its scores reset), and one of the copy's strategies is redrawn with probability 0.1. They give neither the interval nor the rate. Measured (5 seeds): σ²/N falls from 5.6 (rounds 1 000–2 000) to 2.1 (38 000–40 000): the population learns, as they say.",
            |c| {
                minority(c, 1001, 5, 6);
                c.evolution = Evolution {
                    enabled: true,
                    every: 10,
                    strategy_mutation: 0.1,
                    memory_mutation: 0.0,
                };
            },
        ),
        preset(
            "mg-inbred",
            "Fig. 10: cloning without mutation",
            CZ,
            "Our rolling-window selection with perfect copies and no mutation, whereas Fig. 10 illustrates a pure population where Challet and Zhang see 'tremendous waste' once the population is all clones of one player. Measured (5 seeds): σ²/N falls from 5.9 to 2.3 by 40 000 rounds, much as with mutation (2.1); with 101 players, 0.30 against 0.24. Population purity is not measured, so these runs do not test the illustrated monospecies case. A separate homogeneous-population audit probe confirms extreme waste under both tie rules. CZ98 later reports diversity above N/2 even without mutation.",
            |c| {
                minority(c, 1001, 5, 6);
                c.evolution = Evolution {
                    enabled: true,
                    every: 10,
                    strategy_mutation: 0.0,
                    memory_mutation: 0.0,
                };
            },
        ),
        preset(
            "mg-arms-race",
            "Fig. 11: memory evolves",
            CZ,
            "101 players starting with memories of 2, the copies' memories moving up or down by one with probability 0.1 (Fig. 11). Challet and Zhang: an 'arm race' in memory that levels off, at a level depending on how often players are replaced. Measured (5 seeds): mean memory 2.0, 2.6, 3.1, 3.2 at rounds 1 000, 5 000, 10 000, 20 000, then about 3 through 100 000, while σ²/N falls from 3.4 to 0.05. Color the agents by Memory.",
            |c| {
                minority(c, 101, 5, 2);
                c.evolution = Evolution {
                    enabled: true,
                    every: 50,
                    strategy_mutation: 0.1,
                    memory_mutation: 0.1,
                };
            },
        ),
        preset(
            "mg-crowded",
            "m = 2: the crowded phase",
            SMR,
            "Savit, Manuca and Riolo's first phase: 101 players with 2 strategies and memories of 2. With few possible histories and many players, the crowd moves together: σ²/N 1.37, more than five times coin-flippers' 0.25 (10 seeds); nobody beats chance.",
            |c| minority(c, 101, 2, 2),
        ),
        preset(
            "mg-critical",
            "m = 6: near the transition",
            SMR,
            "The transition: memories of 6 for 101 players (2^6/101 ≈ 0.63). Measured (10 seeds): σ²/N 0.068, a quarter of coin-flippers' — the best coordination the game allows; the mg-memory sweep shows the minimum shifting one memory step per doubling of N.",
            |c| minority(c, 101, 2, 6),
        ),
        preset(
            "mg-random-like",
            "m = 12: the random phase",
            SMR,
            "Memories of 12 for 101 players: more histories than the players can learn from. Measured (10 seeds): σ²/N 0.240, the same as coin-flippers' 0.25.",
            |c| minority(c, 101, 2, 12),
        ),
        preset(
            "cmo-binary",
            "The binary El Farol, m = 2",
            CMO,
            "A CMO-inspired binary El Farol variant (step-win scoring, rather than their Eq. 4 linear update): 120 agents, 60 seats, strategies reading only whether each of the last two weeks was crowded, each entry 'go' with probability 0.5 — so, on average, agents want 60 seats. Measured (10 seeds): mean attendance 60.4, but σ²/N 1.67 against coin-flippers' 0.25 — qualitative support for poorer coordination near āN = L in this variant. The cmo-bias sweep shows a small bias, or a longer memory, bringing the swings down.",
            |c| {
                minority(c, 120, 2, 2);
                c.capacity = Some(60);
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, FarolConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Farol(c) => (p.id, c),
                _ => panic!("{} is not an El Farol preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("ef-arthur"), FarolConfig::default());
        assert_eq!(find("ef-payoff").scoring, Scoring::Payoff);
        let m = find("mg-m10");
        assert_eq!(
            (m.game, m.agents, m.strategies, m.memory, m.capacity()),
            (Game::Minority, 1001, 5, 10, 500)
        );
        assert!(m.plain_minority());
        let a = find("mg-arms-race");
        assert_eq!(
            (a.memory, a.evolution.every, a.evolution.memory_mutation),
            (2, 50, 0.1)
        );
        let b = find("cmo-binary");
        assert_eq!(
            (b.agents, b.capacity(), b.memory, b.bias),
            (120, 60, 2, 0.5)
        );
        assert!(!b.plain_minority());
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
