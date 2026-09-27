"""Combat: who kills, when the killing starts, and where (see episodes/war)."""

import collections
import statistics


def kills_by_tick(d):
    """The number of kills in each frame."""
    return [len(f.kills) for f in d.frames]


def burst_start(d, share=0.1):
    """The tick by which `share` of the run's kills had happened (None
    without kills): when the war starts in earnest."""
    counts = kills_by_tick(d)
    total = sum(counts)
    if not total:
        return None
    running = 0
    for tick, n in enumerate(counts):
        running += n
        if running >= share * total:
            return tick
    return len(counts) - 1


def killers(d):
    """Each attacker's kills over the run, most first."""
    return collections.Counter(k.attacker for f in d.frames for k in f.kills).most_common()


def top_killer_share(d):
    """The share of the run's kills made by its busiest killer (None without kills)."""
    ranked = killers(d)
    total = sum(n for _, n in ranked)
    return ranked[0][1] / total if total else None


def nth_kill_tick(d, id_, n):
    """The tick of `id_`'s `n`th kill (None if it never makes that many)."""
    made = 0
    for tick, f in enumerate(d.frames):
        made += sum(1 for k in f.kills if k.attacker == id_)
        if made >= n:
            return tick
    return None


def unstoppable_from(d, id_, tick):
    """Whether, from `tick` until it dies or the run ends, no Flump of
    another tribe is ever richer than `id_` — so, under rule C, none may
    attack it."""
    for f in d.frames[tick:]:
        me = f.agents.get(id_)
        if me is None:
            return True
        group = f.groups[id_]
        if any(f.groups[i] != group and a.sugar > me.sugar for i, a in f.agents.items()):
            return False
    return True


def majority_share(frame):
    """The largest tribe's share of the Flumps alive (0 with none)."""
    counts = collections.Counter(frame.groups.values())
    return max(counts.values()) / len(frame.groups) if frame.groups else 0.0


def quarter_shares(d, width, height):
    """Where the run's victims died, as each quarter's share of the kills
    (NW, NE, SW, SE), from the victims' cells in the frame before."""
    counts = collections.Counter()
    for k in range(1, len(d.frames)):
        before = d.frames[k - 1].agents
        for kill in d.frames[k].kills:
            v = before.get(kill.victim)
            if v is not None:
                counts[("N" if v.y < height // 2 else "S") + ("W" if v.x < width // 2 else "E")] += 1
    total = sum(counts.values())
    return {q: counts[q] / total if total else 0.0 for q in ("NW", "NE", "SW", "SE")}


def victim_ages(d):
    """The victims' ages at death, from the frame before."""
    ages = []
    for k in range(1, len(d.frames)):
        before = d.frames[k - 1].agents
        ages += [before[kill.victim].age for kill in d.frames[k].kills if kill.victim in before]
    return ages


def median_or_none(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None
