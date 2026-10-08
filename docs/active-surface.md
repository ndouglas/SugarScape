# Choosing paid experiments on an uncertain surface

A single experimenting Agent chooses how much to probe, when to attempt communication and when to buy trusted inspection. At the retained prices and four-trial horizon, Agent A buys one probe round; Agent B skips probing and learns during its first communication attempt. These choices improve expected net task utility under the supplied model, while sacrificing some accuracy on individual noncommunicating mechanisms. An informed helper supplies the responses. This is an exact finite engineering experiment.

## Run and interpret the diagnostic

```sh
cargo run --release -p sugarscape-cli -- active-surface diagnose > active-surface.json
```

The command accepts no tuning flags and emits `active-surface-diagnostic-v1`, protocol version 1. Exit 0 requires full current-payload integrity: each compact episode is reconstructed and compared with a freshly evaluated episode, including both Agents' local beliefs, choices, alternative values, charged events, predictions and failures. The report also checks independent exact reference projections. Engineering validation failure emits `passed: false` before exit 2; write or late-flush failure exits 1. A supported wrong answer or an expected unsupported-history episode can occur inside an engineering-valid report.

The retained report is about 250 MB. Redirect output to a file and use a streaming reader. The 11,030-entry history library stores unique complete local prefixes, and checked references preserve exact entry counts and public clocks. A saved `passed` field, bounded header/tail inspection or an aggregate comparison alone cannot validate a modified report.

## Supplied world, partner and information

Agent A privately sees X and predicts Y; B privately sees Y and predicts X. Four fresh independent fair-bit pairs produce 256 equally weighted complete sequences. The same seven raw symbols, generic write acceptance, four-slot round order and physical mechanisms from the [shared-surface study](shared-surface.md) are retained:

| Mechanism | Surface | Across round boundaries |
| --- | --- | --- |
| SP: shared persistent | One common field | Retains its symbol |
| SR: shared resetting | One common field | Clears to blank |
| PP: private persistent | Each Agent reads its own field | Retains its symbol |
| IN: inert | Writes accepted without effect; reads blank | Remains blank |

Phase and trial resets clear the fields for every mechanism. The supplied alphabet is `blank`, `probe0`, `ack0`, `probe1`, `ack1`, `data0`, `data1`. Probe/acknowledgment grammar and bit encoding/decoding are supplied; the physical world does not interpret symbol meanings. Calibration has no task bits. The three probe rounds retain their original role-specific grammar and order; choosing another round does not invent a new experiment. Round 1 has A write a probe, B read and conditionally acknowledge it, then A read. Round 2 has delayed reads by A then B, a second probe written by B, and a read by A. Round 3 has A conditionally acknowledge that probe and B read, with the remaining scheduled slots charged as waits.

The unknown experimenter starts Uniform over SP/SR/PP/IN. The supplied responder knows the actual in-family mechanism and follows the original known-mechanism action rule. For a hypothetical candidate, its responder instead has that candidate's point prior and candidate-local history/bits. Known experimenter controls receive the matching informed point prior at time zero. On DataFlip the responder receives nominal SP, while the unknown experimenter keeps its four-model prior. No controller receives the inversion rule.

Only an Agent's own complete action/outcome prefix, own prior, own current private bit and credits, and public clock/reset/phase events reach inference or planning. Peer histories, actual mechanism, future private bits, target truth, realized reward feedback and write lineage remain privileged evaluation data. Both role orientations keep their original slot ownership: B's first live decision follows A's first physical action, whose realized outcome B has not observed. Own choices are interventions; their selection is not another likelihood factor. Responder target beliefs are replayed under the solved active policy rather than borrowed from the old fixed partner policy.

## Prices and the objective

Each Agent starts with 48 credits. Read, write and wait cost one; trusted target inspection costs four. Every completed probe round costs each Agent two. Stopping is a free public boundary decision: it removes the remaining calibration slots and their charges, then the live phase resets fields and issues private bits. The public probe length is available evidence about the experimenter's model belief; live routine selection stays private to the selecting Agent. Calibration cannot signal a task bit that has not been issued.

Each free prediction earns 12 if correct and zero otherwise; target ties predict zero. Net utility is reward minus all paid credits. The planner maximizes the experimenter's expected remaining net utility across the entire remaining episode. Each live choice selects a supplied inspection routine or send/read communication routine. The responder's assistance, costs and utility are measured separately; there is no participation negotiation or incentive guarantee. Group net is the sum of both Agents' net utilities.

An inspection trial costs nine including its scheduled waits; a communication attempt costs six and does not guarantee a response or correct decode. With isolated correctness probability q, the immediate attempt payoff exceeds inspection iff q > 3/4. A task action can also provide evidence that improves later choices, so that threshold alone is not the dynamic policy. On equal expected value, probing stops and live selection inspects. Computation is accounted for separately from physical credits.

Five policies are frozen: Adaptive chooses probing and live routines by exact remaining value; FixedThree completes three rounds then uses the original threshold controller; NoProbe skips calibration but optimizes live choices; InspectOnly skips calibration and always inspects; Known optimizes live choices with its supplied matching point prior. Alternative values in fixed controls follow their declared continuation policy. They are not all unrestricted Adaptive continuation values.

## Measured decisions and future evidence

Every Adaptive A episode continues once at the initial boundary: Continue = 49/4 versus Stop = 12. After that round, it stops for every actual mechanism. In the shared SP/SR branch, Stop = 33/2 versus Continue = 16; in PP/IN, Stop = 12 versus Continue = 10. It attempts the first live trial only in the shared branch. There Inspect = 15 versus Attempt = 33/2 for the remaining episode. The first attempted trial's immediate expected net is tied with inspection, but its paid read distinguishes persistence and changes the next three decisions. On actual SP it attempts all four trials; on actual SR it attempts once then inspects three times. PP/IN receive four inspections after the one probe.

Every Adaptive B episode stops at zero: Stop = 51/4 versus Continue = 49/4. Its first live choice has Inspect = 12 versus Attempt = 51/4. This attempt identifies SP versus the remaining SR/PP/IN class. It then attempts on SP and inspects on the non-SP class. B never needs to distinguish those three models to choose its useful routine. NoProbe B follows the same realized policy and outcomes. NoProbe A instead inspects all four trials, retaining its Uniform model belief. Role order and the responder's candidate-local action rule make these outcomes asymmetric.

These numbers are measured exact alternatives in the validated first report. The independent precollection mathematical derivations predicted these initial inequalities and were retained before collection; the values above are now bound to the frozen source and report. Expected utility before learning is distinct from utility conditional on an actual mechanism.

## Complete primary results

All 40 primary settings below include all 256 sequences, without cross-role pooling or selection of favorable mechanisms. E is the experimenter and H the supplied helper; ordered pairs are (E, H), so the physical names reverse in the B table. Accuracy is expected correct predictions divided by four. Reward, credits and net are exact per-episode means. All primary settings complete with zero failure mass.

### Experimenter A, helper B

| Mechanism | Policy | Probe rounds | Accuracy (E, H) | Credits (E, H) | Reward (E, H) | Net (E, H) | Group net |
| --- | --- | ---: | --- | --- | --- | --- | ---: |
| SP | Adaptive | 1 | (1, 1) | (26, 26) | (48, 48) | (22, 22) | 44 |
| SP | FixedThree | 3 | (1, 1) | (30, 30) | (48, 48) | (18, 18) | 36 |
| SP | NoProbe | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| SP | InspectOnly | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| SP | Known | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| SR | Adaptive | 1 | (7/8, 1) | (35, 38) | (42, 48) | (7, 10) | 17 |
| SR | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| SR | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| SR | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| SR | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| PP | Adaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| PP | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| PP | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| PP | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| PP | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| IN | Adaptive | 1 | (1, 1) | (38, 38) | (48, 48) | (10, 10) | 20 |
| IN | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| IN | NoProbe | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| IN | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| IN | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |

### Experimenter B, helper A

| Mechanism | Policy | Probe rounds | Accuracy (E, H) | Credits (E, H) | Reward (E, H) | Net (E, H) | Group net |
| --- | --- | ---: | --- | --- | --- | --- | ---: |
| SP | Adaptive | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| SP | FixedThree | 3 | (1, 1) | (30, 30) | (48, 48) | (18, 18) | 36 |
| SP | NoProbe | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| SP | InspectOnly | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| SP | Known | 0 | (1, 1) | (24, 24) | (48, 48) | (24, 24) | 48 |
| SR | Adaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| SR | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| SR | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| SR | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| SR | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| PP | Adaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| PP | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| PP | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| PP | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| PP | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| IN | Adaptive | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| IN | FixedThree | 3 | (1, 1) | (42, 42) | (48, 48) | (6, 6) | 12 |
| IN | NoProbe | 0 | (7/8, 1) | (33, 36) | (42, 48) | (9, 12) | 21 |
| IN | InspectOnly | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |
| IN | Known | 0 | (1, 1) | (36, 36) | (48, 48) | (12, 12) | 24 |

### Uniform distribution over actual mechanisms

This separate summary weights each actual SP/SR/PP/IN panel by 1/4 and retains each role and policy. It is an outer evaluation distribution. For Known it mixes four separately informed controls; its net 15 does not describe a single uncertain Agent with a Uniform own prior. The unknown policies actually start with that Uniform own prior.

| Experimenter | Policy | Accuracy (E, H) | Credits (E, H) | Reward (E, H) | Net (E, H) | Group net |
| --- | --- | --- | --- | --- | --- | --- |
| A | Adaptive | (31/32, 1) | (137/4, 35) | (93/2, 48) | (49/4, 13) | 101/4 |
| A | FixedThree | (1, 1) | (39, 39) | (48, 48) | (9, 9) | 18 |
| A | NoProbe | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |
| A | InspectOnly | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |
| A | Known | (1, 1) | (33, 33) | (48, 48) | (15, 15) | 30 |
| B | Adaptive | (29/32, 1) | (123/4, 33) | (87/2, 48) | (51/4, 15) | 111/4 |
| B | FixedThree | (1, 1) | (39, 39) | (48, 48) | (9, 9) | 18 |
| B | NoProbe | (29/32, 1) | (123/4, 33) | (87/2, 48) | (51/4, 15) | 111/4 |
| B | InspectOnly | (1, 7/8) | (36, 33) | (48, 42) | (12, 9) | 21 |
| B | Known | (1, 1) | (33, 33) | (48, 48) | (15, 15) | 30 |

Adaptive A improves experimenter net over NoProbe/InspectOnly by 1/4, helper net by 4 and group net by 17/4 under this outer distribution. Adaptive B equals NoProbe B; relative to InspectOnly it improves experimenter net by 3/4, helper net by 6 and group net by 27/4. Relative to FixedThree, Adaptive improves experimenter net by 13/4 for A and 15/4 for B. These gains do not hold conditionally on every mechanism: Adaptive A loses 5 experimenter units on SR and 2 on PP/IN versus inspection; Adaptive B loses 3 on every non-SP mechanism. On those first attempted trials, expected correctness is 1/2, reducing episode accuracy to 7/8. Inspection establishes perfect experimenter accuracy in every completed primary panel.

The helper can also lose utility when the experimenter declines communication. In SP InspectOnly panels the experimenter has accuracy 1 and net 12, while the helper sends four times, receives no other-Agent task-bearing writes, has accuracy 1/2 and nets 0. The pair's group net is 12. The experimenter's accuracy ceiling does not imply perfect helper accuracy.

## Transfer, identification and task benefit

The report retains per-Agent paid reads/writes/waits, attempted live sends, trusted inspections and privileged other-Agent task-bearing lineage reads separately from predictions and reward. A received raw symbol is not itself proof of task transfer. Lineage remains evaluator-only and excludes the recipient's own echo; it establishes a write origin, not truthfulness or positive reward.

On primary SP, Adaptive, FixedThree and Known in both orientations, plus NoProbe B, each Agent attempts four sends and receives four other-Agent task-bearing writes, with zero inspections. NoProbe A and InspectOnly in either orientation make no experimenter sends or lineage reads and use four experimenter inspections; their SP helper attempts four sends, receives no peer task writes and uses no inspections. On Adaptive SR A, and on Adaptive/NoProbe non-SP B, the experimenter attempts once, receives zero peer task writes and uses three inspections; the helper attempts zero times and inspects four times. All remaining primary settings have zero live sends/peer task reads and four inspections per Agent. These cases cover the entire primary matrix; probe reads/writes are additional paid activity, not live transfer counts.

Catalog certainty records the first point posterior at any declared model, its model label, checkpoint, cost and `Supplied` or `Observed` source. A point posterior and high confidence are separate from a true-model grade. Every in-family helper starts with supplied certainty at cost zero; Known experimenters do too. Time-zero knowledge is not acquired discovery.

Adaptive A identifies PP/IN at calibration round 1, slot 4 after two own credits; it identifies SP/SR at trial 0, live round 3, slot 1 after seven own credits. Adaptive/NoProbe B identifies SP at trial 0, live round 2, slot 2 after three own credits. On SR/PP/IN it only excludes SP, retaining [0, 1/3, 1/3, 1/3] with no unique identification; that censoring still supports correct later routine selection. FixedThree identifies every in-family model. NoProbe A and InspectOnly in either role retain Uniform model beliefs, despite perfect experimenter task accuracy. Identification, receiving information and benefiting from it are distinct outcomes.

## Complete DataFlip results and failures

DataFlip preserves SP probe behavior but inverts data symbols on writes. It is outside the supplied catalog. The six prospectively declared secondary settings use the frozen prices, priors, grammar and horizon; they are retained together with the primary results.

| Role | Mechanism | Policy | Probe rounds | Accuracy (E, H) | Credits (E, H) | Reward (E, H) | Net (E, H) | Group net |
| --- | --- | --- | ---: | --- | --- | --- | --- | ---: |
| A | DataFlip | Adaptive | 1 | (0, 0) | (26, 26) | (0, 0) | (-26, -26) | -52 |
| A | DataFlip | NoProbe | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| A | DataFlip | InspectOnly | 0 | (1, 1/2) | (36, 24) | (48, 24) | (12, 0) | 12 |
| B | DataFlip | Adaptive | 0 | (0, 0) | (24, 24) | (0, 0) | (-24, -24) | -48 |
| B | DataFlip | NoProbe | 0 | (0, 0) | (24, 24) | (0, 0) | (-24, -24) | -48 |
| B | DataFlip | InspectOnly | 0 | (unavailable, unavailable) | (7, 5) | (unavailable, unavailable) | (unavailable, unavailable) | unavailable |

The first five secondary settings complete with failure mass 0. B InspectOnly fails on all 256 sequences (failure mass 1); its ordered credits above are (experimenter B = 7, helper A = 5). Both terminal accuracy/reward/net and group net are unavailable, even though B has paid for one truthful inspection. Partial predictions, discoveries and charged events are retained; no terminal mean is taken over an invented completed rollout or selected survivors.

Adaptive A, Adaptive B and NoProbe B send and receive four other-Agent task-bearing writes per Agent, but every prediction is wrong: accuracy 0 and reward 0. Their costs yield net −26 per Agent for A's one-probe treatment and −24 for B's zero-probe treatments. They confidently identify nominal SP: experimenter A at live trial 0, round 3, slot 1 (7 own credits), and experimenter B at round 2, slot 2 (3 own credits). Their responder's nominal SP certainty is supplied. True-model probability and identification grades are unavailable for an out-of-catalog mechanism; nominal certainty does not identify DataFlip or certify truth. Adaptive A loses 38 experimenter net units versus its completed InspectOnly control.

NoProbe A and InspectOnly A instead inspect four times, attaining experimenter accuracy 1/net 12, while their nominal-SP helper sends four times, receives no peer task writes and reaches accuracy 1/2/net 0. With B InspectOnly, helper A later reads an inverted echo of its own earlier write at trial 0, round 3, slot 1. That observation contradicts A's nominal SP point prior, producing an immediate unsupported history with charged physical costs [A = 5, B = 7], before any further action. A's partial supplied SP discovery survives; terminal model projections are absent for both Agents. There is no defined terminal paired benefit against this failed B inspection control. Every secondary episode, including this expected failure, passes the engineering semantics checks.

## Planning computation and scientific scope

Exact rational backward induction enumerates finite candidate worlds, action-conditioned local outcomes and continuation values. Cache identity preserves the complete local prefix and latent physical/responder distinctions; equal mechanism marginals alone do not merge histories. The deterministic search statistics are distinct from physical costs:

| Experimenter | Own prior | Candidate/positive worlds | Decision states | Branches |
| --- | --- | --- | ---: | ---: |
| A | Uniform | 1024/1024 | 9532 | 71943 |
| A | SharedPersistent | 1024/256 | 4684 | 39787 |
| A | SharedResetting | 1024/256 | 2076 | 13475 |
| A | PrivatePersistent | 1024/256 | 2076 | 13475 |
| A | Inert | 1024/256 | 2076 | 13475 |
| B | Uniform | 1024/1024 | 8494 | 65205 |
| B | SharedPersistent | 1024/256 | 4684 | 39787 |
| B | SharedResetting | 1024/256 | 2076 | 13475 |
| B | PrivatePersistent | 1024/256 | 2076 | 13475 |
| B | Inert | 1024/256 | 2076 | 13475 |

Unknown policy controls share their role's Uniform candidate domain; their declared decision restrictions and continuation policies still differ. The collector compiles/replays 16 distinct protocol/prior engines per construction or integrity pass and preserves all 46 settings. Its physical-credit optimum is conditional on this finite model, supplied helper, prices, horizon and allowed routines. Separate runtime/memory measurements expose computation without treating it as a physical paid action.

[Khetarpal et al. (2020)](https://proceedings.mlr.press/v119/khetarpal20a.html) supplies an affordance and transition-model anchor. [Wang, Wang and Powell (2016)](https://proceedings.mlr.press/v48/wangb16.html) supplies a value-of-information anchor for choosing costly evidence to improve decisions; its stochastic binary-feedback model and knowledge-gradient algorithm are not implemented here. [Pacheco and Fisher (2019)](https://proceedings.mlr.press/v89/pacheco19a.html) supplies a mutual-information planning anchor for a separately designed information-seeking comparator. This experiment optimizes downstream net task utility and reproduces none of those studies numerically or empirically.

The apparatus supplies roles, clock, action vocabulary, probe grammar, encodings, helper behavior, finite hypotheses and task relevance. It demonstrates neither invented language, unknown exploit discovery, spontaneous cooperation, unrestricted social learning nor Hugging Face incident fidelity. Consistent opaque ID renaming checks spelling dependence within this apparatus, not semantic transfer to new objects or conventions. No price, prior, reward, horizon, probe-order or policy sweep was added after observing results.

## Frozen measurement and reproduction

First collection on October 7, 2026 (local time) used committed runtime/test source `cec392ba4d42001b6b1e23c0b77161330cd8f069` and a copied release executable. Both direct CLI calls exited 0, with empty stderr and all four engineering groups passing: frozen census, compact own histories, current episode semantics, and independent reference v2. Full current-payload validation reconstructs and freshly re-evaluates all 11,776 episodes across the 46 settings and 16 engines before exit 0. Engineering checks encompass expected supported mistakes and retained unsupported failures.

The first and exact byte-identical repeat each contain **249,848,367 bytes**, SHA256 `9e2ee6f7cf43c632fd8c7e764971c6080f3c5a7d4ddd6a9d0726daf981475b01`. Direct child wall times were **71.23 and 73.66 seconds**, with maximum resident sizes **222,642,176 and 226,377,728 bytes**. These are macOS `getrusage(RUSAGE_CHILDREN)` measurements from dedicated per-CLI collector processes containing only that CLI child; they exclude the release build, Python extraction/comparison overhead and other machine processes. Runtime can reflect host contention and is not a learning outcome.

The frozen binary SHA256 is `e173544a2dd29cd4cea01fd026d2abe3ad0d99095a56c53051ffbf96fc45a0ec` (12,922,880 bytes), built with `cargo build --release -p sugarscape-cli`, rustc 1.98.1 and cargo 1.98.1 on macOS arm64. The original freeze records no environment `RUSTFLAGS` override. A separately retained, late read-only configuration observation and release fingerprint confirm the preexisting global Cargo flag `--cfg tokio_unstable` was applied; this observation occurred after the first call began. No rebuild, source, binary or physical-setting change followed that observation. The unchanged original freeze retains its identity.

Artifacts are retained under ignored `.superpowers/sdd/2026-10-07-active-surface-experiments/`: exclusive first/repeat report and stderr, copied `first-sugarscape`, 535-file committed source/test/fixture/dependency snapshot, protocol settings, toolchain/build identity, oracle source/output, review gates, exact comparison and projection receipts. The approved 124-file source manifest SHA256 is `a5412ba4c159c978e516beae0f7ec9a1a1e064b7e152643873dd4d62d656f8a6`. Every one of the expanded 535 executable-source entries equals the committed revision and was rehashed before and after both calls; pending root plan/tracker documentation at freeze did not change the executable. No runtime, test, fixture or physical setting changed after freeze, and no revised measurement series was needed.

The independent integer/`Fraction` enumerator source SHA256 is `6eafbad345a3e23e312929e54ff5c9aac1538496bc1c1a73a26cd769f3300675`, output `e39b6bb0f76a626892223343f7c41a9843a8622363bb421fa54d0aedfe2e9f83`. Its author/reviewer did not read new production source or collected results. Precollection verification separately checked 93,928 decision records, 187,816 exact alternative values, all 46 panels/11,776 episodes, 322 designated complete traces, 86 paired-policy comparisons and 11,776 complete typed ID renames. This oracle/proof is independent verification evidence, not the first production report or its resource measurement.

Guide values are extracted directly from the validated first report by a 64 KiB streaming text reader that decodes one history/episode at a time (largest decoded object: 21,278 characters). It recomputes exact per-Agent metrics over all 256 sequences in every panel, compares every aggregate with both the emitted payload and the source-manifest-bound precollection proof, and retains decision/catalog/failure projections. The separately labeled outer means use all four actual mechanisms per role/policy. The compact `first-active-surface-projection.json` binds its inputs by SHA256; full report SHA256 and exact first/repeat comparison use 1 MiB streaming chunks. This extraction does not replace the CLI's full typed semantic validator.

All seven previous deduction/run commands reproduce both stdout and stderr with the frozen new binary. The original `shared-surface diagnose` also reproduces its retained 4,969,265,658 bytes exactly, SHA256 `d4978f0727d6eeb7558e74a83185f549d67ab7f422c38d2ef758f122a25a1370`, using streaming hash and byte comparison. All 101 protected old source/fixture/dependency hashes match; the original shared-surface guide remains an exact prefix with only a link to this guide appended. Prior experimental reports stay in place and read-only.

See the [experiment viewer guide](experiment-viewer.md) for browser episodes, checkpoint perspectives, and recorded results.
