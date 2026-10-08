//! Exact summaries of matched actual outcomes, separate from Agent priors.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedComparison {
    pub experimenter: Role,
    pub environment: Environment,
    pub ids: Ids,
    pub left_policy: PolicyKind,
    pub right_policy: PolicyKind,
    /// Left minus right, evaluated on each of the same 256 actual sequences.
    pub difference: Aggregate,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualMechanismWeight {
    pub model: Mechanism,
    pub weight: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UniformMechanismSummary {
    pub protocol: Protocol,
    /// An outer distribution of actual mechanisms, never an Agent's OwnPrior.
    /// For Known, this is a mixture of four separately informed point controls.
    pub actual_mechanism_distribution: [ActualMechanismWeight; 4],
    pub aggregate: Aggregate,
}

fn indexed(panel: &Panel) -> Result<BTreeMap<u16, &Episode>, Error> {
    panel.protocol.validate()?;
    if panel.episodes.len() != 256 {
        return Err(Error::InvalidReport(
            "summary requires all 256 sequences".into(),
        ));
    }
    let mut episodes = BTreeMap::new();
    for episode in &panel.episodes {
        if episode.sequence >= 256 || episodes.insert(episode.sequence, episode).is_some() {
            return Err(Error::InvalidReport(
                "summary requires unique sequences 0..255".into(),
            ));
        }
    }
    Ok(episodes)
}
#[derive(Default)]
struct Totals {
    rows: u64,
    failed: u64,
    spent: [i64; 2],
    reward: [i64; 2],
    net: [i64; 2],
}
fn add(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_add(right).ok_or(Error::ArithmeticOverflow)
}
fn subtract(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_sub(right).ok_or(Error::ArithmeticOverflow)
}
fn terminal(value: Option<i64>) -> Result<i64, Error> {
    value.ok_or_else(|| Error::InvalidReport("complete episode lacks terminal metric".into()))
}
impl Totals {
    fn record(&mut self, left: &Episode, right: Option<&Episode>) -> Result<(), Error> {
        self.rows += 1;
        let failed = left.failure.is_some() || right.is_some_and(|e| e.failure.is_some());
        self.failed += u64::from(failed);
        for i in 0..2 {
            let a = &left.metrics[i];
            let b = right.map(|e| &e.metrics[i]);
            self.spent[i] = add(
                self.spent[i],
                i64::from(a.spent) - b.map_or(0, |m| i64::from(m.spent)),
            )?;
            if !failed {
                self.reward[i] = add(
                    self.reward[i],
                    subtract(
                        terminal(a.reward)?,
                        b.map(|m| terminal(m.reward)).transpose()?.unwrap_or(0),
                    )?,
                )?;
                self.net[i] = add(
                    self.net[i],
                    subtract(
                        terminal(a.net)?,
                        b.map(|m| terminal(m.net)).transpose()?.unwrap_or(0),
                    )?,
                )?;
            }
        }
        Ok(())
    }
    fn finish(self) -> Result<Aggregate, Error> {
        let fraction =
            |n| -> Result<ScoreFraction, Error> { Ok(ScoreFraction::new(n, self.rows)?) };
        let spent = [fraction(self.spent[0])?, fraction(self.spent[1])?];
        let (reward, net, group_net) = if self.failed == 0 {
            (
                [
                    Some(fraction(self.reward[0])?),
                    Some(fraction(self.reward[1])?),
                ],
                [Some(fraction(self.net[0])?), Some(fraction(self.net[1])?)],
                Some(fraction(add(self.net[0], self.net[1])?)?),
            )
        } else {
            ([None, None], [None, None], None)
        };
        Ok(Aggregate {
            valid_mass: Probability::new(self.rows - self.failed, self.rows)?,
            failed_mass: Probability::new(self.failed, self.rows)?,
            spent,
            reward,
            net,
            group_net,
        })
    }
}

/// Compare policies on identical actual mechanisms, physical roles and IDs.
/// Public cached aggregates are ignored; means are recomputed from paired rows.
pub fn compare_panels(left: &Panel, right: &Panel) -> Result<PairedComparison, Error> {
    if left.environment != right.environment
        || left.protocol.experimenter != right.protocol.experimenter
        || left.protocol.ids != right.protocol.ids
    {
        return Err(Error::InvalidReport(
            "paired panels differ in environment, role, or IDs".into(),
        ));
    }
    let a = indexed(left)?;
    let b = indexed(right)?;
    let mut totals = Totals::default();
    for (sequence, episode) in a {
        totals.record(episode, Some(b[&sequence]))?;
    }
    Ok(PairedComparison {
        experimenter: left.protocol.experimenter,
        environment: left.environment,
        ids: left.protocol.ids.clone(),
        left_policy: left.protocol.policy,
        right_policy: right.protocol.policy,
        difference: totals.finish()?,
    })
}

/// Equal-weight actual-mechanism summary. No cross-role or cross-policy pooling.
pub fn uniform_actual_mechanism_summary(
    panels: &[Panel],
) -> Result<UniformMechanismSummary, Error> {
    if panels.len() != 4 {
        return Err(Error::InvalidReport(
            "uniform mechanism summary requires four catalog panels".into(),
        ));
    }
    let protocol = &panels[0].protocol;
    let mut seen = [false; 4];
    let mut totals = Totals::default();
    for panel in panels {
        if &panel.protocol != protocol {
            return Err(Error::InvalidReport(
                "uniform mechanism summary requires one policy, role, and ID set".into(),
            ));
        }
        let Environment::InFamily(model) = panel.environment else {
            return Err(Error::InvalidReport(
                "DataFlip is outside the actual catalog mixture".into(),
            ));
        };
        if std::mem::replace(&mut seen[model.index()], true) {
            return Err(Error::InvalidReport(
                "duplicate actual mechanism in summary".into(),
            ));
        }
        for episode in indexed(panel)?.into_values() {
            totals.record(episode, None)?;
        }
    }
    Ok(UniformMechanismSummary {
        protocol: protocol.clone(),
        actual_mechanism_distribution: Mechanism::ALL.map(|model| ActualMechanismWeight {
            model,
            weight: Probability::new(1, 4).expect("fixed catalog weight"),
        }),
        aggregate: totals.finish()?,
    })
}
