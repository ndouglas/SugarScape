"""The Ethnocentrism episode's captioned claims (Hammond & Axelrod 2006),
measured over 20 seeds (see studio/measure.py and
docs/superpowers/specs/2026-09-28-ethnocentrism-spike.md). Every land is
the paper's 50 × 50 torus; shares are the mean over the last 100 periods,
as the paper's are.

- ha-standard to period 2,000.
- The scattering: ha-standard to period 1,000, then each child placed on a
  random empty square anywhere (Jansson 2013's variant) to 2,000.
- ha-cost-2 and ha-cost-2-blind: color-seeing and color-blind Flumps,
  helping at double cost (HA06: 56 % and 14 %).
- Color-blind Flumps at the standard cost (ha-standard, discrimination
  none), against the standard case, seed by seed.
"""

import statistics

import measure as m
from measure import median

TICKS = 2000
SWITCH = 1000
LATE = (1901, 2000)
SCATTER = {"schedule": [{"tick": SWITCH, "set": {"offspring": "anywhere"}}]}
PAPER_BLIND, PAPER_SEEING = 0.14, 0.56  # HA06 p. 7, at double cost: "56 percent" and "14 percent"


def _series(preset, seed, tmp, changes=None):
    rows, _ = m.run(preset, seed, TICKS, tmp, changes)
    out = []
    for r in rows:
        out.append({k: float(v) if v not in ("", "NaN") else float("nan") for k, v in r.items()})
    return out


def _late(rows, key):
    return statistics.fmean(r[key] for r in rows if LATE[0] <= r["tick"] <= LATE[1])


def _count(rows, test):
    return sum(bool(test(r)) for r in rows.values())


def verdicts(rows, config):
    """Each captioned claim, whether the seeds support it, and why. `config`:
    ha-standard's."""
    n = len(rows)
    largest = _count(rows, lambda r: r["own"] > max(r["everyone"], r["none"], r["others"]))
    kin = [r["kin_help"] for r in rows.values()]
    dark = [r["scattered_none"] for r in rows.values()]
    blind = [r["blind"] for r in rows.values()]
    seeing = [r["seeing2"] for r in rows.values()]
    more = _count(rows, lambda r: r["blind1"] > r["cooperation"])
    return [
        ("each Flump wears one of four colors, and passes it on to its children",
         config["colors"] == 4 and config["mutation"] <= 0.01,
         f"{config['colors']} colors; a child's color differs from its parent's with probability {config['mutation']}"),
        ("each carries two rules, help my own color? help the others?, making four kinds",
         config["discrimination"] == "same_other" and sorted(config["allowed"]) == ["E", "H", "S", "T"],
         f"discrimination {config['discrimination']}, kinds {config['allowed']}"),
        ("helping costs a little of your chance to have a child, and gives the neighbor three times as much",
         abs(config["benefit"] - 3 * config["cost"]) < 1e-12,
         f"cost {config['cost']:.0%} of the chance, benefit {config['benefit']:.0%}, from a base of "
         f"{config['base_ptr']:.0%}"),
        ("the land starts empty; newcomers arrive one at a time; children are born next door",
         config["start"] == "empty" and config["immigration"] == 1 and config["offspring"] == "adjacent",
         f"start {config['start']}, {config['immigration']:g} newcomer a period, offspring {config['offspring']}"),
        ("favoritism wins in all 20 worlds: about 3 in 4 Flumps help only their own color",
         0.7 <= median(rows, "own") <= 0.8 and largest == n,
         f"help-own-only over periods {LATE[0]}–{LATE[1]}: {median(rows, 'own'):.1%} (median; "
         f"{min(r['own'] for r in rows.values()):.1%}–{max(r['own'] for r in rows.values()):.1%}), the largest kind "
         f"in {largest} of {n}; cooperation {median(rows, 'cooperation'):.1%}"),
        ("why? their neighbors share ancestors: over 8 in 10 of all helps go to relatives", min(kin) > 0.8,
         f"helps to Flumps with the same founding newcomer: {statistics.median(kin):.1%} (median; "
         f"{min(kin):.1%}–{max(kin):.1%}); neighboring pairs that are relatives {median(rows, 'relatives'):.1%}"),
        ("put each child anywhere, and favoritism collapses: nine in ten Flumps help no one",
         min(dark) >= 0.8 and 0.85 <= statistics.median(dark) <= 0.95,
         f"scattered from period {SWITCH}: help-no-one over {LATE[0]}–{LATE[1]} {statistics.median(dark):.1%} "
         f"(median; {min(dark):.1%}–{max(dark):.1%}); cooperation {median(rows, 'scattered_cooperation'):.1%}; "
         f"helps to relatives {median(rows, 'scattered_kin_help'):.1%}"),
        # HA06 give both numbers; neither reproduces, and cooperation at
        # double cost is steep in the cost (the blind world: 42 % at 2 %,
        # 20 % at 2.5 %, 13 % at 3 %), so the caption shows both pairs and
        # blames no one.
        ("the paper: at double cost, color-seeing Flumps help 56% of the time, color-blind ones 14%; here, 67% and 42%",
         round(statistics.median(seeing), 2) == 0.67 and round(statistics.median(blind), 2) == 0.42,
         f"cooperation over {LATE[0]}–{LATE[1]} at cost 2 %: seeing {statistics.median(seeing):.1%} (median; "
         f"{min(seeing):.1%}–{max(seeing):.1%}), blind {statistics.median(blind):.1%} ({min(blind):.1%}–{max(blind):.1%})"),
        ("at the paper's usual cost, color-blind Flumps help even more: 82% to 76%",
         more >= 15 and f"{median(rows, 'blind1'):.0%}" == "82%" and f"{median(rows, 'cooperation'):.0%}" == "76%",
         f"cost 1 %: color-blind {median(rows, 'blind1'):.1%} (median; {min(r['blind1'] for r in rows.values()):.1%}–"
         f"{max(r['blind1'] for r in rows.values()):.1%}) against seeing {median(rows, 'cooperation'):.1%}; blind above "
         f"seeing in {more} of {n} seeds"),
    ]


def seed_row(tmp, seed):
    std = _series("ha-standard", seed, tmp)
    scattered = _series("ha-standard", seed, tmp, SCATTER)
    blind = _series("ha-cost-2-blind", seed, tmp)
    seeing2 = _series("ha-cost-2", seed, tmp)
    blind1 = _series("ha-standard", seed, tmp, {"discrimination": "none"})
    return {
        "own": _late(std, "ethnocentric"),
        "everyone": _late(std, "humanitarian"),
        "none": _late(std, "selfish"),
        "others": _late(std, "traitorous"),
        "cooperation": _late(std, "cooperation"),
        "kin_help": _late(std, "kin_help"),
        "relatives": _late(std, "relatives"),
        "population": _late(std, "population"),
        "scattered_none": _late(scattered, "selfish"),
        "scattered_cooperation": _late(scattered, "cooperation"),
        "scattered_kin_help": _late(scattered, "kin_help"),
        "blind": _late(blind, "cooperation"),
        "seeing2": _late(seeing2, "cooperation"),
        "blind1": _late(blind1, "cooperation"),
    }


def measure(tmp):
    rows = {seed: seed_row(tmp, seed) for seed in m.SEEDS}
    config = m.preset_config("ha-standard", tmp)
    keys = ["own", "everyone", "none", "others", "cooperation", "kin_help", "relatives", "population",
            "scattered_none", "scattered_cooperation", "scattered_kin_help", "blind", "seeing2", "blind1"]
    lines = [f"## The paper's 50 × 50 land, means over periods {LATE[0]}–{LATE[1]}", ""]
    lines += m.table(rows, keys)
    lines += ["", f"Typical seeds: land {m.typical_seed(rows, ['own', 'kin_help'])}, "
              f"scattered {m.typical_seed(rows, ['scattered_none'])}, blind {m.typical_seed(rows, ['blind'])}."]
    medians = {k: median(rows, k) for k in keys}
    medians["paper_blind"], medians["paper_seeing"] = PAPER_BLIND, PAPER_SEEING
    return lines, verdicts(rows, config), {"medians": medians, "seeds": len(rows)}
