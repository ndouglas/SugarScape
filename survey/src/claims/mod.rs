mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [ch2::claims(), ch3::claims(), ch4::claims(), ch5::claims(), ch6::claims()]
        .into_iter()
        .flatten()
        .collect()
}
