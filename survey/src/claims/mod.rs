mod ch2;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [ch2::claims()].into_iter().flatten().collect()
}
