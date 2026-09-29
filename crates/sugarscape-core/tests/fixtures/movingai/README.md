# Moving AI grid benchmarks (subset)

Two maps and a subset of their scenarios from Nathan Sturtevant's grid-based pathfinding
benchmarks (N. R. Sturtevant, "Benchmarks for Grid-Based Pathfinding", *IEEE Transactions on
Computational Intelligence and AI in Games* 4(2), 2012, 144–148), downloaded from
https://movingai.com/benchmarks/grids.html on 2026-09-27.

The data is made available under the Open Data Commons Attribution License
(https://opendatacommons.org/licenses/by/). These files are unchanged except that the scenario
files keep only every 20th (`random512-10-0`) or every 100th (`maze512-4-0`) scenario line after
the version header.

- `random512-10-0.map`, `random512-10-0.map.scen`: the synthetic random set, 10 % obstacles.
- `maze512-4-0.map`, `maze512-4-0.map.scen`: the synthetic maze set, corridors 4 wide.

Map format: `type octile`, `height`, `width`, `map`, then rows; `.` `G` `S` are passable, `@` `O`
`T` `W` are not. Scenario columns: bucket, map, width, height, start x, start y, goal x, goal y,
optimal length (8-way moves, diagonals cost √2, no corner cutting).
