# Dutch disease: a study stub

**Date:** 2026-09-28
**Status:** a stub, persisted for later. Nothing is scheduled or built; the user may drive the science.
**Separate from:** the reproductions (`docs/papers.md`). This would be our own experiment, not a
reproduction; if an agent-based Dutch disease paper turns up, it moves to the queue instead.

## The question

Does the textbook Dutch disease survive myopic agents and costly moves between sectors — and when a
resource boom ends, does manufacturing come back?

## The mechanism to test

Corden & Neary (1982): a resource boom squeezes manufacturing (a tradable, priced by the world) two
ways. The resource-movement effect pulls labor into the booming sector; the spending effect raises
demand for non-tradables (services), whose price rises against tradables — the real exchange rate
appreciates. The dynamic version (van Wijnbergen 1984; Krugman 1987) adds learning by doing in
manufacturing, so the loss can outlast the boom.

## What the model needs (Sugarscape has none of it yet)

- **Tradables and non-tradables:** at least a manufactured good sold to an outside world at a fixed
  price, a service produced and consumed only locally (priced by local supply and demand), and the
  resource. Sugarscape's goods are all bartered locally.
- **Occupational choice:** agents pick a sector by its returns, with switching costs or
  sector-specific skills. Harvesting by location could be the resource sector.
- **Learning by doing** in manufacturing, per agent or per sector, as a switch.
- **A boom that starts and ends:** a windfall (a newly rich resource region) that runs out.

## What to measure

Manufacturing's share of labor over time; the price of services relative to tradables; output and
welfare by group (resource, manufacturing and service workers); and recovery — or hysteresis — after
the boom.

## Candidate findings

- Hysteresis from heterogeneous skills and switching costs alone, without learning by doing.
- The adjustment path the comparative statics hide, and who bears it.
- Policy switches as sweeps: saving the windfall abroad (a sovereign fund), subsidizing manufacturing,
  taxing resource rents and redistributing.
- The distributional side, alongside `2026-09-27-trade-and-inequality.md`.

## Sources to read first

W. Max Corden and J. Peter Neary, "Booming Sector and De-industrialisation in a Small Open Economy,"
*Economic Journal* 92 (1982); Sweder van Wijnbergen, "The 'Dutch Disease': A Disease After All?"
*Economic Journal* 94 (1984); Paul Krugman, "The Narrow Moving Band, the Dutch Disease, and the
Competitive Consequences of Mrs. Thatcher," *Journal of Development Economics* 27 (1987). Search for
an agent-based treatment before building (none is cited here).
