//! Capacity recurrence and conditional commitments (APSR pp.146–148).
//! GeoSim alone uses pinned libm software transcendentals for exact host parity.
pub fn distance_curve(
    distance: f64,
    offset: f64,
    threshold: f64,
    exponent: f64,
    increasing: bool,
) -> f64 {
    if distance == 0.0 {
        return 1.0;
    }
    let power = libm::pow(
        distance / threshold,
        if increasing { -exponent } else { exponent },
    );
    offset + (1.0 - offset) / (1.0 + power)
}
pub fn capacity(old: f64, yield_capacity: f64, loss: f64, adjustment: f64, add_loss: bool) -> f64 {
    (1.0 - adjustment) * old + adjustment * (yield_capacity + if add_loss { loss } else { -loss })
}
pub fn commitment(
    capacity: f64,
    mobile: f64,
    neighbors: usize,
    opposing: f64,
    enemy: f64,
    active: bool,
) -> f64 {
    if neighbors == 0 {
        return 0.0;
    }
    let fixed = (1.0 - mobile) * capacity / neighbors as f64;
    fixed
        + mobile
            * capacity
            * if enemy == 0.0 {
                1.0
            } else if active {
                opposing / enemy
            } else {
                opposing / (enemy + opposing)
            }
}
pub fn probability(ratio: f64, threshold: f64, exponent: u32) -> f64 {
    if ratio == 0.0 {
        return 0.0;
    }
    let log = (libm::log(ratio) - libm::log(threshold)) * f64::from(exponent);
    if log >= 0.0 {
        1.0 / (1.0 + libm::exp(-log))
    } else {
        let x = libm::exp(log);
        x / (1.0 + x)
    }
}
