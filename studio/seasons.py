"""Seasons (rule S, Animation II-7), as the engine applies them: the north —
rows y < height // 2 — has summer while tick mod 2·period < period, and the
south has winter; then they swap. Used by the Seasons episode's claims and
visuals; pure, so it is tested outside Blender."""


def north_summer(tick, period):
    return tick % (2 * period) < period


def in_north(y, height):
    return y < height // 2


def _winter(north, tick, period):
    return 1.0 if north != north_summer(tick, period) else 0.0


def frost(north, tick, period, ease=2.0):
    """How wintry a hemisphere looks at a fractional tick: 0 in summer, 1 in
    winter, easing across each swap over `ease` ticks."""
    swap = round(tick / period) * period
    if swap > 0 and abs(tick - swap) < ease / 2:
        before, after = _winter(north, swap - 1, period), _winter(north, swap, period)
        return before + (after - before) * ((tick - swap) / ease + 0.5)
    return _winter(north, int(tick), period)


def _alive_through(track, start, end):
    last = track.first + len(track.cells) - 1
    return track.first <= start and last >= end


def switches(track, start, end, height):
    """How often a Flump crossed between hemispheres over ticks start..end,
    or None if it was not alive for all of them."""
    if not _alive_through(track, start, end):
        return None
    sides = [in_north(track.cells[t - track.first][1], height) for t in range(start, end + 1)]
    return sum(a != b for a, b in zip(sides, sides[1:]))


def migrants(tracks, start, end, height):
    """The ids of Flumps alive through ticks start..end that crossed between
    hemispheres at least twice in them (the survey's migrators)."""
    return {i for i, t in tracks.items() if (switches(t, start, end, height) or 0) >= 2}


def summer_share(track, start, end, height, period):
    """The share of ticks start..end a Flump spent in the summer hemisphere
    (None if it was not alive for all of them)."""
    if not _alive_through(track, start, end):
        return None
    ticks = range(start, end + 1)
    hits = sum(in_north(track.cells[t - track.first][1], height) == north_summer(t, period) for t in ticks)
    return hits / len(ticks)
