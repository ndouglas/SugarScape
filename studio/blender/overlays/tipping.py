"""Overlays for Schelling's bounded neighborhood: the fence around the area,
and his plane (Red inside across, Blue inside up) with each color's
tolerance curve and the path the area takes."""

import area
from blender import materials

from .panels import _polyline
from .parts import CREAM, box, card, text


def fence(beat, d, ctx):
    """A low cream fence around the area's squares on the board."""
    w, h = d.width, d.height
    wood = materials.knit("cream")
    left = area.LEFT - w / 2
    right = area.RIGHT - w / 2
    top, bottom = h / 2, h / 2 - area.AREA
    rail = 0.12
    for name, loc, scale in (
        ("fence-n", ((left + right) / 2, top + rail / 2, 0.18), (right - left + rail, rail, 0.3)),
        ("fence-s", ((left + right) / 2, bottom - rail / 2, 0.18), (right - left + rail, rail, 0.3)),
        ("fence-w", (left - rail / 2, (top + bottom) / 2, 0.18), (rail, top - bottom, 0.3)),
        ("fence-e", (right + rail / 2, (top + bottom) / 2, 0.18), (rail, top - bottom, 0.3)),
    ):
        box(name, wood, None, location=loc, scale=scale)
    return lambda frame: None


def tipplane(beat, d, ctx):
    """Top right: Schelling's plane. Red inside across (0 to all Red), Blue
    inside up; Red's curve (the most tolerant n Red abide n·R(n) Blue: Red
    stay below it) and Blue's (Blue stay left of it); the path so far and a
    marker at the step shown."""
    red, blue = d.config["red"], d.config["blue"]
    t_red, t_blue = d.config["tolerances"]
    anchor = ctx.screen.anchor("tipplane", 0.7, 0.55)
    card("tipplane-card", anchor, (0, 0, -0.01), (0.5, 0.5, 0.002))
    ink = materials.fading("tipplane-ink", CREAM, 1.6)
    size, left, bottom = 0.34, -0.17, -0.18

    def at(r, b):
        return left + size * r / max(red, 1), bottom + size * min(b, blue) / max(blue, 1)

    text("tipplane-title", "Schelling's plane", 0.032, ink, anchor, location=(0, 0.215, 0))
    text("tipplane-x", "Red inside →", 0.024, ink, anchor, location=(left + size, bottom - 0.035, 0), align="RIGHT")
    text("tipplane-y", "Blue inside ↑", 0.024, ink, anchor, location=(left - 0.01, bottom + size + 0.03, 0),
         align="LEFT")
    box("tipplane-axis-x", ink, anchor, location=(left + size / 2, bottom, 0.001), scale=(size, 0.003, 0.002))
    box("tipplane-axis-y", ink, anchor, location=(left, bottom + size / 2, 0.001), scale=(0.003, size, 0.002))
    red_ink = materials.fading("tipplane-red", materials.YARN["red"], 2.5)
    blue_ink = materials.fading("tipplane-blue", materials.YARN["blue"], 2.5)
    red_curve = [at(n, n * t_red[n - 1] if n else 0) for n in range(red + 1)]
    blue_curve = [at(min(m * t_blue[m - 1], red) if m else 0, m) for m in range(blue + 1)]
    for name, pts, mat in (("tipplane-redcurve", red_curve, red_ink), ("tipplane-bluecurve", blue_curve, blue_ink)):
        _, spline = _polyline(name, anchor, pts, mat)
        spline.points.foreach_set("co", [v for x, y in pts for v in (x, y, 0.002, 1)])
    path_obj, path = _polyline("tipplane-path", anchor, [(0, 0)] * (d.ticks + 1), ink)
    marker = box("tipplane-now", materials.fading("tipplane-now", (1.0, 0.8, 0.2), 3.0), anchor,
                 location=(0, 0, 0.004), scale=(0.022, 0.022, 0.004))
    counts = list(zip(d.stats["red_in"], d.stats["blue_in"]))

    def update(frame):
        now = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        pts = []
        for k in range(len(counts)):
            x, y = at(*counts[min(k, now)])
            pts += [x, y, 0.003, 1]
        path.points.foreach_set("co", pts)
        x, y = at(*counts[now])
        marker.location = (x, y, 0.004)

    return update
