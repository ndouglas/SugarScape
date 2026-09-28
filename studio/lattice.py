"""Spatial games on the board (see episodes/spatial): where each square sits,
what its Flump shows at a moment of a beat (its side, its hop, the glow on
the felt beneath it), and the measures the captions need.

A Flump that switches sides between generations k and k + 1 hops during the
last `hop` share of that tick and takes its new side at the top of the hop;
the felt under it then glows in Nowak & May's change colors (green for a
cheat turned helper, yellow for a helper turned cheat), fading over a tick.
"""

import math
from dataclasses import dataclass

HOP_HEIGHT = 0.6
GLOW_TICKS = 1.0


def position(i, w, h):
    """Square `i`'s (x, y) on the board: one unit per square, centered."""
    x, y = i % w, i // w
    return (x - w / 2 + 0.5, h / 2 - y - 0.5)


@dataclass(frozen=True)
class Moment:
    """A frame's place in the shot: generation `k`, `frac` of the way to the
    next; the share of a tick a hop takes, and how high it goes."""

    k: int
    frac: float
    hop: float
    lift: float


def moment(timing, frame, ticks):
    t = timing.tick_at(frame)
    k = min(int(math.floor(t)), ticks)
    frac = 0.0 if k >= ticks else t - k
    # No hop when a tick lasts a frame or less (it would only flicker); a
    # full one from six frames a tick.
    lift = HOP_HEIGHT * min(1.0, max(0.0, (timing.tick_frames - 1) / 5))
    return Moment(k, frac, timing.hop, lift)


@dataclass(frozen=True)
class Square:
    helper: bool
    height: float
    # "C" (turned helper) or "D" (turned cheat), and how bright; None: no glow.
    glow: str | None
    strength: float


def squares(d, m):
    """Every square's `Square` at moment `m`, row-major."""
    now = d.frames[m.k].strategies
    nxt = d.frames[m.k + 1].strategies if m.k < d.ticks else now
    before = d.frames[m.k - 1].strategies if m.k > 0 else now
    start, top = 1 - m.hop, 1 - m.hop / 2
    rising = m.frac >= start
    past_top = m.frac >= top
    height = m.lift * math.sin(math.pi * (m.frac - start) / m.hop) if rising and m.hop > 0 else 0.0
    # A switch that topped out during the last tick, and how long ago (ticks).
    old_age = m.frac + m.hop / 2
    out = []
    for a, b, c in zip(now, nxt, before):
        if b != a and rising:
            shown = b if past_top else a
            glow, age = (b, m.frac - top) if past_top else ((a, old_age) if c != a else (None, 0.0))
            out.append(Square(shown == "C", height, glow, max(0.0, 1 - age / GLOW_TICKS) if glow else 0.0))
        elif c != a:
            out.append(Square(a == "C", 0.0, a, max(0.0, 1 - old_age / GLOW_TICKS)))
        else:
            out.append(Square(a == "C", 0.0, None, 0.0))
    return out


def _grid(strategies, w):
    return [strategies[y * w : (y + 1) * w] for y in range(len(strategies) // w)]


def symmetric(strategies, w, h):
    """Whether a board is unchanged by quarter turns and mirrors (the
    kaleidoscope's four-fold symmetry)."""
    if w != h:
        return False
    g = _grid(strategies, w)
    turned = ["".join(g[w - 1 - x][y] for x in range(w)) for y in range(w)]
    mirrored = [row[::-1] for row in g]
    return turned == g and mirrored == g


def reaches_edge(strategies, w, h):
    """Whether a cheat stands on the board's edge."""
    g = _grid(strategies, w)
    return "D" in g[0] or "D" in g[-1] or any(row[0] == "D" or row[-1] == "D" for row in g)


NEIGHBORS = [(dx, dy) for dy in (-1, 0, 1) for dx in (-1, 0, 1) if (dx, dy) != (0, 0)]


def boundary(frame, w, h):
    """Along the frontier (each adjacent helper–cheat pair, the eight
    neighbors, fixed edges): the number of pairs and the share in which the
    cheat out-earns the helper. Needs a frame with scores."""
    s, scores = frame.strategies, frame.scores
    pairs = wins = 0
    for y in range(h):
        for x in range(w):
            i = y * w + x
            if s[i] != "D":
                continue
            for dx, dy in NEIGHBORS:
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and s[ny * w + nx] == "C":
                    pairs += 1
                    wins += scores[i] > scores[ny * w + nx]
    return pairs, (wins / pairs if pairs else float("nan"))
