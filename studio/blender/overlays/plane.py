"""Overlays for the norms plane: each generation's cheats, punishments and
metapunishments, the true generation, and the population's mean traits."""

import math

import plane
from blender import materials

from .parts import CREAM, card, text
from .ring import ARC_POINTS, _curve

# At most this many arcs of each kind a generation (early generations punish
# hundreds of times).
MOST = 150


def _stat(d, name, tick):
    v = d.stats[name][min(max(int(round(tick)), 0), d.ticks)]
    return 0.0 if v is None or (isinstance(v, float) and math.isnan(v)) else v


def events(beat, d, ctx):
    """The generation's events, before its offspring take their squares:
    each cheat as a gold arc hopping up from the cheat, each punishment a red
    arc from punisher to cheat, each metapunishment a purple arc from
    metapunisher to the Flump who looked away; they appear in turn over the
    tick and stay until its hops."""
    gold, red, purple = materials.knit("butter"), materials.knit("red"), materials.knit("lilac")
    pools = {kind: [_curve(f"{kind}{i}", m, ARC_POINTS) for i in range(MOST)]
             for kind, m in (("cheat", gold), ("punish", red), ("meta", purple))}

    def draw(arc, a, b, on, height):
        pts = arc.data.splines[0].points
        if not on:
            for p in pts:
                p.co = (0, 0, -20, 1)
            return
        (x0, y0), (x1, y1) = a, b
        for j, p in enumerate(pts):
            u = j / (ARC_POINTS - 1)
            p.co = (x0 + (x1 - x0) * u, y0 + (y1 - y0) * u, 0.6 + height * 4 * u * (1 - u), 1)

    def update(frame):
        t = ctx.timing.tick_at(frame)
        k = min(int(t), d.ticks)
        frac = t - k if k < d.ticks else 1.0
        f = d.frames[k]
        spots = plane.layout(f)
        window = max(1 - ctx.timing.hop, 0.1)
        share = min(frac / window, 1.0) if frac < 1 - ctx.timing.hop or k >= d.ticks else 0.0
        lists = {
            "cheat": [(spots[i], (spots[i][0], spots[i][1] + 0.01)) for i in f.cheats[:MOST]],
            "punish": [(spots[j], spots[i]) for j, i in f.punishments[:MOST]],
            "meta": [(spots[kk], spots[j]) for kk, j in f.metapunishments[:MOST]],
        }
        for kind, pool in pools.items():
            items = lists[kind]
            shown = int(len(items) * share)
            for i, arc in enumerate(pool):
                on = i < shown
                a, b = items[i] if on else ((0, 0), (0, 0))
                draw(arc, a, b, on, 1.6 if kind == "cheat" else 0.8 + 0.08 * math.dist(a, b))

    return update


def generation(beat, d, ctx):
    """Top left: the true generation of the frame shown."""
    anchor = ctx.screen.anchor("generation", -0.66, 0.84)
    card("generation-card", anchor, (0, 0, -0.01), (0.56, 0.14, 0.002))
    line = text("generation-text", "", 0.05, materials.fading("generation-ink", CREAM, 1.6), anchor,
                location=(-0.24, -0.015, 0), align="LEFT")

    def update(frame):
        line.data.body = f"generation {d.frame(ctx.timing.tick_at(frame)).tick:,}"

    return update


def mean_traits(beat, d, ctx):
    """Top right: the population's mean boldness and vengefulness."""
    anchor = ctx.screen.anchor("mean-traits", 0.64, 0.8)
    card("mean-traits-card", anchor, (0, 0, -0.01), (0.66, 0.2, 0.002))
    ink = materials.fading("mean-traits-ink", CREAM, 1.6)
    bold = text("mean-bold", "", 0.046, ink, anchor, location=(-0.3, 0.03, 0), align="LEFT")
    veng = text("mean-veng", "", 0.046, ink, anchor, location=(-0.3, -0.05, 0), align="LEFT")

    def update(frame):
        t = ctx.timing.tick_at(frame)
        bold.data.body = f"average boldness: {_stat(d, 'mean_boldness', t) * 7:.1f} of 7"
        veng.data.body = f"average vengefulness: {_stat(d, 'mean_vengefulness', t) * 7:.1f} of 7"

    return update
