//! Minds 2's A*: verified against Dijkstra on random grids and against
//! Sturtevant's published optimal lengths (Moving AI benchmarks, ODC-By;
//! see tests/fixtures/movingai/README.md).

use std::collections::BinaryHeap;

use rand::{Rng, SeedableRng};
use sugarscape_core::geometry::{Pos, Torus};
use sugarscape_core::minds::astar::{astar, Graph};
use sugarscape_core::minds::grid::{OctileMap, TorusGrid};

/// Dijkstra's optimal cost, the oracle (no heuristic, no ties to break).
fn dijkstra<G: Graph>(g: &G, start: usize, goal: usize, nodes: usize) -> Option<f64> {
    #[derive(PartialEq)]
    struct Item(f64, usize);
    impl Eq for Item {}
    impl PartialOrd for Item {
        fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Item {
        fn cmp(&self, o: &Self) -> std::cmp::Ordering {
            o.0.total_cmp(&self.0)
        }
    }
    let mut dist = vec![f64::INFINITY; nodes];
    let mut heap = BinaryHeap::new();
    let mut out = Vec::new();
    dist[start] = 0.0;
    heap.push(Item(0.0, start));
    while let Some(Item(d, n)) = heap.pop() {
        if n == goal {
            return Some(d);
        }
        if d > dist[n] {
            continue;
        }
        out.clear();
        g.neighbors(n, &mut out);
        for &(m, c) in &out {
            if d + c < dist[m] {
                dist[m] = d + c;
                heap.push(Item(d + c, m));
            }
        }
    }
    None
}

fn random_mask(rng: &mut impl Rng, len: usize, density: f64) -> Vec<bool> {
    (0..len).map(|_| rng.gen::<f64>() >= density).collect()
}

#[test]
fn astar_matches_dijkstra_on_random_walled_tori() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(20260927);
    let mut checked = 0;
    for case in 0..1000 {
        let (w, h) = (rng.gen_range(3..30u32), rng.gen_range(3..30u32));
        let torus = Torus::new(w, h);
        let density = f64::from(case % 5) * 0.1; // 0-40 %
        let open = random_mask(&mut rng, (w * h) as usize, density);
        let grid = TorusGrid::new(torus, |p: Pos| open[torus.index(p)]);
        let (s, t) = (
            rng.gen_range(0..w * h) as usize,
            rng.gen_range(0..w * h) as usize,
        );
        if !open[s] || !open[t] {
            continue;
        }
        checked += 1;
        let want = dijkstra(&grid, s, t, (w * h) as usize);
        let got = astar(&grid, s, t, usize::MAX);
        match (want, got) {
            (None, None) => {}
            (Some(c), Some(found)) => {
                assert_eq!(found.cost, c, "case {case}");
                assert_eq!(found.path.len() as f64 - 1.0, c, "case {case}: unit steps");
                assert_eq!((found.path[0], *found.path.last().unwrap()), (s, t));
                for pair in found.path.windows(2) {
                    let (a, b) = (torus.pos(pair[0]), torus.pos(pair[1]));
                    assert!(torus.neighbors(a).contains(&b), "case {case}: a real step");
                    assert!(open[pair[1]], "case {case}: through open sites only");
                }
            }
            (w, g) => panic!("case {case}: dijkstra {w:?}, astar {:?}", g.map(|s| s.cost)),
        }
    }
    eprintln!("walled tori: {checked} of 1000 cases checked");
    // A case whose start or goal is a wall is skipped; enough must remain.
    assert!(checked >= 600, "only {checked} of 1000 cases checked");
}

#[test]
fn astar_matches_dijkstra_on_random_octile_maps() {
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(7);
    let mut checked = 0;
    for case in 0..1000 {
        let (w, h) = (rng.gen_range(2..40usize), rng.gen_range(2..40usize));
        let density = (case % 5) as f64 * 0.1;
        let rows: Vec<String> = (0..h)
            .map(|_| {
                (0..w)
                    .map(|_| if rng.gen::<f64>() < density { '@' } else { '.' })
                    .collect()
            })
            .collect();
        let text = format!(
            "type octile\nheight {h}\nwidth {w}\nmap\n{}\n",
            rows.join("\n")
        );
        let map = OctileMap::parse(&text).unwrap();
        let (sx, sy, tx, ty) = (
            rng.gen_range(0..w),
            rng.gen_range(0..h),
            rng.gen_range(0..w),
            rng.gen_range(0..h),
        );
        if !map.passable(sx, sy) || !map.passable(tx, ty) {
            continue;
        }
        checked += 1;
        let (s, t) = (map.index(sx, sy), map.index(tx, ty));
        let want = dijkstra(&map, s, t, w * h);
        let got = astar(&map, s, t, usize::MAX).map(|f| f.cost);
        match (want, got) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!((a - b).abs() < 1e-9, "case {case}: {a} vs {b}"),
            other => panic!("case {case}: {other:?}"),
        }
    }
    eprintln!("octile maps: {checked} of 1000 cases checked");
    // A case whose start or goal is a wall is skipped; enough must remain.
    assert!(checked >= 600, "only {checked} of 1000 cases checked");
}

fn scenarios(map_file: &str, scen_file: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/movingai/");
    let map =
        OctileMap::parse(&std::fs::read_to_string(format!("{dir}{map_file}")).unwrap()).unwrap();
    let scen = std::fs::read_to_string(format!("{dir}{scen_file}")).unwrap();
    let mut n = 0;
    for line in scen.lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        let num = |i: usize| f[i].parse::<usize>().unwrap();
        let optimal: f64 = f[8].parse().unwrap();
        let (s, t) = (map.index(num(4), num(5)), map.index(num(6), num(7)));
        let found = astar(&map, s, t, usize::MAX).unwrap_or_else(|| panic!("{line}: no path"));
        assert!(
            (found.cost - optimal).abs() < 1e-6,
            "{line}: A* {} vs optimal {optimal}",
            found.cost
        );
        n += 1;
    }
    assert!(n >= 80, "{scen_file}: {n} scenarios");
}

#[test]
fn astar_meets_sturtevants_random_benchmark() {
    scenarios("random512-10-0.map", "random512-10-0.map.scen");
}

#[test]
fn astar_meets_sturtevants_maze_benchmark() {
    scenarios("maze512-4-0.map", "maze512-4-0.map.scen");
}
