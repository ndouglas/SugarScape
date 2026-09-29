//! One month of the watershed — rain, water sharing, rice growth, pests and
//! harvest — shared by the world and Janssen's plan search. Written from
//! Lansing and Kremer (1993), Janssen (2007) and Janssen's ODD.

use rand::Rng;

use super::config::{BaliConfig, DamColumns, PestForm, Rain, Routing};
use super::data::{watershed, Watershed};
use crate::rng::SimRng;

/// Evapotranspiration and the rain–runoff threshold (mm a month, as m/day)
/// and the low-rain slope below it (Janssen's ODD).
const ET: f64 = 50.0 / 30000.0;
const RRT: f64 = ET + 50.0 / 30000.0;
const LRS: f64 = 1.0 - ET / RRT;
/// The pest floor.
pub const MIN_PESTS: f64 = 0.01;

/// The network a world runs on: dams per subak, the effective catchments, and
/// the pest links (perhaps perturbed).
#[derive(Clone, Debug)]
pub struct Network {
    pub source: Vec<usize>,
    pub ret: Vec<usize>,
    /// Catchment less the subaks' area charged to each dam, hectares.
    pub effective: Vec<f64>,
    /// Area drawing from each dam, hectares.
    pub served: Vec<f64>,
    /// Out-links (whom a subak imitates, where its pests go) and in-links
    /// (where its pests come from).
    pub out: Vec<Vec<usize>>,
    pub inn: Vec<Vec<usize>>,
}

impl Network {
    pub fn new(c: &BaliConfig, rng: &mut SimRng) -> Network {
        let w = watershed();
        let n = w.subaks.len();
        let (source, ret): (Vec<usize>, Vec<usize>) = w
            .subaks
            .iter()
            .map(|s| match c.dam_columns {
                DamColumns::Code => (s.col3, s.col2),
                DamColumns::Physical => (s.col2, s.col3),
            })
            .unzip();
        let mut charged = vec![0.0; w.dams.len()];
        let mut served = vec![0.0; w.dams.len()];
        for i in 0..n {
            let a = w.subaks[i].area;
            // Janssen: a subak's area counts against its return dam if both
            // are the same, else against its source dam.
            charged[if source[i] == ret[i] {
                ret[i]
            } else {
                source[i]
            }] += a;
            served[source[i]] += a;
        }
        let effective = w
            .dams
            .iter()
            .zip(&charged)
            .map(|(d, a)| d.catchment - a)
            .collect();
        // Janssen's pₑ and pₙ.
        let mut links: Vec<(usize, usize)> = Vec::new();
        for &(a, b) in &w.links {
            if c.remove_links > 0.0 && rng.gen::<f64>() < c.remove_links {
                continue;
            }
            links.push((a, b));
        }
        if c.add_links > 0.0 {
            for a in 0..n {
                for b in 0..n {
                    let shares = a != b
                        && (source[a] == source[b] || ret[a] == ret[b])
                        && !links.contains(&(a, b));
                    if shares && rng.gen::<f64>() < c.add_links {
                        links.push((a, b));
                    }
                }
            }
        }
        let mut out = vec![Vec::new(); n];
        let mut inn = vec![Vec::new(); n];
        for &(a, b) in &links {
            out[a].push(b);
            inn[b].push(a);
        }
        Network {
            source,
            ret,
            effective,
            served,
            out,
            inn,
        }
    }

    /// Undirected links, each once.
    pub fn pairs(&self) -> Vec<(usize, usize)> {
        let mut v: Vec<(usize, usize)> = self
            .out
            .iter()
            .enumerate()
            .flat_map(|(a, bs)| bs.iter().map(move |&b| (a.min(b), a.max(b))))
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }
}

/// What a month needs to know.
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub growth: f64,
    pub dispersal: f64,
    /// Pest damage × this.
    pub damage: f64,
    pub rain_scale: f64,
    /// 0 low, 1 middle, 2 high.
    pub scenario: usize,
    pub routing: Routing,
    pub pest_form: PestForm,
}

impl Params {
    /// The parameters of `year` (1-based) under `c`, with `scenario` drawn.
    pub fn of(c: &BaliConfig, year: u64, scenario: usize) -> Params {
        let perturbed = c.perturb.enabled && year >= u64::from(c.perturb.at);
        Params {
            growth: if perturbed {
                c.perturb.growth
            } else {
                c.growth
            },
            dispersal: if perturbed {
                c.perturb.dispersal
            } else {
                c.dispersal
            },
            damage: if perturbed { c.perturb.damage } else { 1.0 },
            rain_scale: c.rain_scale * if perturbed { c.perturb.rain } else { 1.0 },
            scenario,
            routing: c.routing,
            pest_form: c.pest_form,
        }
    }
}

/// The scenario of a year: fixed, or drawn 25/50/25 %.
pub fn scenario(rain: Rain, rng: &mut SimRng) -> usize {
    match rain {
        Rain::Low => 0,
        Rain::Middle => 1,
        Rain::High => 2,
        Rain::Random => {
            let u = rng.gen::<f64>();
            if u < 0.25 {
                0
            } else if u < 0.75 {
                1
            } else {
                2
            }
        }
    }
}

/// The dynamic state of every subak and dam.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// The crop each subak has this month.
    pub crop: Vec<u8>,
    /// Growth toward harvest (1 = a full crop without water stress).
    pub stage: Vec<f64>,
    pub pests: Vec<f64>,
    /// Harvest this year, t/ha.
    pub harvest: Vec<f64>,
    /// Potential harvest lost to pests, and growing months' water shortfall, this year.
    pub lost: Vec<f64>,
    pub short: Vec<f64>,
    pub growing: Vec<u32>,
    /// Each dam's water stress (the fraction of demand met), inflow and demand.
    pub wsd: Vec<f64>,
    pub inflow: Vec<f64>,
    pub demand: Vec<f64>,
}

impl State {
    pub fn new(n: usize, dams: usize, routing: Routing) -> State {
        State {
            crop: vec![0; n],
            stage: vec![0.0; n],
            pests: vec![MIN_PESTS; n],
            harvest: vec![0.0; n],
            lost: vec![0.0; n],
            short: vec![0.0; n],
            growing: vec![0; n],
            // Janssen's code starts every dam at 0 until it is first drawn.
            wsd: vec![
                if routing == Routing::JanssenCode {
                    0.0
                } else {
                    1.0
                };
                dams
            ],
            inflow: vec![0.0; dams],
            demand: vec![0.0; dams],
        }
    }
}

/// The crop of plan `plan` begun in month `start`, in month `m` of the year.
pub fn crop_of(w: &Watershed, plan: u8, start: u8, m: usize) -> u8 {
    w.plans[plan as usize][(usize::from(start) + m) % 12]
}

/// The stage a crop has reached in month `m` if it grew unstressed since
/// its plan began it (Janssen's starting stages).
pub fn stage_of(w: &Watershed, plan: u8, start: u8, m: usize) -> f64 {
    let c = crop_of(w, plan, start, m);
    if !(1..=3).contains(&c) {
        return 0.0;
    }
    let mut k = 0;
    while k < 11 && crop_of(w, plan, start, (m + 12 - k - 1) % 12) == c {
        k += 1;
    }
    k as f64 / w.devtime[c as usize]
}

/// One month: `crop` must hold this month's crops and `next` next month's.
/// Harvests are added to `s.harvest`.
pub fn month(net: &Network, p: &Params, m: usize, s: &mut State, next: &[u8], rng: &mut SimRng) {
    let w = watershed();
    let n = w.subaks.len();
    let nd = w.dams.len();
    // Rain and runoff at each dam, m/day and m³/day.
    let rain: Vec<f64> = w
        .dams
        .iter()
        .map(|d| w.rain[d.zone][p.scenario][m] * p.rain_scale / 30000.0)
        .collect();
    let runoff: Vec<f64> = (0..nd)
        .map(|j| {
            let r = rain[j];
            let per = if r < RRT { r * LRS } else { (r - ET).max(0.0) };
            per * net.effective[j] * 1e4
        })
        .collect();
    // Demand: a crop's use less the return dam's rain; excess rain returns.
    let (mut d1, mut d3, mut xs) = (vec![0.0; nd], vec![0.0; nd], vec![0.0; nd]);
    for i in 0..n {
        let dmd = (w.water_use[s.crop[i] as usize] - rain[net.ret[i]]) * w.subaks[i].area * 1e4;
        let (src, ret) = (net.source[i], net.ret[i]);
        if src == ret {
            if dmd > 0.0 {
                d1[ret] += dmd;
            } else {
                xs[ret] -= dmd;
            }
        } else if dmd < 0.0 {
            xs[ret] -= dmd;
        } else {
            d3[src] += dmd;
        }
    }
    let base = |j: usize| w.dams[j].flow0 * 86400.0 + runoff[j] + xs[j];
    match p.routing {
        Routing::Network => {
            let mut out = vec![0.0; nd];
            for &j in &w.order {
                let inflow = base(j) + w.upstream[j].iter().map(|&u| out[u]).sum::<f64>();
                let need = d1[j] + d3[j];
                let f = inflow - need;
                s.inflow[j] = inflow;
                s.demand[j] = need;
                if f < 0.0 && need > 0.0 {
                    s.wsd[j] = (1.0 + f / need).max(0.0);
                    out[j] = 0.0;
                } else {
                    s.wsd[j] = 1.0;
                    out[j] = f.max(0.0);
                }
            }
        }
        Routing::JanssenCode => {
            // One random dam balances its own water; no inflow from upstream.
            let j = rng.gen_range(0..nd as u32) as usize;
            let need = d1[j] + d3[j];
            let f = base(j) - need;
            s.inflow[j] = base(j);
            s.demand[j] = need;
            if f < 0.0 {
                if need > 0.0 {
                    s.wsd[j] = (1.0 + f / need).max(0.0);
                }
            } else {
                s.wsd[j] = 1.0;
            }
        }
    }
    // Rice grows by its water.
    for i in 0..n {
        let c = s.crop[i] as usize;
        if (1..=3).contains(&c) {
            let ws = s.wsd[net.source[i]];
            s.stage[i] += ws / w.devtime[c];
            s.short[i] += 1.0 - ws;
            s.growing[i] += 1;
        } else {
            s.stage[i] = 0.0;
        }
    }
    // Pests grow and spread along the links.
    let old = s.pests.clone();
    for i in 0..n {
        let g = if (1..=3).contains(&s.crop[i]) {
            p.growth
        } else if s.crop[i] == 4 {
            0.33
        } else {
            0.1
        };
        let flux = p.dispersal * net.inn[i].iter().map(|&k| old[k] - old[i]).sum::<f64>();
        let v = match p.pest_form {
            PestForm::Shortcut => g * (old[i] + 0.5 * flux) + 0.5 * flux,
            PestForm::Diffusion => g * old[i] + flux,
        };
        s.pests[i] = v.max(MIN_PESTS);
    }
    // A crop ending this month is harvested.
    for (i, &after) in next.iter().enumerate().take(n) {
        let c = s.crop[i] as usize;
        if (1..=3).contains(&c) && (after == 0 || after == 4) {
            let potential = s.stage[i] * w.yield_max[c];
            let kept = (1.0 - s.pests[i] * w.sensitivity[c] * p.damage).max(0.0);
            s.harvest[i] += potential * kept;
            s.lost[i] += potential * (1.0 - kept);
        }
    }
}

/// The area-weighted mean of per-subak values.
pub fn area_mean(v: &[f64]) -> f64 {
    let w = watershed();
    let total: f64 = w.subaks.iter().map(|s| s.area).sum();
    v.iter()
        .zip(&w.subaks)
        .map(|(x, s)| x * s.area)
        .sum::<f64>()
        / total
}

/// A year's harvest per subak under fixed plans, from the second of two
/// years (with pests reset each year every year after the first repeats it).
pub fn steady_year(net: &Network, p: &Params, plans: &[(u8, u8)], rng: &mut SimRng) -> Vec<f64> {
    let w = watershed();
    let n = plans.len();
    let mut s = State::new(n, w.dams.len(), p.routing);
    for (i, &(plan, start)) in plans.iter().enumerate() {
        s.stage[i] = stage_of(w, plan, start, 0);
    }
    for year in 0..2 {
        s.harvest.iter_mut().for_each(|h| *h = 0.0);
        for m in 0..12 {
            for (i, &(plan, start)) in plans.iter().enumerate() {
                s.crop[i] = crop_of(w, plan, start, m);
            }
            let next: Vec<u8> = plans
                .iter()
                .map(|&(plan, start)| crop_of(w, plan, start, m + 1))
                .collect();
            month(net, p, m, &mut s, &next, rng);
        }
        if year == 0 {
            s.pests.iter_mut().for_each(|x| *x = MIN_PESTS);
        }
    }
    s.harvest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn params() -> Params {
        Params::of(&BaliConfig::default(), 1, 1)
    }

    #[test]
    fn starting_stages_follow_the_plan() {
        let w = watershed();
        // Plan 0 (three-month rice): months 0, 1, 2 at stages 0, 1/3, 2/3.
        assert_eq!(stage_of(w, 0, 0, 0), 0.0);
        assert!((stage_of(w, 0, 0, 1) - 1.0 / 3.0).abs() < 1e-12);
        assert!((stage_of(w, 0, 0, 2) - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(stage_of(w, 0, 0, 3), 0.0);
        // Plan 6 from month 3: the six-month crop in its fourth month.
        assert!((stage_of(w, 6, 3, 0) - 3.0 / 6.0).abs() < 1e-12);
    }

    #[test]
    fn an_unstressed_pest_free_crop_yields_its_potential() {
        let mut r = rng::seeded(1);
        let net = Network::new(&BaliConfig::default(), &mut r);
        // Everyone on plan 9 (one six-month crop): plenty of water; pests small.
        let plans = vec![(9u8, 0u8); 172];
        let p = Params {
            growth: 0.0,
            ..params()
        };
        let h = steady_year(&net, &p, &plans, &mut r);
        let w = watershed();
        for (i, x) in h.iter().enumerate() {
            let ws = 1.0;
            let _ = (i, w);
            assert!(*x <= 5.0 * ws + 1e-9);
        }
        assert!(area_mean(&h) > 4.9, "{}", area_mean(&h));
    }

    #[test]
    fn a_shortage_cuts_every_subak_on_the_dam_by_the_same_fraction() {
        let mut r = rng::seeded(1);
        let net = Network::new(&BaliConfig::default(), &mut r);
        let w = watershed();
        let n = w.subaks.len();
        let mut s = State::new(n, w.dams.len(), Routing::Network);
        s.crop = vec![3; n];
        let next = vec![3u8; n];
        // The dry season (August) at low rain: demand outstrips supply somewhere.
        let p = Params {
            scenario: 0,
            ..params()
        };
        month(&net, &p, 7, &mut s, &next, &mut r);
        assert!(s.wsd.iter().any(|&x| x < 1.0), "{:?}", s.wsd);
        assert!(s.wsd.iter().all(|&x| (0.0..=1.0).contains(&x)));
        for i in 0..n {
            // Every growing subak advanced by its source dam's fraction.
            assert!((s.stage[i] - s.wsd[net.source[i]] / 3.0).abs() < 1e-12);
        }
    }

    #[test]
    fn water_passes_downstream_under_the_network_but_not_janssens_code() {
        let mut r = rng::seeded(1);
        let net = Network::new(&BaliConfig::default(), &mut r);
        let w = watershed();
        let n = w.subaks.len();
        let mut a = State::new(n, w.dams.len(), Routing::Network);
        month(&net, &params(), 7, &mut a, &vec![0; n], &mut r);
        // Dam 11 is the last: its inflow includes everything upstream.
        assert!(a.inflow[11] > w.dams[11].flow0 * 86400.0 * 2.0);
        let mut b = State::new(n, w.dams.len(), Routing::JanssenCode);
        let p = Params {
            routing: Routing::JanssenCode,
            ..params()
        };
        month(&net, &p, 7, &mut b, &vec![0; n], &mut r);
        // One dam balanced; the rest still at 0.
        assert_eq!(b.wsd.iter().filter(|&&x| x != 0.0).count(), 1);
    }

    #[test]
    fn pests_follow_either_equation_and_never_fall_below_the_floor() {
        let mut r = rng::seeded(1);
        let net = Network::new(&BaliConfig::default(), &mut r);
        let w = watershed();
        let n = w.subaks.len();
        // A subak with one in-link: find it.
        let i = (0..n).find(|&i| net.inn[i].len() == 1).unwrap();
        let k = net.inn[i][0];
        let mut s = State::new(n, w.dams.len(), Routing::Network);
        s.crop = vec![3; n];
        s.pests[k] = 1.0;
        s.pests[i] = 0.1;
        let before = s.pests.clone();
        month(&net, &params(), 0, &mut s, &vec![3; n], &mut r);
        let flux = 0.3 * (before[k] - before[i]);
        assert!((s.pests[i] - (2.2 * (0.1 + 0.5 * flux) + 0.5 * flux)).abs() < 1e-12);
        let mut d = State::new(n, w.dams.len(), Routing::Network);
        d.crop = vec![3; n];
        d.pests = before.clone();
        let p = Params {
            pest_form: PestForm::Diffusion,
            ..params()
        };
        month(&net, &p, 0, &mut d, &vec![3; n], &mut r);
        assert!((d.pests[i] - (2.2 * 0.1 + flux)).abs() < 1e-12);
        // Fallow fields: pests shrink to the floor.
        let mut f = State::new(n, w.dams.len(), Routing::Network);
        month(&net, &params(), 0, &mut f, &vec![0; n], &mut r);
        assert!(f.pests.iter().all(|&x| x == MIN_PESTS));
    }

    #[test]
    fn link_perturbations_remove_and_add() {
        let mut r = rng::seeded(3);
        let all = Network::new(
            &BaliConfig {
                remove_links: 1.0,
                ..BaliConfig::default()
            },
            &mut r,
        );
        assert!(all.out.iter().all(Vec::is_empty));
        let more = Network::new(
            &BaliConfig {
                add_links: 0.1,
                ..BaliConfig::default()
            },
            &mut r,
        );
        let base = Network::new(&BaliConfig::default(), &mut r);
        let count = |n: &Network| n.out.iter().map(Vec::len).sum::<usize>();
        assert_eq!(count(&base), 323);
        assert!(count(&more) > 323);
    }
}
