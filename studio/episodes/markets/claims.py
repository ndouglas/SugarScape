"""The Markets episode's captioned claims, measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-26-markets-and-war-spikes.md):

- iv-1-spice, with trade off (the preset) and on: who shuttles between the
  sugar hills and the spice hills over ticks 100–200, and who survives to 1000.
- iv-3-trade (the book's market: 200 Flumps, vision 1–5): the first ticks'
  trades and which way they go; prices over 1000 ticks.
- Figure IV-6's sweep over 20 seeds: carrying capacity with and without trade
  at each mean vision 1–6.
- Figure IV-13's setting (iv-3-trade with lifetimes of 60–100 ticks, each
  death replaced): the Gini of sugar plus spice over ticks 500–1000, with
  and without trade.
"""

import csv
import statistics
import subprocess

import markets
import measure as m
from measure import median

SHUTTLE = (100, 200)
EARLY = 20
LONG = 1000
FINITE = {"lifespan.enabled": True, "replacement.enabled": True}


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def _mean(series, key, first, last):
    return statistics.fmean(float(r[key]) for r in series[first : last + 1])


def verdicts(rows, sweep):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    most = 0.9 * n
    near_one = _count(rows, lambda r: abs(r["ln_price_late"]) <= 0.05)
    narrower = _count(rows, lambda r: r["sd_late"] < r["sd_early"])
    walk_more = _count(rows, lambda r: r["shuttle_trade"] > r["shuttle"])
    less_equal = _count(rows, lambda r: r["gini_trade"] > r["gini"])
    right = sum(r["right"] for r in rows.values()) / max(sum(r["trades"] for r in rows.values()), 1)
    first = [r["exchanges1"] for r in rows.values()]
    wins, pairs = sweep
    return [
        ("without trade, about half the Flumps walk back and forth between the hills",
         0.4 <= median(rows, "shuttle") <= 0.6,
         f"{median(rows, 'shuttle'):.0%} of the Flumps alive through ticks {SHUTTLE[0]}–{SHUTTLE[1]} change hills "
         f"at least twice (median; {min(r['shuttle'] for r in rows.values()):.0%}–"
         f"{max(r['shuttle'] for r in rows.values()):.0%})"),
        ("of 400 Flumps, about 120 survive", 110 <= median(rows, "alive") <= 130,
         f"{median(rows, 'alive'):g} alive at tick {LONG} (median; {min(r['alive'] for r in rows.values()):g}–"
         f"{max(r['alive'] for r in rows.values()):g})"),
        ("neighbors swap what they have too much of for what they lack", right >= 0.9,
         f"in {right:.0%} of the trading pairs over ticks 1–{EARLY}, the Flump giving sugar held more sugar for its "
         f"needs, against spice, than its partner"),
        ("at first, hundreds of trades a tick", statistics.median(first) >= 200 and min(first) >= 100,
         f"exchanges at tick 1: {statistics.median(first):g} (median; {min(first)}–{max(first)})"),
        ("nobody sets the price, yet it settles near one sugar per spice", near_one >= most,
         f"mean log price over ticks 500–{LONG} within ±0.05 of zero in {near_one} of {n} seeds "
         f"(median {median(rows, 'ln_price_late'):+.3f})"),
        ("and the prices agree more and more", narrower == n,
         f"spread of log prices {median(rows, 'sd_early'):.3f} over ticks 1–50, {median(rows, 'sd_late'):.3f} over "
         f"{LONG - 50}–{LONG}; narrower in {narrower} of {n} seeds"),
        ("trade feeds more Flumps: in 118 of 120 worlds", (wins, pairs) == (118, 120),
         f"trade's carrying capacity is higher in {wins} of {pairs} seed–vision pairs (Figure IV-6, mean vision 1–6)"),
        ("but it doesn't stop the walking: more Flumps shuttle, not fewer", walk_more >= most,
         f"{median(rows, 'shuttle_trade'):.0%} shuttle with trade, {median(rows, 'shuttle'):.0%} without; more in "
         f"{walk_more} of {n} seeds"),
        ("and it makes them less equal, in all 20 worlds", less_equal == n,
         f"Gini of sugar plus spice over ticks 500–{LONG}, with lifetimes 60–100: {median(rows, 'gini_trade'):.3f} with "
         f"trade, {median(rows, 'gini'):.3f} without; higher with trade in {less_equal} of {n} seeds"),
    ]


def _shuttle(tmp, seed, trade):
    spec = {"preset": "iv-1-spice", "ticks": SHUTTLE[1], "seed": seed, "set": {"trade.enabled": trade}}
    d = m.shot(spec, tmp, f"spice-{seed}-{trade}")
    share = markets.shuttle_share(d.frames, d.width, markets.sides(d.capacity, d.spice_capacity), *SHUTTLE)
    (tmp / f"spice-{seed}-{trade}.frames.json").unlink()  # 200 ticks of two goods: large
    return share


def _early(tmp, seed):
    d = m.shot({"preset": "iv-3-trade", "ticks": EARLY, "seed": seed}, tmp, f"market-{seed}")
    right = trades = 0
    for k in range(1, len(d.frames)):
        r, t = markets.expected_direction(d.frames[k - 1], d.frames[k])
        right, trades = right + r, trades + t
    return {"exchanges1": sum(t.exchanges for t in d.frames[1].trades), "right": right, "trades": trades}


def seed_row(tmp, seed):
    prices, _ = m.run("iv-3-trade", seed, LONG, tmp)
    spice, _ = m.run("iv-1-spice", seed, LONG, tmp)
    finite, _ = m.run("iv-3-trade", seed, LONG, tmp, FINITE)
    barter_free, _ = m.run("iv-3-trade", seed, LONG, tmp, {**FINITE, "trade.enabled": False})
    return {
        "shuttle": _shuttle(tmp, seed, False),
        "shuttle_trade": _shuttle(tmp, seed, True),
        "alive": float(spice[LONG]["population"]),
        **_early(tmp, seed),
        "ln_price_late": _mean(prices, "mean_log_price", 500, LONG),
        "sd_early": _mean(prices, "sd_log_price", 1, 50),
        "sd_late": _mean(prices, "sd_log_price", LONG - 50, LONG),
        "gini_trade": _mean(finite, "gini_total", 500, LONG),
        "gini": _mean(barter_free, "gini_total", 500, LONG),
    }


def fig_iv_6(tmp):
    """Figure IV-6 over the studio's 20 seeds: (seed–vision pairs where trade's
    carrying capacity is higher, pairs), and each vision's medians."""
    runs = tmp / "fig-iv-6.csv"
    subprocess.run([m.CLI, "sweep", "--builtin", "fig-iv-6", "--seeds", str(len(m.SEEDS)), "--quiet",
                    "--runs-csv", runs, "--out", tmp / "fig-iv-6.json"], check=True)
    cells = {}
    with open(runs) as f:
        for r in csv.DictReader(f):
            cells.setdefault((int(r["x"]), int(r["seed"])), {})[r["series"]] = float(r["value"])
    wins = sum(c["1"] > c["0"] for c in cells.values())
    by_vision = {
        v: {s: statistics.median(c[s] for (x, _), c in cells.items() if x == v) for s in ("0", "1")}
        for v in sorted({x for x, _ in cells})
    }
    return (wins, len(cells)), by_vision


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    sweep, by_vision = fig_iv_6(tmp)
    keys = ["shuttle", "shuttle_trade", "alive", "exchanges1", "ln_price_late", "sd_early", "sd_late",
            "gini_trade", "gini"]
    lines = ["## iv-1-spice, iv-3-trade and Figure IV-13's setting", ""]
    lines += m.table(rows, keys)
    lines += ["", "## Figure IV-6: carrying capacity (mean population over ticks 200–300), medians over 20 seeds", "",
              "| mean vision | no trade | trade |", "|---|---|---|"]
    lines += [f"| {v} | {c['0']:.1f} | {c['1']:.1f} |" for v, c in by_vision.items()]
    lines += ["", f"Typical seed (market): {m.typical_seed(rows, ['exchanges1', 'sd_late'])}; "
              f"(walking): {m.typical_seed(rows, ['shuttle', 'shuttle_trade'])}.",
              "Hills: a site belongs to the good it can grow more of; sites growing equal amounts belong to neither."]
    medians = {k: median(rows, k) for k in keys}
    medians.update(pop_trade=by_vision[3]["1"], pop_no_trade=by_vision[3]["0"],
                   trade_wins=sweep[0] / sweep[1])
    return lines, verdicts(rows, sweep), {"medians": medians, "seeds": len(rows)}
