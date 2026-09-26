"""On-screen text and data displays. Each overlay builder takes (beat, dump,
ctx) and returns a per-frame updater; `ctx` carries the camera's `Screen`,
the beat's timing, the agents' tracks, the board's corners and the close-up
rigs. Screen-space things hang from `Screen` anchors, whose units are half
the frame's width, so they keep their size on screen as the lens changes."""

import math
import pathlib

import bmesh
import bpy

import animate
import dump as dump_mod
import seasons
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


def caption_scene(body, y, preview, title=False):
    """A scene holding only the caption — rounded cream text over a soft dark
    shadow — on a transparent film, rendered once per beat and laid over the
    beat by the cut (which fades it), so depth of field never blurs it. A
    title caption is larger, over a dark scrim that dims the whole frame."""
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
    size = 0.105 if title else 0.085
    if title:
        scrim = materials.fading("scrim", INK, 1.0)
        scrim.node_tree.nodes["Mix"].inputs[0].default_value = 0.62
        box("scrim", scrim, None, location=(0, 0, -1), scale=(2.4, 1.4, 0.01))
    text("caption-shadow", body, size, materials.fading("caption-ink", INK, 1.0), anchor, location=(0.005, -0.006, -0.01))
    text("caption", body, size, materials.fading("caption", CREAM, 1.0), anchor)


def _pose(ctx, d, id_, frame):
    return animate.pose(ctx.tracks[id_], ctx.timing, frame, ctx.corners, d.width, d.height)


def _turn_to_camera(obj, ctx):
    direction = ctx.camera.matrix_world.translation - obj.location
    obj.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()


def belly(beat, d, ctx):
    """A meter floating over each focus Flump: a dark case, a glowing gold
    fill for its sugar (full at 20) and the count beside it."""
    meters = {}
    # Bright emission washes out toward white under AgX; a deep orange at
    # modest strength stays gold.
    gold = materials.fading("belly-gold", (1.0, 0.38, 0.0), 0.9)
    cream = materials.fading("belly-ink", (1.0, 0.97, 0.9), 4.0)
    for i in beat.focus:
        id_ = d.placed[i]
        holder = bpy.data.objects.new(f"belly{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        # The holder turns to face the camera, so its local +Z points at the lens.
        _card(f"belly{id_}-case", holder, (0, 0, -0.02), (0.74, 0.2, 0.02))
        fill = box(f"belly{id_}-fill", gold, holder, scale=(0.64, 0.12, 0.02))
        count = text(f"belly{id_}-count", "", 0.3, cream, holder, location=(0.45, -0.02, 0), align="LEFT")
        meters[id_] = (holder, fill, count)

    def update(frame):
        for id_, (holder, fill, count) in meters.items():
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            level = min(max(p.sugar / 20, 0.0), 1.0)
            holder.scale = (1, 1, 1)
            holder.location = (p.x, p.y, p.z + 1.05)
            _turn_to_camera(holder, ctx)
            fill.scale.x = max(level, 0.002) * 0.64
            fill.location.x = -0.32 + fill.scale.x / 2
            count.data.body = f"{p.sugar:.0f}"

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
    """"sees N · eats M" over each focus Flump on a dark pill, turned to the camera."""
    items = []
    cream = materials.fading("label", CREAM, 2.2)
    for i in beat.focus:
        id_ = d.placed[i]
        a = d.frames[ctx.tracks[id_].first].agents[id_]
        holder = bpy.data.objects.new(f"label{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        _card(f"label{id_}-pill", holder, (0, 0, -0.03), (2.6, 0.5, 0.02))
        text(f"label{id_}-text", f"sees {a.vision} · eats {a.metabolism}", 0.34, cream, holder)
        items.append((id_, holder))

    def update(frame):
        for id_, holder in items:
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            holder.location = (p.x, p.y, p.z + 1.2)
            holder.scale = (0.6,) * 3
            _turn_to_camera(holder, ctx)

    return update


def _card(name, parent, location, scale):
    """A dark card behind a display, so it reads on any shot. Its bevel is
    wide, so small cards come out as pills."""
    card = box(name, materials.matte("card", (0.03, 0.025, 0.02)), parent, location=location, scale=scale)
    card.modifiers["round"].width = 0.08
    card.modifiers["round"].segments = 6
    return card


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
            label.data.body = f"average {title}: {v:.2f}   (was {series[0]:.2f})"

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
    anchor = ctx.screen.anchor("histogram", 0.62, -0.55)
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


GROUPS = (("poorest half", "teal", 0.5), ("middle 40%", "cream", 0.4), ("richest 10%", "coral", 0.1))


def wealth(beat, d, ctx):
    """Who holds the sugar: the Flumps split into the poorest half, the
    middle 40 % and the richest tenth, and beneath them the same groups'
    shares of all the sugar, from the frame shown."""
    anchor = ctx.screen.anchor("wealth", 0.62, 0.6)
    _card("wealth-card", anchor, (0, 0.01, -0.01), (0.6, 0.4, 0.002))
    ink = materials.fading("wealth-ink", CREAM, 1.6)
    text("wealth-title", "who holds the sugar", 0.04, ink, anchor, location=(0, 0.165, 0))
    left, width = -0.14, 0.4
    x = left
    # Name the two ends; the middle's label would collide with the narrow top tenth's.
    for (name, _, share), align in zip(GROUPS, ("CENTER", None, "RIGHT")):
        if align:
            at = x + share * width / 2 if align == "CENTER" else left + width
            text(f"wealth-label-{name}", name, 0.024, ink, anchor, location=(at, 0.108, 0), align=align)
        x += share * width
    rows = {}
    for row, y in (("Flumps", 0.06), ("sugar", -0.03)):
        text(f"wealth-{row}", row, 0.035, ink, anchor, location=(left - 0.02, y - 0.01, 0), align="RIGHT")
        rows[row] = [
            box(f"wealth-{row}-{name}", materials.knit(color), anchor, location=(0, y, 0), scale=(0.01, 0.06, 0.004))
            for name, color, _ in GROUPS
        ]
    readout = text("wealth-readout", "", 0.032, ink, anchor, location=(0, -0.12, 0))

    def lay(bars, fractions):
        x = left
        for bar, f in zip(bars, fractions):
            bar.scale.x = max(f * width, 0.002)
            bar.location.x = x + bar.scale.x / 2
            x += f * width

    lay(rows["Flumps"], [g[2] for g in GROUPS])

    def update(frame):
        f = d.frames[min(int(round(ctx.timing.tick_at(frame))), d.ticks)]
        poor, middle, rich = animate.shares([a.sugar for a in f.agents.values()])
        lay(rows["sugar"], (poor, middle, rich))
        readout.data.body = f"richest 10%: {rich:.0%} of the sugar  ·  poorest half: {poor:.0%}"

    return update


SUMMER, WINTER = (1.0, 0.72, 0.25), (0.62, 0.8, 1.0)


def season_card(beat, d, ctx):
    """Top left: which hemisphere has summer and which winter, from the
    engine's schedule for the tick shown."""
    period = d.config["seasons"]["period"]
    anchor = ctx.screen.anchor("season-card", -0.72, 0.78)
    _card("season-card-box", anchor, (0, 0, -0.01), (0.5, 0.24, 0.002))
    ink = materials.fading("season-ink", CREAM, 1.6)
    warm = materials.fading("season-summer", SUMMER, 1.6)
    cold = materials.fading("season-winter", WINTER, 1.6)
    rows = {}
    for side, y in (("north", 0.05), ("south", -0.05)):
        text(f"season-{side}", side, 0.06, ink, anchor, location=(-0.21, y - 0.015, 0), align="LEFT")
        rows[side] = (
            text(f"season-{side}-summer", "summer", 0.06, warm, anchor, location=(0.21, y - 0.015, 0), align="RIGHT"),
            text(f"season-{side}-winter", "winter", 0.06, cold, anchor, location=(0.21, y - 0.015, 0), align="RIGHT"),
        )

    def update(frame):
        summer_north = seasons.north_summer(int(ctx.timing.tick_at(frame)), period)
        for side, (summer, winter) in rows.items():
            on, off = (summer, winter) if summer_north == (side == "north") else (winter, summer)
            on.scale = (1, 1, 1)
            on.location.z = 0
            off.scale = (1e-4, 1e-4, 1e-4)  # hidden in place (see flump.stow)

    return update


def _torus():
    mesh = bpy.data.meshes.get("ring")
    if mesh is None:
        bm = bmesh.new()
        ring, tube = 0.46, 0.08
        rings, sides = 32, 8
        verts = []
        for i in range(rings):
            a = 2 * math.pi * i / rings
            for j in range(sides):
                b = 2 * math.pi * j / sides
                r = ring + tube * math.cos(b)
                verts.append(bm.verts.new((r * math.cos(a), r * math.sin(a), tube * math.sin(b))))
        for i in range(rings):
            for j in range(sides):
                v = lambda ii, jj: verts[(ii % rings) * sides + (jj % sides)]  # noqa: E731
                bm.faces.new((v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1)))
        mesh = bpy.data.meshes.new("ring")
        bm.to_mesh(mesh)
        bm.free()
        for poly in mesh.polygons:
            poly.use_smooth = True
    return mesh


def _rings(name, color, members, d, ctx):
    """A glowing ring at the feet of each Flump in `members`."""
    glow = materials.fading(f"ring-{name}", color, 3.0)
    mesh = _torus()
    rings = {}
    for id_ in members:
        obj = bpy.data.objects.new(f"ring-{name}-{id_}", mesh)
        bpy.context.scene.collection.objects.link(obj)
        # The ring mesh is shared; each group's material lives on its objects.
        if not obj.data.materials:
            obj.data.materials.append(None)
        obj.material_slots[0].link = "OBJECT"
        obj.material_slots[0].material = glow
        rings[id_] = obj

    def update(frame):
        for id_, obj in rings.items():
            p = _pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(obj)
                continue
            s = max(p.sx, 1e-4)
            obj.scale = (s, s, 1)
            obj.location = (p.x, p.y, p.z + 0.04)

    return update


def rings_migrants(beat, d, ctx):
    """Gold rings on the migrants: alive from tick 100 to the end and
    crossing hemispheres at least twice (the claims' definition)."""
    return _rings("migrants", (1.0, 0.7, 0.1), seasons.migrants(ctx.tracks, 100, d.ticks, d.height), d, ctx)


def rings_hungry(beat, d, ctx):
    """Red rings on the hungry — metabolism ≥ 3 — alive when the beat starts."""
    at = d.frames[beat.start_tick].agents
    return _rings("hungry", (1.0, 0.15, 0.1), {i for i, a in at.items() if a.metabolism >= 3}, d, ctx)


def counter(beat, d, ctx):
    """Top right: this world's population against the compare world's (the
    same seed without seasons) at the tick shown."""
    anchor = ctx.screen.anchor("counter", 0.66, 0.78)
    _card("counter-card", anchor, (0, 0, -0.01), (0.62, 0.24, 0.002))
    cold = materials.fading("counter-seasons", WINTER, 1.6)
    warm = materials.fading("counter-calm", SUMMER, 1.6)
    with_ = text("counter-with", "", 0.052, cold, anchor, location=(-0.28, 0.035, 0), align="LEFT")
    without = text("counter-without", "", 0.052, warm, anchor, location=(-0.28, -0.065, 0), align="LEFT")

    def update(frame):
        k = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        with_.data.body = f"with seasons: {len(d.frames[k].agents)} Flumps"
        without.data.body = f"without seasons: {len(ctx.compare.frames[k].agents)} Flumps"

    return update


SURVIVAL_ROWS = [
    ("started rich", "rich_alive", "started poor", "poor_alive"),
    ("born on a hill", "hill_alive", "born on the plains", "plain_alive"),
    ("needs little", "met_low_alive", "needs a lot", "met_high_alive"),
]


def survival_panel(beat, d, ctx):
    """How many survive by what they started with — the medians over the
    measured seeds (measurements.json), not one run, since the caption's
    ranking is theirs: rich vs poor, hill vs plains, needing little vs a lot."""
    medians, seeds = ctx.measured["medians"], ctx.measured["seeds"]
    anchor = ctx.screen.anchor("survival", 0.46, 0.08)
    _card("survival-card", anchor, (0, 0.0, -0.01), (0.9, 0.68, 0.002))
    ink = materials.fading("survival-ink", CREAM, 1.6)
    text("survival-title", "who survives the seasons", 0.055, ink, anchor, location=(0, 0.27, 0))
    text("survival-note", f"median over {seeds} runs", 0.035, ink, anchor, location=(0, 0.215, 0))
    width = 0.3
    for i, (good, good_key, bad, bad_key) in enumerate(SURVIVAL_ROWS):
        y = 0.13 - i * 0.17
        for j, (label, key, color) in enumerate(((good, good_key, "teal"), (bad, bad_key, "coral"))):
            yy = y - j * 0.062
            share = medians[key]
            text(f"survival-{i}-{j}", label, 0.04, ink, anchor, location=(-0.06, yy - 0.012, 0), align="RIGHT")
            bar_w = max(share * width, 0.002)
            box(f"survival-bar-{i}-{j}", materials.knit(color), anchor,
                location=(-0.04 + bar_w / 2, yy, 0), scale=(bar_w, 0.046, 0.004))
            text(f"survival-pct-{i}-{j}", f"{share:.0%}", 0.04, ink, anchor,
                 location=(-0.02 + bar_w, yy - 0.012, 0), align="LEFT")
    return lambda frame: None


# The overlays drawn in screen space (on `Screen` anchors).
SCREEN = {"season-card", "counter", "survival", "dials", "histogram", "wealth"}

BUILDERS = {
    "season-card": season_card,
    "rings-migrants": rings_migrants,
    "rings-hungry": rings_hungry,
    "counter": counter,
    "survival": survival_panel,
    "wealth": wealth,
    "belly": belly,
    "sight": sight,
    "labels": labels,
    "dials": dials,
    "stacks": stacks,
    "histogram": histogram,
}
