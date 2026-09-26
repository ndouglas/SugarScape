//! The paper's runs and the replication's variants.

use super::config::{ClassesConfig, Decision, Interaction, LatticeConfig, Layout, Start};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ClassesConfig),
) -> ModelPreset {
    let mut c = ClassesConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Classes(c),
    }
}

/// AEY's tag runs: 100 agents, 50 of each tag, memory 20, ε = 0.2.
fn tags(c: &mut ClassesConfig) {
    c.tags = true;
    c.memory = 20;
}

const AEY: &str = "Axtell, Epstein & Young 2000";
const PVPLH: &str = "Poza, Villafáñez, Pajares, López-Paredes & Hernández 2011";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "aey-equity",
            "One type: toward the equity norm",
            AEY,
            "Axtell, Epstein and Young's bargaining society: 100 agents paired at random each period play the Nash demand game (demand 30, 50 or 70 percent of a pie; both get their demands if they fit in 100, else nothing). Each remembers its last 10 opponents' demands and best-replies to them, erring at random with probability 0.2. The dots are agents on the memory simplex; the shading is the best reply there. The paper (Fig. 2): the equity norm, everyone demanding half, after about 80 periods. Measured (20 seeds): every agent's best reply is M by period 14 (median) in every seed.",
            |_| {},
        ),
        preset(
            "aey-fractious",
            "One type: a fractious start",
            AEY,
            "The same society started fractious: every memory half H and half L, so everyone demands H at once. The paper (Fig. 3): a fractious state — agents aggressive or passive, pie wasted — that 'persists in excess of 10⁹ time periods'. Measured (20 seeds): the first periods pay about 30 each (the paper: about a quarter of the pie), but every seed reaches equity within a few hundred periods, and no random start ever falls into the fractious state first. At 100 agents with ε = 0.2 the fractious regime does not persist.",
            |c| c.start = Start::Fractious,
        ),
        preset(
            "aey-transition",
            "Ten agents leave the fractious state",
            AEY,
            "Ten agents with memory 10 and ε = 0.1, started fractious, stopping at AEY's transition target: every agent with at least (1 − ε)·m M's in memory. The paper (Figs. 4–5): the waiting time grows exponentially with memory and population, beyond 10⁵ periods at m = 13. Measured (20 seeds): it does grow steeply (at ε = 0.05 a median of 334 periods at m = 8 and 25 300 at m = 14; at m = 10, ε = 0.1, 148 at 20 agents and 11 600 at 60), but at m = 13, ε = 0.1 the median is 600 — two orders of magnitude short of the paper's.",
            |c| {
                c.agents = 10;
                c.noise = 0.1;
                c.start = Start::Fractious;
                c.stop_at_equity = true;
            },
        ),
        preset(
            "aey-tags",
            "Two tags: norms and classes",
            AEY,
            "Two types of 50 agents, told apart by a meaningless tag; each keeps a memory of length 20 per tag and best-replies to what that tag's members did. The paper (Figs. 6–9, ε = 0.2): from random starts, equity within and between types, equity between but not within, classes (equity within types, one tag demanding 70 of the other), and 'equity above, division below'. Measured (20 seeds, 5000 periods): equity everywhere, with 18 of 20 passing through equity between but not within — and no classes and no division below in any seed, as Poza et al. (2011) also found: 'segregation never emerged'.",
            tags,
        ),
        preset(
            "aey-classes",
            "Two tags: a class system from the start",
            AEY,
            "The tag society started as a class system: within types everyone remembers M; darks remember lights demanding L, so they demand H, and lights remember darks demanding H. The paper never ran this ('these events are very rare'). Measured (20 seeds): 18 of 20 are still class systems after 20 000 periods — once classes exist, they last.",
            |c| {
                tags(c);
                c.start = Start::Classes;
            },
        ),
        preset(
            "pvplh-small-tags",
            "Two tags, small and forgetful",
            PVPLH,
            "Poza et al. 2011's tag society, small and forgetful: 20 agents, memory 5, ε = 0.05, where they could finally see segregation. Measured (20 seeds, 5000 periods): every seed passes through equity between types but not within (one type compromising, the other fractious); classes still never appear under AEY's rule.",
            |c| {
                c.tags = true;
                c.agents = 20;
                c.memory = 5;
                c.noise = 0.05;
            },
        ),
        preset(
            "pvplh-mode",
            "Two tags, the mode rule",
            PVPLH,
            "AEY's tag society with Poza et al.'s decision rule: best-reply to the most frequent remembered demand instead of maximizing expected payoff. 'Segregation emerged spontaneously much more often.' Measured (20 seeds, 5000 periods): classes or division below in 10 of 20 seeds, against none under AEY's rule. Open 'AEY's rule vs the mode rule' in the presets menu to run both.",
            |c| {
                tags(c);
                c.decision = Decision::Mode;
            },
        ),
        preset(
            "pvplh-progressive",
            "One type, memories that grow",
            PVPLH,
            "One type, 20 agents, memory 12, ε = 0.1, with memories that start empty and grow (Poza et al. §3.4: the first demands are random). They report a longer way to equity. Measured (20 seeds): a median of 186 periods against 173 with random memories — no measurable difference.",
            |c| {
                c.agents = 20;
                c.memory = 12;
                c.noise = 0.1;
                c.start = Start::Progressive;
            },
        ),
        preset(
            "pvplh-lattice",
            "Two tags on a lattice, in two zones",
            PVPLH,
            "Poza et al.'s 10 × 10 torus: each agent meets one of its eight neighbors, the two tags in two zones, memory 5, ε = 0.05, the mode rule. Only agents on the two borders meet the other tag, so each border can settle on its own norm, and the whole-population regime reads mixed (20 of 20 seeds). With tags laid out at random instead, 12 of 20 seeds reach classes or a split within types — the well-mixed attractors recur, as they found.",
            |c| {
                c.tags = true;
                c.memory = 5;
                c.noise = 0.05;
                c.decision = Decision::Mode;
                c.interaction = Interaction::Lattice;
                c.lattice = LatticeConfig {
                    layout: Layout::TwoZones,
                    ..LatticeConfig::default()
                };
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_the_papers_runs_and_the_replications() {
        let got: Vec<(&str, ClassesConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Classes(c) => (p.id, c),
                _ => unreachable!(),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        let d = ClassesConfig::default();
        assert_eq!(find("aey-equity"), d);
        assert_eq!((find("aey-tags").tags, find("aey-tags").memory), (true, 20));
        assert_eq!(find("aey-classes").start, Start::Classes);
        let t = find("aey-transition");
        assert_eq!(
            (t.agents, t.noise, t.start, t.stop_at_equity),
            (10, 0.1, Start::Fractious, true)
        );
        let small = find("pvplh-small-tags");
        assert_eq!((small.agents, small.memory, small.noise), (20, 5, 0.05));
        assert_eq!(find("pvplh-mode").decision, Decision::Mode);
        let l = find("pvplh-lattice");
        assert_eq!(
            (l.interaction, l.lattice.layout, l.agents),
            (Interaction::Lattice, Layout::TwoZones, 100)
        );
        assert_eq!(got.len(), 9);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
