"""The Credit episode's captioned claims, measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-27-credit-episode.md):
iv-5-credit (rule L₁₀,₁₀: ten-tick loans at 10 % simple interest a tick, the
engine's reading, so a loan is repaid at twice its principal) over 1000
ticks, against the same seeds with credit off.
"""

import statistics

import credit
import measure as m
from measure import median

TICKS = 1000
AT = 500  # the network's snapshot, as the survey's checks use


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows):
    """Each captioned claim, whether the 20 seeds support it, and why."""
    n = len(rows)
    older = _count(rows, lambda r: r["lender_age"] > r["borrower_age"])
    chains = _count(rows, lambda r: r["both"] > 0 and r["depth"] >= 3)
    deepest = [r["max_depth"] for r in rows.values()]
    ratios = [r["births"] / r["births_off"] for r in rows.values()]
    more = _count(rows, lambda r: r["births"] > r["births_off"])
    unequal = _count(rows, lambda r: r["gini"] > r["gini_off"])
    return [
        ("loans flow from old to young: lenders about 60, borrowers about 37",
         older == n and abs(median(rows, "lender_age") - 60) <= 3 and abs(median(rows, "borrower_age") - 37) <= 3,
         f"over the loans outstanding at tick {AT}: lenders' median age {median(rows, 'lender_age'):g}, borrowers' "
         f"{median(rows, 'borrower_age'):g} (medians over seeds); lenders older in {older} of {n}; "
         f"{median(rows, 'past'):.0%} of loans from lenders past childbearing"),
        ("some Flumps borrow and lend at once, and chains of debt form", chains == n,
         f"at tick {AT}, some Flump is both lender and borrower and the network is at least 3 levels deep in "
         f"{chains} of {n} seeds ({median(rows, 'both_share'):.0%} of the network's Flumps are both, median)"),
        # The spike sampled every tenth tick and saw 8; every tick, it is 10.
        ("the book saw five levels; ours reach about ten",
         9 <= statistics.median(deepest) <= 11 and min(deepest) >= 8,
         f"the deepest level over the {TICKS} ticks: {statistics.median(deepest):g} (median; {min(deepest)}–"
         f"{max(deepest)})"),
        ("with credit, about a fifth more Flumps are born, in all 20 worlds",
         more == n and 0.15 <= statistics.median(ratios) - 1 <= 0.25,
         f"births over {TICKS} ticks: {median(rows, 'births'):g} with credit, {median(rows, 'births_off'):g} without "
         f"(+{statistics.median(ratios) - 1:.0%}, median); more in {more} of {n}"),
        ("and they're a little less equal, in 18 of 20", unequal == 18,
         f"Gini at tick {TICKS}: {median(rows, 'gini'):.3f} with credit, {median(rows, 'gini_off'):.3f} without; "
         f"higher in {unequal} of {n}"),
    ]


def seed_row(tmp, seed):
    d = m.shot({"preset": "iv-5-credit", "ticks": TICKS, "seed": seed}, tmp, f"credit-{seed}")
    on, _ = m.run("iv-5-credit", seed, TICKS, tmp)
    off, _ = m.run("iv-5-credit", seed, TICKS, tmp, {"credit.enabled": False})
    fertile = {}
    for f in d.frames:
        fertile.update(f.fertility)
    snap = d.frames[AT]
    live = [l for l in snap.loans if l.lender in snap.agents and l.borrower in snap.agents]
    network = {l.lender for l in live} | {l.borrower for l in live}
    row = {
        "lender_age": statistics.median(snap.agents[l.lender].age for l in live),
        "borrower_age": statistics.median(snap.agents[l.borrower].age for l in live),
        "past": sum(snap.agents[l.lender].age > fertile[l.lender][1] for l in live) / len(live),
        "both": len(credit.both(live)),
        "both_share": len(credit.both(live)) / len(network),
        "depth": credit.depth(live),
        "max_depth": max(credit.depth(f.loans) for f in d.frames),
        "births": sum(float(r["births"]) for r in on[1 : TICKS + 1]),
        "births_off": sum(float(r["births"]) for r in off[1 : TICKS + 1]),
        "pop": float(on[TICKS]["population"]),
        "pop_off": float(off[TICKS]["population"]),
        "gini": float(on[TICKS]["gini"]),
        "gini_off": float(off[TICKS]["gini"]),
    }
    (tmp / f"credit-{seed}.frames.json").unlink()
    return row


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    keys = ["lender_age", "borrower_age", "past", "both_share", "depth", "max_depth", "births", "births_off",
            "pop", "pop_off", "gini", "gini_off"]
    lines = [f"## iv-5-credit, {TICKS} ticks (and the same seeds with credit off)", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seed: {m.typical_seed(rows, ['max_depth', 'lender_age', 'births'])}.",
              "Levels: pure lenders are level 1; every other Flump in the network sits one below its deepest lender."]
    medians = {k: median(rows, k) for k in keys}
    return lines, verdicts(rows), {"medians": medians, "seeds": len(rows)}
