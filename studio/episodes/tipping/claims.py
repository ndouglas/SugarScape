"""Following the Crowd, episode 2, "The tipping point": Schelling's bounded
neighborhood (1971, pp. 167–186; 1969), each captioned claim measured on the
shots' configs (see docs/superpowers/specs/2026-09-30-tipping-spike.md). With
his exact tolerance schedules the model is deterministic, so each result is
one run; where a caption speaks of any start, a grid of starts is run. The
rules are the survey's (`survey/src/claims/tipping.rs`).
"""

import subprocess

import measure as m

TICKS = 2000


def _end(preset, tmp, changes=None):
    rows, _ = m.run(preset, 1, TICKS, tmp, changes)
    last = rows[-1]
    return int(float(last["red_in"])), int(float(last["blue_in"]))


def _mixed(end):
    return end[0] > 0 and end[1] > 0


def verdicts(r, config):
    return [
        ("Schelling's second model: one neighborhood everyone prefers; you're in it, or you're not",
         config["model"] == "tipping", "one area, each person inside or outside"),
        ("each Flump has a limit: how many of the other color, for each of its own, it will live with",
         config["intercept"] == 2.0, "Fig. 18's schedules: straight lines from 2.0 to 0"),
        ("the least tolerant leave first; the most tolerant come in first", config["entry"] == "counting_self",
         "each step, a color's least tolerant insider leaves if discontent, else its most tolerant outsider enters"),
        ("100 Red, 50 Blue, most able to live with some of the other color; start mixed, and one color leaves "
         "entirely", r["fig18"] == (100, 0) and r["fig18_grid_mixed"] == 0,
         f"from 25 and 25: {r['fig18']}; {r['fig18_grid']} starts on a grid, {r['fig18_grid_mixed']} end mixed"),
        ("Schelling drew every mix both colors would stay in; in this one, no mix can last",
         r["fig18_grid_mixed"] == 0, f"{r['fig18_grid_mixed']} of {r['fig18_grid']} starts end mixed"),
        ("make them more tolerant, and a mix holds: 80 and 80, from any start with enough of each",
         r["fig19"] == (80, 80) and r["fig19_grid_missed"] == 0,
         f"from 50 and 50: {r['fig19']}; of {r['fig19_grid']} starts with 41+ of each, {r['fig19_grid_missed']} "
         f"miss 80 and 80"),
        ("start all Red, and it takes 28 Blue arriving together to get in", r["entry_fewest"] == 28,
         f"the fewest Blue entering together that reach the mix: {r['entry_fewest']}"),
        ("make one color twice as many, and the mix is lost", not _mixed(r["fig20"]), f"Fig. 20 ends {r['fig20']}"),
        ("let in only the 40 most tolerant Red, and it holds at 40 and 40", r["cap"] == (40, 40),
         f"Fig. 22 from 10 and 10 ends {r['cap']}"),
        ("make the least tolerant even less tolerant, and it holds too", _mixed(r["lesstol"]),
         f"the least tolerant two-thirds of Red intolerant: {r['lesstol']}"),
        ("make every Red less tolerant, and it doesn't", not _mixed(r["allless"]),
         f"all Red with a third of the tolerance: {r['allless']}"),
        ("every result Schelling reported here, the Flumps reproduce", r["survey_holds"],
         "the survey's tipping.* claims, all ten"),
        ("whether a neighborhood stays mixed depends not only on how tolerant people are, but on who, and how many",
         _mixed(r["lesstol"]) and not _mixed(r["allless"]) and not _mixed(r["fig20"]) and r["fig19"] == (80, 80),
         "who: the less tolerant two-thirds; how many: two to one loses Fig. 19's mix"),
    ]


def measure(tmp):
    r = {"fig18": _end("tipping-fig18", tmp), "fig19": _end("tipping-fig19", tmp),
         "fig20": _end("tipping-fig20", tmp), "cap": _end("tipping-fig22", tmp, {"start.red": 10, "start.blue": 10}),
         "lesstol": _end("tipping-less-tolerant", tmp),
         "allless": _end("tipping-fig20", tmp, {"red_schedule.intercept": 5 / 3, "start.red": 40, "start.blue": 40})}
    grid18 = [(a, b) for a in range(0, 101, 20) for b in range(0, 51, 10) if a + b > 0]
    ends18 = [_end("tipping-fig18", tmp, {"start.red": a, "start.blue": b}) for a, b in grid18]
    r["fig18_grid"], r["fig18_grid_mixed"] = len(ends18), sum(map(_mixed, ends18))
    grid19 = [(a, b) for a in range(41, 101, 12) for b in range(41, 101, 12)]
    ends19 = [_end("tipping-fig19", tmp, {"start.red": a, "start.blue": b}) for a, b in grid19]
    r["fig19_grid"], r["fig19_grid_missed"] = len(ends19), sum(1 for e in ends19 if e != (80, 80))
    r["entry_fewest"] = next(k for k in range(0, 101)
                             if _mixed(_end("tipping-fig19", tmp, {"start.red": 100, "start.blue": k})))
    survey = subprocess.run(["cargo", "run", "--release", "-q", "--", "--only", "tipping"], cwd=m.REPO / "survey",
                            capture_output=True, text=True, check=True).stdout
    verdicts_ = [line.split("|")[3].strip() for line in survey.splitlines() if line.startswith("| tipping.")]
    r["survey_holds"] = len(verdicts_) == 10 and all(v == "Holds" for v in verdicts_)
    config = m.preset_config("tipping-fig18", tmp)
    cfg = {"model": "tipping", "intercept": config["red_schedule"]["intercept"], "entry": config["entry"]}
    lines = ["## Schelling's bounded neighborhood (one run each: exact schedules)", ""]
    lines += [f"- {k}: {v}" for k, v in r.items()]
    medians = {"red_end_fig19": r["fig19"][0], "blue_end_fig19": r["fig19"][1], "entry_fewest": r["entry_fewest"]}
    return lines, verdicts(r, cfg), {"medians": medians, "seeds": 1}
