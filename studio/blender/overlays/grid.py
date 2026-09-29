"""Overlays for the social-structure grid: a few Flumps' partners as yarn
lines, and the population's mean payoff."""

import math

import grid
from blender import materials

from .parts import CREAM, box, card, text
from .ring import ARC_POINTS, _curve


def ties(beat, d, ctx):
    """Yarn lines from each Flump in params `follow` (agent indices) to its
    partners this period; they jump when the partners change."""
    follow = beat.params["follow"]
    yarn = materials.knit("butter")
    most = 8 * len(follow)
    lines = [_curve(f"tie{i}", yarn, ARC_POINTS) for i in range(most)]
    for line in lines:
        line.data.bevel_depth = 0.06

    def update(frame):
        f = d.frame(ctx.timing.tick_at(frame))
        pairs = grid.ties(d, f, follow)
        for i, line in enumerate(lines):
            pts = line.data.splines[0].points
            if i >= len(pairs):
                for p in pts:
                    p.co = (0, 0, -20, 1)
                continue
            (x0, y0), (x1, y1) = pairs[i]
            height = 0.6 + 0.12 * math.hypot(x1 - x0, y1 - y0)
            for j, p in enumerate(pts):
                u = j / (ARC_POINTS - 1)
                p.co = (x0 + (x1 - x0) * u, y0 + (y1 - y0) * u, 0.8 + height * 4 * u * (1 - u), 1)

    return update


def payoff(beat, d, ctx):
    """Top right: the mean payoff per move this period, out of 3 (mutual
    help), as a bar and a number."""
    anchor = ctx.screen.anchor("payoff", 0.62, 0.8)
    card("payoff-card", anchor, (0, 0, -0.01), (0.72, 0.2, 0.002))
    box("payoff-back", materials.knit("cream"), anchor, location=(0, -0.035, 0), scale=(0.6, 0.05, 0.004))
    fill = box("payoff-fill", materials.knit("teal"), anchor, location=(0, -0.035, 0.003), scale=(0.6, 0.05, 0.004))
    words = text("payoff-text", "", 0.045, materials.fading("payoff-ink", CREAM, 1.6), anchor,
                 location=(-0.3, 0.04, 0), align="LEFT")

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        v = d.stats["mean_payoff"][k]
        v = 0.0 if v is None or (isinstance(v, float) and math.isnan(v)) else v
        words.data.body = f"average payoff: {v:.2f} of 3"
        width = max(v / 3, 0.002) * 0.6
        fill.scale.x = width
        fill.location.x = -0.3 + width / 2

    return update
