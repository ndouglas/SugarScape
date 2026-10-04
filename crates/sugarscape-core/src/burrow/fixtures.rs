//! Supplied geometry and researcher-seeded material, distinct from excavation.

use super::{
    config::{checked_cells, checked_workers},
    Fixture, InitialSpoil, LabConfig, Pile, Pos, Setup, Side,
};
use crate::config::FieldError;

pub fn corridor_setup(length: u32, workers: u32) -> Result<Setup, Vec<FieldError>> {
    let config = LabConfig {
        fixture: Fixture::Corridor { length, workers },
        ..Default::default()
    };
    config.validate()?;
    let open: Vec<_> = (0..length).map(|x| Pos { x, y: 1 }).collect();
    let spawns = (0..workers)
        .map(|i| Pos {
            x: length - 1 - i % length,
            y: 1,
        })
        .collect();
    Ok(Setup {
        width: length + 2,
        height: 3,
        exit: Pos { x: 0, y: 1 },
        start_tick: 0,
        open,
        diggable: vec![Pos { x: length, y: 1 }],
        workers: spawns,
        spoil: Vec::new(),
    })
}

pub(super) fn setup(config: &LabConfig) -> Result<Setup, Vec<FieldError>> {
    config.validate()?;
    match config.fixture {
        Fixture::Corridor { length, workers } => corridor_setup(length, workers),
        Fixture::Growing {
            width,
            height,
            workers,
        } => {
            checked_cells(width, height, "fixture").map_err(|e| vec![e])?;
            checked_workers(workers, 28).map_err(|e| vec![e])?;
            let exit = Pos { x: 0, y: 12 };
            let open: Vec<_> = (10..=14)
                .flat_map(|y| (0..=2).map(move |x| Pos { x, y }))
                .collect();
            let spawn: Vec<_> = open.iter().copied().filter(|p| *p != exit).collect();
            let workers = (0..workers)
                .map(|i| spawn[i as usize % spawn.len()])
                .collect();
            let diggable = (0..height)
                .flat_map(|y| (0..width).map(move |x| Pos { x, y }))
                .collect();
            Ok(Setup {
                width,
                height,
                exit,
                start_tick: 0,
                open,
                diggable,
                workers,
                spoil: Vec::new(),
            })
        }
        Fixture::Choice { side, pile } => {
            let start_tick = config.freshness_window * 2;
            let pos = Pos {
                x: if side == Side::Left { 2 } else { 6 },
                y: 3,
            };
            let born = if pile == Pile::OldAccumulation {
                0
            } else {
                start_tick
            };
            let count = if pile == Pile::SingleFresh { 1 } else { 4 };
            Ok(Setup {
                width: 9,
                height: 7,
                exit: Pos { x: 4, y: 3 },
                start_tick,
                open: (2..=6).map(|x| Pos { x, y: 3 }).collect(),
                diggable: vec![Pos { x: 1, y: 3 }, Pos { x: 7, y: 3 }],
                workers: vec![Pos { x: 4, y: 3 }],
                spoil: vec![InitialSpoil { pos, born }; count],
            })
        }
    }
}
