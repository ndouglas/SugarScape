mod agreement;
mod ants;
mod bali;
mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod civil;
mod classes;
mod culture;
mod dpd;
mod ethno;
mod farol;
mod image;
mod minds1;
mod minds2;
mod minds3;
mod minds4;
mod minds5;
mod norms;
mod opinions;
mod punishment;
mod retirement;
mod schelling71;
mod spatial;
mod structure;
mod tags;
mod thresholds;
mod zi;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [
        agreement::claims(),
        ants::claims(),
        bali::claims(),
        ch2::claims(),
        ch3::claims(),
        ch4::claims(),
        ch5::claims(),
        ch6::claims(),
        civil::claims(),
        classes::claims(),
        culture::claims(),
        dpd::claims(),
        ethno::claims(),
        farol::claims(),
        norms::claims(),
        image::claims(),
        minds1::claims(),
        minds2::claims(),
        minds3::claims(),
        minds4::claims(),
        minds5::claims(),
        opinions::claims(),
        punishment::claims(),
        retirement::claims(),
        schelling71::claims(),
        spatial::claims(),
        structure::claims(),
        tags::claims(),
        thresholds::claims(),
        zi::claims(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
