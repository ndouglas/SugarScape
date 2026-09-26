"""On-screen text and data displays. Each overlay builder takes (beat, dump,
ctx) and returns a per-frame updater; `ctx` carries the camera's `Screen`,
the beat's timing, the agents' tracks, the board's corners and the close-up
rigs. Screen-space things hang from `Screen` anchors, whose units are half
the frame's width, so they keep their size on screen as the lens changes."""

import pathlib

import bmesh
import bpy

import animate
from blender import flump, materials

FONT = pathlib.Path(__file__).resolve().parent.parent / "fonts" / "Baloo2.ttf"
CREAM, INK = (1.0, 0.96, 0.88), (0.06, 0.05, 0.04)
SENSOR = 36.0  # mm, horizontal (Blender's default sensor fit for 16:9)
# How far in front of the lens screen-space things float: nearer than
# anything in the scene. (Beats with them have no depth of field.)
DEPTH = 0.4


class Screen:
    """Anchors glued to the camera's view: x in [-1, 1] across the frame's
    width, y in [-1, 1] up its height."""

    def __init__(self, camera_obj):
        self.camera = camera_obj
        self.anchors = []

    def anchor(self, name, x, y, depth=DEPTH):
        e = bpy.data.objects.new(name, None)
        e.parent = self.camera
        bpy.context.scene.collection.objects.link(e)
        self.anchors.append((e, x, y, depth))
        return e

    def update(self, frame):
        for e, x, y, depth in self.anchors:
            half = depth * SENSOR / 2 / self.camera.data.lens
            e.location = (x * half, y * half * 9 / 16, -depth)
            e.scale = (half, half, half)


def _font():
    return bpy.data.fonts.load(str(FONT), check_existing=True)


def text(name, body, size, material, parent, location=(0, 0, 0), align="CENTER"):
    cu = bpy.data.curves.new(name, type="FONT")
    cu.body = body
    cu.font = _font()
    cu.size = size
    cu.align_x = align
    cu.align_y = "CENTER"
    cu.materials.append(material)
    obj = bpy.data.objects.new(name, cu)
    obj.parent = parent
    obj.location = location
    bpy.context.scene.collection.objects.link(obj)
    return obj


def box(name, material, parent, location=(0, 0, 0), scale=(1, 1, 1)):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.parent = parent
    obj.location, obj.scale = location, scale
    bpy.context.scene.collection.objects.link(obj)
    bev = obj.modifiers.new("round", "BEVEL")
    bev.width, bev.segments = 0.02, 3
    return obj


def ball(name, radius, material, parent=None):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=12, v_segments=6, radius=radius)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(material)
    obj = bpy.data.objects.new(name, mesh)
    obj.parent = parent
    bpy.context.scene.collection.objects.link(obj)
    return obj


def caption_scene(body, y, preview):
    """A scene holding only the caption — rounded cream text over a soft dark
    shadow — on a transparent film, rendered once per beat and laid over the
    beat by the cut (which fades it), so depth of field never blurs it."""
    scene = bpy.context.scene
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj)
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.resolution_x, scene.render.resolution_y = (960, 540) if preview else (1920, 1080)
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.view_settings.view_transform = "Standard"
    data = bpy.data.cameras.new("caption-camera")
    data.type = "ORTHO"
    data.ortho_scale = 2.0  # x spans [-1, 1]
    cam = bpy.data.objects.new("caption-camera", data)
    cam.location = (0, 0, 10)
    scene.collection.objects.link(cam)
    scene.camera = cam
    anchor = bpy.data.objects.new("caption", None)
    anchor.location = (0, y * 9 / 16, 0)
    scene.collection.objects.link(anchor)
    text("caption-shadow", body, 0.085, materials.fading("caption-ink", INK, 1.0), anchor, location=(0.005, -0.006, -0.01))
    text("caption", body, 0.085, materials.fading("caption", CREAM, 1.0), anchor)


def _pose(ctx, d, id_, frame):
    return animate.pose(ctx.tracks[id_], ctx.timing, frame, ctx.corners, d.width, d.height)


def belly(beat, d, ctx):
    """A meter floating over each focus Flump, filled by its sugar (full at 20)."""
    meters = {}
    for i in beat.focus:
        id_ = d.placed[i]
        holder = bpy.data.objects.new(f"belly{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        back = box(f"belly{id_}-back", materials.knit("cream"), holder, scale=(0.62, 0.08, 0.14))
        fill = box(f"belly{id_}-fill", materials.knit("butter"), holder, location=(0, -0.05, 0), scale=(0.56, 0.06, 0.09))
        meters[id_] = (holder, back, fill)

    def update(frame):
        for id_, (holder, back, fill) in meters.items():
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            level = min(max(p.sugar / 20, 0.0), 1.0)
            holder.scale = (1, 1, 1)
            holder.location = (p.x, p.y, p.z + 1.0)
            fill.scale.x = max(level, 0.002) * 0.56
            fill.location.x = -0.28 + fill.scale.x / 2

    return update


def sight(beat, d, ctx):
    """Glowing dots over the cells the first focus Flump can see, lit row by
    row (north, east, south, west) before each hop and gone as it lands."""
    id_ = d.placed[beat.focus[0]]
    t = ctx.tracks[id_]
    glow = materials.fading("sight", (1.0, 0.55, 0.08), 2.5)
    rows = [
        animate.sight_cells(x, y, d.frames[t.first + k].agents[id_].vision, d.width, d.height)
        for k, (x, y) in enumerate(t.cells)
    ]
    most = max(sum(len(r) for r in dirs) for dirs in rows)
    pool = [ball(f"sight{i}", 0.09, glow) for i in range(most)]
    hop = ctx.timing.hop

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        k = min(max(int(tick) - t.first, 0), len(rows) - 1)
        a = tick - int(tick)
        # Look during the stillness before the hop; stop as it lands.
        look_end = 1 - hop * (1 - animate.CROUCH)
        used = 0
        for j, row in enumerate(rows[k]):
            start = 0.1 + j * (look_end - 0.2) / 4
            shown = animate.smoothstep((a - start) / 0.06) * (1 - animate.smoothstep((a - (1 - hop * (1 - animate.LAND))) / 0.03))
            for cx, cy in row:
                o = pool[used]
                used += 1
                x, y = animate.cell_center(cx, cy, d.width, d.height)
                o.location = (x, y, animate.cell_height(ctx.corners, cx, cy, d.width) + 0.6)
                o.scale = (max(shown, 1e-4),) * 3
        for o in pool[used:]:
            flump.stow(o)

    return update


def labels(beat, d, ctx):
    """"sees N · eats M" over each focus Flump, turned to the camera."""
    items = []
    cream = materials.fading("label", CREAM, 1.2)
    for i in beat.focus:
        id_ = d.placed[i]
        a = d.frames[ctx.tracks[id_].first].agents[id_]
        holder = bpy.data.objects.new(f"label{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        text(f"label{id_}-text", f"sees {a.vision} · eats {a.metabolism}", 0.3, cream, holder)
        items.append((id_, holder))

    def update(frame):
        for id_, holder in items:
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            holder.location = (p.x, p.y, p.z + 1.15)
            holder.scale = (0.6,) * 3
            direction = ctx.camera.matrix_world.translation - holder.location
            holder.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()

    return update


def _card(name, parent, location, scale):
    """A dark felt card behind a screen-space display, so it reads on any shot."""
    return box(name, materials.matte("card", (0.05, 0.04, 0.035)), parent, location=location, scale=scale)


def _gauge(screen, name, color, x, y):
    anchor = screen.anchor(name, x, y)
    box(f"{name}-back", materials.knit("cream"), anchor, scale=(0.5, 0.05, 0.004))
    fill = box(f"{name}-fill", materials.knit(color), anchor, location=(0, -0.004, 0.003), scale=(0.46, 0.035, 0.004))
    label = text(f"{name}-text", "", 0.045, materials.fading(f"{name}-ink", CREAM, 1.2), anchor, location=(-0.25, 0.06, 0), align="LEFT")
    return fill, label


def dials(beat, d, ctx):
    """Two gauges at the top right: the population's mean sight and hunger."""
    rows = [("mean_vision", "sight", 6.0, "teal", 0.78), ("mean_metabolism", "hunger", 4.0, "coral", 0.56)]
    card = ctx.screen.anchor("dials-card", 0.66, 0.7)
    _card("dials-card-box", card, (0, 0, -0.01), (0.6, 0.3, 0.002))
    parts = []
    for key, title, top, color, y in rows:
        fill, label = _gauge(ctx.screen, title, color, 0.66, y)
        parts.append((d.stats[key], top, title, fill, label))

    def update(frame):
        i = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        for series, top, title, fill, label in parts:
            v = series[i]
            fill.scale.x = max(min(v / top, 1.0), 0.002) * 0.46
            fill.location.x = -0.23 + fill.scale.x / 2
            label.data.body = f"average {title}: {v:.2f}"

    return update


def stacks(beat, d, ctx):
    """A column of sugar over every Flump, as tall as its wealth."""
    sugar = materials.gumdrop()
    columns = {id_: box(f"stack{id_}", sugar, None, scale=(0.2, 0.2, 0.01)) for id_ in ctx.tracks}

    def update(frame):
        for id_, c in columns.items():
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(c)
                continue
            height = max(p.sugar * 0.012, 0.01)
            c.scale = (0.2, 0.2, height)
            c.location = (p.x, p.y, p.z + 0.85 * p.sz + height / 2)

    return update


def histogram(beat, d, ctx):
    """How many Flumps hold how much sugar, as felt bars at the right."""
    bins = 12
    last = sorted(a.sugar for a in d.frames[-1].agents.values())
    top = last[int(0.99 * (len(last) - 1))] if last else 1.0
    anchor = ctx.screen.anchor("histogram", 0.62, -0.45)
    _card("histogram-card", anchor, (0, 0.17, -0.01), (0.6, 0.5, 0.002))
    width = 0.5 / bins
    bars = [
        box(f"bar{i}", materials.knit("butter"), anchor, location=(-0.25 + (i + 0.5) * width, 0, 0), scale=(width * 0.85, 0.01, 0.004))
        for i in range(bins)
    ]
    ink = materials.fading("hist-ink", CREAM, 1.2)
    text("histogram-title", "how many Flumps hold how much sugar", 0.035, ink, anchor, location=(0, 0.38, 0))
    text("histogram-axis", "little  →  lots", 0.035, ink, anchor, location=(0, -0.045, 0))

    def update(frame):
        f = d.frames[min(int(round(ctx.timing.tick_at(frame))), d.ticks)]
        counts = animate.histogram([a.sugar for a in f.agents.values()], bins, top)
        most = max(max(counts), 1)
        for b, n in zip(bars, counts):
            height = max(n / most * 0.32, 0.004)
            b.scale.y = height
            b.location.y = height / 2

    return update


BUILDERS = {
    "belly": belly,
    "sight": sight,
    "labels": labels,
    "dials": dials,
    "stacks": stacks,
    "histogram": histogram,
}
