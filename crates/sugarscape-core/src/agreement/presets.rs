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
const DAW: &str = "Deffuant, Weisbuch, Amblard & Faure 2013";
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
            "The same meetings on a 29 × 29 torus, each agent meeting only its four neighbors, d = 0.3, μ = 0.3 (Fig. 5): a broad central consensus with scattered holdouts. The torus is drawn at the right. The caption's 100 000 meetings are about 119 periods. A broad central region is already visible then; numerical agreement to within 10⁻³ takes longer. Measured (20 seeds, to stability): the largest group holds 92 %, with 27 isolated agents, at a median period of 1 797.",
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
            "Extremists with low moderate uncertainty",
            DAWF,
            "The 20 % most extreme opinions (10 % at each end) start confident (uncertainty 0.1), the rest at 0.4; μ = 0.5, 200 agents (Fig. 5). The paper presents a central-convergence example and reports 4 % of moderates becoming extremists, with y = 0.03; those two numbers are inconsistent because y is the sum of squared converted shares. Our cutoff counts an initially moderate agent beyond the innermost initial extremist minus 0.1; the paper does not specify that cutoff. Measured (1000 seeds): 45.08 % are counted at the end, compared with 12.55 % already counted at period 0. Counted-at-end shares depend on the cutoff and do not measure movement by themselves.",
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
            "Fig. 7's setup",
            DAWF,
            "Fig. 7's setup: 10 % extremists, moderates at uncertainty 1.4, μ = 0.5, 200 agents. The paper presents a single-extreme example and Fig. 8 a central example at the same parameters; μ = 0.5 is also repeated in §5. With our stated cutoff and outcome definitions, 1000 seeds give 924 both-extreme and 76 intermediate outcomes, with no single-extreme or central outcomes. This sample provides no match to those examples; it does not establish impossibility. At μ = 0.2 both single-extreme and central outcomes occur, consistent with §4.8's finding that stronger interactions enlarge the both-extreme zone.",
            |c| {
                regime(c, 0.1, 1.4);
            },
        ),
        preset(
            "ra-literal",
            "Fig. 9's corner with drawn extremists",
            DAWF,
            "Fig. 9's corner: μ = 0.2, extremists' uncertainty 0.1, 5 % extremists drawn as the most extreme of 200 uniform opinions, moderates at 1.4, run until opinions and uncertainties stabilize. A moderate is counted beyond the innermost initial extremist minus 0.1: our explicit cutoff assumption, informed by the 2013 reply, rather than a specification in the 2002 paper. Measured (20 seeds): a single extreme in 19 runs (y 0.95), still by period 270 (median).",
            literal,
        ),
        preset(
            "ra-meadows-cliff",
            "Meadows and Cliff's reading",
            MC,
            "Meadows and Cliff's Java reimplementation of Fig. 9's corner: moderates uniform on (−0.8, 0.8), extremists uniform in [0.8, 1] and [−1, −0.8], y measured after 200 periods of 200 meetings (40 000 meetings) with a moderate counted as extremist only past ±0.8. Their conclusion: 'no conditions under which single extreme convergence will occur in the majority of the simulations'. Measured (20 seeds): y 0.00 in every run — yet the mean opinion has already drifted to ±0.56: the majority is on its way to one extreme but short of 0.8 at period 200. Compare with ra-deffuant-2013.",
            |c| {
                reading(c, 0.0, 200);
            },
        ),
        preset(
            "ra-deffuant-2013",
            "The authors' reply",
            DAW,
            "Deffuant, Weisbuch, Amblard and Faure's reply (2013) changes Meadows and Cliff's Java measurement to 1 200 periods (240 000 meetings) and a cutoff of ±0.7, with the same band placement. The authors specify these adjustments after examining the trajectories; the 2002 paper left the horizon and cutoff unstated. Measured (20 seeds): a single extreme in 19 runs (mean y 0.95). Each adjustment alone can yield a single extreme; together they recover frequent single-extreme readings at this setting. The ra-readings sweep compares the four readings.",
            |c| {
                reading(c, 0.1, 1200);
            },
        ),
        preset(
            "ra-bc-extremists",
            "Bounded confidence with extremists",
            DAWF,
            "§6's plain bounded confidence with extremists (Fig. 20): 1000 agents, 5 % extremists at uncertainty 0.1, moderates at 1.0, μ = 0.2. This alternate window uses the listener's own uncertainty. Measured (20 seeds): a single extreme in every run (y 1.00), consistent with the reported single-extreme zone near U = 1. Eq. 11 prints the influencer's uncertainty instead; compare ra-bc-printed. This parameter setting does not test all of §6.",
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
            "The same population with eq. 11 as printed: |x − x′| < u′, where x′ influences x and u′ is the influencer's uncertainty. Confident extremists can move toward uncertain moderates, while moderates accept extremist influence only at short distances. Measured (20 seeds): consensus near the center in every run, y 0.00. Across the tested Fig. 20 slice at 5 % extremists, the printed window yields central readings while the listener window recovers the reported single-extreme zone near U = 1. These checks cover plain bounded confidence, not every variant in §6.",
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
            "Amblard and Deffuant's lattice (Fig. 3c): a 30 × 30 torus with eight neighbors each, 20 % extremists at ±1, moderate opinions uniform across [−1, 1], moderate uncertainty 1.4, μ = 0.2. We sample links uniformly and count moderates beyond ±0.9; the paper leaves these details unspecified. About 5 % of moderates per side already lie beyond ±0.9 at period zero. Measured (20 seeds): both-extreme readings in every run, mean y 0.493. Settling is slow: median final period 19 358.5, with 9 runs reaching the 20 000-period cap.",
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
            "Amblard and Deffuant's small world (Figs. 4–5): 1000 agents on a ring, each linked to 32 neighbors, 80 % of links rewired at random; 5 % extremists at ±1, moderate opinions uniform across [−1, 1], moderate uncertainty 1.8, μ = 0.1. We sample links uniformly and count moderates beyond ±0.9, assumptions the paper does not specify. Measured (20 seeds): 9 single-extreme and 11 central readings, mean y 0.449, median final period 690.5; no run reaches the cap. A central reading only means too few moderates cross this cutoff; it does not establish that their opinions stay near zero. The ad-connectivity sweep compares connectivity and cutoff sensitivity.",
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
