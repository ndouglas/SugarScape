"""The Living neighbors episode's captioned claims (Epstein's demographic
Prisoner's Dilemma), measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-demographic-pd-spike.md). Every board is
Epstein's: 30 × 30, wrapping.

- dpd-run-1 (Table 1: no maximum age) to cycle 500, under the published
  rules (GSS ch. 9: each neighbor played); and the 1997 working paper's rule
  (one game a turn against a random neighbor, which it prints beside the
  same Table 1) with the founders starting with no wealth, the one detail
  no source gives that way (the CD gives 6).
- dpd-soup: any Flump may meet any other, anywhere.
- dpd-run-4: Run 2's 100-cycle lives, with the reward for mutual help cut
  from 5 to 1 (Epstein: "ceteris paribus"; he allows extinction as one
  outcome, and describes long booms and busts).
"""

import json
import math
import statistics

import episode
import measure as m
from measure import median

TICKS = 500
EPSTEIN, EPSTEIN_SD, EPSTEIN_N = 779, 15, 30  # Table 1's cooperators at cycle 500: mean, s.d., runs
SOUP_GONE = 12


def welch_t(values):
    """Welch's t of a sample's mean against Table 1's (779 ± 15, 30 runs)."""
    mean, sd, n = statistics.fmean(values), statistics.stdev(values), len(values)
    return (mean - EPSTEIN) / math.sqrt(sd ** 2 / n + EPSTEIN_SD ** 2 / EPSTEIN_N)


def _series(preset, seed, tmp, changes=None):
    rows, _ = m.run(preset, seed, TICKS, tmp, changes)
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
    published = [r["helpers"] for r in rows.values()]
    working = [r["working"] for r in rows.values()]
    t_published, t_working = welch_t(published), welch_t(working)
    gone = _count(rows, lambda r: r["soup_gone"] is not None and r["soup_gone"] <= SOUP_GONE)
    ruined = _count(rows, lambda r: r["soup_end"] <= 1)
    dead = _count(rows, lambda r: r["low_end"] == 0)
    stated = median(rows, "helpers")
    return [
        ("each turn, a Flump steps to an empty square and plays each neighbor", facts["walks_and_plays"],
         f"vision {facts['vision']} (the four squares next door), play: {facts['play']}"),
        ("two helpers earn 5 each; a cheat takes 6 from a helper; two cheats lose 5 each", facts["payoffs"] == (6, 5, -5, -6),
         f"T, R, P, S = {facts['payoffs']}"),
        ("reach 11, and a Flump splits in two; go broke, and it's gone", facts["clones"] and facts["shown"],
         f"fission at {facts['fission']}, the clone given {facts['endowment']} of it; in the close-up every clone "
         f"keeps its parent's side and every death is a Flump gone broke ({facts['broke']} of {facts['deaths']}); the "
         f"followed helper 14 clones and cheat 4 goes broke in cycle 1, as the close-up shows: {facts['shown']}"),
        ("start with 100 Flumps: about half helpers, half cheats", facts["start"] == (100, 0.5),
         f"{facts['start'][0]} Flumps, each a helper with probability {facts['start'][1]}"),
        ("the land fills up, mostly with helpers: about 730 to 170",
         full == n and all(700 <= h <= 780 for h in helpers) and round(stated, -1) == 730
         and round(median(rows, "cheats"), -1) == 170,
         f"cycle {TICKS}: {stated:.0f} helpers (median; {min(helpers):.0f}–{max(helpers):.0f}) to "
         f"{median(rows, 'cheats'):.0f} cheats ({min(cheats):.0f}–{max(cheats):.0f}); the board full (899+ of 900) "
         f"in {full} of {n}"),
        ("Epstein counted 779 helpers; his published rules give about 730; his working paper's rule, one game a turn, "
         "comes close if the first Flumps start with nothing",
         abs(t_published) > 2 and round(statistics.fmean(published), -1) == 730 and abs(t_working) < 2,
         f"published rules: {statistics.fmean(published):.0f} ± {statistics.stdev(published):.0f} (mean ± s.d.; "
         f"Welch t {t_published:.1f} against Table 1's {EPSTEIN} ± {EPSTEIN_SD}); the working paper's rule with no "
         f"starting wealth: {statistics.fmean(working):.0f} ± {statistics.stdev(working):.0f} (t {t_working:.1f}); the "
         f"working paper's rule alone: {median(rows, 'working_only'):.0f} (median)"),
        ("the last helper is gone within 12 cycles, in 19 of 20 worlds", gone == 19,
         f"soup: no helper left by cycle {SOUP_GONE} in {gone} of {n} (at "
         f"{sorted(r['soup_gone'] for r in rows.values() if r['soup_gone'] is not None)}); in the others, "
         f"{[r['soup_end_helpers'] for r in rows.values() if r['soup_gone'] is None or r['soup_gone'] > SOUP_GONE]} "
         f"helper(s) left at cycle {TICKS}"),
        ("then the cheats ruin each other; by cycle 500, one Flump is left, or none", ruined == n,
         f"soup, cycle {TICKS}: {_count(rows, lambda r: r['soup_end'] == 0)} worlds empty, "
         f"{_count(rows, lambda r: r['soup_end'] == 1)} with one Flump, of {n}"),
        ("give Flumps 100-cycle lives and make helping pay less: Epstein saw long booms and busts, and sometimes "
         "extinction; here, 19 of 20 worlds die out", dead == 19 and facts["low"] == (1.0, 100),
         f"Run 4 (R, maximum age = {facts['low']}): nobody left at cycle {TICKS} in {dead} of {n} (extinct at cycles "
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
    one = d.frames[1]
    shown = (d.frames[0].groups[14] == 0 and d.frames[0].groups[4] == 1
             and any(parents == (14, 14) for _, parents in one.births.values()) and one.deaths.get(4) == "broke")
    low = m.preset_config("dpd-run-4", tmp)
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
        "shown": shown,
        "low": (low["r"], low["max_age"]),
    }


def episode_shot(name):
    return json.loads((episode.episode_dir("living") / "shots" / f"{name}.json").read_text())


def seed_row(tmp, seed):
    land = _series("dpd-run-1", seed, tmp)
    working = _series("dpd-working-paper", seed, tmp, {"initial_wealth": 0.0})
    working_only = _series("dpd-working-paper", seed, tmp)
    soup = _series("dpd-soup", seed, tmp)
    low = _series("dpd-run-4", seed, tmp)
    return {
        "helpers": land[-1]["cooperators"],
        "cheats": land[-1]["defectors"],
        "population": land[-1]["population"],
        "full_at": next((r["tick"] for r in land if r["population"] >= 899), None),
        "working": working[-1]["cooperators"],
        "working_only": working_only[-1]["cooperators"],
        "soup_gone": next((r["tick"] for r in soup if r["cooperators"] == 0), None),
        "soup_end": soup[-1]["population"],
        "soup_end_helpers": soup[-1]["cooperators"],
        "low_end": low[-1]["population"],
        "low_extinct": next((r["tick"] for r in low if r["population"] == 0), None),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    facts = _facts(tmp)
    keys = ["helpers", "cheats", "population", "working", "working_only", "soup_end", "low_end"]
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
