//! Fixed-world central-place foraging after Hecker and Moses (2015).
//! Angular eight-neighbor geometry follows the historical iAnt reconstruction;
//! supplied engineering fixtures do not establish scientific efficacy.

mod draws;
mod movement;
mod setup;

pub use setup::{Pos, Resource, Setup};

#[cfg(test)]
mod tests;
