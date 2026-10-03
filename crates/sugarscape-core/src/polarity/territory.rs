//! Fixed cell IDs and structural changes from explicit snapshot claims.
use super::config::*;
use serde::Serialize;
use std::collections::{BTreeSet, VecDeque};
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Cell {
    pub id: usize,
    pub capital: usize,
    pub predator: bool,
    pub stock: f64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Claim {
    pub winner: usize,
    pub loser: usize,
    pub target: usize,
    pub domestic: bool,
    pub source: Option<usize>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Change {
    pub applied: Vec<Claim>,
    pub transfer_volume: f64,
    pub cells: Vec<usize>,
    pub conquests: u64,
    pub collapses: u64,
    pub disconnections: u64,
    pub stale: u64,
    pub locked: u64,
}
pub fn adjacent(config: &PolarityConfig, id: usize) -> Vec<usize> {
    let (w, h) = (config.width as usize, config.height as usize);
    let (x, y) = (id % w, id / w);
    let mut out = BTreeSet::new();
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let (mut nx, mut ny) = (x as i32 + dx, y as i32 + dy);
        if config.topology == Topology::Torus {
            nx = nx.rem_euclid(w as i32);
            ny = ny.rem_euclid(h as i32);
        }
        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
            out.insert(ny as usize * w + nx as usize);
        }
    }
    out.into_iter().collect()
}
pub fn members(cells: &[Cell], capital: usize) -> Vec<usize> {
    cells
        .iter()
        .filter(|c| c.capital == capital)
        .map(|c| c.id)
        .collect()
}
pub fn capitals(cells: &[Cell]) -> Vec<usize> {
    cells
        .iter()
        .filter(|c| c.capital == c.id)
        .map(|c| c.id)
        .collect()
}
pub fn neighbors(config: &PolarityConfig, cells: &[Cell], capital: usize) -> Vec<usize> {
    let mut out = BTreeSet::new();
    for i in members(cells, capital) {
        for j in adjacent(config, i) {
            if cells[j].capital != capital {
                out.insert(cells[j].capital);
            }
        }
    }
    out.into_iter().collect()
}
#[cfg(test)]
pub fn apply(config: &PolarityConfig, cells: &mut [Cell], claims: &[Claim]) -> Change {
    apply_locked(config, cells, claims, &mut BTreeSet::new())
}
pub fn apply_locked(
    config: &PolarityConfig,
    cells: &mut [Cell],
    claims: &[Claim],
    locks: &mut BTreeSet<usize>,
) -> Change {
    let mut result = Change::default();
    for claim in claims {
        let Claim {
            winner,
            loser,
            target,
            domestic,
            source,
        } = *claim;
        if source
            .is_some_and(|i| cells[i].capital != winner || !adjacent(config, i).contains(&target))
            || cells[target].capital != loser
            || cells[loser].capital != loser
            || (!domestic && cells[winner].capital != winner)
        {
            result.stale += 1;
            continue;
        }
        let old = members(cells, loser);
        let winner_members = members(cells, winner);
        let share = cells[loser].stock / old.len() as f64;
        let mut candidate = cells.to_vec();
        let mut changed = BTreeSet::from([target]);
        let mut conquests = 0;
        let mut collapses = 0;
        let mut disconnected = 0;
        let mut transfer_volume = 0.0;
        if domestic {
            // A province wins its revolt; victory by the center creates no structural claim.
            candidate[target].capital = target;
        } else if target == loser && old.len() > 1 {
            collapses = 1;
            if !config.provincial() {
                transfer_volume += share * (old.len() - 1) as f64;
            }
            for &i in &old {
                candidate[i].capital = i;
                if !config.provincial() {
                    candidate[i].stock = share;
                }
                changed.insert(i);
            }
            if config.capital_capture == CapitalCapture::CaptureAndFragment {
                let stock = candidate[target].stock;
                transfer_volume += stock;
                candidate[target].capital = winner;
                candidate[winner].stock += stock;
                candidate[target].stock = 0.0;
                conquests = 1;
            }
        } else {
            candidate[target].capital = winner;
            conquests = 1;
            if !config.provincial() {
                let transfer = if old.len() == 1 {
                    candidate[loser].stock
                } else if config.province_transfer == ProvinceTransfer::EqualShare {
                    share
                } else {
                    candidate[target].stock
                };
                transfer_volume += transfer;
                candidate[loser].stock -= transfer;
                candidate[winner].stock += transfer;
                candidate[target].stock = 0.0;
            }
        }
        if candidate[loser].capital == loser {
            let mut reached = BTreeSet::from([loser]);
            let mut queue = VecDeque::from([loser]);
            while let Some(i) = queue.pop_front() {
                for j in adjacent(config, i) {
                    if candidate[j].capital == loser && reached.insert(j) {
                        queue.push_back(j);
                    }
                }
            }
            for &i in &old {
                if candidate[i].capital == loser && !reached.contains(&i) {
                    candidate[i].capital = i;
                    if !config.provincial() {
                        candidate[i].stock = share;
                        candidate[loser].stock -= share;
                        transfer_volume += share;
                    }
                    changed.insert(i);
                    disconnected += 1;
                }
            }
        }
        for i in 0..cells.len() {
            if candidate[i] != cells[i] {
                changed.insert(i);
            }
        }
        if config.locking == Locking::AffectedStates {
            changed.extend(old);
            changed.extend(winner_members);
        }
        if changed.iter().any(|i| locks.contains(i)) {
            result.locked += 1;
            continue;
        }
        cells.clone_from_slice(&candidate);
        locks.extend(changed.iter().copied());
        result.cells.extend(changed);
        result.applied.push(claim.clone());
        result.transfer_volume += transfer_volume;
        result.conquests += conquests;
        result.collapses += collapses;
        result.disconnections += disconnected;
    }
    result.cells.sort_unstable();
    result.cells.dedup();
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    fn cell(id: usize, capital: usize, stock: f64) -> Cell {
        Cell {
            id,
            capital,
            stock,
            predator: false,
        }
    }
    #[test]
    fn capital_fall_releases_every_cell_without_annexation_and_conserves_stock() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..Default::default()
        };
        let mut cells = vec![
            cell(0, 0, 90.0),
            cell(1, 0, 0.0),
            cell(2, 0, 0.0),
            cell(3, 3, 50.0),
            cell(4, 4, 10.0),
            cell(5, 5, 10.0),
        ];
        let result = apply(
            &c,
            &mut cells,
            &[Claim {
                winner: 3,
                loser: 0,
                target: 0,
                domestic: false,
                source: None,
            }],
        );
        assert_eq!(
            cells
                .iter()
                .take(3)
                .map(|x| (x.capital, x.stock))
                .collect::<Vec<_>>(),
            vec![(0, 30.0), (1, 30.0), (2, 30.0)]
        );
        assert_eq!(result.collapses, 1);
        assert_eq!(cells.iter().map(|x| x.stock).sum::<f64>(), 160.0);
    }
    #[test]
    fn province_capture_and_disconnected_enclave_debit_equal_shares_once() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..Default::default()
        };
        let mut cells = vec![
            cell(0, 0, 90.0),
            cell(1, 0, 0.0),
            cell(2, 0, 0.0),
            cell(3, 3, 50.0),
            cell(4, 4, 10.0),
            cell(5, 5, 10.0),
        ];
        let result = apply(
            &c,
            &mut cells,
            &[Claim {
                winner: 4,
                loser: 0,
                target: 1,
                domestic: false,
                source: None,
            }],
        );
        assert_eq!(
            (cells[0].stock, cells[4].stock, cells[2].stock),
            (30.0, 40.0, 30.0)
        );
        assert_eq!(result.disconnections, 1);
        assert_eq!(cells.iter().map(|x| x.stock).sum::<f64>(), 160.0);
    }
    #[test]
    fn provincial_collapse_preserves_individual_stocks() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..PolarityConfig::for_variant(Variant::TwoLevel)
        };
        let mut cells = vec![
            cell(0, 0, 90.0),
            cell(1, 0, -5.0),
            cell(2, 0, 20.0),
            cell(3, 3, 50.0),
            cell(4, 4, 10.0),
            cell(5, 5, 10.0),
        ];
        apply(
            &c,
            &mut cells,
            &[Claim {
                winner: 3,
                loser: 0,
                target: 0,
                domestic: false,
                source: None,
            }],
        );
        assert_eq!(
            cells.iter().take(3).map(|x| x.stock).collect::<Vec<_>>(),
            vec![90.0, -5.0, 20.0]
        );
    }
    #[test]
    fn torus_deduplicates_neighbors_on_two_cell_sides() {
        let c = PolarityConfig {
            width: 2,
            height: 2,
            topology: Topology::Torus,
            ..Default::default()
        };
        assert_eq!(adjacent(&c, 0), vec![1, 2]);
    }
}
#[cfg(test)]
mod structural_motifs {
    use super::*;
    fn cells() -> Vec<Cell> {
        vec![
            Cell {
                id: 0,
                capital: 0,
                predator: false,
                stock: 90.0,
            },
            Cell {
                id: 1,
                capital: 0,
                predator: true,
                stock: 0.0,
            },
            Cell {
                id: 2,
                capital: 0,
                predator: false,
                stock: 0.0,
            },
            Cell {
                id: 3,
                capital: 3,
                predator: false,
                stock: 50.0,
            },
            Cell {
                id: 4,
                capital: 4,
                predator: false,
                stock: 10.0,
            },
            Cell {
                id: 5,
                capital: 5,
                predator: false,
                stock: 10.0,
            },
        ]
    }
    fn config() -> PolarityConfig {
        PolarityConfig {
            width: 3,
            height: 2,
            ..Default::default()
        }
    }
    fn claim(target: usize) -> Claim {
        Claim {
            winner: 3,
            loser: 0,
            target,
            domestic: false,
            source: None,
        }
    }
    #[test]
    fn capture_and_fragment_annexes_only_capital_and_conserves_stock() {
        let mut c = config();
        c.capital_capture = CapitalCapture::CaptureAndFragment;
        let mut cells = cells();
        let r = apply(&c, &mut cells, &[claim(0)]);
        assert_eq!(
            (cells[0].capital, cells[0].stock, cells[3].stock),
            (3, 0.0, 80.0)
        );
        assert_eq!(r.collapses, 1);
        assert_eq!(cells.iter().map(|c| c.stock).sum::<f64>(), 160.0);
    }
    #[test]
    fn stale_snapshot_claim_cannot_capture_a_changed_target() {
        let mut cells = cells();
        let r = apply(&config(), &mut cells, &[claim(1), claim(1)]);
        assert_eq!(r.stale, 1);
        assert_eq!(r.conquests, 1);
    }
    #[test]
    fn affected_capital_lock_prevents_second_stock_transfer() {
        let mut cells = cells();
        let r = apply(&config(), &mut cells, &[claim(2), claim(1)]);
        assert_eq!(r.locked, 1);
        assert_eq!(r.conquests, 1);
    }
    #[test]
    fn source_cell_ownership_is_part_of_snapshot_identity() {
        let mut cells = cells();
        let mut claim = claim(1);
        claim.source = Some(4);
        let r = apply(&config(), &mut cells, &[claim]);
        assert_eq!(r.stale, 1);
        assert_eq!(cells[1].capital, 0);
    }
    #[test]
    fn primitive_stock_only_does_not_take_corporate_share() {
        let mut c = config();
        c.province_transfer = ProvinceTransfer::PrimitiveStockOnly;
        let mut cells = cells();
        apply(&c, &mut cells, &[claim(2)]);
        assert_eq!((cells[0].stock, cells[3].stock), (90.0, 50.0));
    }
    #[test]
    fn negative_corporate_stock_splits_without_floors() {
        let mut cells = cells();
        cells[0].stock = -90.0;
        apply(&config(), &mut cells, &[claim(0)]);
        assert_eq!(
            cells.iter().take(3).map(|c| c.stock).collect::<Vec<_>>(),
            vec![-30.0, -30.0, -30.0]
        );
    }
    #[test]
    fn primitive_provincial_capture_preserves_stock_at_new_province() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..PolarityConfig::for_variant(Variant::TwoLevel)
        };
        let mut cells = cells();
        cells[1].capital = 1;
        cells[1].stock = 25.0;
        let q = Claim {
            winner: 0,
            loser: 1,
            target: 1,
            domestic: false,
            source: Some(0),
        };
        apply(&c, &mut cells, &[q]);
        assert_eq!(
            (cells[1].capital, cells[1].stock, cells[0].stock),
            (0, 25.0, 90.0)
        );
    }
    #[test]
    fn corporate_provincial_capital_capture_transfers_only_center_stock() {
        let mut c = PolarityConfig {
            width: 3,
            height: 2,
            ..PolarityConfig::for_variant(Variant::TwoLevel)
        };
        c.capital_capture = CapitalCapture::CaptureAndFragment;
        let mut cells = cells();
        cells[1].stock = -5.0;
        cells[2].stock = 20.0;
        apply(&c, &mut cells, &[claim(0)]);
        assert_eq!(
            (cells[3].stock, cells[1].stock, cells[2].stock),
            (140.0, -5.0, 20.0)
        );
    }
    #[test]
    fn successful_revolt_restores_latent_type_and_preserves_provincial_stock() {
        let c = PolarityConfig {
            width: 3,
            height: 2,
            ..PolarityConfig::for_variant(Variant::TwoLevel)
        };
        let mut cells = cells();
        cells[1].stock = 25.0;
        apply(
            &c,
            &mut cells,
            &[Claim {
                winner: 1,
                loser: 0,
                target: 1,
                domestic: true,
                source: None,
            }],
        );
        assert_eq!(
            (cells[1].capital, cells[1].stock, cells[1].predator),
            (1, 25.0, true)
        );
        assert_eq!(cells[2].capital, 2);
    }
}
