// Temporary staging allowance: controller consumes these helpers in Task 4.
#![allow(dead_code)]
use crate::config::FieldError;
use rand::Rng;

pub(super) trait DrawSource {
    fn uniform(&mut self) -> Result<f64, Vec<FieldError>>;
}
pub(super) struct PcgDraws<'a>(pub(super) &'a mut crate::rng::SimRng);
impl DrawSource for PcgDraws<'_> {
    fn uniform(&mut self) -> Result<f64, Vec<FieldError>> {
        Ok(self.0.gen::<f64>())
    }
}
/// Check even injected sources at each consumer boundary.
pub(super) fn checked_uniform(draws: &mut impl DrawSource) -> Result<f64, Vec<FieldError>> {
    let value = draws.uniform()?;
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        return Err(vec![FieldError::new("draw", "must be finite in [0,1)")]);
    }
    Ok(value)
}
pub(super) fn draw_index(draws: &mut impl DrawSource, length: u32) -> Result<u32, Vec<FieldError>> {
    if length == 0 {
        return Err(vec![FieldError::new("length", "must be positive")]);
    }
    Ok((checked_uniform(draws)? * f64::from(length)).floor() as u32)
}
