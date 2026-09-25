mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod civil;
mod spatial;
mod tags;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [
        ch2::claims(),
        ch3::claims(),
        ch4::claims(),
        ch5::claims(),
        ch6::claims(),
        civil::claims(),
        spatial::claims(),
        tags::claims(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
