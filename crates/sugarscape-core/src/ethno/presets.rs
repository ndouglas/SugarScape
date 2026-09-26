//! HA06's runs, its appendix's and its code's readings, and its critics'
//! variants. Descriptions quote measurements (release, seeds 1–10, 2,000
//! periods, the mean over periods 1,901–2,000, recorded 2026-09-25).

use super::config::{Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut EthnoConfig),
) -> ModelPreset {
    let mut c = EthnoConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Ethno(c),
    }
}

const HA06: &str = "Hammond & Axelrod 2006, J. Conflict Resolution 50";
const HA_APPENDIX: &str = "Hammond & Axelrod 2006, appendix";
const HA_JAVA: &str = "Hammond & Axelrod's Java code (2003)";
const HKS13: &str = "Hartshorn, Kaznatcheev & Shultz 2013, JASSS 16(3)";
const J13: &str = "Jansson 2013, JASSS 16(3)";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ha-standard",
            "The standard case",
            HA06,
            "HA06's standard case: an empty 50 × 50 torus, four neighbors each. Each period an immigrant with random traits — one of four colours, a bit for helping its own colour and one for helping others — arrives at a random empty site; every agent's potential to reproduce (PTR) is reset to 12%, and each decides once for each neighbor whether to help it (costing it 1% of PTR, giving the neighbor 3%); in random order each reproduces with probability PTR into an empty neighbor (0.5% mutation per trait); each dies with probability 10%. HA06 Table 1 a: 76.3% ethnocentric, 74.2% cooperation. Measured (seeds 1–10, periods 1,901–2,000): 75.9% ethnocentric (13.5% humanitarian, 8.1% selfish, 2.4% traitorous), 76.0% cooperation, about 1,560 agents. Ethnocentrics take longer to dominate than HA06 say: 57% after 500 periods against their 73.9%.",
            |_| {},
        ),
        preset(
            "ha-figure-1",
            "Figure 1: mutation 0.25%",
            HA06,
            "HA06's Figure 1 and Table 1 f: the standard case with half the mutation rate (0.25%). HA06: 82.8% ethnocentric, 79.8% cooperation. Measured: 83.4% and 80.2% — reproduced.",
            |c| c.mutation = 0.0025,
        ),
        preset(
            "ha-appendix-mutation",
            "The appendix's 5% mutation",
            HA_APPENDIX,
            "HA06's appendix gives MutationRate = 0.05; its text, Table 1, the authors' code, NetLogo and every later paper use 0.005. At 5% the lattice never sorts: 36.0% ethnocentric, 25.8% humanitarian, 22.1% selfish, 16.0% traitorous, 56.4% cooperation — nothing like Table 1 a's 76.3/74.2, so the appendix's figure is a slip.",
            |c| c.mutation = 0.05,
        ),
        preset(
            "ha-appendix-double-play",
            "The appendix's loop: every decision twice",
            HA_APPENDIX,
            "HA06's appendix loops \"for each adjacent neighboring agent N of each existing agent A: A decides whether to donate to N … N decides whether to donate to A\" — read literally, every direction is decided twice a period. The authors' code and NetLogo decide once. Twice: 80.9% ethnocentric, 77.3% cooperation, 1,860 agents — five points above Table 1 a (76.3%), where once gives 75.9%.",
            |c| c.pair_play = PairPlay::Twice,
        ),
        preset(
            "ha-java-five-colors",
            "Five colours (the code's draw)",
            HA_JAVA,
            "The authors' Java draws a tag with Ascape's randomInRange(0, numColors), which includes both ends: \"four\" colours are five (tag 4 drawn black, \"if appears means error\"), \"eight\" are nine. Five colours: 75.4% ethnocentric, 76.1% cooperation — indistinguishable from four (75.9/76.0). The sweep ha-colors runs 2 to 9.",
            |c| c.colors = 5,
        ),
        preset(
            "ha-java-archive",
            "The archived code as it runs",
            HA_JAVA,
            "The authors' archived Java (July 2003) as it runs, not as its memo describes it: the immigration block is commented out and setup fills every site with a random agent, with the five-colour draw. Measured: 77.7% ethnocentric, 77.6% cooperation — the same outcome as the paper's empty start (and 43% ethnocentric by period 100, where the standard case has 35%).",
            |c| {
                c.colors = 5;
                c.start = Start::Random;
                c.immigration = 0.0;
            },
        ),
        preset(
            "ha-egoist-start",
            "A full lattice of egoists",
            HA06,
            "HA06: starting from a full lattice of egoists (help no one) with no immigration, ethnocentrism becomes \"just as dominant\". Measured: 78.6% ethnocentric, 79.9% cooperation — reproduced; mutation alone brings it in: 7% at period 100, 26% at 200, 70% at 500.",
            |c| {
                c.start = Start::Selfish;
                c.immigration = 0.0;
            },
        ),
        preset(
            "ha-cost-2",
            "Cost 2%",
            HA06,
            "Table 1 c: helping costs 2% of PTR. HA06: 61.8% ethnocentric, 56.1% cooperation. Measured: 63.5% ethnocentric (21.6% selfish), but 64.7% cooperation — 8.6 points above HA06's. Compare the colour-blind preset.",
            |c| c.cost = 0.02,
        ),
        preset(
            "ha-cost-2-blind",
            "Cost 2%, colour-blind",
            HA06,
            "HA06: \"when agents are unable to distinguish their own color from others, cooperation in the doubled-cost case falls to 14 percent.\" Blind agents have one bit (help everyone or no one). Measured: 41.8% cooperation (40% humanitarian, 60% selfish) — three times HA06's 14%, which blind agents reach only at a cost of 3% (12.7%). Not reproduced; the ha-cost sweep compares blind and seeing agents at every cost.",
            |c| {
                c.cost = 0.02;
                c.discrimination = Discrimination::None;
            },
        ),
        preset(
            "ha-misperception",
            "10% misperception",
            HA06,
            "HA06: with a 10% chance of misperceiving whether the other has the same colour, \"more than two-thirds\" are ethnocentric. Each decision uses the agent's other bit with probability 0.1 (the authors' noise). Measured: 71.7% ethnocentric, 71.1% cooperation — reproduced.",
            |c| c.misperception = 0.1,
        ),
        preset(
            "ha-each-color",
            "Strategies for each colour",
            HA06,
            "HA06: when agents distinguish all four colours, \"80 percent ethnocentric strategies\". Here each agent has one help bit per colour. Helping one's own colour only: 27.0%; helping one's own colour and refusing at least one other: 84.3% — HA06's 80% matches the looser reading. The Strategy view shows the rest as mixed (67%).",
            |c| c.discrimination = Discrimination::EachColor,
        ),
        preset(
            "jansson-offspring-anywhere",
            "Offspring anywhere",
            J13,
            "Jansson 2013 §3.6: put each offspring on a random empty site instead of next to its parent and the results are \"similar to the null model\" (12% cooperators with one partner a round, 3.4% with four). Measured: 4.5% cooperation; 88.7% selfish, 8.3% ethnocentric, 2.5% traitorous — reproduced: without kin next door, discrimination buys nothing.",
            |c| c.offspring = Offspring::Anywhere,
        ),
        preset(
            "jansson-tag-mutation-30",
            "Tag mutation 30%",
            J13,
            "Jansson 2013 §4.4: raise only the tag's mutation rate, so the tag becomes a deceptive marker of kinship: \"At a marker mutation rate of 30%, altruists surpass ethnocentrics.\" Measured: 44.9% humanitarian against 39.9% ethnocentric — reproduced (at 20%, 32.2 against 54.5). The sweep jansson-tag-mutation runs 0.5% to 90%.",
            |c| c.tag_mutation = Some(0.3),
        ),
        preset(
            "jansson-kin",
            "Kin strategies",
            J13,
            "Jansson 2013 §5.2: agents may discriminate on the tag or on a kin marker naming their family's founder (the marker mutates at 0.5%, the mutant founding a new family); the basis is a bit that mutates like the others (Jansson does not say). Table 5: kin 76.2%, ingroup 16.4%, all 2.8, none 2.0, outgroup 1.3, nonkin 1.3. Measured: kin 52.1%, ingroup (ethnocentric) 26.7%, all 12.0, none 7.4, nonkin 1.0, outgroup 0.7 — kin discriminators win, by far less. With the basis fixed at immigration (jansson-kin-fixed) kin take 65.5% and ingroup 12.6%, nearer Table 5.",
            |c| c.kin_strategies = true,
        ),
        preset(
            "jansson-kin-fixed",
            "Kin strategies, basis fixed",
            J13,
            "Jansson 2013 §5.2's kin discriminators with the other reading of an unstated rule: an agent's basis (tag or kin marker) is drawn at immigration and never mutates, so a lineage stays tag- or kin-discriminating. Table 5: kin 76.2%, ingroup 16.4%, all 2.8, none 2.0, outgroup 1.3, nonkin 1.3. Measured (seeds 1–10): kin 65.5%, ingroup 12.6%, all 13.7, none 6.2, nonkin 1.1, outgroup 0.8 (20 seeds: kin 68.9, ingroup 10.6) — closer to Table 5 than the mutating basis (52.1 / 26.7), though humanitarians stay far above Jansson's 2.8%. The jansson-markers sweep runs both readings.",
            |c| {
                c.kin_strategies = true;
                c.kin_basis = KinBasis::Fixed;
            },
        ),
        preset(
            "hks-no-ethnocentrics",
            "No ethnocentrics (H, S, T)",
            HKS13,
            "HKS13 Study 2: immigrants and mutations may produce only humanitarian, selfish and traitorous strategies (a disallowed immigrant is redrawn, a disallowed mutation ignored). HKS13 Table 3: 1,368 humanitarians, 115 selfish, 150 traitors — the one subset where traitors beat the selfish. Measured: 84.7% humanitarian, 7.0% selfish, 8.4% traitorous (1,384, 114, 136 agents) — reproduced; humanitarians reach 84.7% here against the standard case's 75.9% ethnocentrics (1,384 against 1,184 agents in HKS13's Study 2 counts).",
            |c| {
                c.allowed = vec![
                    Strategy::Humanitarian,
                    Strategy::Selfish,
                    Strategy::Traitorous,
                ]
            },
        ),
    ]
}
