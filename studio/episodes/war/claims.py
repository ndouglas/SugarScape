"""The War episode's captioned claims, measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-27-war-episode.md):

- iii-9-combat (rule C with the victim's whole wealth as loot), 2000 ticks,
  against the same seeds with combat off.
- iii-11-combat-fixed (a reward of 2 per kill, lifetimes of 60–100, every
  death replaced), 500 ticks.

Blue starts on the south-west hill, Red on the north-east.
"""

import statistics

import measure as m
import war
from measure import median

TICKS = 2000
FIXED_TICKS = 500
W = H = 50
SKIRMISH = 25  # at most this many kills over the run: no warlord rose


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def _home_share(frame, group, west, south):
    """The share of the Flumps on a hill's quarter that belong to `group`."""
    here = [i for i, a in frame.agents.items() if (a.x < W // 2) == west and (a.y >= H // 2) == south]
    return sum(frame.groups[i] == group for i in here) / len(here) if here else 0.0


def verdicts(rows, fixed):
    """Each captioned claim, whether the seeds support it, and why."""
    n = len(rows)
    conquests = {s: r for s, r in rows.items() if r["conquest"]}
    wiped = _count(conquests, lambda r: r["tribes_left"] == 1)
    unstoppable = _count(conquests, lambda r: r["unstoppable"])
    bursts = [r["burst"] for r in conquests.values()]
    shares = [r["top_share"] for r in conquests.values()]
    ratios = [r["richest"] / r["richest_peace"] for r in conquests.values()]
    skirmishes = {s: r for s, r in rows.items() if r["kills"] <= SKIRMISH}
    kept = _count(skirmishes, lambda r: min(r["blue_home"], r["red_home"]) >= 0.9)
    spread = _count(fixed, lambda r: min(r["quarters"].values()) >= 0.15)
    young = _count(fixed, lambda r: r["young"] > 0.5)
    return [
        ("for hundreds of ticks, almost nothing happens",
         bool(bursts) and statistics.median(bursts) >= 200,
         f"in the worlds that end in conquest, the first tenth of the kills is reached at tick "
         f"{statistics.median(bursts):g} (median; {min(bursts)}–{max(bursts)}); the first kill comes at tick "
         f"{median(rows, 'first'):g} (median)"),
        ("then one Flump gets rich enough that no one can stop it", unstoppable == len(conquests),
         f"in {unstoppable} of {len(conquests)} conquest worlds, from its third kill on, no Flump of the other tribe "
         f"is ever richer than the top killer, so under rule C none may attack it"),
        ("in 14 of 20 worlds, one tribe is wiped out, or nearly", len(conquests) == 14,
         f"one tribe holds at least 90% of the Flumps at tick {TICKS} in {len(conquests)} of {n} seeds; the other "
         f"tribe is gone in {wiped}"),
        ("nearly all the killing is done by a single Flump", bool(shares) and statistics.median(shares) >= 0.85,
         f"in the conquest worlds the top killer makes {statistics.median(shares):.0%} of the kills (median; "
         f"{min(shares):.0%}–{max(shares):.0%})"),
        ("the winner ends up about 25 times richer than the richest Flump in a world without war",
         bool(ratios) and 20 <= statistics.median(ratios) <= 30 and min(ratios) > 5,
         f"the richest Flump at tick {TICKS} against the richest in the same seed with combat off: "
         f"{statistics.median(ratios):.0f} times (median; {min(ratios):.0f}–{max(ratios):.0f}) in the conquest worlds"),
        ("in 5 of 20, no warlord rises: a few skirmishes, then the tribes keep their hills",
         len(skirmishes) == 5 and kept == len(skirmishes),
         f"{len(skirmishes)} of {n} seeds have at most {SKIRMISH} kills in {TICKS} ticks "
         f"({', '.join(str(r['kills']) for r in skirmishes.values())}); both home hills stay at least 90% their "
         f"own tribe in {kept} of them"),
        ("under its own rules there isn't a front: the killing is everywhere, and most of the dead are newcomers",
         spread >= 0.9 * len(fixed) and young >= 0.9 * len(fixed),
         f"iii-11 over {FIXED_TICKS} ticks: every quarter of the board holds at least 15% of the kills in {spread} of "
         f"{len(fixed)} seeds (smallest quarter {median(fixed, 'smallest'):.0%}, median); victims aged 5 ticks or "
         f"less: {median(fixed, 'young'):.0%} (median), a majority in {young} of {len(fixed)}"),
    ]


def seed_row(tmp, seed):
    d = m.shot({"preset": "iii-9-combat", "ticks": TICKS, "seed": seed}, tmp, f"war-{seed}")
    _, peace = m.run("iii-9-combat", seed, TICKS, tmp, {"combat.enabled": False})
    ranked = war.killers(d)
    top = ranked[0][0] if ranked else None
    third = war.nth_kill_tick(d, top, 3) if top is not None else None
    end = d.frames[-1]
    counts = war.kills_by_tick(d)
    row = {
        "kills": sum(counts),
        "first": next((k for k, c in enumerate(counts) if c), TICKS),
        "burst": war.burst_start(d),
        "top_share": war.top_killer_share(d) or 0.0,
        "unstoppable": third is not None and war.unstoppable_from(d, top, third),
        "majority": war.majority_share(end),
        "conquest": war.majority_share(end) >= 0.9,
        "tribes_left": len(set(end.groups.values())),
        "richest": max(a.sugar for a in end.agents.values()),
        "richest_peace": max(float(a["sugar"]) for a in peace),
        "blue_home": _home_share(end, 0, west=True, south=True),
        "red_home": _home_share(end, 1, west=False, south=False),
    }
    (tmp / f"war-{seed}.frames.json").unlink()  # 2000 ticks: about 50 MB
    return row


def fixed_row(tmp, seed):
    d = m.shot({"preset": "iii-11-combat-fixed", "ticks": FIXED_TICKS, "seed": seed}, tmp, f"fixed-{seed}")
    quarters = war.quarter_shares(d, W, H)
    ages = war.victim_ages(d)
    (tmp / f"fixed-{seed}.frames.json").unlink()
    return {"quarters": quarters, "smallest": min(quarters.values()),
            "young": sum(a <= 5 for a in ages) / len(ages) if ages else 0.0,
            "age": statistics.median(ages) if ages else 0.0}


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    fixed = {seed: fixed_row(tmp, seed) for seed in m.SEEDS}
    keys = ["kills", "first", "top_share", "majority", "richest", "richest_peace"]
    lines = [f"## iii-9-combat, {TICKS} ticks (and the same seeds with combat off)", ""]
    lines += m.table(rows, keys)
    conquest = sorted(s for s, r in rows.items() if r["conquest"])
    lines += ["", f"Conquest (one tribe ≥ 90 %): seeds {conquest}.",
              "Per seed (kills, war start, top killer's share, majority): "
              + "; ".join(f"{s}: {r['kills']}, {r['burst']}, {r['top_share']:.0%}, {r['majority']:.0%}"
                          for s, r in rows.items()),
              "", f"## iii-11-combat-fixed, {FIXED_TICKS} ticks", ""]
    lines += m.table(fixed, ["smallest", "young", "age"])
    conquests = [r for r in rows.values() if r["conquest"]]
    medians = {
        "conquest_seeds": len(conquests) / len(rows),
        "wiped_seeds": sum(r["tribes_left"] == 1 for r in conquests) / len(rows),
        "top_share": statistics.median(r["top_share"] for r in conquests),
        "others_share": 1 - statistics.median(r["top_share"] for r in conquests),
        "richest_war": statistics.median(r["richest"] for r in conquests),
        "richest_peace": statistics.median(r["richest_peace"] for r in conquests),
        "young": median(fixed, "young"),
    }
    return lines, verdicts(rows, fixed), {"medians": medians, "seeds": len(rows)}
