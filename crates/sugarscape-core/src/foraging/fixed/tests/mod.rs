use super::draws::DrawSource;
use super::{Pos, Setup};
use crate::config::FieldError;

mod setup_movement;

pub(super) fn setup() -> Setup {
    Setup {
        width: 5,
        height: 5,
        nest: Pos { x: 2, y: 2 },
        agents: 1,
        resources: vec![],
        parameters: crate::foraging::CpfaParameters {
            p_search: 1.0,
            p_return: 0.0,
            omega: 0.0,
            lambda_informed: 0.0,
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
    fn uniform(&mut self) -> Result<f64, Vec<FieldError>> {
        let value = self
            .values
            .get(self.next)
            .copied()
            .ok_or_else(|| vec![FieldError::new("draw", "script exhausted")])?;
        self.next += 1;
        if !value.is_finite() || !(0.0..1.0).contains(&value) {
            return Err(vec![FieldError::new("draw", "must be finite in [0,1)")]);
        }
        Ok(value)
    }
}

mod ledger;

mod server;
