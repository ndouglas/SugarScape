"""Following the Crowd, episode 4, "One culture or many": Axelrod's culture
model (1997), with Castellano et al.'s traits transition, Klemm et al.'s drift
and the docking paper's wandering agents, each captioned claim measured over
20 seeds (1,000 where it is about his samples; see studio/measure.py and
docs/superpowers/specs/2026-09-30-culture-spike.md). The rules follow the
survey's `culture.*` (survey/src/claims/culture.rs).

Runs go until the map is stable (at most 200,000 steps), except drift (20,000
steps: drift never stops) and the docked Sugarscape (its own stop, at most
20,000 ticks).
"""

import concurrent.futures
import json

import measure as m
from measure import median

CAP = 200_000
SAMPLE = range(1, 1001)


def _end(preset, seed, tmp, changes=None, ticks=CAP, series="regions"):
    rows, _ = m.run(preset, seed, ticks, tmp, changes)
    return float(rows[-1][series]), int(rows[-1]["tick"])


def seed_row(tmp, seed):
    out = {}
    out["sample"], out["sample_stable"] = _end("ac-sample-run", seed, tmp)
    out["q15"], _ = _end("ac-sample-run", seed, tmp, {"traits": 15})
    out["f15"], _ = _end("ac-sample-run", seed, tmp, {"features": 15})
    out["four"] = out["sample"]
    out["twelve"], _ = _end("ac-sample-run", seed, tmp, {"neighborhood": "diamond"})
    out["small_map"], _ = _end("ac-many-regions", seed, tmp)
    out["big_map"], _ = _end("ac-sample-run", seed, tmp, {"width": 50, "height": 50, "traits": 15})
    out["q25_20"], _ = _end("ac-sample-run", seed, tmp, {"width": 20, "height": 20, "traits": 25})
    out["q25_30"], _ = _end("ac-sample-run", seed, tmp, {"width": 30, "height": 30, "traits": 25})
    out["no_drift"], _ = _end("ac-many-regions", seed, tmp, {"stop_when_stable": False}, ticks=20_000)
    out["drift"], _ = _end("ac-drift", seed, tmp, {"stop_when_stable": False}, ticks=20_000)
    out["wander"], out["wander_stop"] = _end("dock-mobility-15", seed, tmp, ticks=20_000, series="distinct_cultures")
    return out


def sample_row(tmp, seed):
    return _end("ac-sample-run", seed, tmp)[0]


def verdicts(rows, sample, config):
    n = len(rows)
    mean = lambda k: sum(r[k] for r in rows.values()) / n  # noqa: E731
    md = {k: median(rows, k) for k in next(iter(rows.values()))}
    s_mean = sum(sample) / len(sample)
    s_median = sorted(sample)[len(sample) // 2]
    return [
        ("Robert Axelrod's villages: a hundred on a map; each has five features, each one of ten traits",
         config["sample"] == (10, 10, 5, 10), "10 × 10, five features of ten traits"),
        ("neighbors talk as often as they're alike; when they talk, one copies a feature from the other",
         config["rule"] == ("von_neumann", "random", "active", "random"),
         "four neighbors, a random site each event, the active site copying a random differing feature"),
        ("neighbors grow alike, until each pair is either the same or shares nothing; then nothing changes",
         all(r["sample_stable"] < CAP for r in rows.values()),
         f"every one of {n} runs stops (median at step {md['sample_stable']:.0f})"),
        ("a few cultures survive, about four here; Axelrod's runs, about three",
         3.5 <= s_mean <= 5 and s_median == 4,
         f"mean {s_mean:.2f}, median {s_median:.0f} over {len(sample)} seeds (his: 3.2 over 10 runs, median 3 over 100; "
         "within sampling, survey culture.sample.*)"),
        ("more traits to differ on, and more survive", mean("q15") > 2 * mean("sample"),
         f"mean {mean('q15'):.1f} regions at 15 traits against {mean('sample'):.1f} at 10"),
        ("more features to share, and one culture wins", all(r["f15"] == 1 for r in rows.values()),
         f"{sum(r['f15'] == 1 for r in rows.values())} of {n} end as one region at 15 features"),
        ("talk to more neighbors, and fewer survive", mean("twelve") < mean("four"),
         f"mean {mean('twelve'):.2f} regions with 12 neighbors against {mean('four'):.2f} with 4"),
        ("the surprise: a bigger map ends with fewer cultures", mean("big_map") < mean("small_map") / 2,
         f"15 traits: mean {mean('small_map'):.1f} regions at 12 × 12, {mean('big_map'):.1f} at 50 × 50"),
        ("a map of 50 by 50: a handful", 3 <= mean("big_map") <= 9,
         f"15 traits: mean {mean('big_map'):.1f} regions at 50 × 50 (Axelrod: \"about 6\")"),
        ("unless there are enough traits; then a big map shatters",
         md["q25_30"] > md["q25_20"] and md["q25_30"] > 100,
         f"25 traits: median {md['q25_20']:.0f} regions at 20 × 20, {md['q25_30']:.0f} at 30 × 30"),
        ("let a trait change at random, now and then, and the borders melt", md["drift"] < md["no_drift"] / 2,
         f"12 × 12, 15 traits, step 20,000: median {md['no_drift']:.0f} regions without drift, {md['drift']:.0f} "
         "with one change in 10,000 events"),
        ("let them wander a sugar mountain, and one culture takes everyone",
         md["wander"] == 1 and mean("wander") <= 1.5 and all(r["wander_stop"] < 20_000 for r in rows.values()),
         f"mean {mean('wander'):.2f} cultures, median {md['wander']:.0f}, every run stopping (theirs 1.1 ± 0.3)"),
        ("everything Axelrod reported, the Flumps reproduce", config["survey"],
         "survey culture.*: every claim holds"),
        ("neighbors grow alike, and the map stays divided",
         sum(x > 1 for x in sample) >= 0.8 * len(sample),
         f"{sum(x > 1 for x in sample)} of {len(sample)} sample runs end with more than one region"),
    ]


def _config(tmp):
    c = m.preset_config("ac-sample-run", tmp)
    return {"sample": (c["width"], c["height"], c["features"], c["traits"]),
            "rule": (c["neighborhood"], c["activation"], c["changes"], c["pick"]),
            "survey": _survey_holds()}


def _survey_holds():
    """Whether every claim in the survey's last full culture run holds
    (survey/out/results-culture.json, from `cargo run --release -- --only culture`)."""
    path = m.REPO / "survey" / "out" / "results-culture.json"
    results = json.loads(path.read_text())
    return bool(results) and all(r["verdict"] == "Holds" for r in results)


def measure(tmp):
    config = _config(tmp)
    for p in ("ac-sample-run", "ac-many-regions", "ac-drift", "dock-mobility-15"):
        m.preset_config(p, tmp)
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        rows = dict(zip(m.SEEDS, pool.map(lambda s: seed_row(tmp, s), m.SEEDS)))
        sample = list(pool.map(lambda s: sample_row(tmp, s), SAMPLE))
    keys = list(next(iter(rows.values())))
    lines = ["## Axelrod's culture", ""] + m.table(rows, keys)
    lines += ["", f"Sample setup over {len(sample)} seeds: mean {sum(sample) / len(sample):.2f}, median "
              f"{sorted(sample)[len(sample) // 2]:.0f}.", "",
              "Typical seeds: sample " + str(m.typical_seed(rows, ["sample"])) + ", q15 "
              + str(m.typical_seed(rows, ["q15"])) + ", f15 " + str(m.typical_seed(rows, ["sample_stable"]))
              + ", twelve " + str(m.typical_seed(rows, ["twelve"])) + ", small map "
              + str(m.typical_seed(rows, ["small_map"])) + ", big map " + str(m.typical_seed(rows, ["big_map"]))
              + ", q25 30 " + str(m.typical_seed(rows, ["q25_30"])) + ", drift " + str(m.typical_seed(rows, ["drift"]))
              + ", wander " + str(m.typical_seed(rows, ["wander", "wander_stop"])) + "."]
    return lines, verdicts(rows, sample, config), {"medians": {k: median(rows, k) for k in keys}, "seeds": len(rows)}
