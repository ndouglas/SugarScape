//! The Relative Agreement model's parameters: Deffuant et al.'s (2002)
//! relative agreement and the three bounded-confidence rules they compare
//! it with, extremists, the networks of Deffuant et al. (2000), Amblard and
//! Deffuant (2004) and Weisbuch (2004), and the readings the papers leave
//! open (Meadows and Cliff's, the 2013 reply's, eq. 11's window) as switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::opinions::Neighborhood;
use crate::schema::{Apply, Param};

/// How one agent influences another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Relative agreement (DAWF eqs. 1–6).
    Ra,
    /// Bounded confidence (DAWF eq. 11; DNAW).
    Bc,
    /// Bounded confidence with averaging uncertainties (eqs. 11–12).
    BcAveraging,
    /// Bounded confidence with uncertainty from variance (eqs. 13–14).
    BcVariance,
}

/// Whose uncertainty the bounded-confidence rules' window uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    /// Eq. 11 as printed: |x − x′| < u′, the influencer's.
    Influencer,
    /// The listener's own: |x − x′| < u (RA's hᵢⱼ > uᵢ when uᵢ < uⱼ).
    Listener,
}

/// Where the extremists start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// DAWF: N uniform draws; the most extreme take ue.
    Drawn,
    /// AD: the same agents, set to ±1.
    Bounds,
    /// M&C: extremists uniform in [b, 1] and [−1, −b], moderates on (−b, b).
    Band,
}

/// How the two agents of a meeting update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairUpdate {
    /// Both from their old values (`melsimp.c`, M&C).
    Simultaneous,
    /// The first acts on the second, then the second, updated, on the first.
    Sequential,
    /// Only the first updates (Weisbuch 2004).
    OneWay,
}

/// Who can meet whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Anyone (DNAW §2, DAWF).
    All,
    /// A torus (DNAW §3, AD §3.1).
    Lattice,
    /// Watts–Strogatz (AD §3.2).
    SmallWorld,
    /// Barabási–Albert (Weisbuch).
    ScaleFree,
}

/// How a meeting's pair is drawn on a network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// A uniform link, its order uniform (DNAW, AD).
    Edge,
    /// A uniform agent, then a uniform neighbor (Weisbuch).
    Node,
}

/// A small world's regular start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Substrate {
    /// A circle, k/2 neighbors each side (AD Fig. 4).
    Ring,
    /// The lattice's torus with a Moore neighborhood of radius r (AD Fig. 6).
    Grid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
}

impl Default for LatticeConfig {
    /// DNAW's 29 × 29 square lattice, four neighbors each.
    fn default() -> Self {
        LatticeConfig {
            width: 29,
            height: 29,
            neighborhood: Neighborhood::VonNeumann,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SmallWorldConfig {
    pub substrate: Substrate,
    /// k: neighbors each agent starts with.
    pub degree: u32,
    /// p: the chance each link is rewired.
    pub rewire: f64,
}

impl Default for SmallWorldConfig {
    fn default() -> Self {
        SmallWorldConfig {
            substrate: Substrate::Ring,
            degree: 8,
            rewire: 0.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScaleFreeConfig {
    /// m: links per new node (mean degree 2m).
    pub links: u32,
}

impl Default for ScaleFreeConfig {
    /// Weisbuch's two links per new node (mean degree 4).
    fn default() -> Self {
        ScaleFreeConfig { links: 2 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgreementConfig {
    /// N; under a lattice or a grid-substrate small world, width × height.
    pub agents: u32,
    pub rule: Rule,
    pub window: Window,
    /// μ.
    pub mu: f64,
    /// α under `bc_variance`.
    pub alpha: f64,
    /// U: the moderates' starting uncertainty.
    pub uncertainty: f64,
    /// pe: the proportion of extremists.
    pub extremists: f64,
    /// ue.
    pub extremist_uncertainty: f64,
    /// δ = |p₊ − p₋| / pe; the surplus is positive.
    pub delta: f64,
    pub placement: Placement,
    /// b under `band`.
    pub band: f64,
    /// A moderate is a new extremist beyond its side's boundary minus this.
    pub extreme_margin: f64,
    pub pair_update: PairUpdate,
    pub network: Network,
    pub lattice: LatticeConfig,
    pub small_world: SmallWorldConfig,
    pub scale_free: ScaleFreeConfig,
    pub pairing: Pairing,
    /// Stop at the first stable period.
    pub stop_when_stable: bool,
    /// Stop at this period (0: never): the cap under stability.
    pub stop_at: u32,
}

impl Default for AgreementConfig {
    /// Relative agreement with extremists at Fig. 9's μ and ue: N 200, U 1,
    /// pe 0.1, the extremists drawn, run to stability.
    fn default() -> Self {
        AgreementConfig {
            agents: 200,
            rule: Rule::Ra,
            window: Window::Influencer,
            mu: 0.2,
            alpha: 0.8,
            uncertainty: 1.0,
            extremists: 0.1,
            extremist_uncertainty: 0.1,
            delta: 0.0,
            placement: Placement::Drawn,
            band: 0.8,
            extreme_margin: 0.1,
            pair_update: PairUpdate::Simultaneous,
            network: Network::All,
            lattice: LatticeConfig::default(),
            small_world: SmallWorldConfig::default(),
            scale_free: ScaleFreeConfig::default(),
            pairing: Pairing::Edge,
            stop_when_stable: true,
            stop_at: 20_000,
        }
    }
}

/// The largest lattice side.
pub const MAX_SIDE: u32 = 64;
/// The most agents.
pub const MAX_AGENTS: u32 = 4000;

impl AgreementConfig {
    /// Whether agents sit on the lattice's torus (a lattice, or a small world
    /// grown from it).
    pub fn on_torus(&self) -> bool {
        self.network == Network::Lattice
            || (self.network == Network::SmallWorld
                && self.small_world.substrate == Substrate::Grid)
    }

    /// The number of agents: width × height on the torus, else `agents`.
    pub fn population(&self) -> usize {
        if self.on_torus() {
            self.lattice.width as usize * self.lattice.height as usize
        } else {
            self.agents as usize
        }
    }

    /// The grid substrate's Moore radius for degree k = (2r + 1)² − 1, if k
    /// is of that form.
    pub fn grid_radius(&self) -> Option<u32> {
        (1..=MAX_SIDE / 2).find(|r| (2 * r + 1) * (2 * r + 1) - 1 == self.small_world.degree)
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
            (2..=MAX_AGENTS).contains(&self.agents),
            "agents",
            "must be between 2 and 4000",
        );
        check(unit(self.mu), "mu", "must be between 0 and 1");
        check(unit(self.alpha), "alpha", "must be between 0 and 1");
        check(
            self.uncertainty > 0.0 && self.uncertainty <= 4.0,
            "uncertainty",
            "must be above 0 and at most 4",
        );
        check(
            unit(self.extremists),
            "extremists",
            "must be between 0 and 1",
        );
        check(
            self.extremist_uncertainty > 0.0 && self.extremist_uncertainty <= 4.0,
            "extremist_uncertainty",
            "must be above 0 and at most 4",
        );
        check(unit(self.delta), "delta", "must be between 0 and 1");
        check(
            self.band > 0.0 && self.band < 1.0,
            "band",
            "must be between 0 and 1 (exclusive)",
        );
        check(
            unit(self.extreme_margin),
            "extreme_margin",
            "must be between 0 and 1",
        );
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if self.on_torus() {
            let l = &self.lattice;
            check(
                (3..=MAX_SIDE).contains(&l.width) && (3..=MAX_SIDE).contains(&l.height),
                "lattice",
                "sides must be between 3 and 64",
            );
        }
        let n = self.population() as u64;
        match self.network {
            Network::SmallWorld => {
                let s = &self.small_world;
                check(
                    unit(s.rewire),
                    "small_world",
                    "rewire must be between 0 and 1",
                );
                match s.substrate {
                    Substrate::Ring => check(
                        s.degree >= 2
                            && s.degree <= 256
                            && s.degree.is_multiple_of(2)
                            && u64::from(s.degree) < n,
                        "small_world",
                        "degree must be even, between 2 and 256, and below the number of agents",
                    ),
                    Substrate::Grid => {
                        let fits = self.grid_radius().is_some_and(|r| {
                            2 * r < self.lattice.width && 2 * r < self.lattice.height
                        });
                        check(
                            fits,
                            "small_world",
                            "on a grid the degree must be (2r + 1)² − 1 (8, 24, 48 …) with 2r below each side",
                        );
                    }
                }
            }
            Network::ScaleFree => {
                let m = self.scale_free.links;
                check(
                    (1..=8).contains(&m) && u64::from(m) + 2 <= n,
                    "scale_free",
                    "links must be between 1 and 8, with at least links + 2 agents",
                );
            }
            Network::All | Network::Lattice => {}
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &AgreementConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("uncertainty", self.uncertainty == next.uncertainty),
            ("extremists", self.extremists == next.extremists),
            (
                "extremist_uncertainty",
                self.extremist_uncertainty == next.extremist_uncertainty,
            ),
            ("delta", self.delta == next.delta),
            ("placement", self.placement == next.placement),
            ("band", self.band == next.band),
            ("network", self.network == next.network),
            ("lattice", self.lattice == next.lattice),
            ("small_world", self.small_world == next.small_world),
            ("scale_free", self.scale_free == next.scale_free),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }

    /// Whether `next` changes how agents move (a stable run then resumes).
    pub(crate) fn moves_differently(&self, next: &AgreementConfig) -> bool {
        self.rule != next.rule
            || self.window != next.window
            || self.mu != next.mu
            || self.alpha != next.alpha
            || self.pair_update != next.pair_update
            || self.pairing != next.pairing
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Population", "agents", "Agents (N)", (2, MAX_AGENTS), Reset)
            .with_help("DAWF: 200 (Figs. 5–8) or 1000 (Fig. 9); Weisbuch: 900. On a lattice, or a small world grown on one, width × height instead."),
        Param::number(
            "Population",
            "uncertainty",
            "Moderates' uncertainty (U)",
            (0.05, 4.0, 0.05),
            Reset,
        )
        .with_help("Opinions run from −1 to 1. Deffuant 2000's threshold d on [0, 1] is U = 2d here."),
        Param::choice(
            "Interaction",
            "rule",
            "Rule",
            &[
                ("ra", "Relative agreement (DAWF)"),
                ("bc", "Bounded confidence (eq. 11)"),
                ("bc_averaging", "BC, averaging uncertainties (eq. 12)"),
                ("bc_variance", "BC, uncertainty from variance (eqs. 13–14)"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "window",
            "BC window",
            &[
                ("influencer", "The influencer's uncertainty (eq. 11 as printed)"),
                ("listener", "The listener's own uncertainty"),
            ],
            Live,
        )
        .with_help("Eq. 11 reads |x − x′| < u′, the influencer's; §6's results need the listener's (measured). Unused by relative agreement."),
        Param::number("Interaction", "mu", "Speed (μ)", (0.0, 1.0, 0.01), Live),
        Param::number("Interaction", "alpha", "Memory (α)", (0.0, 1.0, 0.01), Live)
            .shown_if("rule", "bc_variance")
            .with_help("Eqs. 13–14's α; DAWF give no value (0.8 = 1 − μ here)."),
        Param::choice(
            "Interaction",
            "pair_update",
            "A meeting updates",
            &[
                ("simultaneous", "Both, from their old values"),
                ("sequential", "One, then the other"),
                ("one_way", "Only the first (Weisbuch)"),
            ],
            Live,
        )
        .with_help("DAWF do not say; melsimp.c and Meadows & Cliff update both from the old values."),
        Param::number(
            "Extremists",
            "extremists",
            "Extremists (pe)",
            (0.0, 1.0, 0.0125),
            Reset,
        ),
        Param::number(
            "Extremists",
            "extremist_uncertainty",
            "Their uncertainty (ue)",
            (0.01, 1.0, 0.01),
            Reset,
        ),
        Param::number("Extremists", "delta", "Lean (δ)", (0.0, 1.0, 0.05), Reset)
            .with_help("δ = |p₊ − p₋| / pe; the surplus goes to +1."),
        Param::choice(
            "Extremists",
            "placement",
            "Placement",
            &[
                ("drawn", "The most extreme draws (DAWF)"),
                ("bounds", "Set to ±1 (Amblard & Deffuant)"),
                ("band", "A band at the ends (Meadows & Cliff)"),
            ],
            Reset,
        ),
        Param::number("Extremists", "band", "Band edge (b)", (0.05, 0.95, 0.05), Reset)
            .shown_if("placement", "band"),
        Param::number(
            "Extremists",
            "extreme_margin",
            "New-extremist margin",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("y counts a moderate as a new extremist beyond its side's boundary minus this: 0.1 in Deffuant et al.'s 2013 reply, 0 in Meadows & Cliff."),
        Param::choice(
            "Network",
            "network",
            "Who meets whom",
            &[
                ("all", "Anyone"),
                ("lattice", "Lattice neighbors"),
                ("small_world", "A small world"),
                ("scale_free", "A scale-free network"),
            ],
            Reset,
        ),
        Param::integer("Network", "lattice.width", "Width", (3, MAX_SIDE), Reset)
            .with_help("Also the grid a small world can grow from."),
        Param::integer("Network", "lattice.height", "Height", (3, MAX_SIDE), Reset),
        Param::choice(
            "Network",
            "lattice.neighborhood",
            "Neighbors",
            &[("von_neumann", "4 (von Neumann)"), ("moore", "8 (Moore)")],
            Reset,
        )
        .shown_if("network", "lattice"),
        Param::choice(
            "Network",
            "small_world.substrate",
            "Grown from",
            &[("ring", "A ring"), ("grid", "The lattice")],
            Reset,
        )
        .shown_if("network", "small_world"),
        Param::integer(
            "Network",
            "small_world.degree",
            "Neighbors (k)",
            (2, 256),
            Reset,
        )
        .shown_if("network", "small_world")
        .with_help("Even on a ring; (2r + 1)² − 1 on the lattice (8, 24, 48 …)."),
        Param::number(
            "Network",
            "small_world.rewire",
            "Rewiring (p)",
            (0.0, 1.0, 0.05),
            Reset,
        )
        .shown_if("network", "small_world"),
        Param::integer(
            "Network",
            "scale_free.links",
            "Links per new agent (m)",
            (1, 8),
            Reset,
        )
        .shown_if("network", "scale_free"),
        Param::choice(
            "Network",
            "pairing",
            "Pairs",
            &[
                ("edge", "A random link (Deffuant, Amblard)"),
                ("node", "An agent, then a neighbor (Weisbuch)"),
            ],
            Live,
        )
        .with_help("Unused when anyone meets anyone."),
        Param::bool("Stopping", "stop_when_stable", "Stop when stable", Live)
            .with_help("Stable: no opinion or uncertainty moved more than 10⁻⁶ in a period of N meetings."),
        Param::integer("Stopping", "stop_at", "Stop at period", (0, 1_000_000), Live)
            .with_help("0: never. With the stop when stable, a cap; Meadows & Cliff stop at 200, Deffuant et al.'s reply at 1200."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_relative_agreement_at_fig_9s_speed() {
        let c = AgreementConfig::default();
        assert_eq!((c.agents, c.rule, c.mu), (200, Rule::Ra, 0.2));
        assert_eq!(
            (c.uncertainty, c.extremists, c.extremist_uncertainty),
            (1.0, 0.1, 0.1)
        );
        assert_eq!(
            (c.placement, c.extreme_margin, c.pair_update, c.network),
            (
                Placement::Drawn,
                0.1,
                PairUpdate::Simultaneous,
                Network::All
            )
        );
        assert!(c.stop_when_stable && c.stop_at == 20_000 && c.validate().is_ok());
        assert_eq!(c.population(), 200);
    }

    #[test]
    fn validation_names_fields() {
        let bad = AgreementConfig {
            agents: 1,
            mu: 1.5,
            alpha: -0.1,
            uncertainty: 0.0,
            extremists: 2.0,
            extremist_uncertainty: 0.0,
            delta: 1.5,
            band: 1.0,
            extreme_margin: -1.0,
            stop_at: 2_000_000,
            ..AgreementConfig::default()
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
                "agents",
                "mu",
                "alpha",
                "uncertainty",
                "extremists",
                "extremist_uncertainty",
                "delta",
                "band",
                "extreme_margin",
                "stop_at"
            ]
        );
    }

    #[test]
    fn networks_validate_their_own_fields() {
        let ring = |degree| AgreementConfig {
            network: Network::SmallWorld,
            small_world: SmallWorldConfig {
                degree,
                ..SmallWorldConfig::default()
            },
            ..AgreementConfig::default()
        };
        assert!(ring(8).validate().is_ok());
        for bad in [7, 0, 200, 258] {
            assert_eq!(
                ring(bad).validate().unwrap_err()[0].field,
                "small_world",
                "{bad}"
            );
        }
        let grid = |degree, side| AgreementConfig {
            network: Network::SmallWorld,
            lattice: LatticeConfig {
                width: side,
                height: side,
                ..LatticeConfig::default()
            },
            small_world: SmallWorldConfig {
                substrate: Substrate::Grid,
                degree,
                rewire: 0.1,
            },
            ..AgreementConfig::default()
        };
        assert_eq!(grid(24, 29).grid_radius(), Some(2));
        assert!(grid(24, 29).validate().is_ok());
        assert_eq!(
            grid(24, 29).population(),
            841,
            "the grid sets the population"
        );
        assert!(grid(10, 29).validate().is_err(), "not (2r + 1)² − 1");
        assert!(
            grid(48, 5).validate().is_err(),
            "radius 3 wraps a side of 5"
        );
        let lattice = AgreementConfig {
            network: Network::Lattice,
            lattice: LatticeConfig {
                width: 70_000,
                height: 70_000,
                ..LatticeConfig::default()
            },
            ..AgreementConfig::default()
        };
        assert_eq!(lattice.validate().unwrap_err()[0].field, "lattice");
        let sf = |links, agents| AgreementConfig {
            network: Network::ScaleFree,
            agents,
            scale_free: ScaleFreeConfig { links },
            ..AgreementConfig::default()
        };
        assert!(sf(2, 900).validate().is_ok());
        assert!(sf(0, 900).validate().is_err());
        assert!(sf(8, 9).validate().is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Agreement(AgreementConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
