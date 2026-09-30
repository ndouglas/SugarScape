"""Overlays for the variations on Schelling: a card naming whose rules are
shown, a statistic as a number, and the felt arc that closes Pancs &
Vriend's ring."""

import math

from blender import materials

from .parts import CREAM, card, text
from .ring import _curve, _stat


def paper(beat, d, ctx):
    """Top left: whose rules these are. params: `paper` (one or two lines)."""
    anchor = ctx.screen.anchor("paper", -0.66, 0.8)
    lines = beat.params["paper"].split("\n")
    card("paper-card", anchor, (0, 0, -0.01), (0.66, 0.1 + 0.06 * len(lines), 0.002))
    ink = materials.fading("paper-ink", CREAM, 1.6)
    top = 0.03 * (len(lines) - 1)
    for i, line in enumerate(lines):
        text(f"paper-{i}", line, 0.05, ink, anchor, location=(-0.3, top - 0.06 * i - 0.015, 0), align="LEFT")
    return lambda frame: None


def figure(beat, d, ctx):
    """Top right: a statistic of the step shown as a number. params: `stat`
    (a series name), `label`, and `format` ("int", "pct" or "2f")."""
    name, label, fmt = beat.params["stat"], beat.params["label"], beat.params.get("format", "int")
    anchor = ctx.screen.anchor("figure", 0.62, 0.8)
    card("figure-card", anchor, (0, 0, -0.01), (0.72, 0.16, 0.002))
    words = text("figure-text", "", 0.055, materials.fading("figure-ink", CREAM, 1.6), anchor,
                 location=(-0.3, -0.02, 0), align="LEFT")

    def update(frame):
        v = _stat(d, name, ctx.timing.tick_at(frame))
        shown = {"int": f"{v:,.0f}", "pct": f"{v:.0%}", "2f": f"{v:.2f}"}[fmt]
        words.data.body = f"{shown} {label}"

    return update


ARC = 48


def ringjoin(beat, d, ctx):
    """A felt arc behind the row from its last square back to its first: on
    Pancs & Vriend's ring the two ends are neighbors."""
    half = d.width / 2
    arc = _curve("ringjoin", materials.knit("cream"), ARC)
    arc.data.bevel_depth = 0.12
    pts = arc.data.splines[0].points
    for j, p in enumerate(pts):
        t = math.pi * j / (ARC - 1)
        p.co = (half * math.cos(t), 0.6 + half * 0.45 * math.sin(t), 0.08, 1)
    return lambda frame: None
