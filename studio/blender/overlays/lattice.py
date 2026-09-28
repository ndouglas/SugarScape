"""Overlays for spatial games: the helpers gauge, average earnings, each
Flump's score in a close-up, and the games one Flump plays."""

import math

import bpy
from mathutils import Vector

import lattice
from blender import materials

from .parts import CREAM, box, card, text, turn_to_camera

HELPER_YARN, CHEAT_YARN = "blue", "red"


def _stat(d, name, tick):
    return d.stats[name][min(max(int(round(tick)), 0), d.ticks)]


def helpers(beat, d, ctx):
    """Top right: the share of Flumps helping at the generation shown, as a
    bar and a number."""
    anchor = ctx.screen.anchor("helpers", 0.66, 0.8)
    card("helpers-card", anchor, (0, 0, -0.01), (0.62, 0.2, 0.002))
    box("helpers-back", materials.knit(CHEAT_YARN), anchor, location=(0, -0.035, 0), scale=(0.5, 0.05, 0.004))
    fill = box("helpers-fill", materials.knit(HELPER_YARN), anchor, location=(0, -0.035, 0.003),
               scale=(0.5, 0.05, 0.004))
    label = text("helpers-text", "", 0.05, materials.fading("helpers-ink", CREAM, 1.6), anchor,
                 location=(-0.25, 0.04, 0), align="LEFT")

    def update(frame):
        share = _stat(d, "fraction_c", ctx.timing.tick_at(frame))
        label.data.body = f"helping: {share:.0%}"
        width = max(share, 0.002) * 0.5
        fill.scale.x = width
        fill.location.x = -0.25 + width / 2

    return update



def earnings(beat, d, ctx):
    """Top right: what the average helper and the average cheat earned in
    the generation shown. params: `top`, the earnings a full bar stands for."""
    top = beat.params.get("top", 10.0)
    anchor = ctx.screen.anchor("earnings", 0.62, 0.72)
    card("earnings-card", anchor, (0, 0, -0.01), (0.72, 0.34, 0.002))
    ink = materials.fading("earnings-ink", CREAM, 1.6)
    text("earnings-title", "average earnings, this generation", 0.045, ink, anchor, location=(0, 0.11, 0))
    rows = []
    for j, (label, key, yarn) in enumerate((("helper", "mean_payoff_c", HELPER_YARN),
                                            ("cheat", "mean_payoff_d", CHEAT_YARN))):
        y = 0.01 - j * 0.09
        text(f"earnings-{label}", label, 0.045, ink, anchor, location=(-0.16, y - 0.013, 0), align="RIGHT")
        bar = box(f"earnings-bar-{label}", materials.knit(yarn), anchor, location=(0, y, 0), scale=(0.01, 0.05, 0.004))
        value = text(f"earnings-value-{label}", "", 0.045, ink, anchor, location=(0, y - 0.013, 0), align="LEFT")
        rows.append((key, bar, value, y))
    width = 0.3

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        for key, bar, value, y in rows:
            v = _stat(d, key, tick)
            v = 0.0 if v is None or (isinstance(v, float) and math.isnan(v)) else v
            w = max(min(v / top, 1.0) * width, 0.002)
            bar.scale.x = w
            bar.location.x = -0.14 + w / 2
            value.location.x = -0.12 + w
            value.data.body = f"{v:.1f}"

    return update


def scores(beat, d, ctx):
    """A close-up: the score of one Flump (params `square`: [x, y]) and of
    each of its neighbors, at the generation shown, on dark pills facing the
    camera; the best of them, whose side the Flump takes next, in gold."""
    x0, y0 = beat.params["square"]
    w, h = d.width, d.height
    cream = materials.fading("score-ink", CREAM, 2.2)
    gold = materials.fading("score-best", (1.0, 0.62, 0.08), 4.0)
    items = []
    for dx, dy in [(0, 0), *lattice.NEIGHBORS]:
        x, y = x0 + dx, y0 + dy
        if not (0 <= x < w and 0 <= y < h):
            continue
        i = y * w + x
        holder = bpy.data.objects.new(f"score{i}", None)
        bpy.context.scene.collection.objects.link(holder)
        card(f"score{i}-pill", holder, (0, 0, -0.03), (1.2, 0.55, 0.02))
        px, py = lattice.position(i, w, h)
        holder.location = (px, py, 1.05)
        holder.scale = (0.56,) * 3
        # Two labels, one per ink, the other shrunk away: swapping a label's
        # material mid-animation recompiles shaders, and crashed Eevee.
        items.append((i, holder, text(f"score{i}-text", "", 0.36, cream, holder),
                      text(f"score{i}-best", "", 0.36, gold, holder)))

    def update(frame):
        f = d.frame(ctx.timing.tick_at(frame))
        best = max(f.scores[i] for i, *_ in items)
        for i, holder, plain, gilded in items:
            body = f"{round(f.scores[i], 1):g}"
            shown, hidden = (gilded, plain) if f.scores[i] == best else (plain, gilded)
            if shown.data.body != body:
                shown.data.body = body
            shown.scale = (1, 1, 1)
            hidden.scale = (1e-4, 1e-4, 1e-4)
            holder.rotation_euler = ctx.camera.rotation_euler

    return update


def play(beat, d, ctx):
    """A close-up: yarn lines from one Flump (params `square`: [x, y]) to each
    of its neighbors, drawn out over the beat's first second."""
    x0, y0 = beat.params["square"]
    w, h = d.width, d.height
    here = Vector((*lattice.position(y0 * w + x0, w, h), 0.45))
    yarn = materials.knit("cream")
    lines = []
    for dx, dy in lattice.NEIGHBORS:
        x, y = x0 + dx, y0 + dy
        if not (0 <= x < w and 0 <= y < h):
            continue
        there = Vector((*lattice.position(y * w + x, w, h), 0.45))
        bpy.ops.mesh.primitive_cylinder_add(radius=0.035, depth=1, vertices=12)
        line = bpy.context.active_object
        line.data.materials.append(yarn)
        lines.append((line, here, there))

    def update(frame):
        grow = min(max((frame - 1) / 30, 0.0), 1.0)
        for line, a, b in lines:
            end = a.lerp(b, grow * 0.72 + 0.14)
            start = a.lerp(b, 0.14)
            line.location = (start + end) / 2
            line.scale = (1, 1, max((end - start).length, 1e-4))
            line.rotation_euler = (end - start).to_track_quat("Z", "Y").to_euler()

    return update
