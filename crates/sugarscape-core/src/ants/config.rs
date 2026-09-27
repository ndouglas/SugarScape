//! The Ants and Recruitment model's parameters: Kirman's (1993) recruitment
//! chain, the three extensions he names (Becker's majority pull, more
//! sources, meetings over a network), and Alfarano and Milaković's (2007)
//! agent rule, as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How ants change sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Kirman: random pairwise meetings (eq. 1).
    Kirman,
    /// Alfarano and Milaković (eq. 18): each ant in turn switches with
    /// probability (a + λ·neighbors elsewhere)/(a + λN).
    Alfarano,
}

/// How Kirman's self-conversion and recruitment combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conversion {
    /// Eq. (1) as written: switch with probability ε + (1 − δ)·[the other
    /// ant is elsewhere], clamped to 1.
    Kirman,
    /// Footnote 9: switch with probability ε; otherwise meet, and join with
    /// probability γ = (1 − δ)/(1 − ε), the γ that gives the same δ.
    Footnote,
}

/// Who can meet whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Anyone meets anyone (Kirman).
    Complete,
    /// Each ant linked to the degree/2 nearest on each side.
    Ring,
    /// The ring plus 0.1·N shortcuts (Alfarano and Milaković).
    SmallWorld,
    /// Each pair linked with probability `link`.
    Random,
    /// Barabási–Albert growth, degree/2 links per new ant.
    ScaleFree,
}

/// Where the ants start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Each ant at a uniformly random source (Kirman does not say).
    Random,
    /// All at the first source.
    One,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AntsConfig {
    /// N.
    pub ants: u32,
    pub rule: Rule,
    /// ε: the chance an ant changes source on its own.
    pub epsilon: f64,
    /// δ: an ant met at another source recruits with probability 1 − δ.
    pub delta: f64,
    pub conversion: Conversion,
    /// Meetings per step under Kirman's rule (Figure II plots every
    /// fiftieth); a step under Alfarano and Milaković's is one sweep.
    pub meetings: u32,
    pub sources: u32,
    /// Becker's externality: recruiting scaled by 1 + pull·(xⱼ − xᵢ).
    pub pull: f64,
    /// Alfarano and Milaković's a and λ.
    pub a: f64,
    pub lambda: f64,
    pub network: Network,
    /// The ring's, small world's and scale-free network's degree.
    pub degree: u32,
    /// The random graph's link probability.
    pub link: f64,
    /// q: the share of ants that never herd.
    pub independent: f64,
    pub start: Start,
    /// Stop at this step (0: never).
    pub stop_at: u32,
}

impl Default for AntsConfig {
    /// Kirman's Figure IIb: 100 ants, ε 0.002, δ 0.01, 50 meetings a step.
    fn default() -> Self {
        AntsConfig {
            ants: 100,
            rule: Rule::Kirman,
            epsilon: 0.002,
            delta: 0.01,
            conversion: Conversion::Kirman,
            meetings: 50,
            sources: 2,
            pull: 0.0,
            a: 0.5,
            lambda: 1.0,
            network: Network::Complete,
            degree: 10,
            link: 0.1,
            independent: 0.0,
            start: Start::Random,
            stop_at: 0,
        }
    }
}

/// The most ants and sources.
pub const MAX_ANTS: u32 = 5000;
pub const MAX_SOURCES: u32 = 6;

impl AntsConfig {
    /// How many ants never herd.
    pub fn independent_count(&self) -> usize {
        (self.independent * f64::from(self.ants)).round() as usize
    }

    /// Footnote 9's γ for this δ and ε.
    pub fn gamma(&self) -> f64 {
        if self.epsilon < 1.0 {
            ((1.0 - self.delta) / (1.0 - self.epsilon)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=MAX_ANTS).contains(&self.ants),
            "ants",
            "must be between 2 and 5000",
        );
        check(unit(self.epsilon), "epsilon", "must be between 0 and 1");
        check(unit(self.delta), "delta", "must be between 0 and 1");
        check(
            self.conversion == Conversion::Kirman || self.delta >= self.epsilon,
            "delta",
            "footnote 9's δ = 1 − γ + γε is at least ε",
        );
        check(
            (1..=100_000).contains(&self.meetings),
            "meetings",
            "must be between 1 and 100000",
        );
        check(
            (2..=MAX_SOURCES).contains(&self.sources),
            "sources",
            "must be between 2 and 6",
        );
        check(
            self.rule == Rule::Kirman || self.sources == 2,
            "sources",
            "Alfarano and Milaković's rule has two states",
        );
        check(
            (0.0..=10.0).contains(&self.pull),
            "pull",
            "must be between 0 and 10",
        );
        check(
            self.a >= 0.0
                && self.lambda >= 0.0
                && self.a + self.lambda > 0.0
                && self.a <= 1e6
                && self.lambda <= 1e6,
            "a",
            "a and λ must be at least 0, not both 0, and at most 1000000",
        );
        let linked = matches!(
            self.network,
            Network::Ring | Network::SmallWorld | Network::ScaleFree
        );
        check(
            !linked
                || (self.degree >= 2 && self.degree.is_multiple_of(2) && self.degree < self.ants),
            "degree",
            "must be even, at least 2 and less than the number of ants",
        );
        check(
            self.link > 0.0 && self.link <= 1.0,
            "link",
            "must be above 0 and at most 1",
        );
        check(
            unit(self.independent),
            "independent",
            "must be between 0 and 1",
        );
        check(
            self.stop_at <= 10_000_000,
            "stop_at",
            "must be at most 10000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &AntsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("ants", self.ants == next.ants),
            ("rule", self.rule == next.rule),
            ("sources", self.sources == next.sources),
            ("network", self.network == next.network),
            ("degree", self.degree == next.degree),
            ("link", self.link == next.link),
            ("independent", self.independent == next.independent),
            ("start", self.start == next.start),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Colony", "ants", "Ants (N)", (2, MAX_ANTS), Reset)
            .with_help("Kirman's figures: 100."),
        Param::choice(
            "Colony",
            "rule",
            "Ants change source by",
            &[
                ("kirman", "Meeting another ant (Kirman)"),
                ("alfarano", "Counting their neighbors (Alfarano & Milaković)"),
            ],
            Reset,
        ),
        Param::choice(
            "Colony",
            "start",
            "Start",
            &[
                ("random", "Each ant at a random source"),
                ("one", "All at the first source"),
            ],
            Reset,
        ),
        Param::integer("Colony", "sources", "Food sources", (2, MAX_SOURCES), Reset)
            .shown_if("rule", "kirman")
            .with_help("Kirman: 'Generalizing to a larger number of sources would not change the analysis.' A self-converting ant goes to a random other source."),
        Param::number("Kirman", "epsilon", "Self-conversion (ε)", (0.0, 1.0, 0.001), Live)
            .shown_if("rule", "kirman"),
        Param::number("Kirman", "delta", "Resistance to recruiting (δ)", (0.0, 1.0, 0.001), Live)
            .shown_if("rule", "kirman")
            .with_help("An ant met at another source recruits with probability 1 − δ."),
        Param::choice(
            "Kirman",
            "conversion",
            "ε and δ combine",
            &[
                ("kirman", "As in eq. (1): ε + (1 − δ)"),
                ("footnote", "As in footnote 9: ε, then γ"),
            ],
            Live,
        )
        .shown_if("rule", "kirman"),
        Param::integer("Kirman", "meetings", "Meetings per step", (1, 100_000), Live)
            .shown_if("rule", "kirman")
            .with_help("Kirman's Figure II plots every fiftieth meeting."),
        Param::number("Kirman", "pull", "Majority pull (Becker)", (0.0, 10.0, 0.05), Live)
            .shown_if("rule", "kirman")
            .with_help("Recruiting scaled by 1 + pull × (the recruiter's share − the recruit's). Kirman: 'This would make the process more extreme.'"),
        Param::number("Alfarano & Milaković", "a", "Idiosyncratic rate (a)", (0.0, 1000.0, 0.001), Live)
            .shown_if("rule", "alfarano"),
        Param::number("Alfarano & Milaković", "lambda", "Herding rate (λ)", (0.0, 1000.0, 0.01), Live)
            .shown_if("rule", "alfarano")
            .with_help("Each ant in turn switches with probability (a + λ × neighbors elsewhere)/(a + λN)."),
        Param::choice(
            "Network",
            "network",
            "Who meets whom",
            &[
                ("complete", "Anyone (Kirman)"),
                ("ring", "A ring"),
                ("small_world", "A ring with shortcuts"),
                ("random", "A random network"),
                ("scale_free", "A network with hubs"),
            ],
            Reset,
        ),
        Param::integer("Network", "degree", "Degree (D)", (2, MAX_ANTS - 1), Reset)
            .with_help("For the ring, the ring with shortcuts and the network with hubs; even."),
        Param::number("Network", "link", "Link probability (p)", (0.001, 1.0, 0.001), Reset)
            .shown_if("network", "random"),
        Param::number("Network", "independent", "Ants who never herd (q)", (0.0, 1.0, 0.01), Reset)
            .with_help("Alfarano & Milaković: a few such ants on the network calm the whole colony."),
        Param::integer("Stopping", "stop_at", "Stop at step", (0, 10_000_000), Live)
            .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_kirman_s_figure_ii_b() {
        let c = AntsConfig::default();
        assert_eq!(
            (c.ants, c.epsilon, c.delta, c.meetings, c.sources),
            (100, 0.002, 0.01, 50, 2)
        );
        assert_eq!((c.rule, c.network), (Rule::Kirman, Network::Complete));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = AntsConfig {
            ants: 1,
            epsilon: 1.5,
            delta: -0.1,
            meetings: 0,
            sources: 7,
            pull: -1.0,
            a: -1.0,
            network: Network::Ring,
            degree: 3,
            link: 0.0,
            independent: 2.0,
            stop_at: 20_000_000,
            ..AntsConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "ants",
                "epsilon",
                "delta",
                "meetings",
                "sources",
                "pull",
                "a",
                "degree",
                "link",
                "independent",
                "stop_at"
            ]
        );
    }

    #[test]
    fn the_footnote_needs_delta_at_least_epsilon_and_am_two_sources() {
        let f = AntsConfig {
            conversion: Conversion::Footnote,
            epsilon: 0.1,
            delta: 0.05,
            ..AntsConfig::default()
        };
        assert_eq!(f.validate().unwrap_err()[0].field, "delta");
        let ok = AntsConfig {
            delta: 0.3,
            ..f.clone()
        };
        assert!(ok.validate().is_ok());
        // δ = 1 − γ + γε.
        let g = ok.gamma();
        assert!((1.0 - g + g * ok.epsilon - ok.delta).abs() < 1e-12);
        let am = AntsConfig {
            rule: Rule::Alfarano,
            sources: 3,
            ..AntsConfig::default()
        };
        assert_eq!(am.validate().unwrap_err()[0].field, "sources");
        let degree_off_ring = AntsConfig {
            degree: 3,
            ..AntsConfig::default()
        };
        assert!(
            degree_off_ring.validate().is_ok(),
            "degree unused on complete"
        );
    }

    #[test]
    fn the_network_changes_only_on_reset() {
        let next = AntsConfig {
            network: Network::Ring,
            epsilon: 0.1,
            ..AntsConfig::default()
        };
        let changes = AntsConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "network");
    }

    /// Whether the panel shows `p` for `c`: every condition holds.
    fn shown(p: &Param, c: &AntsConfig) -> bool {
        let json = serde_json::to_value(c).unwrap();
        let holds = |cond: &Option<crate::schema::ShowIf>| {
            cond.is_none_or(|cond| {
                let v = cond.path.split('.').fold(&json, |v, key| &v[key]);
                let s = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                s == cond.equals
            })
        };
        holds(&p.show_if) && holds(&p.also_if)
    }

    #[test]
    fn fields_show_only_under_their_rule() {
        let k = AntsConfig::default();
        let am = AntsConfig {
            rule: Rule::Alfarano,
            ..AntsConfig::default()
        };
        for p in schema() {
            match p.group {
                "Kirman" => assert!(!shown(&p, &am), "{} under Alfarano", p.path),
                "Alfarano & Milaković" => assert!(!shown(&p, &k), "{} under Kirman", p.path),
                _ => {}
            }
        }
        let link = schema().into_iter().find(|p| p.path == "link").unwrap();
        assert!(!shown(&link, &k));
        assert!(shown(
            &link,
            &AntsConfig {
                network: Network::Random,
                ..k
            }
        ));
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Ants(AntsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
