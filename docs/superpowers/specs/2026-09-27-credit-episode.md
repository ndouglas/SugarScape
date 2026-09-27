# Credit (episode 8): spike, storyboard and tune

**Date:** 2026-09-27
**Builds on:** the series plan. Approved storyboard and tune direction. Every caption is re-measured
by `studio/episodes/credit/claims.py` over 20 seeds (iv-5-credit, 1000 ticks, against the same seeds
with credit off).

## Spike: what the book says and what holds

| The book | Measured (20 seeds) | Verdict |
|---|---|---|
| Older agents lend to younger ones ("a primary consequence") | lenders' median age 60, borrowers' 37, older on 20 of 20; 66 % of loans from lenders past childbearing | Holds |
| Hierarchy: pure lenders on top, pure borrowers at the bottom; "as many as five levels" in its run | some Flump both lends and borrows, and the network is 3 or more levels deep, on 20 of 20; the deepest level over a run is **10** (8–11) | Holds, and deeper than the book's |
| "Many turn out to be both" | 12 % of the network's Flumps (median) | Holds as "some" (the book's other wording) |
| "Credit augments agent fertility" | births +20 % (5,335 against 4,466), population +19 %, both on 20 of 20 | Holds |
| (the series' thread) | Gini 0.180 with credit against 0.160 without, higher on 18 of 20 | Holds, and small |

**A reading, not the book's words:** L₁₀,₁₀ is a ten-tick loan at 10 %. The engine reads the rate as
simple interest per tick, so a loan is repaid at twice its principal (`CreditRule`'s doc). The book
doesn't say, so the captions say "with interest" and never "double".

**A caption that changed:** the spike sampled every tenth tick and saw 8 levels. Measured every tick,
the depth is 10, so the caption says "about ten".

## Storyboard (about 80 s)

| # | Caption | Shot |
|---|---|---|
| 1 | "An old Flump, past having children, lends out half its sugar…" | close-up `loan`: an old lender (62) beside a young neighbor (25), short of its endowment |
| 2 | "…to a young neighbor, to pay back in ten ticks, with interest." | close-up: the loan line; at tick 11, the repayment |
| 3 | "Loans flow from old to young: lenders about 60, borrowers about 37." | wide: loan lines across the world |
| 4 | "Some Flumps borrow and lend at once, and chains of debt form." | wide: lines colored by level; a debt-ladder panel |
| 5 | "The book saw five levels. Ours reach about ten." | the ladder at its deepest |
| 6 | "With credit, about a fifth more Flumps are born, in all 20 worlds." | bars: births with and without |
| 7 | "And they're a little less equal, in 18 of 20." | bars: Gini |
| 8 | "Every generation borrows from the one before." | title |
| 9 | "Credit — after Epstein & Axtell, 1996 / ndouglas.github.io/SugarScape" | end card |

## Tune: "Borrowed Time"

An original blues in 7/4 (the user's suggestion: "Money"'s feel, not its riff). A: a walking
bassline on electric bass, alone at first, over coin percussion (tambourine, cabasa, tinkle bell).
B: electric piano comping, a light kit, and a tenor sax on the melody. C: a bigger turnaround. Sting:
a "ka-ching" (tinkle bell, triangle, cowbell) as the first loan is made.

## New studio work

- **Engine:** loans and fertile ages in the frame dump; placements can set age and birth endowment.
- **Blender:** loan lines colored by level, a debt-ladder panel, age labels in the close-ups.
