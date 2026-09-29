"""Social structure on the board (see episodes/friends): 256 Flumps on a
16 × 16 grid, colored by p, how readily each helps after being helped (red
wary, green friendly). On the torus each Flump stands on its own square, so
its four partners are next door; for the other structures the grid is just
the agents in order, and partners are anywhere.
"""

import colorsys
import math

SIDE = 16
SPACING = 1.15
SHADES = 10


def position(agent, d):
    """An agent's (x, y) on the board."""
    square = d.site[agent] if d.site else agent
    x, y = square % SIDE, square // SIDE
    return ((x - (SIDE - 1) / 2) * SPACING, ((SIDE - 1) / 2 - y) * SPACING)


def shade(p):
    """The yarn bucket (0 … SHADES − 1) of a friendliness p in [0, 1]."""
    return min(int(p * SHADES), SHADES - 1)


def shade_color(k):
    """Bucket k's yarn, as linear RGB: red (0) through yellow to green."""
    r, g, b = colorsys.hsv_to_rgb(0.33 * (k + 0.5) / SHADES, 0.6, 0.9)
    return (r ** 2.2, g ** 2.2, b ** 2.2)


def ties(d, frame, agents):
    """For each followed agent, the (from, to) board points of its lines to
    this period's partners."""
    out = []
    for a in agents:
        if a < len(frame.partners):
            for b in frame.partners[a]:
                out.append((position(a, d), position(b, d)))
    return out


def reach(d, frame, agents):
    """The mean board distance from each followed agent to its partners."""
    lines = ties(d, frame, agents)
    return sum(math.dist(a, b) for a, b in lines) / len(lines) if lines else 0.0
