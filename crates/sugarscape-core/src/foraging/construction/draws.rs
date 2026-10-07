use super::{Checked, Pos};
use crate::config::FieldError;
use rand::Rng;
// Staged Tasks 2–4 controller draws.
#[allow(dead_code)]
pub(super) trait DrawSource {
    fn uniform(&mut self) -> Checked<f64>;
}
#[allow(dead_code)]
pub(super) struct PcgDraws<'a>(pub(super) &'a mut crate::rng::SimRng);
impl DrawSource for PcgDraws<'_> {
    fn uniform(&mut self) -> Checked<f64> {
        Ok(self.0.gen::<f64>())
    }
}
#[allow(dead_code)]
pub(super) fn checked_uniform(draws: &mut impl DrawSource) -> Checked<f64> {
    let value = draws.uniform()?;
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        return Err(vec![FieldError::new("draw", "must be finite in [0,1)")]);
    }
    Ok(value)
}
#[allow(dead_code)]
pub(super) fn draw_index(draws: &mut impl DrawSource, length: u32) -> Checked<u32> {
    if length == 0 {
        return Err(vec![FieldError::new("length", "must be positive")]);
    }
    Ok((checked_uniform(draws)? * f64::from(length)).floor() as u32)
}
#[allow(dead_code)]
pub(super) fn choose(candidates: &[Pos], draws: &mut impl DrawSource) -> Checked<Option<Pos>> {
    if candidates.is_empty() {
        return Ok(None);
    }
    let length = u32::try_from(candidates.len())
        .map_err(|_| vec![FieldError::new("length", "exceeds u32")])?;
    Ok(Some(candidates[draw_index(draws, length)? as usize]))
}
