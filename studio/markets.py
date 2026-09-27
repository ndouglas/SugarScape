"""Sugar and spice: which hill a site belongs to, Flumps walking between the
hills, which way trades go and at what price (see episodes/markets)."""

import math


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


def shuttlers(tracks, width, side_of, first, last):
    """The ids of the tracks alive from tick `first` to `last` that change
    side at least twice in between (as `shuttle_share` counts them)."""
    out = set()
    for id_, t in tracks.items():
        end = t.first + len(t.cells) - 1
        if t.first > first or end < last:
            continue
        path = [y * width + x for x, y in t.cells[first - t.first : last - t.first + 1]]
        if crossings(path, side_of) >= 2:
            out.add(id_)
    return out


def price_series(frames):
    """Each tick's price in spice per sugar — the geometric mean of the pairs'
    prices, each weighted by its exchanges — and the spread, the standard
    deviation of their log prices. None before the first trade; a tick
    without trades keeps the last."""
    prices, spreads = [], []
    price = spread = None
    for f in frames:
        logs = [(math.log(t.price), t.exchanges) for t in f.trades]
        n = sum(w for _, w in logs)
        if n:
            mean = sum(v * w for v, w in logs) / n
            price = math.exp(mean)
            spread = math.sqrt(sum(w * (v - mean) ** 2 for v, w in logs) / n)
        prices.append(price)
        spreads.append(spread)
    return prices, spreads


def smoothed(prices, spreads, window):
    """Rolling averages over the last `window` ticks with a price: the
    geometric mean of the prices and the mean of the spreads. A tick's
    price swings when few pairs trade; the average shows where it settles."""
    out_p, out_s = [], []
    for k in range(len(prices)):
        seen = [(p, s) for p, s in zip(prices[max(k - window + 1, 0) : k + 1], spreads[max(k - window + 1, 0) : k + 1])
                if p is not None]
        if not seen:
            out_p.append(None)
            out_s.append(None)
            continue
        out_p.append(math.exp(sum(math.log(p) for p, _ in seen) / len(seen)))
        out_s.append(sum(s for _, s in seen) / len(seen))
    return out_p, out_s
