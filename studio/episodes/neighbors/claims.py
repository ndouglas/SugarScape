"""Following the Crowd, episode 1, "Neighbors like me": Schelling's own line
and checkerboard (1971), each captioned claim measured over 20 seeds (see
studio/measure.py and docs/superpowers/specs/2026-09-29-schelling-spike.md).
The decision rules are the survey's (`survey/src/claims/schelling71.rs`),
their numbers Schelling's.

- s71-line: his line (70, four neighbors each side, half alike), 50 rounds.
- s71-board: his checkerboard (13 × 16, 138 agents, eight neighbors, no fewer
  than half alike, the nearest square, rounds), 60 rounds; s71-third,
  s71-congregate, s71-integrate his Figs. 11, 16 and 17; the integrationists'
  moves compared with the same population (two-thirds Red) wanting half.
"""

import measure as m
from measure import median

LINE, BOARD = 50, 60
FIG8 = {"like": 0.90, "none": 2 / 3}
FIG9 = {"like": (5 / 6 + 4 / 5) / 2, "none": 0.40}


def _last(preset, seed, ticks, tmp, changes=None):
    rows, _ = m.run(preset, seed, ticks, tmp, changes)
    last = {k: float(v) for k, v in rows[-1].items()}
    last["total_moves"] = sum(float(r["moves"]) for r in rows)
    return last


def seed_row(tmp, seed):
    line = _last("s71-line", seed, LINE, tmp)
    board = _last("s71-board", seed, BOARD, tmp)
    third = _last("s71-third", seed, BOARD, tmp)
    company = _last("s71-congregate", seed, BOARD, tmp)
    mixed = _last("s71-integrate", seed, BOARD, tmp)
    same = _last("s71-board", seed, BOARD, tmp, {"red_share": 2 / 3})
    return {
        "groups": line["groups"], "mean_group": line["mean_group"], "line_like": line["like_share"],
        "like": board["segregation"], "none": board["no_unlike"], "ratio": board["like_ratio"],
        "third_ratio": third["like_ratio"], "third_like": third["segregation"],
        "company_like": company["segregation"],
        "mixed_moves": mixed["total_moves"], "mixed_left": mixed["unsatisfied"], "same_moves": same["total_moves"],
    }


def verdicts(rows, config):
    md = {k: median(rows, k) for k in next(iter(rows.values()))}
    return [
        ("Schelling worked it out by hand: a row of stars and zeros, then a checkerboard",
         config["board"] == (16, 13, 138) and config["line"] == 70,
         f"his board {config['board'][1]} × {config['board'][0]} with {config['board'][2]} chips, his line of "
         f"{config['line']} (1971, pp. 149, 155–156: \"done by hand and eye\")"),
        ("70 Flumps in a row; each wants at least half of its eight nearest neighbors like itself",
         config["line"] == 70 and config["radius"] == 4 and config["line_preference"] == 0.5,
         "length 70, four each side, half alike"),
        ("a few rounds later: about seven clusters of ten; nobody asked for more than half",
         6.5 <= md["groups"] <= 7.5 and 9 <= md["mean_group"] <= 11,
         f"groups {md['groups']:.1f} of {md['mean_group']:.1f} at round {LINE} (medians; his range five to eight, "
         f"9 to 14); like share {md['line_like']:.3f}"),
        ("then a checkerboard: 138 Flumps and 70 empty squares", config["board"] == (16, 13, 138),
         "16 × 13, 138 agents"),
        ("four in five neighbors end up alike; nearly two in five Flumps see no one of the other color",
         0.75 <= md["like"] <= 0.85 and 0.33 <= md["none"] < 0.4,
         f"like share {md['like']:.3f}, no opposite neighbor {md['none']:.3f} at round {BOARD} (medians)"),
        ("Schelling worked his boards by hand, and said they were too few to generalize; his came out a little more "
         "sorted", md["like"] < FIG9["like"] and md["none"] < FIG9["none"],
         f"here {md['like']:.3f} and {md['none']:.3f}; his Fig. 9 {FIG9['like']:.3f} and {FIG9['none']:.2f}, "
         f"Fig. 8 {FIG8['like']:.2f} and {FIG8['none']:.2f}"),
        ("ask for only a third, and the sorting is slight", md["third_ratio"] < 1.5,
         f"like-to-unlike ratio {md['third_ratio']:.2f} (median; his \"less than 1.5\"); like share "
         f"{md['third_like']:.3f}"),
        ("ask only for company — three of your own — and the town sorts anyway", md["company_like"] >= 0.75,
         f"like share {md['company_like']:.3f} (median; his \"just over 75%\")"),
        ("even Flumps who want a mixed street move more, and some are never satisfied",
         md["mixed_moves"] > md["same_moves"] and md["mixed_left"] > 0,
         f"moves {md['mixed_moves']:.0f} against {md['same_moves']:.0f} for the same population wanting half; "
         f"{md['mixed_left']:.1%} unsatisfied at the end (medians)"),
        ("nobody wanted a divided town; they just didn't want to be outnumbered", config["preference"] == 0.5,
         "the board's demand: no fewer than half alike"),
    ]


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    board = m.preset_config("s71-board", tmp)
    line = m.preset_config("s71-line", tmp)
    config = {"board": (board["width"], board["height"], board["population"]), "line": line["length"],
              "radius": line["radius"], "line_preference": line["preference"], "preference": board["preference"]["min"]}
    keys = list(next(iter(rows.values())))
    lines = ["## Schelling's line and checkerboard", ""] + m.table(rows, keys)
    lines += ["", "Typical seeds: line " + str(m.typical_seed(rows, ["groups", "mean_group"])) + ", board "
              + str(m.typical_seed(rows, ["like", "none"])) + ", third " + str(m.typical_seed(rows, ["third_ratio"]))
              + ", company " + str(m.typical_seed(rows, ["company_like"])) + ", mixed "
              + str(m.typical_seed(rows, ["mixed_moves", "mixed_left"])) + "."]
    medians = {k: median(rows, k) for k in keys}
    medians.update({"fig8_like": FIG8["like"], "fig9_like": FIG9["like"], "fig8_none": FIG8["none"],
                    "fig9_none": FIG9["none"]})
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
