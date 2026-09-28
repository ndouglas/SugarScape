"""The Living neighbors episode's captioned claims (Epstein's demographic
Prisoner's Dilemma), measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-demographic-pd-spike.md). Every board is
Epstein's: 30 × 30, wrapping.

- dpd-run-1 (Table 1: no maximum age) to cycle 500, under the published
  rules; dpd-closest, the same with three rules Epstein never states.
- dpd-soup: any Flump may meet any other, anywhere.
- dpd-run-4: the reward for mutual help cut from 5 to 1.
"""

import json

import episode
import measure as m
from measure import median

TICKS = 500
EPSTEIN = 779  # Table 1's cooperators at cycle 500 (± 15)
SOUP_GONE = 12


def _series(preset, seed, tmp):
    rows, _ = m.run(preset, seed, TICKS, tmp)
    return [{k: float(v) for k, v in r.items()} for r in rows]


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows, facts):
    """Each captioned claim, whether the seeds support it, and why. `facts`:
    the rules, read from the configs and checked on the close-up's dump."""
    n = len(rows)
    helpers = [r["helpers"] for r in rows.values()]
    cheats = [r["cheats"] for r in rows.values()]
    full = _count(rows, lambda r: r["population"] >= 899)
    reached = _count(rows, lambda r: r["helpers"] >= EPSTEIN)
    gone = _count(rows, lambda r: r["soup_gone"] is not None and r["soup_gone"] <= SOUP_GONE)
    ruined = _count(rows, lambda r: r["soup_end"] <= 1)
    dead = _count(rows, lambda r: r["low_end"] == 0)
    stated, closest = median(rows, "helpers"), median(rows, "closest")
    return [
        ("each turn, a Flump steps to an empty square and plays each neighbor", facts["walks_and_plays"],
         f"vision {facts['vision']} (the four squares next door), play: {facts['play']}"),
        ("two helpers earn 5 each; a cheat takes 6 from a helper; two cheats lose 5 each", facts["payoffs"] == (6, 5, -5, -6),
         f"T, R, P, S = {facts['payoffs']}"),
        ("reach 11, and a Flump splits in two; go broke, and it's gone", facts["clones"],
         f"fission at {facts['fission']}, the clone given {facts['endowment']} of it; in the close-up every clone "
         f"keeps its parent's side and every death is a Flump gone broke ({facts['broke']} of {facts['deaths']})"),
        ("start with 100 Flumps: half helpers, half cheats", facts["start"] == (100, 0.5),
         f"{facts['start'][0]} Flumps, each a helper with probability {facts['start'][1]}"),
        ("the land fills up, mostly with helpers: about 730 to 170",
         full == n and all(700 <= h <= 780 for h in helpers) and round(stated, -1) == 730
         and round(median(rows, "cheats"), -1) == 170,
         f"cycle {TICKS}: {stated:.0f} helpers (median; {min(helpers):.0f}–{max(helpers):.0f}) to "
         f"{median(rows, 'cheats'):.0f} cheats ({min(cheats):.0f}–{max(cheats):.0f}); the board full (899+ of 900) "
         f"in {full} of {n}"),
        ("Epstein counted 779 helpers; his stated rules give about 730; his count needs rules he never wrote down",
         reached == 0 and 720 <= stated <= 740 and abs(closest - EPSTEIN) <= 15,
         f"no seed reaches {EPSTEIN} ({reached} of {n}); with the working paper's one game a turn, founders with no "
         f"wealth and newborns acting at once: {closest:.0f} (median; "
         f"{min(r['closest'] for r in rows.values()):.0f}–{max(r['closest'] for r in rows.values()):.0f})"),
        ("the last helper is gone within 12 cycles, in 19 of 20 worlds", gone == 19,
         f"soup: no helper left by cycle {SOUP_GONE} in {gone} of {n} (at "
         f"{sorted(r['soup_gone'] for r in rows.values() if r['soup_gone'] is not None)}); in the others, "
         f"{[r['soup_end_helpers'] for r in rows.values() if r['soup_gone'] is None or r['soup_gone'] > SOUP_GONE]} "
         f"helper(s) left at cycle {TICKS}"),
        ("then the cheats ruin each other; by cycle 500, one Flump is left, or none", ruined == n,
         f"soup, cycle {TICKS}: {_count(rows, lambda r: r['soup_end'] == 0)} worlds empty, "
         f"{_count(rows, lambda r: r['soup_end'] == 1)} with one Flump, of {n}"),
        ("make helping pay less: Epstein saw booms and busts; here, 19 of 20 worlds die out", dead == 19,
         f"R = 1: nobody left at cycle {TICKS} in {dead} of {n} (extinct at cycles "
         f"{sorted(r['low_extinct'] for r in rows.values() if r['low_extinct'] is not None)})"),
    ]


def _facts(tmp):
    """The rules: from dpd-run-1's config and the close-up's dump."""
    config = m.preset_config("dpd-run-1", tmp)
    d = m.shot(episode_shot("closeup"), tmp, "closeup")
    clones_ok = all(
        d.frames[k].groups[child] == (d.frames[k].groups.get(p) if p in d.frames[k].groups else d.frames[k - 1].groups[p])
        for k in range(1, len(d.frames)) for child, (_, parents) in d.frames[k].births.items() if parents
        for p in parents[:1]
    )
    deaths = [c for f in d.frames for c in f.deaths.values()]
    return {
        "walks_and_plays": config["vision"] == 1 and config["play"] == "each_neighbor" and config["pairing"] == "space",
        "vision": config["vision"],
        "play": config["play"],
        "payoffs": (config["t"], config["r"], config["p"], config["s"]),
        "fission": config["fission_wealth"],
        "endowment": config["endowment"],
        "clones": clones_ok and all(c == "broke" for c in deaths) and config["fission_wealth"] == 11,
        "broke": deaths.count("broke"),
        "deaths": len(deaths),
        "start": (config["agents"], config["initial_cooperators"]),
    }


def episode_shot(name):
    return json.loads((episode.episode_dir("living") / "shots" / f"{name}.json").read_text())


def seed_row(tmp, seed):
    land = _series("dpd-run-1", seed, tmp)
    closest = _series("dpd-closest", seed, tmp)
    soup = _series("dpd-soup", seed, tmp)
    low = _series("dpd-run-4", seed, tmp)
    return {
        "helpers": land[-1]["cooperators"],
        "cheats": land[-1]["defectors"],
        "population": land[-1]["population"],
        "full_at": next((r["tick"] for r in land if r["population"] >= 899), None),
        "closest": closest[-1]["cooperators"],
        "soup_gone": next((r["tick"] for r in soup if r["cooperators"] == 0), None),
        "soup_end": soup[-1]["population"],
        "soup_end_helpers": soup[-1]["cooperators"],
        "low_end": low[-1]["population"],
        "low_extinct": next((r["tick"] for r in low if r["population"] == 0), None),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    facts = _facts(tmp)
    keys = ["helpers", "cheats", "population", "closest", "soup_end", "low_end"]
    lines = [f"## Epstein's 30 × 30 board, cycle {TICKS}", ""]
    lines += m.table(rows, keys)
    filled = {s: {"full_at": r["full_at"] or TICKS, "soup_gone": r["soup_gone"] or TICKS,
                  "low_extinct": r["low_extinct"] or TICKS} for s, r in rows.items()}
    lines += m.table(filled, ["full_at", "soup_gone", "low_extinct"])[2:]
    lines += ["", f"Typical seeds: land {m.typical_seed(rows, ['helpers', 'cheats'])}, "
              f"soup {m.typical_seed(filled, ['soup_gone'])}, low {m.typical_seed(filled, ['low_extinct'])}.",
              "full_at, soup_gone and low_extinct read the last cycle when it never happened."]
    medians = {k: median(rows, k) for k in keys}
    medians["epstein"] = float(EPSTEIN)
    return lines, verdicts(rows, facts), {"medians": medians, "seeds": len(rows)}
