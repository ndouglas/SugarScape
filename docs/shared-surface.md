# Learning a shared surface

Two Agents learn whether a generic status field can carry useful task information. The fixed diagnostic separates learning its physical behavior, receiving another Agent's data, and benefiting from that data. A shared persistent field saves inspection costs once both Agents have enough evidence to use it. A same-round handshake leaves persistence ambiguous, asymmetric participation can lose utility, and an unmodeled data-flipping field can produce confident wrong answers.

## Run the diagnostic

```sh
cargo run --release -p sugarscape-cli -- shared-surface diagnose > shared-surface.json
```

`shared-surface diagnose` accepts no tuning flags. It emits `shared-surface-diagnostic-v1`, protocol version 1. Exit 0 requires full current-payload integrity and independent exact references. Failed engineering validation emits the report with `passed: false` before exit 2; write/flush failures exit 1. Expected controller failures and adversarial mistakes remain valid diagnostic findings.

The first report is approximately 5 GB; use file redirection and streaming readers. Its trace library retains complete local histories, while episode references separately retain privileged lineage, failures and metrics. Integrity replays the frozen protocol and checks the current payload; stored success flags alone cannot validate a report. This standalone diagnostic has no browser-world or network-provider interface.

## Supplied rules and learned behavior

Agent A predicts Y, privately observed by B; B predicts X, privately observed by A. Four trials have fresh independent fair X/Y bits. Each Agent can instead inspect its own target through a trusted local operation. There are no task bits during calibration.

The field holds one raw symbol and writes overwrite it. Legal operations are read, write, wait, and live target inspection. A write returns generic acceptance without proving effect or delivery. The supplied alphabet contains `blank`, `probe0`, `ack0`, `probe1`, `ack1`, `data0`, and `data1`; blank is a reset value. The world stores symbols without interpreting their message meanings.

| Hidden mechanism | Write effect and read visibility | Round boundary |
| --- | --- | --- |
| Shared persistent (SP) | One common field | Retains symbol |
| Shared resetting (SR) | One common field | Clears to blank |
| Private persistent (PP) | Writer's own field; each Agent reads its own | Retains symbol |
| Inert (IN) | Accepted writes have no effect; reads are blank | Remains blank |

Every round has four public slots. Explicit phase/trial resets clear every mechanism's fields; memory and model beliefs persist. SP and SR can both support same-round acknowledgment. Only SP carries the delayed live messages in this schedule.

Roles, turn order, probe/acknowledgment grammar, data encoder/decoder, four-model family, priors, and participation rule are supplied. Agents learn the mechanism from their own paid action/outcome histories. They receive public schedule/reset events, own budget and private bit, but no other Agent's realized history, actual mechanism, hidden target, evaluator reward, lineage, seed or archive. Exact inference marginalizes possible hidden peer histories and private bits using the original episode prior; later observations update without recounting earlier evidence. An informed hypothetical peer receives the candidate mechanism's point mass, avoiding leakage of the actual mechanism.

Unknown controls start uniform over SP/SR/PP/IN. Known controls start at the actual in-family mechanism's point mass and run the same conditional calibration probes. Their time-zero certainty is supplied knowledge. No-communication controls wait through calibration, always inspect live targets, and retain a uniform unexplored belief; their mechanism scores are ungraded.

## Paid calibration and identification

Round 1 schedules A's probe, B's read and conditional acknowledgment, then A's read. Round 2 begins with delayed reads, then B writes a second probe and A reads it. Round 3 lets A acknowledge that probe and B read the result. Each Agent pays two credits per calibration round, including waits.

With one round on SP or SR, both posteriors are `[1/2, 1/2, 0, 0]`: shared visibility is certain but persistence remains ambiguous. After two rounds both uniquely identify SP or SR. For PP/IN, A identifies its mechanism in round 1; B remains split equally between PP and IN until its round-3 read. Zero-round unknown controls retain uniform beliefs in these primary panels. Complete three-round calibration identifies every in-family mechanism for both Agents in all enumerated episodes. This is designed identifiability under supplied deterministic probes.

| Actual mechanism | A's first unique identification | B's first unique identification |
| --- | --- | --- |
| SP or SR | Round 2, slot 1; 3 own credits spent | Round 2, slot 2; 3 own credits spent |
| PP or IN | Round 1, slot 4; 2 own credits spent | Round 3, slot 2; 5 own credits spent |

On SP, visibility reaches one at A's round-1 slot 4 (2 credits) and B's round-1 slot 2 (1 credit). Retention and useful-channel belief reach one at their round-2 identification slots. The report retains complete posteriors, true-model probability, unique identification, visibility `P(SP)+P(SR)`, retention `P(SP)+P(PP)` and useful delayed-channel belief `P(SP)`. Tied maximum-weight labels do not count as knowledge. Properties never established at probability one are censored, with no invented zero-cost discovery; supplied certainty is distinguished from acquired certainty.

## Communication, accuracy and utility

At its send/inspect opportunity an Agent writes its observed bit iff `P(SP) > 1/2`; equality selects inspection. A sends in round 1 and reads in round 3 iff it sent. B makes a pre-read choice in round 2, then chooses send/inspection using its updated belief. Inspection can replace an inferred target with verified truth. Prediction uses the exact target-posterior majority, ties predicting zero. The rule and assumed reciprocal return are supplied behavior, without a guarantee that the peer reciprocates.

Read/write/wait cost one credit, inspection four. Each Agent starts with 48 credits for the entire episode; a fully inspected trial costs nine credits and full three-round calibration plus four inspected trials costs 42. Correct predictions earn 12 units each. Net utility is gross reward minus every spent credit; group utility sums the two Agents' net utilities.

Every one of the 48 in-family primary panels has target accuracy 1, four correct predictions and gross reward 48 per Agent. The fully inspecting no-communication baseline already reaches this accuracy ceiling. Communication's positive net benefit here comes entirely from saving three inspection credits on each of four trials, rather than additional correct answers.

| SP calibration rounds | Unknown/Unknown net per Agent | Known/Known net per Agent | No communication net per Agent |
| --- | ---: | ---: | ---: |
| 0 | 12 | 24 | 12 |
| 1 | 10 | 22 | 10 |
| 2 | 20 | 20 | 8 |
| 3 | 18 | 18 | 6 |

Known SP saves 12 net units per Agent at every length; unknown SP saves 12 only with two or three calibration rounds. Its communicating credits are 28 and 30 respectively, versus 40 and 42 for inspection. Each communicating Agent attempts four live sends, reads four live messages, decodes four data symbols and receives four other-Agent task-bearing writes. Under SR, PP and IN every primary treatment instead inspects, yielding per-Agent net `[12, 10, 8, 6]` at lengths 0–3 and zero net benefit over no communication. Group utility is twice each symmetric table entry.

Asymmetric SP pairs expose the participation cost. At lengths 0 and 1, the informed Agent sends but receives no reciprocal data: accuracy 1/2, two expected correct predictions, credits 24 or 26, and net 0 or −2. Its net benefit is −12. The unknown Agent inspects, reaches accuracy 1 and nets 12 or 10. This holds for either informed role; group utility is 12 or 8. At lengths 2 and 3 both communicate, netting 20 or 18 each.

Attempted sends, reads, decoded symbols, other-Agent task-bearing lineage reads, inspections, sender credits, accuracy, gross/net recipient benefit and group utility are separate measurements. Paired-bit sensitivity holds recipient private information and other trials fixed, changes the sender's bit, and reruns both controllers and transitions before truth disclosure. It measures dependence, not a reward effect. Inspection alone also makes predictions sensitive to target changes, so prediction sensitivity alone proves no communication. Privileged lineage remains evaluator-only; symbols and correlated guesses alone do not establish transfer.

## Restart, stale beliefs and DataFlip

Changed-mechanics episodes announce a possible change and reset fields. Operational inference restarts uniformly; the previous posterior is recorded only as a diagnostic. All 64 old/new/length restart panels complete all 256 sequences with no controller failures and reproduce the matching unknown new-mechanism panel. Old acquisition costs six credits per Agent in a separate episode; new budgets are 48 and relearning costs follow the identification table. No-change episodes receive the same public restart. This measures relearning, without a zero-shot transfer claim.

The stale diagnostic retains the old acquired point mass. With three calibration rounds, all twelve changed old/new pairs fail on every sequence: 3,072 failed episodes out of 4,096. The four unchanged pairs complete normally. For example, stale SP facing SR fails after A's delayed round-2 read, with spent credits `[3, 2]`; stale SP facing PP/IN fails at B's round-1 read, with `[1, 1]`. Unsupported history stops immediately, retains the contradictory observation's cost and trace, and never resets silently to uniform. Shorter panels may lack evidence to expose a change.

Any positive failed episode mass makes unconditional terminal accuracy, reward, net benefit and group utility unavailable (`null` on the wire, `None` in typed results). Spent credits and failure mass remain available. Missing terminal metrics mean an unfinished rollout, not zero reward or an average over selected survivors.

DataFlip is outside the supplied family: it preserves SP calibration behavior but swaps `data0`/`data1` on writes. Here the nominal known control is explicitly an SP-assumption control, not knowledge of the actual DataFlip mechanism. Unknown pairs at lengths 2/3 and SP-assumption pairs at all lengths complete without unsupported histories, confidently retaining SP while getting every target wrong. With three rounds they receive four peer task-bearing writes each but have accuracy 0, gross reward 0, credits 30, net −30 each and group net −60. Their gross benefit is −48 and net benefit −36 per Agent relative to inspection. Unknown zero/one-round and no-communication DataFlip controls inspect and remain perfectly accurate. Successful calibration and detectable task dependence cannot certify an unmodeled channel's truthfulness.

## Scope and scientific anchors

All 256 four-trial bit sequences have equal exact mass. The frozen grid has 48 primary, 32 asymmetric, 64 restart, 64 stale and 12 DataFlip settings: 220 settings and 56,320 episode rows. Another 12,288 primary episodes rerun with consistently renamed opaque surface/Agent identifiers; full posterior/action/cost/utility histories are invariant after normalization. This checks reliance on identifier spelling, without demonstrating semantic transfer, language invention, role discovery or discovery of new object categories.

Learning action effects is inspired by the affordance and transition-model framework of [Khetarpal et al. (2020)](https://proceedings.mlr.press/v119/khetarpal20a.html). [Derex and Boyd (2016)](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/) supplies a later anchor for communication topology and cumulative innovation; [Steels (1995)](https://pubmed.ncbi.nlm.nih.gov/8925502/) supplies one for evolving vocabularies. This finite experiment supplies its communication convention and topology and replicates none of those studies. It validates neither spontaneous cooperation nor a Hugging Face incident scenario. There is no unrestricted exploit discovery, learned language, spatial navigation, distributional training, strategic equilibrium or adversarial guarantee.

## Frozen measurement and reproducibility

First collection on October 7, 2026 passed all five check groups and full current-payload integrity. The independently authored Python integer/`Fraction` oracle computes its references from the approved mathematical definitions, separately from Rust. Its numeric author read no production Rust or collected results. The supplemental complete-trace adapter uses the original oracle, approved specification/contract and public-clock notes; its author read no production source or outputs. The current trace projection also incorporates the producer's textual clock-discrepancy report and the explicit public-clock ruling. Computational independence from production is claimed, with oracle reuse and these clarifications disclosed. Both retained trace fixture versions remain available; their numerical results agree.

The first and byte-identical repeat each contain 4,969,265,658 bytes with SHA256 `d4978f0727d6eeb7558e74a83185f549d67ab7f422c38d2ef758f122a25a1370`. They took 85.32 and 83.12 seconds for the direct CLI subprocess. Child maximum resident set sizes were 2,432,008,192 and 2,437,611,520 bytes, from macOS `getrusage(RUSAGE_CHILDREN)` in dedicated collector processes; these exclude collector/comparison overhead and are not whole-machine peaks. Runtime, bytes and memory describe collection resources, not learning outcomes.

Artifacts, exclusive first/repeat stdout and stderr, copied release executable, source/settings/oracle snapshots and exact reference/preservation records are retained under ignored `.superpowers/sdd/2026-10-06-shared-surface/`. Frozen binary SHA256 is `e62018cb786e5fe4b8fa565e44d3c9ded3b6fe50687f9396219ef526258e6714`. Source identity is the frozen per-file capsule, which explicitly records uncommitted runtime/test additions; baseline HEAD alone does not identify the collected source. No runtime, test, fixture, prior, controller or setting changed after freeze.

For manageable extraction, guide aggregate values come from all 220 retained setting proofs, each bound to the same source manifest and independently checked oracle. Their hashes were rechecked; the frozen source matches that manifest, and first-report validation separately replays the full typed payload before exit 0. This extraction does not load the multi-GB report into memory. Bounded report-header/tail checks, streaming SHA256 and exact byte comparison retain first/repeat identity.

All seven previous CLI outputs reproduce their retained bytes with the new frozen binary. All 66 preserved source hashes match, and the original deduction guide remains an unchanged prefix with only a new guide link appended.

For paid probe selection and task-directed stopping, see the [active-surface study](active-surface.md).
