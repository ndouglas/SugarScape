"""Overlays in the world, following Flumps: meters, sight, labels, sugar
stacks, rings, bequests and traits."""

import math

import bmesh
import bpy

import animate
import lineage
import seasons
from blender import flump, materials

from .parts import CREAM, ball, box, card, flump_pose, text, turn_to_camera


def belly(beat, d, ctx):
    """A meter floating over each focus Flump (or every Flump, with params
    belly="all"): a dark case, a glowing gold fill for its sugar (full at
    params belly_full, default 20) and the count beside it."""
    meters = {}
    full = beat.params.get("belly_full", 20)
    ids = list(ctx.tracks) if beat.params.get("belly") == "all" else [d.placed[i] for i in beat.focus]
    # Bright emission washes out toward white under AgX; a deep orange at
    # modest strength stays gold.
    gold = materials.fading("belly-gold", (1.0, 0.38, 0.0), 0.9)
    cream = materials.fading("belly-ink", (1.0, 0.97, 0.9), 4.0)
    for id_ in ids:
        holder = bpy.data.objects.new(f"belly{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        # The holder turns to face the camera, so its local +Z points at the lens.
        card(f"belly{id_}-case", holder, (0, 0, -0.02), (0.74, 0.2, 0.02))
        fill = box(f"belly{id_}-fill", gold, holder, scale=(0.64, 0.12, 0.02))
        count = text(f"belly{id_}-count", "", 0.3, cream, holder, location=(0.45, -0.02, 0), align="LEFT")
        meters[id_] = (holder, fill, count)

    def update(frame):
        for id_, (holder, fill, count) in meters.items():
            p = flump_pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            level = min(max(p.sugar / full, 0.0), 1.0)
            holder.scale = (1, 1, 1)
            holder.location = (p.x, p.y, p.z + 1.05)
            turn_to_camera(holder, ctx)
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
        card(f"label{id_}-pill", holder, (0, 0, -0.03), (2.6, 0.5, 0.02))
        text(f"label{id_}-text", f"sees {a.vision} · eats {a.metabolism}", 0.34, cream, holder)
        items.append((id_, holder))

    def update(frame):
        for id_, holder in items:
            p = flump_pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            holder.location = (p.x, p.y, p.z + 1.2)
            holder.scale = (0.6,) * 3
            turn_to_camera(holder, ctx)

    return update


def stacks(beat, d, ctx):
    """A column of sugar over every Flump, as tall as its wealth (params
    stack_scale: height per unit of sugar, default 0.012)."""
    per = beat.params.get("stack_scale", 0.012)
    sugar = materials.gumdrop()
    columns = {id_: box(f"stack{id_}", sugar, None, scale=(0.2, 0.2, 0.01)) for id_ in ctx.tracks}

    def update(frame):
        for id_, c in columns.items():
            p = flump_pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(c)
                continue
            height = max(p.sugar * per, 0.01)
            c.scale = (0.2, 0.2, height)
            c.location = (p.x, p.y, p.z + 0.85 * p.sz + height / 2)

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
            p = flump_pose(ctx, d, id_, frame)
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


def bequests(beat, d, ctx):
    """Sugar flowing from each Flump who dies with some to each child alive to
    inherit it (rule I; nothing without inheritance): glowing gumdrops arc
    from the parent to its heirs as it poofs, one per heir, each carrying an
    equal share."""
    flows = []
    gold = materials.gumdrop()
    for tick, parent, held, heirs in lineage.bequests(d):
        if parent not in ctx.tracks:
            continue  # not alive during this beat
        start = ctx.timing.frame(tick) - ctx.timing.poof_frames
        home = ctx.tracks[parent].cells[-1]
        for heir in (h for h in heirs if h in ctx.tracks):
            obj = ball(f"bequest-{parent}-{heir}", 0.16, gold)
            flows.append((start, home, heir, obj))

    def update(frame):
        for start, home, heir, obj in flows:
            u = (frame - start) / BEQUEST_FRAMES
            if not 0 <= u <= 1:
                flump.stow(obj)
                continue
            x0, y0 = animate.cell_center(*home, d.width, d.height)
            z0 = animate.cell_height(ctx.corners, *home, d.width) + 0.6
            p = flump_pose(ctx, d, heir, frame)
            e = animate.smoothstep(u)
            obj.location = (x0 + (p.x - x0) * e, y0 + (p.y - y0) * e, z0 + (p.z + 0.6 - z0) * e + 1.2 * math.sin(math.pi * u))
            s = 1 - 0.6 * u
            obj.scale = (s, s, s)

    return update


BEQUEST_FRAMES = 24


def traits(beat, d, ctx):
    """Each focus Flump's cultural tags (its eleven traits) on a dark pill
    over it, updated as they flip; the text is its tribe's color."""
    items = []
    inks = [materials.fading(f"traits-{c}", materials.YARN[c], 3.0) for c in materials.TRIBE_YARN]
    for i in beat.focus:
        id_ = d.placed[i]
        holder = bpy.data.objects.new(f"traits{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        card(f"traits{id_}-pill", holder, (0, 0, -0.03), (2.3, 0.5, 0.02))
        labels = [text(f"traits{id_}-{g}", "", 0.34, ink, holder) for g, ink in enumerate(inks)]
        items.append((id_, holder, labels))

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        for id_, holder, labels in items:
            p = flump_pose(ctx, d, id_, frame)
            if not p.visible or id_ not in d.frames[k].tags:
                flump.stow(holder)
                continue
            holder.location = (p.x, p.y, p.z + 1.2)
            holder.scale = (0.6,) * 3
            turn_to_camera(holder, ctx)
            group = d.frames[k].groups[id_]
            for g, label in enumerate(labels):
                label.data.body = d.frames[k].tags[id_] if g == group else ""

    return update
