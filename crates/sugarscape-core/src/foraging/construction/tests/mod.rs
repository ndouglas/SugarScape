use super::{draws::*, knowledge::*, observation::*, terrain::*, *};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};
mod setup_learning;
fn pos(x: u32, y: u32) -> Pos {
    Pos { x, y }
}
fn setup() -> Setup {
    Setup {
        width: 5,
        height: 3,
        open: vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(0, 1)],
        diggable: vec![pos(3, 0)],
        nest: vec![pos(0, 0), pos(1, 0)],
        waste: pos(0, 1),
        workers: vec![pos(0, 0)],
        food: vec![Resource {
            id: u64::MAX,
            pos: pos(3, 0),
        }],
        parameters: Parameters {
            p_search: 1.0,
            p_return: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
#[derive(Clone)]
struct Scripted {
    values: Vec<f64>,
    next: usize,
}
impl Scripted {
    fn new(values: &[f64]) -> Self {
        Self {
            values: values.to_vec(),
            next: 0,
        }
    }
}
impl DrawSource for Scripted {
    fn uniform(&mut self) -> Checked<f64> {
        let value = self
            .values
            .get(self.next)
            .copied()
            .ok_or_else(|| vec![FieldError::new("draw", "script exhausted")])?;
        self.next += 1;
        Ok(value)
    }
}
