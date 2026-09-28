"""Overlays for the tags ring: each generation's gifts as arcs, a gauge of
any statistic, and a Flump's tolerance as a sliver of the ring."""

import math

import bpy

import ring
from blender import materials

from .parts import CREAM, box, card, text


def _stat(d, name, tick):
    v = d.stats[name][min(max(int(round(tick)), 0), d.ticks)]
    return 0.0 if v is None or (isinstance(v, float) and math.isnan(v)) else v


def rate(beat, d, ctx):
    """Top right: a statistic of the generation shown as a bar and a share.
    params: `stat` (a series name) and `label`."""
    name, label = beat.params["stat"], beat.params["label"]
    anchor = ctx.screen.anchor("rate", 0.62, 0.8)
    card("rate-card", anchor, (0, 0, -0.01), (0.72, 0.2, 0.002))
    box("rate-back", materials.knit("cream"), anchor, location=(0, -0.035, 0), scale=(0.6, 0.05, 0.004))
    fill = box("rate-fill", materials.knit("teal"), anchor, location=(0, -0.035, 0.003), scale=(0.6, 0.05, 0.004))
    words = text("rate-text", "", 0.045, materials.fading("rate-ink", CREAM, 1.6), anchor,
                 location=(-0.3, 0.04, 0), align="LEFT")

    def update(frame):
        v = _stat(d, name, ctx.timing.tick_at(frame))
        words.data.body = f"{v:.0%} {label}"
        width = max(v, 0.002) * 0.6
        fill.scale.x = width
        fill.location.x = -0.3 + width / 2

    return update


def _curve(name, material, points):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = 0.04
    cu.bevel_resolution = 2
    spline = cu.splines.new("POLY")
    spline.points.add(points - 1)
    cu.materials.append(material)
    obj = bpy.data.objects.new(name, cu)
    bpy.context.scene.collection.objects.link(obj)
    return obj


ARC_POINTS = 9


def gifts(beat, d, ctx):
    """Each gift of the generation shown as a yarn arc from giver to
    receiver, drawn before the generation's leaps and fading into them."""
    most = max((len(f.gifts) for f in d.frames), default=0)
    yarn = materials.knit("butter")
    arcs = [_curve(f"gift{i}", yarn, ARC_POINTS) for i in range(most)]

    def update(frame):
        t = ctx.timing.tick_at(frame)
        k = min(int(t), d.ticks)
        frac = t - k if k < d.ticks else 0.0
        f = d.frames[k]
        spots = ring.layout(f)
        # Arcs grow over the first part of the tick, and go before the leaps.
        grow = min(frac / max(1 - ctx.timing.hop, 0.1) * 1.5, 1.0) if k < d.ticks else 1.0
        shown = frac < 1 - ctx.timing.hop or k >= d.ticks
        for i, arc in enumerate(arcs):
            pts = arc.data.splines[0].points
            if not shown or i >= len(f.gifts):
                for p in pts:
                    p.co = (0, 0, -20, 1)
                continue
            (x0, y0), (x1, y1) = spots[f.gifts[i][0]], spots[f.gifts[i][1]]
            height = 0.6 + 0.12 * math.hypot(x1 - x0, y1 - y0)
            for j, p in enumerate(pts):
                u = j / (ARC_POINTS - 1) * grow
                p.co = (x0 + (x1 - x0) * u, y0 + (y1 - y0) * u, 0.5 + height * 4 * u * (1 - u), 1)

    return update


def tolerance(beat, d, ctx):
    """The tolerance of the Flumps at the places in params `places`, each
    as a sliver of the ring from tag − tolerance to tag + tolerance, just
    inside the wheel of shades."""
    places = beat.params["places"]
    ink = materials.fading("tolerance-ink", (1.0, 0.95, 0.8), 3.0)
    slivers = [_curve(f"tolerance{k}", ink, 25) for k in places]
    for s in slivers:
        s.data.bevel_depth = 0.12

    def update(frame):
        f = d.frame(ctx.timing.tick_at(frame))
        for k, s in zip(places, slivers):
            a = f.agents[k]
            r = ring.RADIUS - 2.1
            for j, p in enumerate(s.data.splines[0].points):
                tag = a.tag - a.tolerance + 2 * a.tolerance * j / 24
                x, y = ring.on_ring(tag, r)
                p.co = (x, y, 0.05, 1)

    return update
