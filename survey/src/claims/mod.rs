mod agreement;
mod ants;
mod auctions;
mod bali;
mod ch2;
mod ch3;
mod ch4;
mod ch5;
mod ch6;
mod civil;
mod classes;
mod collusion;
mod culture;
mod dpd;
mod ethno;
mod farol;
mod firms;
mod image;
mod minds1;
mod minds2;
mod minds3;
mod minds4;
mod minds5;
mod minds6;
mod minds7;
mod minds8;
pub(crate) mod minds8b;
pub(crate) mod minds9;
mod norms;
mod opinions;
pub(crate) mod protection;
pub(crate) mod protection_archive;
pub(crate) mod protection_report;
mod punishment;
mod retirement;
mod schelling71;
mod spatial;
mod structure;
mod tags;
mod thresholds;
mod tipping;
mod variations;
mod zi;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [
        agreement::claims(),
        auctions::claims(),
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
        firms::claims(),
        collusion::claims(),
        norms::claims(),
        image::claims(),
        minds1::claims(),
        minds2::claims(),
        minds3::claims(),
        minds4::claims(),
        minds5::claims(),
        minds6::claims(),
        minds7::claims(),
        minds8::claims(),
        minds8b::claims(),
        opinions::claims(),
        punishment::claims(),
        retirement::claims(),
        schelling71::claims(),
        spatial::claims(),
        structure::claims(),
        tags::claims(),
        thresholds::claims(),
        tipping::claims(),
        variations::claims(),
        zi::claims(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
