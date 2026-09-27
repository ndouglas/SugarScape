mod agreement;
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
mod norms;
mod image;
mod minds1;
mod opinions;
mod spatial;
mod structure;
mod tags;

use crate::claim::Claim;

pub fn all() -> Vec<Claim> {
    [
        agreement::claims(),
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
        opinions::claims(),
        spatial::claims(),
        structure::claims(),
        tags::claims(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
