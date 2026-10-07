use super::*;
use super::{draws::*, knowledge::*, metrics::*, observation::*};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};
mod setup_learning;
fn pos(x: u32, y: u32) -> Pos {
    Pos { x, y }
}
pub(super) fn setup() -> Setup {
    Setup {
        width: 5,
        height: 5,
        open: vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(3, 0), pos(3, 1)],
        nest: vec![pos(0, 0), pos(1, 0)],
        workers: vec![pos(0, 0)],
        resources: vec![],
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

mod navigation;

mod ledger_server;

mod controller;

mod runner;
