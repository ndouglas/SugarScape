//! Epstein's runs, the working paper's rule, soup, the shifted payoffs and
//! metabolism, footnote 27, Radax & Rengs' best fit and the coordination
//! game. Descriptions quote measurements (release, seeds 1–30, the value at
//! cycle 500 unless noted, mean ± s.d. (range), recorded 2026-09-26).

use super::config::{DpdConfig, EndowmentFrom, NewbornsAct, Pairing, Play, Removal};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut DpdConfig),
) -> ModelPreset {
    let mut c = DpdConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Dpd(c),
    }
}

const GSS: &str = "Epstein, Generative Social Science (2006), ch. 9";
const WP: &str = "Epstein, SFI Working Paper 97-12-094 (1997)";
const APPENDIX: &str = "Epstein, Generative Social Science (2006), ch. 9 appendix";
const RR: &str = "Radax & Rengs 2009, MPRA 14419";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "dpd-run-1",
            "Run 1: no maximum age",
            GSS,
            "Epstein's Table 9.1: 100 agents with random fixed strategies on a 30 × 30 torus, wealth 6 (the CD's). In turn each moves to a random unoccupied neighbouring site, plays each neighbour (T 6, R 5, P −5, S −6), clones onto an empty neighbouring site once its wealth reaches 11 (giving the offspring 6), and dies if its wealth goes negative. Table 1: 779 ± 15 cooperators, 121 ± 15 defectors. Measured: 729 ± 17 (698–778) and 171 ± 17 (122–201) — cooperation dominates, but 50 fewer cooperators than Epstein (t = 11.9: not reproduced), and about 4.3 to 1 by t = 50, not 5 to 1.",
            |_| {},
        ),
        preset(
            "dpd-run-2",
            "Run 2: maximum age 100",
            GSS,
            "Run 1 with a maximum age of 100 (initial and newborn ages random in 1–100). Table 2: 784 ± 29 cooperators, 99 ± 25 defectors. Measured: 695 ± 29 (628–741) and 196 ± 28 (152–263) — twice Epstein's defectors (not reproduced). Repeating Radax & Rengs' factorial of the six timing switches here, one setting of 64 matches (removal at once, death on the agent's own turn, a full shuffle): 785 ± 23 / 110 ± 22, not one of theirs.",
            |c| c.max_age = 100,
        ),
        preset(
            "dpd-run-3",
            "Run 3: R = 2",
            GSS,
            "Run 2 with the reward for mutual cooperation cut from 5 to 2: \"cooperators do worse … defectors do better, and the oscillations are now more evident.\" Measured: 339 ± 162 cooperators, 209 ± 98 defectors; 5 of 30 populations die out by cycle 500 (Radax & Rengs saw 250–450 cooperators and about 200 defectors).",
            |c| {
                c.max_age = 100;
                c.r = 2.0;
            },
        ),
        preset(
            "dpd-run-4",
            "Run 4: R = 1",
            GSS,
            "R cut to 1: Epstein describes predator–prey cycles of cooperative zones, with outcomes that differ by seed — coexistence, cooperator monopoly or extinction — and cooperators that \"ultimately do better with a low payoff (R = 1) than with a high one (R = 5)\". Measured: 26 of 30 populations are extinct by cycle 500 and all 30 by 2,000, after at most two swings (cooperators above 400 then below 100; 0.6 per run); no cooperator monopoly at R = 1 (and none at R = 5, where all 30 coexist). Not reproduced.",
            |c| {
                c.max_age = 100;
                c.r = 1.0;
            },
        ),
        preset(
            "dpd-run-5",
            "Run 5: 50% mutation",
            GSS,
            "Run 2 with a 50% chance that an offspring takes the other strategy: cooperation \"is intact through 10 thousand cycles\". Measured: 268 ± 87 cooperators and 306 ± 91 defectors at cycle 500; cooperators persist through 10,000 cycles in 27 of 30 runs (the other three populations die out entirely), averaging 260 against 295 defectors over cycles 5,001–10,000. At 25% mutation (Epstein: about 350 and 400): 420 and 393.",
            |c| {
                c.max_age = 100;
                c.mutation = 0.5;
            },
        ),
        preset(
            "dpd-working-paper",
            "The working paper's rule",
            WP,
            "The 1997 working paper's rule: \"Choose a random site within your vision; go there and play your strategy against a random neighbor\" — one game a turn, where the published text plays each neighbour. On Run 1's settings: 759 ± 18 cooperators, 141 ± 18 defectors — nearer Table 1 (779/121) than the published rule (729/171), but still rejected (t = 4.8); on Run 2's: 737 ± 21 and 156 ± 21.",
            |c| c.play = Play::RandomNeighbor,
        ),
        preset(
            "dpd-closest",
            "Closest to Tables 1 and 2",
            WP,
            "Three choices no source settles, changed from the defaults: the working paper's one random neighbour a turn, initial agents with no wealth (the prose never gives one; 6 is the 2006 CD's), and newborns acting in the cycle they are born. On Run 1's settings: 786 ± 21 cooperators, 114 ± 21 defectors (Table 1: 779 ± 15, 121 ± 15; t = −1.5); with a maximum age of 100, 792 ± 24 and 101 ± 23 (Table 2: 784 ± 29, 99 ± 25; t = −1.2) — both tables at once, which none of Radax & Rengs' 64 timing settings does with the published rule. It is fragile: over seeds 31–60 and 61–90 Run 1 still holds but Run 2's defectors come out 116 and 121 (t = −2.6, −3.2); of 512 combinations of these three choices with the six timing switches, 46 reproduce Run 1 (every one with no initial wealth), 24 Run 2 and 7 both.",
            |c| {
                c.play = Play::RandomNeighbor;
                c.initial_wealth = 0.0;
                c.newborns_act = NewbornsAct::ThisCycle;
            },
        ),
        preset(
            "dpd-soup",
            "Soup",
            GSS,
            "\"If we replace space and local interactions with soup (equiprobable random agent pairings), then the system runs to pure defection.\" Each agent moves to a random empty site anywhere, plays one random other agent and places offspring anywhere. Measured: the last cooperator dies by cycle 8 (4–14) in 29 of 30 runs; the defectors then kill one another, leaving a lone agent or nobody by cycle 500 — reproduced.",
            |c| c.pairing = Pairing::Soup,
        ),
        preset(
            "dpd-shifted",
            "Payoffs shifted by 6",
            GSS,
            "GSS: \"with maximum age of 100, zero mutation, and all payoffs shifted up by 6, so that T = 12, R = 11, P = 1, and S = 0, the spatial system again converges to pure defection\" (Fig. 9.13). Measured: no run converges — all 30 coexist at cycle 500 (418 ± 78 cooperators, 478 ± 77 defectors) and at 2,000. With nothing negative only old age kills, the lattice stays full and everyone can afford to clone, so births go to whoever finds an empty site. Run 5's 50% mutation (the working paper's context) or Run 1's unlimited lives do not converge either. Not reproduced.",
            shifted,
        ),
        preset(
            "dpd-metabolism",
            "Shifted payoffs, metabolism 6",
            GSS,
            "Epstein: the shifted payoffs with a metabolism of 6 are \"equivalent mathematically\" to the negative payoffs, metabolism being \"a fixed decrement to accumulated payoff per cycle\". Charged per cycle (this preset): 644 ± 97 cooperators, 253 ± 97 defectors against Run 2's 695 and 196 — cooperative, but another model. Charged per game (metabolism_per: interaction, the text's \"after every interaction\"), the run is Run 2 exactly, to the fingerprint.",
            |c| {
                shifted(c);
                c.metabolism = 6.0;
            },
        ),
        preset(
            "dpd-footnote-27",
            "Footnote 27",
            GSS,
            "Footnote 27: \"with all payoffs hiked by ten (so that T = 16, R = 11, P = 5, S = 4) and the maximum lifetime reduced from 100 to 10 cycles … we again see an evolution to cooperative monopoly.\" (Hiked by ten, R would be 15.) Measured: all 30 runs coexist at cycle 500 (385 ± 158 cooperators, 485 ± 158 defectors); by 2,000 one run of 30 is a cooperative monopoly. R = 15 gives the same. Not reproduced.",
            |c| {
                (c.t, c.r, c.p, c.s) = (16.0, 11.0, 5.0, 4.0);
                c.max_age = 10;
            },
        ),
        preset(
            "dpd-rr-best",
            "Radax & Rengs' best fit",
            RR,
            "Radax & Rengs' best fit to Table 2 (their Repast replication: 780 ± 25 cooperators, 97 ± 22 defectors): the dead leave at the end of the cycle, death as wealth goes negative, the endowment granted rather than taken from the parent, random newborn ages, asynchronous updating, Epstein's swaps. Here: 703 ± 26 and 164 ± 23 (t = 11.3 against Table 2) — their best fit does not carry over to this implementation of the same switches.",
            |c| {
                c.max_age = 100;
                c.removal = Removal::EndOfCycle;
                c.endowment_from = EndowmentFrom::Granted;
            },
        ),
        preset(
            "dpd-coordination",
            "The coordination game",
            APPENDIX,
            "GSS's appendix: the same demography with the CD's coordination payoffs [1, −3, −3, 1] (two conventions — say, driving on the left or the right — paying 1 when neighbours match and −3 when they do not), maximum age 1,000, no mutation: persistent \"norm maps\", regions of each convention with accidents at their borders. Measured: both conventions persist in 17 of 30 runs at cycle 500, 16 at 2,000 and 11 at 5,000 (the rest settle on one); where both persist 3.6% of neighbouring pairs differ, against 35% if mixed at random.",
            |c| {
                (c.r, c.s, c.t, c.p) = (1.0, -3.0, -3.0, 1.0);
                c.max_age = 1000;
            },
        ),
    ]
}

/// GSS p. 216: "maximum age of 100, zero mutation, and all payoffs shifted
/// up by 6" (12, 11, 1, 0).
fn shifted(c: &mut DpdConfig) {
    c.max_age = 100;
    (c.t, c.r, c.p, c.s) = (12.0, 11.0, 1.0, 0.0);
}
