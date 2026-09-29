//! The Oos and Petanu watershed and the model's tables, parsed from Janssen's
//! data files (data/bali/: GPL-2.0, see its NOTICE).

use std::sync::OnceLock;

const SUBAKS: &str = include_str!("../../../../data/bali/subakdata.txt");
const DAMS: &str = include_str!("../../../../data/bali/damdata.txt");
const SUBAK_DAMS: &str = include_str!("../../../../data/bali/subakdamdata.txt");
const LINKS: &str = include_str!("../../../../data/bali/subaksubakdata.txt");
const TABLES: &str = include_str!("../../../../data/bali/tables.txt");

/// A subak (farmers' association) of the watershed.
#[derive(Clone, Debug, PartialEq)]
pub struct SubakData {
    /// Janssen's grid coordinates.
    pub x: f64,
    pub y: f64,
    /// Hectares.
    pub area: f64,
    /// Its masceti temple (1–14), and the data's second temple column (1–2).
    pub masceti: u32,
    pub ulun: u32,
    /// The subak–dam file's second and third columns (Janssen's code reads
    /// them as the return and the source dam).
    pub col2: usize,
    pub col3: usize,
}

/// A dam (weir) of the watershed.
#[derive(Clone, Debug, PartialEq)]
pub struct DamData {
    pub x: f64,
    pub y: f64,
    /// Base flow, m³/s.
    pub flow0: f64,
    pub elevation: f64,
    /// Catchment, hectares.
    pub catchment: f64,
    /// Rain zone (0–4).
    pub zone: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Watershed {
    pub subaks: Vec<SubakData>,
    pub dams: Vec<DamData>,
    /// Directed pest links (a, b): pests spread from a to b; a imitates b.
    pub links: Vec<(usize, usize)>,
    /// Each dam's upstream dams.
    pub upstream: Vec<Vec<usize>>,
    /// Dams in an order where every dam follows its upstream dams.
    pub order: Vec<usize>,
    /// The 21 twelve-month plans: the crop each month.
    pub plans: Vec<[u8; 12]>,
    /// Rain, mm a month: `rain[zone][scenario][month]`.
    pub rain: Vec<[[f64; 12]; 3]>,
    /// By crop (fallow, six-month, four-month, three-month, vegetables).
    pub devtime: [f64; 5],
    pub yield_max: [f64; 5],
    pub sensitivity: [f64; 5],
    pub water_use: [f64; 5],
}

/// Whitespace-separated numbers, skipping `;` comments.
fn numbers(text: &str) -> Vec<f64> {
    text.split(['\n', '\r'])
        .map(|l| l.split(';').next().unwrap_or(""))
        .flat_map(|l| l.split_whitespace())
        .map(|w| w.parse::<f64>().expect("a number in the Bali data"))
        .collect()
}

fn parse() -> Watershed {
    let s = numbers(SUBAKS);
    let sd = numbers(SUBAK_DAMS);
    let subaks: Vec<SubakData> = s
        .as_chunks::<6>()
        .0
        .iter()
        .zip(sd.as_chunks::<3>().0.iter())
        .map(|(r, d)| SubakData {
            x: r[1],
            y: r[2],
            area: r[3],
            masceti: r[4] as u32,
            ulun: r[5] as u32,
            col2: d[1] as usize,
            col3: d[2] as usize,
        })
        .collect();
    let dams: Vec<DamData> = numbers(DAMS)
        .as_chunks::<7>()
        .0
        .iter()
        .map(|r| DamData {
            x: r[1],
            y: r[2],
            flow0: r[3],
            elevation: r[4],
            catchment: r[5],
            zone: r[6] as usize,
        })
        .collect();
    let links: Vec<(usize, usize)> = numbers(LINKS)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0] as usize, p[1] as usize))
        .collect();
    // The tables: named sections.
    let mut plans = Vec::new();
    let mut rain = Vec::new();
    let (mut devtime, mut yield_max, mut sensitivity, mut water_use) =
        ([0.0; 5], [0.0; 5], [0.0; 5], [0.0; 5]);
    let mut upstream = vec![Vec::new(); dams.len()];
    let mut section = "";
    let mut rain_rows: Vec<[f64; 12]> = Vec::new();
    for line in TABLES.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_whitespace();
        let head = words.clone().next().unwrap_or("");
        if head.parse::<f64>().is_err() {
            section = head;
            words.next();
        }
        let v: Vec<f64> = words
            .map(|w| w.parse().expect("a number in tables.txt"))
            .collect();
        match section {
            "plans" if v.len() == 12 => {
                plans.push(std::array::from_fn(|m| v[m] as u8));
            }
            "rain" if v.len() == 12 => rain_rows.push(std::array::from_fn(|m| v[m])),
            "devtime" => devtime.copy_from_slice(&v),
            "yield" => yield_max.copy_from_slice(&v),
            "sensitivity" => sensitivity.copy_from_slice(&v),
            "use" => water_use.copy_from_slice(&v),
            "dams" => {
                for p in v.as_chunks::<2>().0 {
                    upstream[p[1] as usize].push(p[0] as usize);
                }
            }
            _ => {}
        }
    }
    for z in rain_rows.as_chunks::<3>().0 {
        rain.push([z[0], z[1], z[2]]);
    }
    // Upstream dams first.
    let mut order = Vec::new();
    let mut done = vec![false; dams.len()];
    while order.len() < dams.len() {
        for d in 0..dams.len() {
            if !done[d] && upstream[d].iter().all(|&u| done[u]) {
                done[d] = true;
                order.push(d);
            }
        }
    }
    Watershed {
        subaks,
        dams,
        links,
        upstream,
        order,
        plans,
        rain,
        devtime,
        yield_max,
        sensitivity,
        water_use,
    }
}

/// The watershed, parsed once.
pub fn watershed() -> &'static Watershed {
    static W: OnceLock<Watershed> = OnceLock::new();
    W.get_or_init(parse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_watershed_matches_janssens_description() {
        let w = watershed();
        assert_eq!(
            (w.subaks.len(), w.dams.len(), w.links.len()),
            (172, 12, 323)
        );
        let area: f64 = w.subaks.iter().map(|s| s.area).sum();
        assert_eq!(area, 5861.0);
        let mut sizes = [0; 15];
        for s in &w.subaks {
            sizes[s.masceti as usize] += 1;
        }
        assert_eq!(
            &sizes[1..],
            &[3, 8, 30, 10, 4, 17, 4, 11, 9, 31, 11, 10, 11, 13]
        );
        assert_eq!((w.plans.len(), w.rain.len()), (21, 5));
        assert_eq!(w.plans[6], [1, 1, 1, 1, 1, 1, 0, 2, 2, 2, 2, 0]);
        assert_eq!(w.rain[0][1][0], 252.0);
        assert_eq!(w.upstream[8], [6, 7]);
        assert_eq!(w.upstream[9], [2, 3, 4]);
        let pos = |d: usize| w.order.iter().position(|&x| x == d).unwrap();
        assert!(pos(0) < pos(5) && pos(5) < pos(6) && pos(6) < pos(8) && pos(9) < pos(11));
        assert_eq!(
            (w.yield_max[3], w.devtime[1], w.water_use[1]),
            (10.0, 6.0, 0.015)
        );
    }
}
