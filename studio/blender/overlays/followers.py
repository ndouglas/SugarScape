"""Overlays in the world, following Flumps: meters, sight, labels, sugar
stacks, rings, bequests, traits, trades, loot, the warlord and where kills
happened."""

import math

import bmesh
import bpy

import animate
import lineage
import markets
import seasons
import credit
import war
from blender import board, flump, materials

from .parts import CREAM, ball, box, card, flump_pose, text, turn_to_camera


def _spice_at(d, id_, tick):
    """A Flump's spice at a fractional tick, rising or falling through the tick."""
    k = min(max(int(math.floor(tick)), 0), d.ticks)
    now = d.frames[k].spice_agents.get(id_)
    if now is None:
        return 0.0
    after = d.frames[min(k + 1, d.ticks)].spice_agents.get(id_, now)
    return now[0] + (after[0] - now[0]) * (tick - k)


def belly(beat, d, ctx):
    """A meter floating over each focus Flump (or every Flump, with params
    belly="all"): a dark case, a glowing gold fill for its sugar (full at
    params belly_full, default 20) and the count beside it. In a two-good
    world, a red spice fill beneath, full at the same level."""
    meters = {}
    full = beat.params.get("belly_full", 20)
    ids = list(ctx.tracks) if beat.params.get("belly") == "all" else [d.placed[i] for i in beat.focus]
    two = bool(d.spice_capacity)
    # Bright emission washes out toward white under AgX; a deep orange at
    # modest strength stays gold.
    gold = materials.fading("belly-gold", (1.0, 0.38, 0.0), 0.9)
    red = materials.fading("belly-spice", (0.9, 0.05, 0.02), 0.9)
    cream = materials.fading("belly-ink", (1.0, 0.97, 0.9), 4.0)
    rows = ((0.09, gold), (-0.09, red)) if two else ((0.0, gold),)
    for id_ in ids:
        holder = bpy.data.objects.new(f"belly{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        # The holder turns to face the camera, so its local +Z points at the lens.
        card(f"belly{id_}-case", holder, (0, 0, -0.02), (0.74, 0.38 if two else 0.2, 0.02))
        bars = []
        for i, (y, ink) in enumerate(rows):
            fill = box(f"belly{id_}-fill{i}", ink, holder, location=(0, y, 0), scale=(0.64, 0.12, 0.02))
            size = 0.22 if two else 0.3
            count = text(f"belly{id_}-count{i}", "", size, cream, holder, location=(0.45, y - 0.02, 0), align="LEFT")
            bars.append((fill, count))
        meters[id_] = (holder, bars)

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        for id_, (holder, bars) in meters.items():
            p = flump_pose(ctx, d, id_, frame)
            death = ctx.tracks[id_].death
            # Gone with the poof, so a victim's meter never overlaps its killer's.
            poofing = death is not None and frame >= ctx.timing.frame(death) - ctx.timing.poof_frames
            if not p.visible or poofing:
                flump.stow(holder)
                continue
            holder.scale = (1, 1, 1)
            holder.location = (p.x, p.y, p.z + (1.15 if two else 1.05))
            turn_to_camera(holder, ctx)
            amounts = (p.sugar, _spice_at(d, id_, tick)) if two else (p.sugar,)
            for (fill, count), amount in zip(bars, amounts):
                level = min(max(amount / full, 0.0), 1.0)
                fill.scale.x = max(level, 0.002) * 0.64
                fill.location.x = -0.32 + fill.scale.x / 2
                count.data.body = f"{amount:.0f}"

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
    """"sees N · eats M" over each focus Flump on a dark pill, turned to the
    camera; with params label="age", its age at the tick shown instead, and
    with label="diseases", how many diseases it carries ("well" for none)."""
    items = []
    kind = beat.params.get("label")
    ages = kind in ("age", "diseases")
    cream = materials.fading("label", CREAM, 2.2)
    for i in beat.focus:
        id_ = d.placed[i]
        a = d.frames[ctx.tracks[id_].first].agents[id_]
        holder = bpy.data.objects.new(f"label{id_}", None)
        bpy.context.scene.collection.objects.link(holder)
        card(f"label{id_}-pill", holder, (0, 0, -0.03), (2.6, 0.5, 0.02))
        body = "" if ages else f"sees {a.vision} · eats {a.metabolism}"
        items.append((id_, holder, text(f"label{id_}-text", body, 0.34, cream, holder)))

    def update(frame):
        # Rounded, as the Flumps' live colors are, so a label never contradicts its tint.
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        for id_, holder, label in items:
            p = flump_pose(ctx, d, id_, frame)
            if not p.visible:
                flump.stow(holder)
                continue
            if kind == "age" and id_ in d.frames[k].agents:
                label.data.body = f"age {d.frames[k].agents[id_].age}"
            elif kind == "diseases" and id_ in d.frames[k].agents:
                n = d.frames[k].diseases.get(id_, 0)
                label.data.body = "well" if n == 0 else f"{n} disease{'s' if n > 1 else ''}"
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


def _rings(name, color, members, d, ctx, size=1.0):
    """A glowing ring at the feet of each Flump in `members` (`size` times a
    Flump's width across)."""
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
            obj.scale = (s * size, s * size, 1)
            obj.location = (p.x, p.y, p.z + 0.04)

    return update


def rings_migrants(beat, d, ctx):
    """Gold rings on the migrants: alive from tick 100 to the end and
    crossing hemispheres at least twice (the claims' definition)."""
    return _rings("migrants", (1.0, 0.7, 0.1), seasons.migrants(ctx.tracks, 100, d.ticks, d.height), d, ctx)


def rings_shuttlers(beat, d, ctx):
    """Gold rings on the shuttlers: alive from tick 100 to 200 and changing
    hills at least twice in between (the claims' definition)."""
    sides = markets.sides(d.capacity, d.spice_capacity)
    return _rings("shuttlers", (1.0, 0.7, 0.1), markets.shuttlers(ctx.tracks, d.width, sides, 100, 200), d, ctx)


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


# The share of a tick the arcs fly, from its start: before the next hop
# when ticks are slow; while the Flumps move, following them, when fast.
TRADE_FLIGHT = 0.8
# Partners are neighbors, but across the board's wrapped edge they stand at
# opposite sides of it: an arc between them would cross the whole board.
TRADE_REACH = 1.5


def trades(beat, d, ctx):
    """Each tick's trades as two drops crossing between the partners: a sugar
    gumdrop from the one giving sugar, a spice drop back, sized by how much
    moved, flying early in the tick after the one the dump lists them in
    (where the partners stand then). Pairs that are neighbors only across
    the wrapped edge get no arc."""
    first, last = int(ctx.timing.tick_at(1)), int(ctx.timing.tick_at(beat.frames + 1)) + 1
    busiest = max((len(d.frames[k].trades) for k in range(first, min(last, d.ticks) + 1)), default=0)
    sugar = [ball(f"trade-sugar{i}", 0.1, materials.gumdrop()) for i in range(busiest)]
    spice = [ball(f"trade-spice{i}", 0.1, materials.spice_drop()) for i in range(busiest)]
    for o in sugar + spice:
        flump.stow(o)

    def fly(obj, a, b, u, lift, amount):
        e = animate.smoothstep(u)
        obj.location = (a.x + (b.x - a.x) * e, a.y + (b.y - a.y) * e,
                        a.z + 0.7 + (b.z - a.z) * e + lift * math.sin(math.pi * u))
        s = min(0.6 + 0.15 * math.sqrt(amount), 1.2) * (1 - 0.3 * u)
        obj.scale = (s, s, s)

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        k = min(int(math.floor(tick)), d.ticks)
        u = (tick - k) / TRADE_FLIGHT
        used = 0
        if 0 <= u <= 1:
            for t in d.frames[k].trades:
                if t.sugar_giver not in ctx.tracks or t.spice_giver not in ctx.tracks:
                    continue
                a = flump_pose(ctx, d, t.sugar_giver, frame)
                b = flump_pose(ctx, d, t.spice_giver, frame)
                if not (a.visible and b.visible) or math.hypot(a.x - b.x, a.y - b.y) > TRADE_REACH:
                    continue
                # The drops pass each other: sugar arcs high, spice low.
                fly(sugar[used], a, b, u, 0.9, t.sugar)
                fly(spice[used], b, a, u, 0.45, t.spice)
                used += 1
        for o in sugar[used:] + spice[used:]:
            flump.stow(o)

    return update


LOOT_FRAMES = 18


def loot(beat, d, ctx):
    """At each kill, a glowing gumdrop — the victim's sugar — arcs from where
    the victim stood to its killer, sized by how much it carried."""
    flows = []
    gold = materials.gumdrop()
    for k in range(1, d.ticks + 1):
        before = d.frames[k - 1].agents
        for kill in d.frames[k].kills:
            if kill.attacker not in ctx.tracks or kill.victim not in before:
                continue
            v = before[kill.victim]
            start = ctx.timing.frame(k) - ctx.timing.poof_frames
            size = min(0.12 + 0.03 * math.sqrt(kill.loot), 0.45)
            flows.append((start, (v.x, v.y), kill.attacker, ball(f"loot-{k}-{kill.victim}", size, gold)))

    def update(frame):
        for start, (vx, vy), killer, obj in flows:
            u = (frame - start) / LOOT_FRAMES
            if not 0 <= u <= 1:
                flump.stow(obj)
                continue
            x0, y0 = animate.cell_center(vx, vy, d.width, d.height)
            z0 = animate.cell_height(ctx.corners, vx, vy, d.width) + 0.5
            p = flump_pose(ctx, d, killer, frame)
            e = animate.smoothstep(u)
            obj.location = (x0 + (p.x - x0) * e, y0 + (p.y - y0) * e, z0 + (p.z + 0.5 - z0) * e + math.sin(math.pi * u))
            s = 1 - 0.5 * u
            obj.scale = (s, s, s)

    return update


WARLORD_SCALE = 3.0


def warlord(beat, d, ctx):
    """A red ring at the feet of the run's busiest killer and, over it, its
    sugar and its kills so far."""
    ranked = war.killers(d)
    if not ranked or ranked[0][0] not in ctx.tracks:
        return lambda frame: None
    top = ranked[0][0]
    # Wide shots: the ring and the label are several times a Flump's size, so
    # the one Flump the beat is about can be found among 200.
    ring = _rings("warlord", (1.0, 0.12, 0.08), {top}, d, ctx, size=WARLORD_SCALE)
    holder = bpy.data.objects.new("warlord-label", None)
    bpy.context.scene.collection.objects.link(holder)
    card("warlord-pill", holder, (0, 0, -0.03), (3.4, 0.5, 0.02))
    label = text("warlord-text", "", 0.34, materials.fading("warlord-ink", CREAM, 2.2), holder)
    made = [0]
    for f in d.frames:
        made.append(made[-1] + sum(1 for k in f.kills if k.attacker == top))

    def update(frame):
        ring(frame)
        p = flump_pose(ctx, d, top, frame)
        if not p.visible:
            flump.stow(holder)
            return
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        holder.location = (p.x, p.y, p.z + 1.2 + 0.5 * WARLORD_SCALE)
        holder.scale = (WARLORD_SCALE,) * 3
        turn_to_camera(holder, ctx)
        label.data.body = f"{p.sugar:,.0f} sugar · {made[k + 1]} kills"

    return update


# A kill mark's size as a gumdrop level (4 is a full gumdrop, GUMDROP_SIZE
# across): small, flat and nearly black, so it reads as a stain, not a spice.
KILL_MARK = 1.2


def killmap(beat, d, ctx):
    """A dark mark where each Flump was killed, appearing as it dies and
    staying: where the fighting has been."""
    spots, ticks = [], []
    for k in range(1, d.ticks + 1):
        before = d.frames[k - 1].agents
        for kill in d.frames[k].kills:
            v = before.get(kill.victim)
            if v is None:
                continue
            x, y = animate.cell_center(v.x, v.y, d.width, d.height)
            spots.append((x, y, animate.cell_height(ctx.corners, v.x, v.y, d.width) + 0.08))
            ticks.append(k)
    mesh = bpy.data.meshes.new("killmap")
    mesh.from_pydata(spots, [], [])
    mesh.attributes.new("level", "FLOAT", "POINT")
    obj = bpy.data.objects.new("killmap", mesh)
    bpy.context.scene.collection.objects.link(obj)
    proto_mesh = bpy.data.meshes.new("kill-mark")
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=6, radius=0.5)
    bm.to_mesh(proto_mesh)
    bm.free()
    proto_mesh.materials.append(materials.matte("kill-mark", (0.1, 0.01, 0.01)))
    proto = bpy.data.objects.new("kill-mark", proto_mesh)
    proto.scale = (1, 1, 0.08)
    bpy.context.scene.collection.objects.link(proto)
    proto.hide_render = proto.hide_viewport = True
    obj.modifiers.new("marks", "NODES").node_group = board.instancer_tree(proto)

    def update(frame):
        now = ctx.timing.tick_at(frame)
        mesh.attributes["level"].data.foreach_set("value", [KILL_MARK if t <= now else 0.0 for t in ticks])
        mesh.update()

    return update


# Loan lines take the book's colors: lenders green, Flumps that both lend and
# borrow yellow, and deeper levels toward red; a line takes its lender's.
LEVEL_COLORS = ((0.2, 0.85, 0.3), (0.95, 0.85, 0.2), (1.0, 0.55, 0.1), (1.0, 0.25, 0.1), (0.85, 0.05, 0.1))
LOAN_REACH = 1.5  # neighbors only across the wrapped edge get no line
LOAN_FLIGHT = 20  # frames a loan's or a repayment's sugar takes to fly


def _loan_key(loan):
    return (loan.lender, loan.borrower, loan.due_tick)


def loanlines(beat, d, ctx):
    """An arc from each lender to each of its borrowers for the loans
    outstanding at the tick shown, in its lender's level's color. With
    params flows=True, a gumdrop also flies from lender to borrower as a loan
    is made and back, larger, as it is repaid (the loan gone at its due tick
    with both alive). params reach: the longest line drawn (default
    LOAN_REACH, neighbors only; a close-up can follow a pair apart)."""
    reach = beat.params.get("reach", LOAN_REACH)
    first, last = int(ctx.timing.tick_at(1)), min(int(ctx.timing.tick_at(beat.frames + 1)) + 1, d.ticks)
    most = max((len(d.frames[k].loans) for k in range(first, last + 1)), default=0)
    cu = bpy.data.curves.new("loanlines", "CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = beat.params.get("line_width", 0.05)
    for c in LEVEL_COLORS:
        cu.materials.append(materials.fading(f"loan-{c}", c, 2.0))
    splines = []
    for _ in range(max(most, 1)):
        s = cu.splines.new("POLY")
        s.points.add(2)
        splines.append(s)
    obj = bpy.data.objects.new("loanlines", cu)
    bpy.context.scene.collection.objects.link(obj)
    flows = []
    if beat.params.get("flows"):
        gold = materials.gumdrop()
        for k in range(max(first, 1), last + 1):
            before = {_loan_key(l): l for l in d.frames[k - 1].loans}
            now = {_loan_key(l): l for l in d.frames[k].loans}
            alive = d.frames[k].agents
            for key, l in now.items():
                if key not in before and l.lender in ctx.tracks and l.borrower in ctx.tracks:
                    flows.append((ctx.timing.frame(k), l.lender, l.borrower, ball(f"lend-{k}-{key}", 0.16, gold)))
            for key, l in before.items():
                if key not in now and l.due_tick == k - 1 and l.lender in alive and l.borrower in alive:
                    flows.append((ctx.timing.frame(k), l.borrower, l.lender, ball(f"repay-{k}-{key}", 0.24, gold)))

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        loans = d.frames[k].loans
        level = credit.levels(loans)
        used = 0
        for l in loans:
            if l.lender not in ctx.tracks or l.borrower not in ctx.tracks:
                continue
            a, b = flump_pose(ctx, d, l.lender, frame), flump_pose(ctx, d, l.borrower, frame)
            if not (a.visible and b.visible) or math.hypot(a.x - b.x, a.y - b.y) > reach or used >= len(splines):
                continue
            s = splines[used]
            mid = ((a.x + b.x) / 2, (a.y + b.y) / 2, max(a.z, b.z) + 1.3)
            s.points.foreach_set("co", [a.x, a.y, a.z + 0.7, 1, *mid, 1, b.x, b.y, b.z + 0.7, 1])
            s.material_index = min(level.get(l.lender, 1), len(LEVEL_COLORS)) - 1
            used += 1
        for s in splines[used:]:
            s.points.foreach_set("co", [0, 0, -20, 1] * 3)
        for start, src, dst, ball_obj in flows:
            u = (frame - start) / LOAN_FLIGHT
            if not 0 <= u <= 1:
                flump.stow(ball_obj)
                continue
            p, q = flump_pose(ctx, d, src, frame), flump_pose(ctx, d, dst, frame)
            e = animate.smoothstep(u)
            ball_obj.location = (p.x + (q.x - p.x) * e, p.y + (q.y - p.y) * e,
                                 p.z + 0.8 + (q.z - p.z) * e + 1.2 * math.sin(math.pi * u))
            ball_obj.scale = (1, 1, 1)

    return update


INFECTION_REACH = 1.5  # neighbors only across the wrapped edge get no line
INFECTION_FLIGHT = 0.8  # the share of a tick a new infection's line shows


def infections(beat, d, ctx):
    """The book's transmission network (Animation V-3): as each infection
    happens, a red line arcs from the Flump who passed the disease on to the
    one who caught it, showing early in the tick and fading."""
    first, last = int(ctx.timing.tick_at(1)), min(int(ctx.timing.tick_at(beat.frames + 1)) + 1, d.ticks)
    most = max((len(d.frames[k].infections) for k in range(first, last + 1)), default=0)
    cu = bpy.data.curves.new("infections", "CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = beat.params.get("line_width", 0.06)
    cu.materials.append(materials.fading("infection", (0.95, 0.12, 0.08), 2.5))
    splines = []
    for _ in range(max(most, 1)):
        s = cu.splines.new("POLY")
        s.points.add(2)
        splines.append(s)
    obj = bpy.data.objects.new("infections", cu)
    bpy.context.scene.collection.objects.link(obj)

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        k = min(int(math.floor(tick)), d.ticks)
        u = (tick - k) / INFECTION_FLIGHT
        used = 0
        if 0 <= u <= 1:
            for i in d.frames[k].infections:
                if i.infector is None or i.infector not in ctx.tracks or i.infected not in ctx.tracks:
                    continue
                a, b = flump_pose(ctx, d, i.infector, frame), flump_pose(ctx, d, i.infected, frame)
                if not (a.visible and b.visible) or math.hypot(a.x - b.x, a.y - b.y) > INFECTION_REACH:
                    continue
                if used >= len(splines):
                    break
                mid = ((a.x + b.x) / 2, (a.y + b.y) / 2, max(a.z, b.z) + 1.1)
                # Drawn from the infector outward as the tick begins.
                e = min(u * 2, 1.0)
                end = (a.x + (b.x - a.x) * e, a.y + (b.y - a.y) * e, a.z + 0.7 + (b.z - a.z) * e)
                splines[used].points.foreach_set("co", [a.x, a.y, a.z + 0.7, 1, *mid, 1, *end, 1])
                used += 1
        for s in splines[used:]:
            s.points.foreach_set("co", [0, 0, -20, 1] * 3)

    return update
