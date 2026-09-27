//! The papers' runs, the replication's and the reply's readings, and the
//! network studies.

use super::config::{
    AgreementConfig, LatticeConfig, Network, PairUpdate, Pairing, Placement, Rule, ScaleFreeConfig,
    SmallWorldConfig, Substrate, Window,
};
use crate::model::ModelConfig;
use crate::opinions::Neighborhood;
use crate::presets::ModelPreset;

const DNAW: &str = "Deffuant, Neau, Amblard & Weisbuch 2000";
const DAWF: &str = "Deffuant, Amblard, Weisbuch & Faure 2002";
const MC: &str = "Meadows & Cliff 2012";
const DAW: &str = "Deffuant, Amblard & Weisbuch 2013";
const AD: &str = "Amblard & Deffuant 2004";
const W: &str = "Weisbuch 2004";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut AgreementConfig),
) -> ModelPreset {
    let mut c = AgreementConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Agreement(c),
    }
}

/// DNAW's pairwise model: bounded confidence, no extremists, d on [0, 1]
/// as U = 2d here.
fn dnaw(c: &mut AgreementConfig, d: f64, mu: f64) {
    c.rule = Rule::Bc;
    c.extremists = 0.0;
    c.uncertainty = 2.0 * d;
    c.mu = mu;
}

/// DAWF's Figs. 5–8: N 200, μ 0.5.
fn regime(c: &mut AgreementConfig, pe: f64, u: f64) {
    c.agents = 200;
    c.mu = 0.5;
    c.extremists = pe;
    c.uncertainty = u;
}

/// Fig. 9's μ and ue at pe 0.05, U 1.4, N 200.
fn literal(c: &mut AgreementConfig) {
    c.extremists = 0.05;
    c.uncertainty = 1.4;
}

/// A reading that stops at a fixed period.
fn reading(c: &mut AgreementConfig, margin: f64, stop_at: u32) {
    literal(c);
    c.placement = Placement::Band;
    c.extreme_margin = margin;
    c.stop_when_stable = false;
    c.stop_at = stop_at;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "dnaw-consensus",
            "Pairs meet: consensus",
            DNAW,
            "Deffuant, Neau, Amblard and Weisbuch's pairwise bounded confidence: a thousand agents with opinions spread evenly; each period every agent meets, on average twice, a random partner, and when their opinions differ by less than d both move toward each other by μ = 0.5 of the gap. With d = 0.5 (Fig. 1) the population agrees. Opinions run from −1 to 1 here, so d = 0.5 on the paper's [0, 1] is an uncertainty of 1.0. Lines are agents, colored by uncertainty (here all the same); the right panel plots each agent's start against its opinion now. Measured (20 seeds): consensus in every run, still by period 32 (median).",
            |c| {
                c.agents = 1000;
                dnaw(c, 0.5, 0.5);
            },
        ),
        preset(
            "dnaw-clusters",
            "Pairs meet: two clusters",
            DNAW,
            "The same meetings with d = 0.2 (Fig. 2, uncertainty 0.4 here): two clusters. Fig. 4 counts peaks falling as d grows, about 1/(2d) at most (2.5 here). Measured (20 seeds): two major clusters in every run, holding about half each, plus a median of 3 stragglers near the ends (the paper's 'wings'); still by period 73.",
            |c| {
                c.agents = 1000;
                dnaw(c, 0.2, 0.5);
            },
        ),
        preset(
            "dnaw-lattice",
            "On a lattice: one cluster and stragglers",
            DNAW,
            "The same meetings on a 29 × 29 torus, each agent meeting only its four neighbors, d = 0.3, μ = 0.3 (Fig. 5): 'a large majority … reached consensus … apart from isolated agents which have extremist opinions'. The torus is drawn at the right. The caption's '100 000 iterations' are 119 periods of 841 meetings. Measured (20 seeds): at period 119 the largest cluster (opinions within 10⁻³) holds only 41 % — the lattice is still settling; run to stability (median period 1 797) it holds 92 %, with 27 isolated agents: the figure's picture, but much later than its caption says.",
            |c| {
                dnaw(c, 0.3, 0.3);
                c.network = Network::Lattice;
            },
        ),
        preset(
            "dnaw-lattice-clusters",
            "On a lattice: many local clusters",
            DNAW,
            "The lattice with d = 0.15 (Fig. 6): one percolating cluster and 'smaller non-percolating clusters with similar but not equal opinions'. Measured (20 seeds, to stability, median period 3 963; one run reached the 20 000-period cap): a median of 16.5 clusters holding 1 % of agents or more, 141 isolated agents, and a largest cluster of 6 % — similar opinions, split across many local clusters.",
            |c| {
                dnaw(c, 0.15, 0.3);
                c.network = Network::Lattice;
            },
        ),
        preset(
            "ra-uniform",
            "Relative agreement, no extremists",
            DAWF,
            "Deffuant, Amblard, Weisbuch and Faure's relative agreement: each agent has an opinion and an uncertainty (a segment around the opinion); when two meet, each moves the other by μ times the overlap of their segments beyond half the influencer's width, divided by that width — so confident agents sway uncertain ones, and uncertainties move too. Without extremists (Fig. 3: 200 agents, uncertainty 0.4) the clusters number about w/2u = 2.5 (Fig. 4). Lines are colored from confident red to uncertain green. Measured (50 seeds): a median of 3 clusters (2 to 6), still by period 96.",
            |c| {
                regime(c, 0.0, 0.4);
            },
        ),
        preset(
            "ra-central",
            "Extremists: central convergence",
            DAWF,
            "Extremists: the 20 % most extreme opinions (10 % at each end) start confident (uncertainty 0.1), the rest at 0.4; μ = 0.5, 200 agents (Fig. 5). The paper: central convergence, 'only a marginal part of the initially non-extremists became extremist (4%)'. y (DAWF's indicator) is the sum of the squared shares of moderates that end up extremists at each end: 0 central, 0.5 both extremes, 1 a single extreme. Measured (40 seeds): 23 % of moderates per side end past the innermost extremist less 0.1 (y 0.12) — everyone within the extremists' reach joins them; the caption's 4 % does not reproduce (11 % per side with extremists set to ±1). The majority does stay central.",
            |c| {
                regime(c, 0.2, 0.4);
            },
        ),
        preset(
            "ra-both",
            "Extremists: both extremes",
            DAWF,
            "Fig. 6's extremists: a quarter of the agents, moderates much less sure (uncertainty 1.2), μ = 0.5. The paper: the moderates split between the two extremes (43 % and 56 %; y 0.49). Measured (20 seeds): both extremes in every run, y 0.51.",
            |c| {
                regime(c, 0.25, 1.2);
            },
        ),
        preset(
            "ra-single",
            "Extremists: a single extreme",
            DAWF,
            "Fig. 7's setup: 10 % extremists, moderates at uncertainty 1.4, μ = 0.5, 200 agents. The paper's run ends with 98.33 % of moderates at one extreme (y 0.97), and Fig. 8, 'for the same parameters', stays central — the instability the paper is about. Measured: at the stated μ = 0.5 both extremes in 39 of 40 runs (y 0.53), under every placement and update order tried; single and central appear only at Fig. 9's μ = 0.2 (10 and 10 of 20 runs), as §4.8 predicts ('when the intensity of interactions (μ) increases, the both extremes convergence zone increases'). The figures' μ looks misstated.",
            |c| {
                regime(c, 0.1, 1.4);
            },
        ),
        preset(
            "ra-literal",
            "Fig. 9's corner, as the paper states it",
            DAWF,
            "Fig. 9's corner, as the paper states it: μ = 0.2, extremists' uncertainty 0.1, 5 % extremists drawn as the most extreme of 200 uniform opinions, moderates at 1.4, run 'until … invariant opinions and uncertainties', a moderate counted as extremist once past the innermost extremist less 0.1. Meadows and Cliff (2012) could not reproduce Fig. 9 here; this reading does. Measured (20 seeds): a single extreme in 19 runs (y 0.95), still by period 270 (median).",
            literal,
        ),
        preset(
            "ra-meadows-cliff",
            "Meadows and Cliff's reading",
            MC,
            "Meadows and Cliff's reimplementation of Fig. 9's corner: moderates uniform on (−0.8, 0.8), extremists uniform in [0.8, 1] and [−1, −0.8], y measured after 200 periods of 200 meetings (40 000 meetings) with a moderate counted as extremist only past ±0.8. Their conclusion: 'no conditions under which single extreme convergence will occur in the majority of the simulations'. Measured (20 seeds): y 0.00 in every run — yet the mean opinion has already drifted to ±0.56: the majority is on its way to one extreme but short of 0.8 at period 200. Compare with ra-deffuant-2013.",
            |c| {
                reading(c, 0.0, 200);
            },
        ),
        preset(
            "ra-deffuant-2013",
            "The authors' reply",
            DAW,
            "Deffuant, Amblard and Weisbuch's reply (2013): Meadows and Cliff 'compute indicator y before model convergence'; with 1 200 periods (240 000 meetings) and new extremists counted past ±0.7 ('a threshold … lower of 0.1 than the threshold for initial extremists') their own program gives Fig. 9. Same placement as ra-meadows-cliff. Measured (20 seeds): a single extreme in 19 runs (y 0.95). Neither fix is enough alone (the ra-readings sweep): the drifted majority settles between 0.7 and 0.8.",
            |c| {
                reading(c, 0.1, 1200);
            },
        ),
        preset(
            "ra-bc-extremists",
            "Bounded confidence with extremists",
            DAWF,
            "§6's plain bounded confidence with extremists (Fig. 20): 1000 agents, 5 % extremists at uncertainty 0.1, moderates at 1.0, μ = 0.2; an agent moves toward a partner within its window. With the window read as the listener's own uncertainty the paper's claims hold: single extreme 'limited to a zone of parameters around … U = 1'. Measured (20 seeds): a single extreme in every run (y 1.00). Eq. 11 as printed uses the influencer's uncertainty instead — see ra-bc-printed.",
            |c| {
                c.agents = 1000;
                c.rule = Rule::Bc;
                c.window = Window::Listener;
                c.extremists = 0.05;
                c.uncertainty = 1.0;
            },
        ),
        preset(
            "ra-bc-printed",
            "Bounded confidence, eq. 11 as printed",
            DAWF,
            "The same society with eq. 11 as printed: 'If |x − x′| < u′ the influence of x′ on x …', u′ being the influencer's uncertainty. Then the confident extremists move toward every uncertain moderate who meets them, and the moderates ignore them. Measured (20 seeds): consensus near the center in every run, y 0.00 — and 0.00 over the whole of Fig. 20's grid (the ra-rules sweep). §6's results need the listener's window.",
            |c| {
                c.agents = 1000;
                c.rule = Rule::Bc;
                c.window = Window::Influencer;
                c.extremists = 0.05;
                c.uncertainty = 1.0;
            },
        ),
        preset(
            "ad-moore",
            "Extremists on a Moore lattice",
            AD,
            "Amblard and Deffuant's lattice (Fig. 3c): a 30 × 30 torus with eight neighbors each, 20 % extremists set to ±1, moderates at uncertainty 1.4, μ = 0.2. 'The single extreme convergence never occurs': extremism spreads from each extremist through its neighborhood until it meets the other side's. Measured (20 seeds): both extremes in every run (y 0.49); settling is slow (median period 18 475; some runs reach the 20 000-period cap).",
            |c| {
                c.network = Network::Lattice;
                c.lattice = LatticeConfig {
                    width: 30,
                    height: 30,
                    neighborhood: Neighborhood::Moore,
                };
                c.extremists = 0.2;
                c.uncertainty = 1.4;
                c.placement = Placement::Bounds;
            },
        ),
        preset(
            "ad-small-world",
            "Extremists in a small world",
            AD,
            "Amblard and Deffuant's small world (Figs. 4–5): 1000 agents on a ring, each linked to 32 neighbors, 80 % of links rewired at random; 5 % extremists at ±1, moderates at 1.8, μ = 0.1. Fully connected, these parameters give a single extreme; on sparse rings the paper reports both extremes, which appear here only when moderates are counted as extremists past 0.7 (the ad-connectivity sweep) — with this preset's cutoff at 0.9 they read as central. Measured (20 seeds): a single extreme in 11 runs, central in 9 — k = 32 is near the transition (the ad-connectivity sweep).",
            |c| {
                c.agents = 1000;
                c.mu = 0.1;
                c.uncertainty = 1.8;
                c.extremists = 0.05;
                c.placement = Placement::Bounds;
                c.network = Network::SmallWorld;
                c.small_world = SmallWorldConfig {
                    substrate: Substrate::Ring,
                    degree: 32,
                    rewire: 0.8,
                };
            },
        ),
        preset(
            "w-scale-free",
            "A scale-free network",
            W,
            "Weisbuch's scale-free network (Fig. 4): 900 agents grown by preferential attachment, two links each, pairwise bounded confidence with d = 0.2 (uncertainty 0.4 here), μ = 0.5; a random agent picks a random neighbor and only the first moves. Hubs are influential without being more influenced. Measured (20 seeds): two major clusters, but 116 agents (median) left isolated — 15 % never moved at all, Weisbuch's 'outlying' nodes; still by period 1 331 (median).",
            |c| {
                c.agents = 900;
                dnaw(c, 0.2, 0.5);
                c.network = Network::ScaleFree;
                c.scale_free = ScaleFreeConfig { links: 2 };
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, AgreementConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Agreement(c) => (p.id, c),
                _ => panic!("{} is not a relative agreement preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        let d = find("dnaw-clusters");
        assert_eq!(
            (d.rule, d.extremists, d.uncertainty, d.mu, d.agents),
            (Rule::Bc, 0.0, 0.4, 0.5, 1000)
        );
        assert_eq!(find("dnaw-lattice").population(), 841);
        let s = find("ra-single");
        assert_eq!(
            (s.agents, s.mu, s.extremists, s.uncertainty),
            (200, 0.5, 0.1, 1.4)
        );
        let lit = find("ra-literal");
        assert_eq!(
            (lit.mu, lit.extremists, lit.uncertainty, lit.placement),
            (0.2, 0.05, 1.4, Placement::Drawn)
        );
        let mc = find("ra-meadows-cliff");
        assert_eq!(
            (
                mc.placement,
                mc.extreme_margin,
                mc.stop_when_stable,
                mc.stop_at
            ),
            (Placement::Band, 0.0, false, 200)
        );
        let daw = find("ra-deffuant-2013");
        assert_eq!((daw.extreme_margin, daw.stop_at), (0.1, 1200));
        assert_eq!(find("ra-bc-extremists").window, Window::Listener);
        assert_eq!(find("ra-bc-printed").window, Window::Influencer);
        let sw = find("ad-small-world");
        assert_eq!((sw.small_world.degree, sw.small_world.rewire), (32, 0.8));
        let w = find("w-scale-free");
        assert_eq!(
            (w.network, w.pairing, w.pair_update, w.uncertainty),
            (Network::ScaleFree, Pairing::Node, PairUpdate::OneWay, 0.4)
        );
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
