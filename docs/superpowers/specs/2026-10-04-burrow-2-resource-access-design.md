# Burrow 2 resource access design

**Date:** 2026-10-04
**Status:** architectural design approved by the user on 2026-10-04; [implementation plan](../plans/2026-10-04-burrow-2-resource-access.md) approved 2026-10-04; execution has not started. No runtime changes or scientific execution have occurred.
**Baseline:** [implemented Burrow 1](2026-10-03-burrow-1-excavation-design.md).
**Programme:** [collective construction B3](../../studies/2026-10-03-cultures-construction-and-underworlds.md#b3-add-inhabitation-and-competing-functions).
**Parallel study:** [Burrow 1 protocol draft](2026-10-04-burrow-1-measured-protocol.md).

## Intent and first functional question

The long-term objective is diverse, useful, historically individual underworlds constructed by inspectable agents. This increment gives excavation a specific function: opening a route to a resource site. It asks how construction cost and resulting access differ between undirected builders and a benchmark supplied with the site's location.

First success means structural access for the existing one-cell four-neighbor mover: the resource cell has been excavated and connects to the entrance through open cells. It does not mean food was harvested, carried home, consumed or distributed fairly. Actual use, resource quantities and competing inhabitants follow as separate increments. This distinction prevents an empty cavity from being counted as a pantry or a useful room merely because a researcher named it.

The first comparison fixes direct transport and blind spoil cues. It changes only the explicitly supplied task-information policy. A later protocol can cross task policy with Burrow 1 transport/cue treatments after the simpler functional case is understood.

## Alternatives and selected scope

1. **Structural resource access with a known-goal benchmark, selected.** Adds a task overlay, a local target-weight factor and access measurements while reusing physical transactions. The benchmark knows one coordinate, so it isolates supplied task guidance rather than discovery or learning.
2. **Food collection and delivery.** More directly measures inhabitant benefit, but needs a resource ledger, collection/deposit actions and rules for sharing carrying capacity with spoil. Defer to Burrow 2B after access is verified.
3. **An emitted resource signal.** Could make exploration depend on local sensory opportunities, but needs a justified signal propagation/attenuation model, sensing and memory. A Manhattan heuristic is not diffusion physics; do not disguise it as scent.

This is a deliberately small first functional layer. Support, collapse, ventilation, vertical levels, doors, body variation, learned practices and culture retain their places in the wider programme.

## Scientific support and supplied assumptions

The [source audit](../../studies/2026-10-04-burrow-studies-reading.md) distinguishes transport/cue evidence from this new benchmark. Pielström and Roces provide the Burrow 1 anchor. [Prasath and colleagues](https://pubmed.ncbi.nlm.nih.gov/36214457/) study collective excavation of a confining corral with models and robots, which makes task completion a useful related research question. Escape is not resource access, and this design imports no numerical model parameters or reproduction targets from that study.

Resource coordinates, positive goal weights, worker task memory and structural-access evaluation below are engineering assumptions. No source currently establishes that animals use this particular known-coordinate rule. The proposed access predicate is an explicit relation between a layout and the present movement capability, a first practical step toward later affordance studies; body-dependent, cultural or ecological-psychology conclusions require additional designs and sources.

## World and task overlay

Reuse finite bounded four-neighbor geometry, material accounting, carried capacity one, two-worker occupancy, unchanged action prices and sequential seeded scheduling. Keep uniform diggable soil, traversable loose spoil and supplied exit navigation. Do not add a second material system or alter spoil birth/fate semantics.

The public access configuration composes the existing lab configuration:

```rust
pub enum AccessObjective { Explore, KnownGoal }
pub struct AccessTask {
    pub goal: Pos,
    pub objective: AccessObjective,
    pub goal_weight: u32,
}
pub struct AccessConfig { pub lab: LabConfig, pub task: AccessTask }
pub fn run_access_episode(config: AccessConfig, seed: u64, options: RunOptions)
    -> Result<AccessEpisode, Vec<FieldError>>;
```

Use snake_case enum strings and deny unknown fields. Resolve all task fields explicitly; no implicit goal coordinate is read from researcher diagnostics. Accept only growing lab fixtures in this first public route. Validate the task against the constructed setup before stepping: goal is in bounds, initially solid and diggable; weight is positive; cell counts and cue × goal-weight products are checked before weighted selection. Existing clock, opportunity and conservative ASCII limits continue to apply. Constructor errors have contextual task fields and cause no output writes.

Initial demonstration configuration is the existing 41×25/eight-worker growing setup, with task goal `(7,12)`, goal weight three, 512 ticks and sampling every 32 ticks. Direct transport and blind spoil selection remain fixed. The initial staging reaches x=2; at least five new cells are needed to connect to that goal along the shortest geometric route. This is a lower bound on excavated cells, not a bound on total paid actions or a prediction of achieved behavior.

The resource marker is researcher-supplied metadata attached to an initially solid cell. It neither adds a new obstacle nor changes digging cost. Builders can only open it through legal adjacent excavation. The geometry continues growing from the accessible frontier.

## Agent information and decision rule

**Explore:** exactly the existing Burrow 1 controller. No goal coordinate, completion truth or extra random draw enters its view. Access is measured outside its decision process. For identical lab config, seed and options, its nested base Episode must be byte-identical to `run_episode`, including fingerprints and search counters.

**KnownGoal:** every worker receives the same supplied goal coordinate, its own position, ordinary local observation and one private boolean `goal_seen_open`. There is no global frontier list, path through unknown soil or global completion signal. The boolean starts false and latches true only when the worker's ordinary observation contains the goal as an open cell; update it before its decision, including when carrying spoil. It never resets during an episode. This is supplied finite-state task memory, not learned knowledge or communicated discovery.

Preserve loaded transport priority, disposal, pickup probability, retained-target validity, occupancy-aware routing and no-target wandering. Modify only the weighting of a newly selected local frontier. While `goal_seen_open` is false, multiply the ordinary cue weight by `goal_weight` if the frontier cell has strictly smaller Manhattan distance to the supplied goal than the worker's current position. Otherwise multiply by one. Once the worker has observed the opened goal, use ordinary weights. Draw one integer ticket from the resulting checked weights, as the existing controller does.

Manhattan distance is a coordinate heuristic, not a global navigable route. Nonimproving local faces remain eligible. A solid barrier or congestion can defeat the heuristic, and success is not guaranteed. A retained target is not replaced each turn merely to follow the goal; preserving the current commitment is part of the shared state machine. There is no reward optimizer, global blueprint, desired room shape, central task allocation or forced success.

## Resource access and event-time measurements

Evaluate access after every committed action. In this world all opened cells connect to the exit, so opening the goal is sufficient for structural access. Reuse the current topology field to record actual shortest open-cell exit distance at that event. Do not use Manhattan distance as the measured route length. This predicate ignores transient occupancy; report congestion separately and do not claim that a particular consumer traversed the route.

Retain an initial access state and the first successful access event, if any:

```rust
pub struct AccessMilestone {
    pub tick: u64,
    pub opportunity: u64, // one-based committed action index
    pub worker: u32,
    pub goal: Pos,
    pub digs: u64,
    pub disposed: u64,
    pub carried: u64,
    pub loose: u64,
    pub exit_distance: u32,
}
pub struct AccessSummary {
    pub structurally_accessible: bool,
    pub first_access: Option<AccessMilestone>,
    pub final_exit_distance: Option<u32>,
    pub observed_opportunities: u64,
    pub deadline_censored: bool,
}
pub struct GoalObservation {
    pub tick: u64,
    pub opportunities_before: u64,
    pub worker: u32,
}
pub struct AccessDiagnostics {
    pub completion_observations: Vec<GoalObservation>,
    pub local_completion_checks: u64,
    pub goal_weight_evaluations: u64,
}
pub struct AccessEpisode {
    pub config: AccessConfig,
    pub episode: Episode,
    pub access: AccessSummary,
    pub task_assumptions: Vec<String>,
    pub task_diagnostics: AccessDiagnostics,
}
```

The outer access configuration is necessary to reproduce goal-guided runs; the nested lab config alone is insufficient. Validators require `config.lab == episode.config`. Whole-record native/WASM comparisons use the outer record. Labels state supplied coordinates, the local completion latch, the heuristic and structural rather than realized consumer access.

Task diagnostics retain one first local completion observation per worker, with worker ID, tick and committed-opportunity count before the observation. Also count known-goal completion checks and frontier weight evaluations as separate deterministic computation proxies. These diagnostics consume no RNG and provide no information to another worker. Completion-observation records are bounded by worker count; account for their logical storage separately. Explore has no completion latch/check/evaluation work, so its nested base record remains unchanged.

Do not stop on first access in the initial design. Complete the fixed budget for both policies and preserve its actual horizon; this supports matched action cost and records subsequent work. Task completion can change a worker's preference only through its local latch. First-access cost is recorded before later excavation and shortcuts, while final distance may change. Undisposed spoil remains in the ordinary material histories and censored endpoint quantities.

At the endpoint, inaccessible goals have null first-access/distance and `deadline_censored=true`; do not substitute infinity or omit them. A goal first opened by the last budgeted action is successful, distinct from censoring. A zero-tick export records no milestone and zero observed opportunities. Any later study should report access frequency and censored cost jointly, rather than compare successful-only mean completion time. A restricted opportunity-to-access endpoint may use `first_access.opportunity` when present and the complete budget otherwise, but must be labelled restricted waiting and accompanied by the success count.

## Integration and compatibility

Keep the task layer inside `sugarscape_core::burrow`, in a separate access module. Reuse shared world transactions, observation, local route searches and runner accounting. Refactor only the target-weight seam and runner hosting needed for the optional task; do not duplicate the transport state machine or physical action loop.

The legacy `LabConfig`, `Setup`, `Episode`, `burrow` CLI outputs and `burrow_replay_json` schema remain unchanged. Existing entry points construct no active task. Add optional private task state to the world only where necessary; legacy fingerprints must not hash additional sentinel values. Known-goal fingerprints include task identity, coordinate, weight and every worker's completion latch because these affect continuation. Explore remains the legacy physical/controller state with external task diagnostics.

Add separate checked adapters: proposed CLI `sugarscape burrow-access --config FILE --seed U64 --ticks U32 --sample-every U32 --out DIRECTORY` and WASM `burrow_access_replay_json(config_json, seed, ticks, sample_every)`. Both serialize one shared AccessEpisode. Reuse full-width decimal seeds, f64 finite/integral boundary validation before u32 conversion, existing error codes, fixed exclusive output files and partial-output reporting. Exports are normalized access `config.json`, outer `episode.json`, sampled `maps.txt` and combined access/base integer `summary.json`.

Keep base ASCII maps unchanged; attach task coordinate and access state in outer metadata and adapter headings rather than overwriting a solid glyph with a misleading passable resource marker. No web controls, ModelKind registration, Minds changes or Hornvale modifications belong here.

## Exact verification before implementation acceptance

Use constructed tests before runtime code. Required cases include:

- Invalid/out-of-bounds/initially-open/non-diggable goals, zero or overflowing weights and unsupported public fixture kinds reject before mutation or output.
- On a private three-cell corridor setup, a single legal dig opens the goal, records one opportunity and exact route length two, and creates one carried spoil unit. No food unit is created or consumed.
- Full hands cannot open the goal. Failed digs, waits and moves consume ordinary opportunities but do not fabricate access. A far-away dig does not complete the task.
- Initial local view of a solid goal does not latch completion. Opening it does not remotely inform every worker; each latches only on a later local observation containing it. Latching while loaded still preserves transport priority.
- Enumerated integer tickets verify the goal multiplier and its weight-one reduction. Goal policy does not replace a valid retained target, remove nonimproving faces, bypass full occupancy or expose hidden frontiers.
- Explore produces the exact base Episode of the legacy route; KnownGoal at weight one produces the same physical action/choice projection, although its fingerprint/task state differ.
- A later shortcut changes final goal distance without rewriting the first-access milestone. Last-action success and an inaccessible deadline remain distinguishable.
- Sampling leaves all actions, task memory and fingerprints unchanged; zero ticks and off-cadence terminal frames are present. Clock, storage and opportunity checks retain their existing behavior.
- Serialize/replay deterministically, and compare complete native/WASM records for both policies and seeds 7 and maximum u64. Numeric rejection and CLI output-preservation tests apply at the new boundary.

Run relevant core/CLI/parity checks and existing regressions after an approved staged implementation plan. Acceptance is a fixed-seed scene and exact records, not a scientific treatment result. Construction seed outcomes must not be folded into a later judged protocol.

## Research sequence and remaining questions

First review this design and write its implementation plan. In parallel, the Burrow 1 protocol can become an executable archive/analysis implementation after its own review. Neither track needs to wait for the other's treatment outcomes, and neither should tune against those outcomes.

After resource access works, Burrow 2B can add actual extraction and delivery, accounting for finite resources and shared hands, and then ask who benefits. Later distinguish supplied destination knowledge from locally discovered resources, add physical signals or practices with appropriate source support, vary body and substrate, and preserve/repair layouts across cohorts. That creates the route toward differentiated homes, storage, fortifications, neighborhoods and inherited underworlds without crediting supplied goals or labels as emergent culture.
