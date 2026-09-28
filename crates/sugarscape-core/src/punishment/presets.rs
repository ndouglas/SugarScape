//! Boyd, Gintis, Bowles and Richerson's base case and figures, the variants
//! their text describes, and Janssen's readings of what it leaves open.

use super::config::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Structure, Traits, Victory,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const BGBR: &str = "Boyd, Gintis, Bowles & Richerson 2003, PNAS 100: 3531";
const JANSSEN: &str = "Janssen's replication, CoMSES 2223 (readings only)";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut PunishmentConfig),
) -> ModelPreset {
    let mut c = PunishmentConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Punishment(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "bg-base",
            "The base case, groups of 32",
            BGBR,
            "Boyd, Gintis, Bowles and Richerson's model: 128 groups of 32. Each period contributors and punishers cooperate at a cost c = 0.2 (erring 2 % of the time), defectors do not, and each punisher fines every defector p/n = 0.8/n at a cost k/n = 0.2/n to itself. Then everyone meets someone, from another group with probability m = 0.01, and copies them with probability Wⱼ/(Wⱼ + Wᵢ); groups are paired at random and each pair fights with probability ε = 0.015, the group with fewer defectors more likely to win and replace the loser; 1 % mutate. It starts with one group of punishers among 127 of defectors. The payoff costs are subtracted from is never stated; here 1, which fits their calibration (a trait with advantage c spreads from 10 % to 90 % in about 40 periods). Colors: contributors blue, punishers green, defectors red. Measured (10 seeds, the last 1 000 of 2 000 periods): 69 % cooperate (the figure: 83 %), 44 % are punishers. The stated model's cooperation holds to groups of about 32; the figure's reaches 256 — see bg-either.",
            |_| {},
        ),
        preset(
            "bg-either",
            "Fig. 1b as plotted: either group starts a conflict",
            BGBR,
            "The base case, with one reading changed: 'groups are paired at random, and with probability ε, intergroup conflict results' — either group of a pair can start the conflict, each with probability ε, so a pair fights with probability 2ε − ε², about twice the text's. Their Methods derive ε from an extinction rate of 0.0075 as if pairs fought at ε; the figures fit this reading instead. Measured (10 seeds, the last 1 000 of 2 000 periods): 84 % cooperate at n 32 (the figure: 83 %); across Fig. 1's six curves the mean gap to the figure is 0.010–0.033 (bg-fig1-either), and it reproduces Figs. 2 and 3 as well — all but Fig. 4's fixed cost, which falls a group size sooner.",
            |c| c.pairing = Pairing::Either,
        ),
        preset(
            "bg-none",
            "Fig. 1a: no punishment",
            BGBR,
            "Fig. 1a: no punishment (p = k = 0), the base case otherwise. Cooperation costs c and brings nothing to the cooperator; only conflict between groups favors it. Measured (10 seeds, the last 1 000 of 2 000 periods): 10 % cooperate, the floor mutation keeps up — as in the figure (0.09 from n 32). Group selection alone holds cooperation only in groups of 4 or 8.",
            |c| {
                c.fine = 0.0;
                c.punish_cost = 0.0;
            },
        ),
        preset(
            "bg-large",
            "Fig. 1b: groups of 128",
            BGBR,
            "Fig. 1b's groups of 128. Measured (10 seeds, the last 1 000 of 2 000 periods): 17 % cooperate (from 7 % to 43 % by seed), where the figure has 64 % and the Discussion 'cooperation is sustained in groups on the order of 100 individuals'. When either group can start a conflict (bg-either's reading), 59 %.",
            |c| c.size = 128,
        ),
        preset(
            "bg-weak",
            "Fig. 3: p = 0.4",
            BGBR,
            "Fig. 3: the cost of being punished p = 0.4, twice the cost of cooperating, where the base case has four times. 'Lower values of p result in much lower levels of cooperation.' Measured (10 seeds, the last 1 000 of 2 000 periods): 9 % cooperate at n 32 (the figure: 21 %; 83 % with p = 0.8) — much lower, as stated.",
            |c| c.fine = 0.4,
        ),
        preset(
            "bg-fixed",
            "Fig. 4: a fixed cost",
            BGBR,
            "Fig. 4: punishers pay a fixed cost equal to c every period, whether or not anyone defects, instead of k/n for each defector. 'Punishment does not aid in the evolution of cooperation when the costs born by punishers are fixed.' Measured (10 seeds, the last 1 000 of 2 000 periods): 9 % cooperate at n 32 — no better than no punishment (10 %), as stated.",
            |c| c.punishing = Punishing::Fixed,
        ),
        preset(
            "bg-mixing",
            "Fig. 2: m = 0.05",
            BGBR,
            "Fig. 2: mixing m = 0.05, five times the base case: an agent's model comes from another group one time in 20. 'When the migration rate increases, levels of cooperation fall precipitously.' Measured (10 seeds, the last 1 000 of 2 000 periods): 28 % cooperate at n 32 (from 15 % to 47 % by seed), against 69 % at m = 0.01 — falling, as stated (the figure keeps 72 % here, at n 32 its last size before the fall).",
            |c| c.mixing = 0.05,
        ),
        preset(
            "bg-benefit",
            "A benefit, and conflict over payoffs",
            BGBR,
            "One of the paper's structural variants: each cooperative act gives b/n = 0.8/n (b = 4c) to every other member, and conflicts are decided by groups' average payoffs, normalized by the widest difference possible (Cooney's eq. 3.18; the paper's form is not given). 'For reasonable values of b (2c, 4c, and 8c), the results … are qualitatively similar.' Measured (10 seeds, the last 1 000 of 2 000 periods): 63 % cooperate at n 32, against 11 % without punishment — similar, as stated.",
            |c| {
                c.benefit = 0.8;
                c.victory = Victory::Payoff;
            },
        ),
        preset(
            "bg-continuous",
            "Continuous traits",
            BGBR,
            "Continuous traits: each agent cooperates with probability x and punishes with probability y, both in [0, 1]; mutants draw both uniformly. 'The steady-state mean levels of cooperation in this model are similar to the base model.' Measured (10 seeds, the last 1 000 of 2 000 periods): 94 % cooperate at n 32 (the base model: 69 %), and still 90 % at n 256 (the base model: 12 %) — not similar. Uniform mutants keep the mean punishment near ½, and a defector then pays about p/2 = 0.4, more than c: cooperation pays within groups. Starting from all defectors it still collapses from n 128.",
            |c| c.traits = Traits::Continuous,
        ),
        preset(
            "bg-ring",
            "A ring without extinction",
            BGBR,
            "The paper's last variant: groups on a ring, no conflict, migrants only from the two neighboring groups, and each cooperative act giving b/n = 0.4/n (b = 2c) to the others, so that cooperative groups earn more and are imitated. 'We could find no reasonable parameter combination that led to significant long run average levels of cooperation.' Measured (10 seeds, the last 1 000 of 2 000 periods): 18 % cooperate at n 32 — low, as stated — but about half in groups of 4 or 8 (bg-ring).",
            |c| {
                c.structure = Structure::Ring;
                c.benefit = 0.4;
            },
        ),
        preset(
            "bg-janssen",
            "Janssen's readings",
            JANSSEN,
            "Marco Janssen's NetLogo replication (CoMSES 2223) fills the paper's gaps differently: payoffs 1 plus a benefit 0.5 × the share cooperating; each group challenges a random group with probability ε (about twice the text's conflict); groups fight over the share who cooperated this period; agents imitate in turn; a punisher who errs fines itself. His readings, not his code, are used here. Measured (10 seeds, the last 1 000 of 2 000 periods): 84 % cooperate at n 32 (the figure: 83 %). Of his readings the doubled conflict does the work (bg-readings); his benefit lifts cooperation in small groups without punishment.",
            |c| {
                c.benefit = 0.5;
                c.pairing = Pairing::Challenge;
                c.counted = Counted::Acts;
                c.imitation = Imitation::InTurn;
                c.erring = Erring::Itself;
            },
        ),
    ]
}
