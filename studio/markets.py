"""Sugar and spice: which hill a site belongs to, Flumps walking between the
hills, and which way trades go (see episodes/markets)."""


def sides(sugar_capacity, spice_capacity):
    """Each site's side: "sugar" or "spice", whichever it grows more of, or
    None where they are equal (the middle, and the bare ground)."""
    return [
        "sugar" if a > b else "spice" if b > a else None
        for a, b in zip(sugar_capacity, spice_capacity)
    ]


def crossings(cells, side_of):
    """How many times a path (site indexes) changes side, skipping sites on
    neither side."""
    count, last = 0, None
    for c in cells:
        s = side_of[c]
        if s is None:
            continue
        if last is not None and s != last:
            count += 1
        last = s
    return count


def shuttle_share(frames, width, side_of, first, last):
    """The share of Flumps alive from tick `first` to `last` that change
    side at least twice in between (there and back)."""
    alive = set(frames[first].agents)
    for f in frames[first : last + 1]:
        alive &= set(f.agents)
    if not alive:
        return float("nan")
    shuttlers = 0
    for id_ in alive:
        path = [f.agents[id_].y * width + f.agents[id_].x for f in frames[first : last + 1]]
        shuttlers += crossings(path, side_of) >= 2
    return shuttlers / len(alive)


def _sugar_rich(frame, id_):
    """How much sugar a Flump holds against its spice, each measured in ticks
    of its own need: above 1, it is richer in sugar."""
    a = frame.agents[id_]
    spice, spice_need = frame.spice_agents[id_]
    return (a.sugar / a.metabolism) / max(spice / spice_need, 1e-9)


def expected_direction(before, after):
    """Of `after`'s trades between Flumps alive in `before`, how many had the
    sugar giver relatively richer in sugar than its partner (by holdings over
    needs in `before`), and how many there were: (right, total)."""
    right = total = 0
    for t in after.trades:
        if not {t.sugar_giver, t.spice_giver} <= set(before.agents) or not before.spice_agents:
            continue
        total += 1
        right += _sugar_rich(before, t.sugar_giver) > _sugar_rich(before, t.spice_giver)
    return right, total
