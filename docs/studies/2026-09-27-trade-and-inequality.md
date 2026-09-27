# Where trade's inequality comes from: a study note

**Date:** 2026-09-27
**Status:** a question with a first answer from throwaway probes, persisted for a proper study
later. Nothing here is in a video or a caption.
**Separate from:** the reproductions and Flump Studio (like `2026-09-26-war-and-society.md`). The
Markets episode's caption "And it makes them less equal, in all 20 worlds" stands on its own
measurement (`studio/episodes/markets/measurements.md`); this note asks *why*.

## The question

In Figure IV-13's setting (iv-3-trade with lifetimes of 60–100 ticks, each death replaced), the Gini
of sugar plus spice over ticks 500–1000 is higher with trade on 20 of 20 seeds (0.365 against 0.320).
Where does that inequality come from, and when does it build up?

## What the probes found

Seeds 1–20 for the first probe and 1–10 for the second. Both used the studio's `measure.run` (with
config changes) and frame dumps from `measure.shot`.

**Survivorship is ruled out in this setting.** With replacement, both worlds hold exactly 200 Flumps,
so there are no extra poor survivors to explain the gap. Trimming the trade world to the no-trade
population changes nothing (0.369 either way, still higher on 20 of 20). Survivorship may still matter
where population is free to differ, as in Figure IV-6's setting.

**When it opens:**

| Tick | 0 | 1 | 5 | 10 | 25 | 50 | 100 | 200 | 500 | 1000 |
|---|---|---|---|---|---|---|---|---|---|---|
| Gini gap (trade − none), median | 0 | +0.004 | +0.010 | +0.031 | +0.065 | +0.058 | +0.036 | +0.043 | +0.050 | +0.050 |
| Seeds with trade higher | — | 19 | 20 | 20 | 20 | 20 | 18 | 17 | 20 | 20 |

- The gap opens in the first tick, before anyone has died or been replaced.
- It peaks while trade is heavy and prices are unsettled, around tick 25.
- It settles near +0.05 once the market does.

**Where in the distribution:** the bottom half holds 25.1 % with trade against 28.6 % without, and the
top tenth 26.0 % against 24.4 %. Mean holdings are *lower* with trade (84.0 against 88.7 units), and
Flumps live a little longer (mean age 33.2 against 31.3).

**The mechanism: the desperate pay more.**
- An exchange conserves the pair's total of sugar plus spice. But at a price other than one, one
  partner ends with more units and the other with fewer.
- In trades that shift units, the **richer partner gains them 59 %** of the time (median; more than
  half on 10 of 10 seeds).
- The partner **closer to starving** (fewer ticks of its scarcer good left) **loses them 62 %** of
  the time (10 of 10).
- A Flump short of what it needs values it highly, agrees to a high price and pays in units. The net
  flow from poorer to richer partners is about 740 units over ticks 1–50 and about 4,100 over ticks
  51–300: a steady 15–16 units a tick, on every seed.
- Both partners still gain welfare in every exchange (rule T requires it). It's the holdings that
  drift apart.

**It builds over a lifetime.** Among Flumps under 30 the Gini is 0.261 with trade against 0.218
without. Among those 30 and older it's 0.390 against 0.311, higher on 10 of 10 in both bands, and
the gap nearly doubles with age. Small, repeated transfers add up, and trade's longer lives give them
longer to add up.

## Caveats

- Units: sugar and spice are counted as equal units. Prices sit near one, so this is close but not
  exact.
- Holdings, not welfare: this measures holdings. The Markets spike found the welfare gap much smaller
  (+0.02 at mean vision 1, shrinking to +0.002 at vision 13).
- "Closer to starving" is a stand-in for desperation, not the rule's own MRS.
- The second probe used 10 seeds; the scripts were throwaway and aren't in the repository.

## The study to run

1. **Per-trade transfer against desperation:** for every exchange, the units shifted against each
   partner's MRS (the rule's own measure of need) and against its wealth. Is the transfer explained
   by need, by wealth or by both?
2. **The price rule as the switch:** the book's geometric-mean price (the default) against Chapter
   IV note 15's random price (`PriceRule::Random`, already in the engine) and a fixed price of one (a
   new named switch). At a fixed price of one, no exchange shifts units, so if the mechanism is
   right, trade's extra inequality in holdings should vanish while trade's other effects remain.
3. **Holdings against welfare:** report both Ginis side by side, since the answer to "less equal?"
   depends on which one is measured.
4. **By setting:** repeat in Figure IV-6's setting (no replacement), where population can differ and
   survivorship could add to the mechanism or replace it.
5. **Over a lifetime:** the Gini within age bands over time, to separate transfers from the time
   they have to add up.

Ground rules as elsewhere: the book's rule stays the default, every departure is a named switch with
a sweep, 20 seeds, and a finding goes into the repository only with its measurement.

## A note on the Markets episode

The closing card, "Trade made the pie bigger. / It didn't share it out.", joins two settings. The
bigger pie is Figure IV-6's carrying capacity (no replacement). The unequal sharing is Figure IV-13's
setting (finite lives, replacement), where mean holdings are actually a little *lower* with trade.
Each half is measured; together they read as one world. Left as it is for now.
