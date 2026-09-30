"""Where the Flumps of Schelling's bounded neighborhood stand (pure Python).

The board is a row of three blocks: Red's queue on the left, the area (a
square fenced in the middle), Blue's queue on the right, a gap between each.
Insiders keep a square of the area while inside, taken in a fixed shuffled
order as they enter; outsiders stand at their rank's spot in their queue,
the most tolerant nearest the gate (the area's side), so the queue shows the
tolerance schedule and the gaps show who is inside.
"""

import random

AREA = 15            # the area's side, in squares
QUEUE = 7            # a queue's width
GAP = 1
LEFT = QUEUE + GAP   # the area's first column
RIGHT = LEFT + AREA  # the column after the area
WIDTH = RIGHT + GAP + QUEUE
HEIGHT = AREA

# The area's squares in the order entrants take them: shuffled, so the two
# colors mingle rather than filling rows.
_ORDER = [(x, y) for y in range(AREA) for x in range(LEFT, RIGHT)]
random.Random(7).shuffle(_ORDER)


def in_area(x, y):
    return LEFT <= x < RIGHT and 0 <= y < AREA


def _rows():
    """Rows from the middle out: 7, 6, 8, 5, 9, …"""
    mid = AREA // 2
    out = [mid]
    for k in range(1, AREA):
        for y in (mid - k, mid + k):
            if 0 <= y < AREA:
                out.append(y)
    return out


def queue_spot(red, rank):
    """The square of an outsider of rank `rank` (0 the most tolerant): the
    column nearest the gate first, from the middle row out."""
    col, row = divmod(rank, AREA)
    if col >= QUEUE:
        raise ValueError(f"rank {rank} does not fit a queue of {QUEUE} × {AREA}")
    y = _rows()[row]
    x = LEFT - GAP - 1 - col if red else RIGHT + GAP + col
    return x, y


def layout(frames):
    """For each frame (a list of `(red, rank)` insiders, in order), the square
    of each person inside, keeping squares across frames: `{(red, rank): (x, y)}`."""
    held, out = {}, []
    for insiders in frames:
        now = set(insiders)
        for key in [k for k in held if k not in now]:
            del held[key]
        taken = set(held.values())
        free = (sq for sq in _ORDER if sq not in taken)
        for key in insiders:
            if key not in held:
                held[key] = next(free)
        out.append(dict(held))
    return out
