//! Axelrod's runs and dominance variant, and Galán & Izquierdo's long runs
//! and departures.

use super::config::{GroupsConfig, NormsConfig, Selection};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const AXELROD: &str = "Axelrod 1986";
const GI: &str = "Galán & Izquierdo 2005";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut NormsConfig),
) -> ModelPreset {
    let mut c = NormsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Norms(c),
    }
}

/// Axelrod's metanorms for G&I's long runs.
fn long(c: &mut NormsConfig) {
    c.metanorms = true;
    c.stop_at = 20_000;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("ax-norms", "The norms game", AXELROD, "Axelrod's norms game: 20 agents, each with a boldness and a vengefulness (eight levels, 0/7 to 7/7). Four times a generation each gets a chance to defect, seen by each other agent with a random chance S; it defects if S is below its boldness, gaining 3 and costing everyone else 1. Whoever sees a defection punishes it with probability equal to their vengefulness: −9 to the defector, −2 to the punisher. Agents one standard deviation above the mean payoff get two offspring, those one below none; each bit mutates at 1 %. The plane is boldness (right) against vengefulness (up), with Galán & Izquierdo's norm-established (green) and norm-collapsed (red) corners. Axelrod's five runs of 100 generations ended in three kinds of state; the norm did not hold. Measured (100 seeds, generation 100): low boldness with vengefulness ≥ 3/7 in 35, both low in 31, high boldness with low vengefulness in 27; collapsed in 19, established in 4. Run on, it collapses: 99 of 100 by generation 1000.", |c| {
            c.stop_at = 100
        }),
        preset("ax-metanorms", "The metanorms game", AXELROD, "The metanorms game: whoever sees a defection and does not punish it can be seen by the others, with the same chance S, and punished for it (−9, costing the punisher −2), with the same vengefulness. Axelrod: 'In all five runs a norm against defection was established.' Measured (100 seeds, generation 100): established in 92, collapsed in none. Galán & Izquierdo's point is what happens later (gi-metanorms-long).", |c| {
            c.metanorms = true;
            c.stop_at = 100;
        }),
        preset(
            "ax-dominance",
            "Dominance: two groups",
            AXELROD,
            "Axelrod's dominance: 20 whites punished less (P = −3) and 10 blacks (P = −9), each group reproducing within itself; a player's defections hurt, and are punished by, only the other group (p. 1103). Axelrod: without metanorms 'even members of the stronger group tend to be free riders … low vengefulness and high boldness in both groups'. Measured (100 seeds, generation 100): the strong group bold in 97 (mean 0.96, vengefulness 0.13); the weak group bold (0.6 or more) in only 50 and kept below 0.2 in 39 (mean 0.52). He gives no counts. With everyone touched instead (groups.rule: everyone): both bold, 0.94 and 0.86.",
            |c| {
                c.groups = GroupsConfig {
                    enabled: true,
                    ..GroupsConfig::default()
                };
                c.stop_at = 100;
            },
        ),
        preset(
            "ax-dominance-metanorms",
            "Dominance with metanorms",
            AXELROD,
            "Dominance with metanorms; punishment for not punishing stays within a group (p. 1103). Axelrod: 'it becomes relatively easier for the strong group to keep the weak group from being bold, while it is not so easy for the weak group to keep the strong one from defecting'. Measured (100 seeds, generation 100): boldness 0.15 (strong) and 0.04 (weak), vengefulness 0.76 and 0.87 — as he says. With everyone touched instead: 0.06 and 0.02, both kept down.",
            |c| {
                c.groups = GroupsConfig {
                    enabled: true,
                    ..GroupsConfig::default()
                };
                c.metanorms = true;
                c.stop_at = 100;
            },
        ),
        preset("gi-metanorms-long", "Metanorms, run long", GI, "The metanorms game run for 20 000 generations. Galán & Izquierdo (1,000 runs to 10⁶ generations): 'Even though after 100 generations the norm is almost always established, as time goes by … the norm usually collapses.' Measured (50 seeds): established in 43 of 50 at generation 100 and 38 at 20 000, collapsed in 5; (100 seeds) 52 established and 43 collapsed by 10⁵ — the drift toward collapse is there, slower than their 10⁶-generation horizon shows. How fast depends on details Axelrod's text leaves ambiguous: see the Readings switches (under his own words for ties, all_equal: keep, the norm mostly holds even at 10⁶).", long),
        preset(
            "gi-low-mutation",
            "Metanorms, mutation 0.001",
            GI,
            "Metanorms with a mutation rate of 0.001 instead of 0.01. Galán & Izquierdo: the norm collapses 'much more quickly' and stays collapsed. Measured (50 seeds): established in 47 at generation 100, collapsed in 46 by 20 000 (100 seeds: 100 by 10⁵).",
            |c| {
                long(c);
                c.mutation = 0.001;
            },
        ),
        preset(
            "gi-mild-metanorms",
            "Metanorms, milder metapunishment",
            GI,
            "Metanorms with the meta-payoffs divided by ten (ME −0.2, MP −0.9). Galán & Izquierdo: 'the norm quickly collapses … Axelrod's conclusions are reversed.' Measured (50 seeds): established in 3 at generation 100; collapsed in all 50 by 20 000.",
            |c| {
                long(c);
                c.meta_enforcement = -0.2;
                c.meta_punishment = -0.9;
            },
        ),
        preset(
            "gi-temptation-10",
            "Metanorms, temptation 10",
            GI,
            "Metanorms with a temptation of 10. Galán & Izquierdo's counterintuitive result: a larger temptation keeps the norm, because defections stay frequent enough for non-punishers to be caught. Measured (50 seeds): established in every run at generation 100 and at 20 000.",
            |c| {
                long(c);
                c.temptation = 10.0;
            },
        ),
        preset(
            "gi-tournament",
            "Metanorms, random tournament",
            GI,
            "Metanorms with Galán & Izquierdo's random tournament: twenty times, the better of two random agents is copied. They found the norm collapses quickly under it and under two other reasonable selection rules. Measured (50 seeds): established in 21 at generation 100; collapsed in 46 by 20 000.",
            |c| {
                long(c);
                c.selection = Selection::Tournament;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, NormsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Norms(c) => (p.id, c),
                _ => panic!("{} is not a norms preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        let n = find("ax-norms");
        assert_eq!((n.metanorms, n.stop_at), (false, 100));
        assert!(find("ax-metanorms").metanorms);
        assert!(find("ax-dominance").groups.enabled && !find("ax-dominance").metanorms);
        let m = find("gi-mild-metanorms");
        assert_eq!(
            (m.meta_enforcement, m.meta_punishment, m.stop_at),
            (-0.2, -0.9, 20_000)
        );
        assert_eq!(find("gi-temptation-10").temptation, 10.0);
        assert_eq!(find("gi-low-mutation").mutation, 0.001);
        assert_eq!(find("gi-tournament").selection, Selection::Tournament);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
