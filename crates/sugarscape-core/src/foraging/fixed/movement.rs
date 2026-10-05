// Temporary staging allowance: controller consumes these helpers in Task 4.
#![allow(dead_code)]
use super::draws::{checked_uniform, draw_index, DrawSource};
use super::{Pos, Setup};
use crate::config::FieldError;
use std::f64::consts::{PI, TAU};

type Checked<T> = Result<T, Vec<FieldError>>;
fn error(field: &str, message: &str) -> Vec<FieldError> {
    vec![FieldError::new(field, message)]
}
fn check_heading(heading: f64) -> Checked<()> {
    if !heading.is_finite() || !(0.0..TAU).contains(&heading) {
        return Err(error("heading", "must be finite in [0,2*pi)"));
    }
    Ok(())
}
/// Historical edge order: top, left, bottom, right. Corners belong to two edges.
pub(super) fn edge_target(setup: &Setup, draws: &mut impl DrawSource) -> Checked<Pos> {
    match draw_index(draws, 4)? {
        0 => Ok(Pos {
            x: draw_index(draws, setup.width)?,
            y: 0,
        }),
        1 => Ok(Pos {
            x: 0,
            y: draw_index(draws, setup.height)?,
        }),
        2 => Ok(Pos {
            x: draw_index(draws, setup.width)?,
            y: setup.height - 1,
        }),
        _ => Ok(Pos {
            x: setup.width - 1,
            y: draw_index(draws, setup.height)?,
        }),
    }
}
pub(super) fn normal_increment(stddev: f64, draws: &mut impl DrawSource) -> Checked<f64> {
    if !stddev.is_finite() || stddev < 0.0 {
        return Err(error("stddev", "must be finite and nonnegative"));
    }
    let u = checked_uniform(draws)?;
    let v = checked_uniform(draws)?;
    Ok((-2.0 * (1.0 - u).ln()).sqrt() * (TAU * v).cos() * stddev)
}
pub(super) fn turn(heading: f64, stddev: f64, draws: &mut impl DrawSource) -> Checked<(f64, u32)> {
    check_heading(heading)?;
    let delta = normal_increment(stddev, draws)?.clamp(-PI, PI);
    let wrapped = (heading + delta).rem_euclid(TAU);
    let new_heading = if wrapped >= TAU { 0.0 } else { wrapped };
    let delay = (delta.abs() / (PI / 4.0 + 0.001)).floor() as u32 + 1;
    Ok((new_heading, delay))
}
/// Rounded endpoints use signed intermediates to preserve outward proposals.
pub(super) fn ahead(setup: &Setup, from: Pos, heading: f64) -> Option<Pos> {
    if check_heading(heading).is_err() {
        return None;
    }
    let x = (f64::from(from.x) + heading.cos()).round() as i64;
    let y = (f64::from(from.y) + heading.sin()).round() as i64;
    if x < 0 || y < 0 || x >= i64::from(setup.width) || y >= i64::from(setup.height) {
        None
    } else {
        Some(Pos {
            x: x as u32,
            y: y as u32,
        })
    }
}
pub(super) fn search_step(
    setup: &Setup,
    from: Pos,
    heading: &mut f64,
    draws: &mut impl DrawSource,
) -> Checked<Pos> {
    check_heading(*heading)?;
    if let Some(pos) = ahead(setup, from, *heading) {
        return Ok(pos);
    }
    for _ in 0..32 {
        *heading = checked_uniform(draws)? * TAU;
        if let Some(pos) = ahead(setup, from, *heading) {
            return Ok(pos);
        }
    }
    Err(error(
        "movement",
        "no legal search proposal after 32 replacement headings",
    ))
}
fn distance(a: Pos, b: Pos) -> f64 {
    (f64::from(a.x) - f64::from(b.x)).hypot(f64::from(a.y) - f64::from(b.y))
}
pub(super) fn directed_step(
    setup: &Setup,
    from: Pos,
    target: Pos,
    draws: &mut impl DrawSource,
) -> Checked<Pos> {
    if from == target {
        return Ok(from);
    }
    let current = distance(from, target);
    let mut candidates = Vec::with_capacity(8);
    let mut total = 0.0;
    for dx in -1i64..=1 {
        for dy in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let x = i64::from(from.x) + dx;
            let y = i64::from(from.y) + dy;
            if x < 0 || y < 0 || x >= i64::from(setup.width) || y >= i64::from(setup.height) {
                continue;
            }
            let pos = Pos {
                x: x as u32,
                y: y as u32,
            };
            if pos == target {
                return Ok(pos);
            }
            let weight = current - distance(pos, target);
            if weight > 0.0 {
                total += weight;
                candidates.push((pos, weight));
            }
        }
    }
    if candidates.is_empty() {
        return Err(error("movement", "no neighbor reduces target distance"));
    }
    let draw = checked_uniform(draws)?;
    let mut upper = 0.0;
    for (index, (pos, weight)) in candidates.iter().enumerate() {
        upper = if index + 1 == candidates.len() {
            1.0
        } else {
            upper + weight / total
        };
        if draw < upper {
            return Ok(*pos);
        }
    }
    Err(error(
        "movement",
        "normalized neighbor intervals did not select a cell",
    ))
}
