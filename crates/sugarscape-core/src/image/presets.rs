//! NS98's Figs. 1–4 and own-score strategies, LH01's Figs. 1–4, and the
//! run without the payoff offset. Descriptions quote measurements (release,
//! recorded 2026-09-26; seeds and run lengths in each).

use super::config::{ImageConfig, Information, Initial, Offset, Seeded};
use super::strategy::{Class, Strategy};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ImageConfig),
) -> ModelPreset {
    let mut c = ImageConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Image(c),
    }
}

const NS98: &str = "Nowak & Sigmund, Nature 393 (1998) 573–577";
const LH01: &str = "Leimar & Hammerstein, Proc. R. Soc. B 268 (2001) 745–753";

/// NS98 Fig. 3: observers 10, m = 10n, ν 0.001, a group of n.
fn fig_3(c: &mut ImageConfig, n: u32) {
    c.information = Information::Observers;
    c.group_size = n;
    c.rounds = 10 * n;
    c.mutation = 0.001;
}

/// LH01's island model: 100 groups of 100, m 500, c 0.25, p 0.9.
fn island(c: &mut ImageConfig) {
    c.groups = 100;
    c.rounds = 500;
    c.c = 0.25;
    c.local = 0.9;
}

fn seeded(only: Strategy, invader: Option<Strategy>, share: f64) -> Initial {
    Initial::Seeded(Seeded {
        only,
        invader,
        share,
    })
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ns-fig-1",
            "Fig. 1: cooperation wins",
            NS98,
            "NS98 Fig. 1: 100 players with k from −5 (always help) to +6 (never), drawn uniformly; in each generation 125 random donor–recipient pairs; a donor helps if the recipient's score is at least its k (cost 0.1, benefit 1, 0.1 added to both players every round), helping raises its score by one and refusing lowers it, within ±5; offspring in proportion to payoff. NS98 show one run in which k = 0 is fixed after 166 generations. Measured (seeds 1–100, run to fixation): k = 0 fixes in 20 runs (median generation 56) and some k ≤ 0 in 40 — defection wins the other 60. With m = 300, some k ≤ 0 wins 91.",
            |_| {},
        ),
        preset(
            "ns-fig-2",
            "Fig. 2: cycles under mutation",
            NS98,
            "Fig. 1 with m = 300 and mutation 0.001 (uniform over k): \"endless cycles\", with k = −4 or −5 drifting in and letting defectors back. Measured (seeds 1–10, 10⁵ generations each): 172 collapses of cooperation (k ≤ 0 falling from at least 90% to at most 10%) and 167 recoveries, 1.7 per 10,000 generations; k ≤ −4 averages 8% of cooperative populations but 68% over the 51 generations up to a collapse's last cooperative generation; cooperative strategies 67% of the time. Reproduced.",
            |c| {
                c.rounds = 300;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-3-n20",
            "Fig. 3: observers, n = 20",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 20),
        ),
        preset(
            "ns-fig-3-n50",
            "Fig. 3: observers, n = 50",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 50),
        ),
        preset(
            "ns-fig-3-n100",
            "Fig. 3: observers, n = 100",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 100),
        ),
        preset(
            "ns-fig-4a",
            "Fig. 4a: AND, perfect information",
            NS98,
            "NS98 Fig. 4a: AND strategies (help if the recipient's score is at least k and one's own is below h; k and h −5 … +6), perfect information, m = 500, mutation 0.001. NS98: 55% of interactions cooperative, (k 0, h 1) the most frequent strategy. Measured (seeds 1–10, generations 1,001–50,000): 53%, (0, 1) the most frequent (22%). Reproduced.",
            |c| {
                c.strategies = vec![Class::And];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-4b",
            "Fig. 4b: AND, observers",
            NS98,
            "NS98 Fig. 4b: AND strategies with observers, \"as in figure 3 with n = 20\" (m = 10n = 200). NS98: 57%, (k 0, h 4) the most frequent. Measured (seeds 1–10, generations 1,001–50,000): 53%, (k 0, h 5) the most frequent (11%) — a high own threshold, as NS98 argue.",
            |c| {
                fig_3(c, 20);
                c.strategies = vec![Class::And];
            },
        ),
        preset(
            "ns-fig-4c",
            "Fig. 4c: OR, perfect information",
            NS98,
            "NS98 Fig. 4c: OR strategies (help if the recipient's score is at least k or one's own is below h), perfect information, m = 500. NS98: 70%, with the defectors (k 6, h −5) the most frequent single strategy. Measured (seeds 1–10, generations 1,001–50,000): 78%; the most frequent is (k 3, h 4) (10%), the defectors second (6%).",
            |c| {
                c.strategies = vec![Class::Or];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-4d",
            "Fig. 4d: OR, observers",
            NS98,
            "NS98 Fig. 4d: OR strategies with observers (n = 20, m = 200). NS98: 80%, the defectors most frequent. Measured (seeds 1–10, generations 1,001–50,000): 85%; the most frequent is (k 2, h 5) (4.5%), the defectors (k 6, h −5) third (3.1%).",
            |c| {
                fig_3(c, 20);
                c.strategies = vec![Class::Or];
            },
        ),
        preset(
            "ns-own-only",
            "Own score only",
            NS98,
            "Strategies that consider only their own score (help while it is below h, h −5 … +6), m = 500, mutation 0.001. NS98: \"less than 0.1% cooperation\". Measured (seeds 1–10, generations 1,001–20,000): 0.19% — the mutants (5 of 12 help at a generation's start) keep it above 0.1%.",
            |c| {
                c.strategies = vec![Class::OwnOnly];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-no-offset",
            "Fig. 1 without the offset",
            NS98,
            "Fig. 1 with nothing added to payoffs: a helping donor ends the round 0.1 down, and negative payoffs weigh nothing in reproduction. (The FAIR23 NetLogo model, said to omit the offset, in fact adds c to both players every round, as LH01 say NS98 did.) Measured (seeds 1–100): k = 0 fixes in 10 runs (median generation 93) and some k ≤ 0 in 63, against 40 with the offset; with Fig. 2's settings, 78% cooperative against 67%. The offset weakens selection, and cooperation loses by it.",
            |c| c.offset = Offset::None,
        ),
        preset(
            "lh-fig-1a",
            "Fig. 1a: h = 1 invades k = 0",
            LH01,
            "LH01 Fig. 1a: 100 groups of 100 (a parent from the offspring's own group with probability 0.9), m = 500, c = 0.25, no mutation or errors; everyone plays k = 0 except 1% of each group playing h = 1 (help while one's own score is below 1, whatever the recipient's). LH01: h = 1 invades (about 80% by generation 150). Measured (seeds 1–10): 37% at generation 50, fixed in every run by 150. Reproduced, faster.",
            |c| {
                island(c);
                c.strategies = vec![Class::K, Class::H];
                c.initial = seeded(Strategy::K(0), Some(Strategy::H(1)), 0.01);
            },
        ),
        preset(
            "lh-fig-1b",
            "Fig. 1b: h = 1 invades (k 0, h 1) with errors",
            LH01,
            "LH01 Fig. 1b: the same with (k 0, h 1) and execution errors 0.05: h = 1 invades but does not wipe out (k 0, h 1). Measured (seeds 1–10): 1.8% at generation 150, 13% at 500, 42% at 1,000 — it invades, an order of magnitude more slowly than LH01 show.",
            |c| {
                island(c);
                c.strategies = vec![Class::And, Class::H];
                c.initial = seeded(Strategy::And { k: 0, h: 1 }, Some(Strategy::H(1)), 0.01);
                c.execution_error = 0.05;
            },
        ),
        preset(
            "lh-fig-2a",
            "Fig. 2a: one group",
            LH01,
            "LH01 Fig. 2a: AND strategies in one group of 100 (NS98's setting), m = 500, c = 0.25, mutation 0.001, no errors. LH01: help in 39% of rounds over 10⁶ generations, (k 0, h 1) dominant. Measured (seeds 1–10, generations 1,001–100,000): 37%, (0, 1) the most frequent (38%). Reproduced.",
            |c| {
                c.strategies = vec![Class::And];
                c.rounds = 500;
                c.c = 0.25;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-2b",
            "Fig. 2b: island model, p = 0.9",
            LH01,
            "LH01 Fig. 2b: the island model (100 groups, p = 0.9) with execution errors 0.02. LH01: with drift limited, image scoring fades — help in 9% of rounds over 10⁵ generations. Measured (seeds 1–10, generations 1,001–5,000): 44% (to 20,000: 35%, runs from 2% to 54%; 10,001–50,000: 31%). Not reproduced: cooperative AND strategies persist in most runs.",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.mutation = 0.001;
                c.execution_error = 0.02;
            },
        ),
        preset(
            "lh-fig-2c",
            "Fig. 2c: island model, p = 0.5",
            LH01,
            "LH01 Fig. 2c: stronger gene flow (p = 0.5). LH01: 2% help, \"mainly a consequence of execution errors\". Measured (seeds 1–10, generations 1,001–5,000): 15% (to 20,000: 9%). Not reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.mutation = 0.001;
                c.execution_error = 0.02;
                c.local = 0.5;
            },
        ),
        preset(
            "lh-fig-3a",
            "Fig. 3a: a small cost",
            LH01,
            "LH01 Fig. 3a: a small cost (c = 0.1), initial payoff 5, p = 0.5, execution errors 0.02. LH01: help in 45% of rounds over 2 × 10⁵ generations. Measured (seeds 1–10, generations 1,001–3,000): 52% (to 10,000: 47%).",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.c = 0.1;
                c.u0 = 5.0;
                c.local = 0.5;
                c.execution_error = 0.02;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-3b",
            "Fig. 3b: with q strategies",
            LH01,
            "LH01 Fig. 3b: Fig. 3a with LH01's q strategies (help when helping is estimated to raise the chance of being helped by more than Δq, 0.01 … 0.99, from group tallies of who was helped). LH01: help 15%, q strategies 12% of the population. Measured (seeds 1–10, generations 1,001–3,000): help 17%, q strategies 26% (to 10,000: 15% and 18%).",
            |c| {
                island(c);
                c.strategies = vec![Class::And, Class::Q];
                c.c = 0.1;
                c.u0 = 5.0;
                c.local = 0.5;
                c.execution_error = 0.02;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-4a",
            "Fig. 4a: standing invades discriminators",
            LH01,
            "LH01 Fig. 4a: binary scorers (scores 0 and −1): everyone a discriminator (k 0) but 1% of each group playing standing (help when in bad standing or when the recipient is in good standing; standing is lost only by refusing a recipient in good standing), execution errors 0.05, no mutation. LH01: standing takes over. Measured (seeds 1–10): standing 69% at generation 500, 98% at 1,000. Reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.initial = seeded(Strategy::Binary(0), Some(Strategy::Standing), 0.01);
                c.execution_error = 0.05;
            },
        ),
        preset(
            "lh-fig-4b",
            "Fig. 4b: … with perception errors",
            LH01,
            "LH01 Fig. 4b: Fig. 4a with execution and perception errors 0.025 (each member keeps its own view of the others' scores and standing). LH01: standing still invades. Measured (seeds 1–5): 35% at generation 500, 75% at 1,000. Reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.initial = seeded(Strategy::Binary(0), Some(Strategy::Standing), 0.01);
                c.execution_error = 0.025;
                c.perception_error = 0.025;
            },
        ),
        preset(
            "lh-fig-4c",
            "Fig. 4c: standing in the long run",
            LH01,
            "LH01 Fig. 4c: cooperators, discriminators, defectors and standing from a uniform start, errors 0.025, mutation 0.0001. LH01 (10⁵ generations): standing dominates, cooperators stay appreciable, discriminators occasionally reach 5%, defectors below 1% — though their condition vrb < c < rb fails here (v = 0.5, r = 0.833). Measured (seeds 1–3, generations 1,001–1,500): standing 53%, cooperators 39%, discriminators 8%, defectors 0.02% (seeds 1–10 to 3,000: 77%, 20%, 3%, 0.01%).",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.execution_error = 0.025;
                c.perception_error = 0.025;
                c.mutation = 0.0001;
            },
        ),
    ]
}
