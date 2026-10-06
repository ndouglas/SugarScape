# Deduction experiment

Frozen before first collection: `deduction-v1`, policy version 1, world seeds 0..128 (128 games per policy). Six-Agent default Wink: watched capability 0 sets Inactive after one round; attention 1; detection 750/1000; memory capacity 64, retention 8; one accusation, wrong accusation deactivates accuser; threat wins with at most one non-holder survivor or after 12 rounds.

Evidence, Reckless, Passive civilians are paired by world seed against an identical threat controller that uses capability 0 against a uniformly random public active nonself target, and never watches, talks, or accuses. Assignment and engine randomness use world seed. Controller seed arithmetic is wrapping u64: `world * 6364136223846793005 + identity * 1442695040888963407 + agent`. Identities: Evidence 1, Random 2, Reckless 3, Passive 4, frozen threat 5. Each controller owns a separate PCG stream. Evidence rotates public candidates using `(round + actor) % active_other_count`, trusts only retained direct Use events of objective capability, and reports each sighting at most once. No claim reliability model is present.

Predeclared directional hypotheses: Evidence captures more often than Reckless; Evidence makes fewer false accusations than Reckless. Failed hypotheses will remain reported as failed; rules, seeds, and thresholds will not be adjusted after collection.

Capture means Accuser victory; threat wins means Threat victory. Civilian losses count original non-holder Agents whose public final status is Inactive, excluding the hidden holder even after capture. False accusations count accepted accusations targeting an original non-holder (default scenario has no grant/revoke). Rounds are completed rounds. Per-game paired differences are Evidence minus Reckless and Evidence minus Passive for captures, threat wins, civilian losses, false accusations, and rounds. Direct evidence availability counts civilian Discuss requests containing retained direct objective-capability Use evidence of a currently legal accusation target; evidence accusations count accusations supported by that request. This availability measures delivered evidence, never inaccessible truth.

Perception fixture measures eligible watched uses, detected uses, and missed uses at detection endpoints 0 and 1000 with one scripted threat use per fixture. Inference is a separate noisy three-label finite channel: uniform prior and likelihood [0.75,0.25,0.25], exact posterior [0.6,0.2,0.2]. Maximum absolute posterior error compares that exact reference. Decision utility is independently enumerated over the three truth labels; regret is maximum reference utility including abstention minus chosen utility, with rewards +1/-1 and abstention 0. These exact checks do not describe Bayesian reasoning in whole games; Evidence uses direct sightings.

Archives are privileged host records containing the engine seed and transcript from which the hidden assignment is reconstructed. They must never be passed to controllers. Controllers receive only TurnRequest. This experimental module is independent of the model registry and browser UI.

First frozen collection (128 games per policy): Evidence captured 128, lost 55 civilians, made 0 false accusations, completed 186 rounds; Reckless captured 31, lost 565 civilians, made 565 false accusations, completed 128 rounds; Passive captured 0, lost 512 civilians, made 0 false accusations, completed 739 rounds. Both predeclared directional hypotheses held. This is descriptive evidence for these exact defaults and seed set, not a general superiority claim. Exact posterior error was 1.1102230246251565e-16, decision regret 0, and both perception endpoint fixtures passed. Full first results are retained in `.superpowers/sdd/2026-10-04-deduction-capabilities/first-diagnostics.json`.


## Run and control an Agent

The CLI defaults to Wink, seed 7, and Evidence. `run` uses the named built-in policy for every Agent, including the threat; the fixed random-target threat controller belongs only to `diagnose`. Policies are Evidence, Random, Reckless, and Passive. Random samples legal actions using its private random stream; Reckless accuses the lowest legal other Agent without evidence; Passive passes. Evidence uses its owned objective capability, rotates watch targets, accuses only from retained direct sightings, and can report a sighting once.

```sh
cargo run -p sugarscape-cli -- deduction run --scenario wink --seed 7 --policy evidence
cargo run -p sugarscape-cli -- deduction play --scenario wink --seed 7 --agent 0 --policy evidence --archive session.json
cargo run -p sugarscape-cli -- deduction play --resume session.json --agent 0
cargo run -p sugarscape-cli -- deduction diagnose --out diagnostics.json
cargo run -p sugarscape-cli -- deduction replay session.json
```

Run prints result JSON; the seed-7 Evidence run produced `{"fingerprint":2685439217561942331,"outcome":{"completed_rounds":1,"reason":"no_active_threat_holders","winner":"accuser"}}`. Diagnose prints JSON unless `--out` is supplied. Replay accepts either a strict CLI session file or a core replay archive and reports its fingerprint and finished/paused state. Fingerprints are deterministic reproducibility checksums, not authentication.

Play writes one compact JSON request per line, flushes it, and waits for one response line. Only the designated Agent's request is exposed. Here is the actual first request from `play --scenario wink --seed 7 --agent 0 --policy evidence`, pretty-printed for readability:

```json
{
  "actor": 0,
  "legal": {
    "accusation_budget": 1,
    "attention_capacity": 1,
    "can_accuse": false,
    "can_say": false,
    "can_use": false,
    "can_watch": true,
    "capabilities": [],
    "targets": [
      1,
      2,
      3,
      4,
      5
    ]
  },
  "observation": {
    "accusation_budget": 1,
    "capabilities": [
      {
        "delay": 1,
        "effect": {
          "kind": "set_status",
          "status": "inactive"
        },
        "id": 0,
        "label": "wink",
        "target": "other_active",
        "visibility": "watched"
      }
    ],
    "events": [],
    "grants": [],
    "objective": "accuser",
    "objectives": {
      "accuser_team": [
        0,
        1,
        2,
        3,
        4,
        5
      ],
      "capability": 0,
      "horizon_winner": "threat",
      "non_holder_survivor_threshold": 1
    },
    "roster": [
      {
        "id": 0,
        "status": "active"
      },
      {
        "id": 1,
        "status": "active"
      },
      {
        "id": 2,
        "status": "active"
      },
      {
        "id": 3,
        "status": "active"
      },
      {
        "id": 4,
        "status": "active"
      },
      {
        "id": 5,
        "status": "active"
      }
    ],
    "rules": {
      "accusation": {
        "budget": 1,
        "eligible": [
          0,
          1,
          2,
          3,
          4,
          5
        ],
        "enabled": true,
        "wrong_cost": "deactivate_accuser"
      },
      "display_name": "Watch for the wink",
      "max_rounds": 12,
      "memory": {
        "capacity": 64,
        "retention_rounds": 8
      },
      "observation": {
        "attention_capacity": 1,
        "detection_per_mille": 750
      }
    }
  },
  "phase": "attention",
  "protocol_version": 1,
  "request_id": 0,
  "round": 0
}
```

A valid response to that exact request, entered as one line, is:

```json
{"request_id":0,"actor":0,"action":{"kind":"watch","agents":[1]}}
```

Actions use `kind`: `pass`, `watch` with `agents`, `use` with `capability` and `target`, `say` with `claim`, or `accuse` with `target`. Claims use `kind`: `suspect` with `agent`, `saw_use` with `source`, `target`, and `round`, or `deny_use` with `round`. Copy actor/request_id from the current request and follow `legal`. Attention accepts Watch/Pass; Act Use/Pass; Discuss Say/Accuse/Pass. No extra fields are accepted, including inside Pass. Malformed, stale, wrong-actor, or illegal replies receive only `invalid response` on stderr and the identical outstanding request on stdout; rejected replies never enter history.

EOF pauses successfully at the outstanding request and never supplies a Pass. With `--archive`, the host saves a versioned privileged session including scenario, selected seat, policy kind, policy seed derivation, and the core archive. Resume restores the exact request and rebuilds internal controllers by feeding their historical private requests before applying accepted responses; the selected Agent's responses never advance an internal controller. Conflicting seed/scenario/seat/policy overrides fail. Agent IDs outside the scenario fail explicitly. If the selected Agent becomes Inactive before terminal victory, play emits `{"actor":0,"reason":"selected_agent_inactive","status":"paused"}` (actor is the selected ID), saves if requested, and stops. A completed game emits only `status: finished` and its public outcome. It never substitutes another Agent's request. Exit codes are 0 for completion/pause, 1 for I/O failure, and 2 for invalid flags/configuration/archive. Invalid response lines are recoverable retries.

An archive exposes the world seed, full configuration, accepted transcript, and full-state fingerprint; together these reconstruct hidden assignment, pending effects, and every Agent's private history. Keep archives out of controller input. Play stdout and generic protocol errors contain no archive, seed, assignment, or world fingerprint. The local host necessarily holds privileged state: this is an observation boundary, not a sandbox against someone reading their own files. External controllers can retain past requests beyond server memory limits. There is no bundled network provider or LLM API; an external JSON host can pipe the same protocol.

## Rules and limits

Each phase collects actions from active Agents in ascending ID order before resolving them. Silenced Agents remain physically active but cannot say or accuse. A watched use is visible only when the observer watches its source and detection succeeds; the actor always knows its own use. Recipient notices omit source. Public status changes omit cause. Claims are attributed speech and may be false; they are never promoted into direct evidence. Memory drops events older than eight rounds, then oldest visible entries over capacity 64. Public roster and own grants remain available.

Uses accepted at phase start resolve even if another same-phase use eliminates their source. At the Act boundary existing due effects precede new ones; creation order breaks ties. A round-r Wink is due at the end of Act in round r+1. Inactive dominates Silenced; effects on inactive targets are harmless no-ops but still yield configured use evidence. Accusations resolve together at Discuss using phase-start grants, so repeated correct accusations are all correct. Correct accusation removes the objective threat capability; wrong accusation spends the token and deactivates its accuser. After the complete boundary, victory precedence is no active threat holders (Accuser), then at most one active non-holder (Threat), then 12 completed rounds (Threat).

The general engine supports 2..32 Agents, 1..32 capabilities, bounded Grant/Revoke/SetStatus effects, 0..1000-round delay, 0..1000-round memory retention, capacity 0..4096, detection 0..1000, and attention 0..Agent-count minus one. Wink assembly accepts 4..12 Agents; this CLI fixes six Agents and does not expose custom config flags. It is separate from the model registry and browser UI. The first experiment has no learned reliability model, universal rules language, renderer, or AIWolf compatibility.

Secret winking and varied detective/accusation penalties motivated this reconstruction ([Wink murder](https://en.wikipedia.org/wiki/Wink_murder)); the exact phase structure, delayed collapse, stochastic detection, budgets, and thresholds are our explicit choices. Structured claims draw inspiration from [AIWolf Protocol 3.6](https://aiwolf.org/control-panel/wp-content/uploads/2019/05/protocol_3_6.pdf), but only the three claim forms above are implemented. The finite Bayesian reference diagnostic is distinct from whole-game Evidence's direct-sighting policy. Initial frozen findings above remain unchanged; no policy/rule/seed tuning followed collection.

## Frozen testimony-v1 diagnostic

Before collection, testimony-v1 fixes tolerance 1e-12, truth priors 1/2,
copy/invert strategy priors 3/4 and 1/4, and signal accuracy 4/5.
Its references are: empty 1/2; uncertain speaker 13/20 and copy 3/4;
two independent signals 49/68; repeated shared signal 13/20;
two copying witnesses independent 16/17 and shared 4/5; inversion 1/5;
always-positive and half-accurate signals 1/2; verified truth copy 12/13,
unobserved second proposition 1/2, and transferred positive report 49/65.
Duplicate evidence must preserve the whole belief; contradictory shared-signal
reports must return ZeroEvidence and preserve the whole belief.
Decisions permit positive accusation (+1 correct, -1 incorrect) or abstention
(0). Uncertain-speaker utility is 3/10; inversion accusation utility -3/5,
optimal abstention utility 0 and credulous regret 3/5; uniform ties abstain.
These exact references come from independent rational enumeration.

Run the exact diagnostic with:

```sh
cargo run -p sugarscape-cli -- deduction testimony
```

The collected JSON reports `version: "testimony-v1"`, 13 fixture results,
three decision results and `passed: true`. Every fixture includes its complete
model, supplied evidence records, exact numeric references and Boolean checks.
The maximum collected posterior/reference error was 2.220446049250313e-16,
below the frozen 1e-12 tolerance. The uncertain-speaker posterior was
0.6500000000000001, giving an accusation utility of 0.30000000000000027.
Known inversion gave posterior 0.19999999999999996 and selected abstention;
a credulous accusation had utility -0.6000000000000001 and regret
0.6000000000000001. Uniform prior selected abstention with zero utility.
Tiny signed regret residuals (here -2.7755575615628914e-16 for the first
fixture) are retained and checked against tolerance, rather than clamped.
Duplicate replay passed both success and no-mutation checks. The contradictory
shared-signal fixture passed actual `ZeroEvidence` and no-mutation checks;
it does not invent a posterior for the rejected observation.

The command takes no flags. Successful references exit 0; failed references
are printed with their checks before exit 2. Argument errors also exit 2.
Output write or flush failures exit 1.

Reporting strategy persists across observations: copying a noisy signal can
produce an incorrect statement, and inversion can produce a true statement.
Signal accuracy and strategy are distinct assumptions. Shared signal groups
must be declared by the caller; different groups remain dependent through
latent truth and persistent profiles. A verified proposition requires an
independent legitimate observation channel. An accusation or matching claim
alone supplies no such verification.

This bounded model supports only explicitly declared worlds (up to 16
propositions, 32 speakers, 16 profiles, 256 hypotheses, 256 signal groups and
512 distinct accepted records). Impossible evidence reports a model
contradiction without changing belief. Unknown provenance or uncertain signal
quality are not automatically inferred. Testimony updates belief without
becoming an engine fact. There is no claim-aware game controller or strategic
opponent adaptation in this increment, and these exact diagnostics establish
no whole-game performance advantage.


## Finite testimony decision game

Run `sugarscape deduction testimony-game` for one deterministic JSON report. It takes no tuning flags. Exit 0 means correctness checks passed; failed exact or integrity checks print the report then exit 2, and output failures exit 1. Search success is descriptive and never changes the correctness gate. This separate diagnostic does not integrate the game into Wink `play` or add human/LLM hosting.

Two reporters receive private signals and persistent copy/invert profiles; a third Agent decides. Explicit permissions assign receiving signals and reporting to the reporters, verification to all three, and deciding only to the decider. Calibration reports are buffered together, independently verified, then followed by buffered live reports and an irreversible intervene/abstain decision. A copying profile can be wrong and an inverting profile can be truthful: these are fixed behaviors, with no optimized speaker utilities or strategic equilibrium.

The deciding observation includes public q/rho and permissions, calibration reports/truth, and live reports. It excludes private signals, actual profiles, live truth, search/game seeds and privileged archives. Reporting observations include only that reporter's own signal/profile; the second reporter cannot inspect the first buffered report. Archives contain config, seed, accepted responses and checkpoints for deterministic replay; they are privileged and unauthenticated. External reporters can deviate, but inference optimality applies only to the declared fixed reporting model.

Independent fair calibration/live truths and independent persistent profile priors define four conditional independent signal channels. Complete enumeration assigns exact masses to 256 worlds and 32 supported public histories per environment. Intervening pays +1 for true live T, -1 for false T; abstaining pays zero. Expected payoff and expected regret describe decisions, not wins or guaranteed loss in every realized game. Independent Python Fraction references, copied as source literals, check masses, baseline payoff/regret, intervention accounting, posterior errors/Brier scores and each Bayesian history. Bayesian regret is zero in all four environments. DirectEvidence and Passive both abstain in this assembly; their behavior is redundant, though only DirectEvidence defines a posterior.

The hand-supplied evolved structure has four integer genes (b,k in -16..16; u,d in 0..16). Copy trust is `sigmoid(logit(rho)+b/4+adjustment)`, with u/4 on a calibration match and -d/4 otherwise, preserving exact rho endpoints. Its score sums `(2q-1)(2trust-1)(2report-1)` over speakers; it intervenes only when the score exceeds k/8. Evolution learns parameters in that supplied trust/aggregation family. It produces no probability or calibration score.

Training uses q=4/5, rho=3/4 and exact expected payoff only. Both methods use seeds 0..19, derived independently of world generation by wrapping `seed*6364136223846793005 + identity*1442695040888963407` (GA identity 1, random identity 2; testimony-search-seed-v1). GA starts with 64 uniformly initialized genomes, keeps two elites through 50 replacement generations, uses tournaments of three with replacement, independent half-probability per-gene crossover and quarter-probability +/-1 clamped mutation. Ranking prefers higher exact payoff, then smaller integer L1 size, then lexicographic gene tuple, with stable exact ties. L1 preference is not evidence of simpler behavior. Repeats count toward the budget. Random search draws the same 3,164 candidate budget per seed with the same initialization and ranking. All 40 champions are selected on training only, owned and frozen before evaluation on any holdout. No settings, seeds, thresholds or objectives were tuned after collection.

First measured collection (testimony-game-v1) found all 20 GA and all 20 random-search champions attained the training optimum 57/250 = 0.228. Every GA champion was (b,u,d,k)=(-3,0,2,0). There is no GA training-payoff advantage over equal-budget random search. Training ties can still produce different holdout behavior. Holdouts publish their new q/rho to each listener while retaining the selected genes: this measures transfer under a declared channel, not unknown-channel misspecification.

| Environment | q | rho | Bayesian optimum | Credulous payoff | Skeptical payoff | DirectEvidence / Passive |
| --- | --- | --- | --- | --- | --- | --- |
| Training | 4/5 | 3/4 | 57/250 | 3/20 | 3/20 | 0 |
| Inversion prevalent | 4/5 | 1/4 | 57/250 | -3/20 | 3/20 | 0 |
| Less accurate | 3/5 | 3/4 | 23/400 | 1/20 | 1/20 | 0 |
| Uninformative | 1/2 | 3/4 | 0 | 0 | 0 | 0 |

Credulous incurs expected regret 189/500 under prevalent inversion and negative expected payoff; Passive loses attainable payoff whenever the optimum above is positive. These are expected comparisons, not guaranteed realized losses.

| Method | Environment | Minimum | Median | Maximum | Runs |
| --- | --- | --- | --- | --- | --- |
| genetic | training | 0.2280 | 0.2280 | 0.2280 | 20 |
| random | training | 0.2280 | 0.2280 | 0.2280 | 20 |
| genetic | inversion | 0.2175 | 0.2175 | 0.2175 | 20 |
| random | inversion | 0.2175 | 0.2175 | 0.2280 | 20 |
| genetic | less_accurate | 0.0440 | 0.0440 | 0.0440 | 20 |
| random | less_accurate | 0.0200 | 0.0440 | 0.0440 | 20 |
| genetic | uninformative | 0.0000 | 0.0000 | 0.0000 | 20 |
| random | uninformative | 0.0000 | 0.0000 | 0.0000 | 20 |

All GA inversion holdout payoffs were 0.2175, below the 0.228 Bayesian optimum; random seed 3 reached 0.228. All GA weaker-signal payoffs were 0.044, below optimum 0.0575; some random champions transferred worse. Neither method universally wins across environments. At q=1/2 all evaluated policies earned exactly zero, but random seeds with k<0 always intervened; this distinguishes information-free expected payoff from sensible tie behavior. The median is the average of the two central sorted results. The report retains all curves, exact payoff/mass numerators and denominators, per-history actions/regret, frozen evaluations and paired GA-minus-random differences. No statistical significance or general social-reasoning claim follows from these 20 seeds.

Each row below records the random champion in (b,u,d,k) order; every GA champion is (-3,0,2,0), with payoff 0.228 / 0.2175 / 0.044 / 0 across training / inversion / less-accurate / uninformative. Every random training payoff is 0.228 and uninformative payoff is zero. Differences shown are GA minus random for inversion and less-accurate; training and uninformative differences are zero for all seeds.

| Seed | Random genome | Random inversion | Random less-accurate | Inversion difference | Less-accurate difference | Random uninformative intervention rate |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | (0,3,6,1) | 0.2175 | 0.0267 | 0.0000 | 0.0172 | 0.0 |
| 1 | (-1,4,5,-1) | 0.2175 | 0.0267 | 0.0000 | 0.0172 | 1.0 |
| 2 | (1,1,7,1) | 0.2175 | 0.0267 | 0.0000 | 0.0172 | 0.0 |
| 3 | (0,8,6,0) | 0.2280 | 0.0440 | -0.0105 | 0.0000 | 0.0 |
| 4 | (-4,1,1,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 5 | (-5,4,0,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 6 | (1,0,7,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 7 | (1,1,6,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 8 | (1,1,8,-1) | 0.2175 | 0.0200 | 0.0000 | 0.0240 | 1.0 |
| 9 | (-5,4,2,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 10 | (0,3,7,-1) | 0.2175 | 0.0200 | 0.0000 | 0.0240 | 1.0 |
| 11 | (1,0,8,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 12 | (-5,5,0,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 13 | (-3,2,3,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 14 | (-5,3,0,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 15 | (-1,1,5,-1) | 0.2175 | 0.0267 | 0.0000 | 0.0172 | 1.0 |
| 16 | (0,4,8,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 17 | (-1,1,5,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 18 | (-1,1,4,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |
| 19 | (-5,3,0,0) | 0.2175 | 0.0440 | 0.0000 | 0.0000 | 0.0 |

The first raw report and source/settings snapshot are retained in the ignored `.superpowers/sdd/2026-10-04-testimony-game/` evidence directory. Later additions of readable rates and additional integrity records change reporting only; the frozen experimental rules and first results remain intact. Existing `deduction diagnose`, `deduction testimony` and Wink run output are byte-compared against retained pre-change outputs. Full workspace verification is recorded separately before final delivery.

## Strategic reporting against frozen listeners

Run `sugarscape deduction strategic-reporting` for the separate fixed JSON diagnostic. It accepts no tuning arguments and adds no interactive host or provider mode. Exit 0 means independent exact references and report integrity passed; a false integrity report is printed before exit 2, while operational output failures use the existing error handling. Search success and GA superiority never determine this exit code.

One strategic reporter, one fixed-profile reporter and one decider have distinct permissions. The reporters receive private signals, report and observe verification; the decider observes verification and decides. Utility is a separate validated table. Intervention pays the decider +1 when live T is true and -1 when false; abstention pays zero. The strategic reporter uses opposed utility, the negative of that payoff. A lie or a report that opposes a private signal earns no direct reward; a noisy private signal can already disagree with truth.

The five responses are strategic/fixed calibration reports, strategic/fixed live reports, then the decider's irreversible action. Both reporting exchanges are buffered; neither reporter sees the other's current report. Only after both calibration reports does verified C become public. A strategic observation contains its own signal, and during live reporting its remembered calibration signal, public calibration reports and C. It has no assigned copy/invert profile. The fixed reporter additionally knows its persistent private profile. The decider sees calibration reports/C and live reports, q, fixed-profile prior, declared permissions and strategic utility. No request exposes T, other private signals/profiles, seeds, archives or evaluator state. Outcome reveals T and both utility scores only after the decision. Version-1 draws are C,T,fixed profile, strategic/fixed calibration signals, strategic/fixed live signals; rational endpoints still consume scheduled draws. Rejected responses preserve every later request and outcome. Archives replay all accepted prefixes and are privileged, unauthenticated records.

The finite policy is an 18-bit truth table: calibration bits 0/1 use the private signal; live row `8*calibration_signal + 4*own_calibration_report + 2*C + live_signal` occupies bit `2+row`. The other calibration report is public but deliberately ignored by this policy family. Calibration choices make some live rows unreachable; search retains redundant bits, and canonical exact optimization zeroes them. The unsigned encoding tie break is arbitrary and does not establish psychological simplicity.

The actual model has independent fair C/T, four conditionally independent signal draws at accuracy q, and one persistent fixed copy/invert profile with copy prior rho. Exact enumeration retains all 128 worlds and uses integer mass denominator `4*rho.denominator*q.denominator^4`. Actual-policy reference posteriors are true/total history masses; zero-mass histories have no posterior/action. The informed reference knows the candidate reporting policy and true generative rules, but no private realized truth or signal, and abstains on exact ties. Its decision regret is zero by construction and independently checked.

Frozen legacy listeners have an explicitly serialized assumed copy prior, separate from the actual fixed-profile prior. The original Bayesian listener continues to model both reporters as persistent copy/invert channels despite the disclosed strategic objective. Credulous assumes both copy; Skeptical and the published Evolved genome (-3,0,2,0) remain their original algorithms. Their channel assumptions can be wrong here. An assumed-model belief is not an actual-policy posterior; an impossible assumed history returns an error instead of fabricated evidence. Training and holdout configurations support all assumed histories; unsupported endpoint panels are rejected before optimization.

Training uses q=4/5, actual rho=3/4, opposed utility, and equally weighted frozen Bayesian-assumption/Credulous listeners. Assumed rho is 3/4. Independent Python Fraction world generation, history references and exhaustive enumeration of all 262,144 encodings agree with separate row decomposition: canonical optimum 81942 earns reporter utility 63/1000. There are 256 optimal encodings. Copy-calibration/invert-live (87382) is behaviorally equivalent at reachable rows; its redundant bits provide no extra utility. Receiver regret compares a listener to the policy-aware decision reference; reporter regret compares its utility to that panel's best response. These are different expected quantities, never guaranteed realized losses.

| Reporting control | Encoding | Opposed training utility | Reporter regret |
| --- | ---: | ---: | ---: |
| Copy signals | 174762 | -477/2000 | 603/2000 |
| Invert signals | 87381 | -81/2000 | 207/2000 |
| Always positive | 262143 | -63/400 | 441/2000 |
| Always negative | 0 | -9/200 | 27/250 |
| Copy calibration, invert live | 87382 | 63/1000 | 0 |
| Canonical optimum | 81942 | 63/1000 | 0 |

At the optimum, the informed receiver earns 3/10; legacy Bayesian earns -51/1000 (receiver regret 351/1000), and Credulous earns -3/40 (regret 3/8). Calibration report/truth agreement and report/private-signal opposition are reported separately for both phases.

Both search methods use seeds 0..19 and exactly 3,164 evaluated candidates per seed, counting repetitions. GA uses population 64, two elites, 50 replacement generations, tournaments of three with replacement, independent half-probability per-bit crossover and mutation probability 1/18 per bit. Random search samples the same uniform 18-bit representation and budget. Ranking maximizes exact training utility, then minimizes unsigned encoding, then preserves stable order. Streams use `strategic-reporting-search-seed-v1`, wrapping `seed*6364136223846793005 + identity*1442695040888963407` with identities 3/4 for GA/random, separate from the earlier testimony search. All 40 champions are frozen on training alone before evaluating the five holdouts: the training pair at q=3/5, and Skeptical/Evolved separately at q=4/5 and 3/5; actual and assumed rho remain 3/4. Passive is a separate zero-utility diagnostic control. Holdouts do not select or tune champions.

First measured collection (`strategic-reporting-diagnostic-v1`) passed all 4,178 exact/integrity checks. All 20 GA and all 20 equal-budget random champions reached training utility 63/1000 with zero reporter regret. Every GA champion used canonical encoding 81942. Random champions have different unreachable bits but identical behavior on reachable rows. There is no measured GA utility advantage: all 120 paired GA-minus-random differences across the six evaluated panels are exactly zero. No settings, seeds, thresholds or objectives changed after measurement.

| Target panel | Exact reporter optimum | Canonical optimum encoding | GA utility (all 20) | Random utility (all 20) | Reporter regret (both) |
| --- | ---: | ---: | ---: | ---: | ---: |
| training | 63/1000 | 81942 | 63/1000 | 63/1000 | 0 |
| training_pair_q3_5 | 1/40 | 81942 | 1/40 | 1/40 | 0 |
| skeptical_q4_5 | 3/40 | 5140 | 3/40 | 3/40 | 0 |
| skeptical_q3_5 | 1/40 | 5140 | 1/40 | 1/40 | 0 |
| evolved_q4_5 | 9/125 | 98342 | 51/1000 | 51/1000 | 21/1000 |
| evolved_q3_5 | 31/1000 | 98342 | 13/1000 | 13/1000 | 9/500 |

All champions transfer optimally to the weaker-signal training pair and both Skeptical panels. They remain suboptimal against the withheld Evolved listener: reporter regrets are 21/1000 at q=4/5 and 9/500 at q=3/5. This is transfer to these particular frozen listeners, not an unseen-policy population, equilibrium or general social reasoning. Receiver regret is still positive: the selected policies induce Evolved payoff -51/1000 with receiver regret 351/1000 at q=4/5, and payoff -13/1000 with regret 113/1000 at q=3/5.

Selected reporters copy their calibration private signal and invert their live private signal. Calibration report/truth agreement is 4/5 at training accuracy and 3/5 at weaker accuracy; live agreement is respectively 1/5 and 2/5. Private-signal opposition is exactly zero in calibration and one in live reporting. These phase measurements describe this finite strategy; they do not establish inferred motives, long-term reputation, human deception or psychological simplicity.

Each seed's selected unsigned encodings are shown below. Every row earns reporter utility 63/1000 on training, 1/40 on the weaker training pair, 3/40 and 1/40 against Skeptical at q=4/5 and 3/5, and 51/1000 and 13/1000 against Evolved at those accuracies. Both methods have the same utilities/regrets in every row; encoding differences occur only in unreachable policy rows.

| Seed | GA encoding | Random encoding |
| --- | ---: | ---: |
| 0 | 81942 | 88214 |
| 1 | 81942 | 86550 |
| 2 | 81942 | 86358 |
| 3 | 81942 | 85142 |
| 4 | 81942 | 91158 |
| 5 | 81942 | 82838 |
| 6 | 81942 | 86486 |
| 7 | 81942 | 84246 |
| 8 | 81942 | 82198 |
| 9 | 81942 | 82902 |
| 10 | 81942 | 89494 |
| 11 | 81942 | 83542 |
| 12 | 81942 | 83542 |
| 13 | 81942 | 85782 |
| 14 | 81942 | 88342 |
| 15 | 81942 | 94934 |
| 16 | 81942 | 83862 |
| 17 | 81942 | 82070 |
| 18 | 81942 | 82198 |
| 19 | 81942 | 83542 |

The report retains all 40 complete search curves, named controls and exact optima, full rules and separate listener assumptions, all per-champion listener evaluations/history masses/actions/posteriors, phase agreement/opposition, action/error accounting, reporter and receiver regret, summaries and paired differences. The median is the mean of the two central sorted exact utilities. Neither learning success nor manufactured trust is a correctness gate.

The first raw JSON, byte-identical repeat, premeasurement source/settings hashes, retained executable and independent reference evidence live under ignored `.superpowers/sdd/2026-10-05-strategic-reporting/`. Existing `deduction diagnose`, `deduction testimony`, `deduction testimony-game` and seed-7 Wink output are byte-compared with the retained pre-change executable outputs; the entire earlier guide remains an unchanged prefix. This separate finite experiment establishes no equilibrium, arbitrary utility language, repeated reputation mechanism or new interactive host mode.


## Strategy-aware listener inference

Run `sugarscape deduction strategy-inference` for the fixed, search-free JSON diagnostic. It accepts no seed, tuning, output-path or positional arguments. Exit 0 means exact independent reference comparisons and full report integrity passed; an inconsistent numerical report is emitted with `passed=false` before exit 2. Write and flush failures follow the existing operational error handling.

The listener receives public rules, verified calibration truth C, both calibration reports, both live reports when available, and a declared policy catalog. It marginalizes over the reporting policy, truths, fixed copy/invert profile and signal noise. The reporting policy is selected once, independently of those latent variables, and retained across calibration and live phases. Actual policy identity, private realized truth, seeds and evaluation scores never enter listener inference. The four reported inference models (two priors at q=4/5 and 3/5) are separate from the actual-distribution scoring rows; each retains eight calibration views/beliefs and 32 complete public-history observations/decisions, including every conditional policy probability and live predictive mass.

Both catalogs contain Copy, Invert, Always positive, Always negative, and Copy-calibration/invert-live. Uniform assigns named-order weights [1,1,1,1,1]; optimization-informed assigns [1,1,1,1,16]. These are supplied prior assumptions. The second prior reflects the former equally weighted frozen Bayesian/Credulous reporter-training panel at q=4/5, rho=3/4 and opposed utility; the diagnostic performs no new search, optimization, RNG sampling or reporter learning. Actual and legacy assumed fixed-copy priors remain 3/4. Listeners are frozen to the two Strategy priors, Bayesian, Credulous, Skeptical, Evolved(-3,0,2,0), and Passive.

Copy and Copy-calibration/invert-live have the same calibration behavior, so all eight calibration observations preserve their odds: Copy:Copy-calibration/invert-live is 1:1 under Uniform and 1:16 under optimization-informed at either accuracy. Calibration updates other policy odds without consuming live evidence. Live reports then update policy probabilities and the T posterior. For example, with verified C=false and both calibration and live reports false at q=4/5, the complete-history Copy:Copy-calibration/invert-live odds become 212:113 under Uniform and 53:452 under optimization-informed; inferred T probabilities are respectively 8/31 and 2312/4909. Those live-conditioned changes are distinct from the supplied calibration odds.

Intervention pays the receiver +1 for true T and -1 for false T; abstention pays zero. The opposed reporter utility is the negative of receiver payoff in every row below. The Strategy listener intervenes only when inferred P(T=true)>1/2 and abstains on exact ties. Its belief error is inferred T probability minus the actual-distribution T posterior; the reported maximum takes absolute error over supported positive-mass histories. Legacy listeners provide actions only, so their listener posterior and belief-error fields remain unavailable. The actual-distribution posterior and informed decision reference are privileged evaluation quantities, separate from inferred listener beliefs. A positive-mass history without assumed support is an explicit failure with no action or unconditional score; support failure is not abstention. Zero actual-mass histories have no action and do not count as unsupported. All 108 measured scoring rows have full support; that result applies to these frozen settings.

### Separate self-mixture comparisons

Each actual mixture uses one catalog and its matching Strategy listener plus all five legacy listeners. The following table contains all 24 mixture payoffs. Distinct actual priors define different opponent distributions and are evaluated separately. The matching Strategy payoff is also that mixture's informed benchmark; its decision regret and maximum belief error are exactly zero in all four cases. For any legacy column, mixture decision regret equals the matching Strategy benchmark minus that column's payoff.

| q | Actual prior | Matching Strategy / mixture benchmark | Bayesian | Credulous | Skeptical | Evolved | Passive |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 4/5 | Uniform | 951/5000 | 153/1250 | 9/200 | 9/200 | 153/1250 | 0 |
| 4/5 | Optimization-informed | 1011/4000 | -153/20000 | -9/200 | -9/200 | -153/20000 | 0 |
| 3/5 | Uniform | 253/5000 | 21/1000 | 3/200 | 3/200 | 12/625 | 0 |
| 3/5 | Optimization-informed | 2/25 | -27/2000 | -3/200 | -3/200 | -99/20000 | 0 |

The higher optimization-informed self-mixture payoff reflects its different supplied opponent distribution. Common-opponent transfer below compares the two Strategy priors against the same actual fixed policies.

### Common fixed-policy transfer

All 40 retained GA/random champions from the previous experiment canonicalize to the single Copy-calibration/invert-live behavior 81942. Their method, seeds and original encodings remain in report provenance and in the preceding seed table; they are behavioral clones, not 40 independent challenges. No GA utility advantage is implied. Together with the other four named controls and withheld encoding 98342, this gives six distinct fixed behaviors. Encoding 98342 was the previous Evolved-target optimum and is absent from both current prior catalogs; no new best response is computed here. The report retains 46 provenance entries: five controls, the withheld policy, and all 40 champions.

The next table contains all 84 common-opponent receiver payoffs: six canonical policies × two accuracies × seven listeners. U and O mean Strategy with Uniform and optimization-informed priors. Canonical encoding removes only unreachable live rows; it preserves each retained raw policy's behavior.

| q | Actual policy | Canonical encoding | Strategy U | Strategy O | Bayesian | Credulous | Skeptical | Evolved | Passive |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 4/5 | Copy | 163882 | 39/250 | -3/25 | 63/250 | 9/40 | 9/40 | 63/250 | 0 |
| 4/5 | Invert | 5441 | 63/250 | 57/200 | 39/250 | -3/40 | -3/40 | 39/250 | 0 |
| 4/5 | Always positive | 246723 | 69/400 | 33/200 | 33/200 | 3/20 | 3/20 | 33/200 | 0 |
| 4/5 | Always negative | 0 | 69/400 | 33/200 | 9/100 | 0 | 0 | 9/100 | 0 |
| 4/5 | Copy calibration, invert live | 81942 | 99/500 | 57/200 | -51/1000 | -3/40 | -3/40 | -51/1000 | 0 |
| 4/5 | Withheld Evolved-target policy | 98342 | 39/250 | 51/250 | -9/125 | -3/200 | -3/200 | -9/125 | 0 |
| 3/5 | Copy | 163882 | 19/500 | -1/10 | 2/25 | 3/40 | 3/40 | 7/125 | 0 |
| 3/5 | Invert | 5441 | 59/1000 | 1/10 | -1/100 | -1/40 | -1/40 | 1/125 | 0 |
| 3/5 | Always positive | 246723 | 1/20 | 0 | 17/400 | 1/20 | 1/20 | 7/200 | 0 |
| 3/5 | Always negative | 0 | 1/20 | 0 | 7/400 | 0 | 0 | 1/100 | 0 |
| 3/5 | Copy calibration, invert live | 81942 | 7/125 | 1/10 | -1/40 | -1/40 | -1/40 | -13/1000 | 0 |
| 3/5 | Withheld Evolved-target policy | 98342 | 19/500 | 1/50 | -7/1000 | 3/200 | 3/200 | -31/1000 | 0 |

The informed benchmark in the next table uses the actual fixed policy and its history masses, without private realized signals or truth. It intervenes when the actual-distribution T posterior exceeds 1/2. For every listener in the payoff table, decision regret is exactly this benchmark minus that listener's payoff. The Strategy regrets are shown explicitly alongside maximum absolute belief errors; legacy belief errors remain unavailable. Mixture regret above uses the mixture benchmark, while these transfer regrets use the actual-policy benchmark.

| q | Actual policy | Actual-policy benchmark | Strategy U regret | Strategy O regret | Strategy U max belief error | Strategy O max belief error |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 4/5 | Copy | 3/10 | 18/125 | 21/50 | 7600/15023 | 30400/53203 |
| 4/5 | Invert | 3/10 | 6/125 | 3/200 | 1080/3503 | 70560/462961 |
| 4/5 | Always positive | 9/50 | 3/400 | 3/200 | 1824/24185 | 27816/116165 |
| 4/5 | Always negative | 9/50 | 3/400 | 3/200 | 1824/24185 | 27816/116165 |
| 4/5 | Copy calibration, invert live | 3/10 | 51/500 | 3/200 | 1080/3503 | 70560/462961 |
| 4/5 | Withheld Evolved-target policy | 3/10 | 18/125 | 12/125 | 7600/15023 | 30400/53203 |
| 3/5 | Copy | 1/10 | 31/500 | 1/5 | 175/1221 | 2450/12987 |
| 3/5 | Invert | 1/10 | 41/1000 | 0 | 3720/41339 | 13020/425249 |
| 3/5 | Always positive | 1/20 | 0 | 1/20 | 14/795 | 2632/35085 |
| 3/5 | Always negative | 1/20 | 0 | 1/20 | 14/795 | 2632/35085 |
| 3/5 | Copy calibration, invert live | 1/10 | 11/250 | 0 | 3720/41339 | 13020/425249 |
| 3/5 | Withheld Evolved-target policy | 1/10 | 31/500 | 2/25 | 175/1221 | 2450/12987 |

The optimization-informed assumption benefits the behavior it emphasizes: against Copy-calibration/invert-live, payoff rises from 99/500 to 57/200 at q=4/5 (gain 87/1000) and from 7/125 to 1/10 at q=3/5 (gain 11/250). This is a prior-attributed benefit on those common opponents. Calibration cannot identify that behavior against Copy, because the two share their calibration rule. Against honest Copy, the same prior lowers payoff from 39/250 to -3/25 at q=4/5 and from 19/500 to -1/10 at q=3/5; the informed benchmarks are 3/10 and 1/10, and optimization-informed regrets are 21/50 and 1/5. Uniform also has positive transfer regret there. These negative honest-copy results are valid outcomes, and favorable self-mixture results do not establish general transfer reliability.

Against withheld policy 98342, optimization-informed payoffs are 51/250 and 1/50 at q=4/5 and 3/5, with regrets 12/125 and 2/25. The finite comparisons establish no adversarial guarantee or equilibrium. Neither payoff superiority nor a successful optimization result determines diagnostic correctness.

First measured collection (`strategy-inference-diagnostic-v1`) passed all 33,548 exact/reference checks and full payload integrity. A computationally separate Python Fraction full-joint oracle supplies posterior, prediction, action, mass, score and support references; the retained reference provenance report records incidental exposure to earlier diagnostic outputs. Comparisons use exact rational cross-products, and report integrity reconstructs the fixed protocol rather than trusting stored flags. The first 8,640,981-byte JSON and byte-identical repeat have SHA256 `5ee51e46dbd6b26f5fbab1e64d7a568afcae8a0e774ce4399d6f8d34cdbf0a0e` and are retained with source/settings/oracle/binary snapshots under ignored `.superpowers/sdd/2026-10-05-strategy-aware-listeners/`. No production source, priors, utility, listeners or settings changed after measurement; this appendix is a documentation-only addition. All five earlier diagnostic outputs were reproduced byte-for-byte with the rebuilt release CLI, and the complete earlier guide remains an unchanged prefix.
