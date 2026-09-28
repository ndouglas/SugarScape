"""Overlays on the screen: gauges, charts and cards hung from `Screen`
anchors, reading the frame shown, a compare world or the measurements."""

import math

import bpy

import animate
import credit
import markets
import seasons
import tribes
from blender import flump, materials

from .parts import CREAM, SUMMER, WINTER, box, card, gauge, text


def dials(beat, d, ctx):
    """Two gauges at the top right: the population's mean sight and hunger."""
    rows = [("mean_vision", "sight", 6.0, "teal", 0.78), ("mean_metabolism", "hunger", 4.0, "coral", 0.56)]
    anchor = ctx.screen.anchor("dials-card", 0.66, 0.7)
    card("dials-card-box", anchor, (0, 0, -0.01), (0.6, 0.3, 0.002))
    parts = []
    for key, title, top, color, y in rows:
        fill, label = gauge(ctx.screen, title, color, 0.66, y)
        parts.append((d.stats[key], top, title, fill, label))

    def update(frame):
        i = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        for series, top, title, fill, label in parts:
            v = series[i]
            fill.scale.x = max(min(v / top, 1.0), 0.002) * 0.46
            fill.location.x = -0.23 + fill.scale.x / 2
            label.data.body = f"average {title}: {v:.2f}   (was {series[0]:.2f})"

    return update


def histogram(beat, d, ctx):
    """How many Flumps hold how much sugar, as felt bars at the right."""
    bins = 12
    last = sorted(a.sugar for a in d.frames[-1].agents.values())
    top = last[int(0.99 * (len(last) - 1))] if last else 1.0
    anchor = ctx.screen.anchor("histogram", 0.62, -0.55)
    card("histogram-card", anchor, (0, 0.17, -0.01), (0.6, 0.5, 0.002))
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
    card("wealth-card", anchor, (0, 0.01, -0.01), (0.6, 0.4, 0.002))
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


def season_card(beat, d, ctx):
    """Top left: which hemisphere has summer and which winter, from the
    engine's schedule for the tick shown."""
    period = d.config["seasons"]["period"]
    anchor = ctx.screen.anchor("season-card", -0.72, 0.78)
    card("season-card-box", anchor, (0, 0, -0.01), (0.5, 0.24, 0.002))
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


def counter(beat, d, ctx):
    """Top right: this world's population against the compare world's (the
    same seed without the rule this episode is about) at the tick shown.
    Labels from params: `with`, `without`."""
    anchor = ctx.screen.anchor("counter", 0.66, 0.78)
    card("counter-card", anchor, (0, 0, -0.01), (0.62, 0.24, 0.002))
    cold = materials.fading("counter-seasons", WINTER, 1.6)
    warm = materials.fading("counter-calm", SUMMER, 1.6)
    with_ = text("counter-with", "", 0.052, cold, anchor, location=(-0.28, 0.035, 0), align="LEFT")
    without = text("counter-without", "", 0.052, warm, anchor, location=(-0.28, -0.065, 0), align="LEFT")
    labels = (beat.params.get("with", "with seasons"), beat.params.get("without", "without seasons"))

    def update(frame):
        k = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        with_.data.body = f"{labels[0]}: {len(d.frames[k].agents)} Flumps"
        without.data.body = f"{labels[1]}: {len(ctx.compare.frames[k].agents)} Flumps"

    return update


def hills(beat, d, ctx):
    """Top right: the share of Flumps living on hill sites (capacity ≥ 3) at
    the tick shown, against the compare world's."""
    anchor = ctx.screen.anchor("hills", 0.66, 0.78)
    card("hills-card", anchor, (0, 0, -0.01), (0.62, 0.24, 0.002))
    here = text("hills-here", "", 0.052, materials.fading("hills-here", CREAM, 1.6), anchor,
                location=(-0.28, 0.035, 0), align="LEFT")
    there = text("hills-there", "", 0.052, materials.fading("hills-there", SUMMER, 1.6), anchor,
                 location=(-0.28, -0.065, 0), align="LEFT")
    label = beat.params.get("without", "without pollution")

    def share(dd, k):
        agents = dd.frames[k].agents.values()
        return sum(dd.capacity[a.y * dd.width + a.x] >= 3 for a in agents) / max(len(agents), 1)

    def update(frame):
        k = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        here.data.body = f"on the hills: {share(d, k):.0%}"
        there.data.body = f"{label}: {share(ctx.compare, k):.0%}"

    return update


SURVIVAL = {
    "title": "who survives the seasons",
    "rows": [
        [("started rich", "rich_alive"), ("started poor", "poor_alive")],
        [("born on a hill", "hill_alive"), ("born on the plains", "plain_alive")],
        [("needs little", "met_low_alive"), ("needs a lot", "met_high_alive")],
    ],
}


def bars(beat, d, ctx):
    """A panel of paired bars from the medians over the measured seeds
    (measurements.json) — not one run, since captions rank the medians.
    params: `title`, `rows` (groups of (label, key) pairs, the first teal and
    the second coral), `format` ("pct" or "num"), `top` (the bar scale; for
    numbers, the value of a full bar)."""
    params = {**SURVIVAL, **beat.params}
    medians, seeds = ctx.measured["medians"], ctx.measured["seeds"]
    percent = params.get("format", "pct") == "pct"
    top = params.get("top", 1.0)
    anchor = ctx.screen.anchor("bars", 0.46, 0.08)
    groups = params["rows"]
    height = 0.28 + 0.17 * len(groups)
    card("bars-card", anchor, (0, 0.0, -0.01), (0.9, height, 0.002))
    ink = materials.fading("bars-ink", CREAM, 1.6)
    text("bars-title", params["title"], 0.055, ink, anchor, location=(0, height / 2 - 0.07, 0))
    text("bars-note", f"median over {seeds} runs", 0.035, ink, anchor, location=(0, height / 2 - 0.125, 0))
    width = 0.3
    tops = params.get("tops", [top] * len(groups))
    for i, group in enumerate(groups):
        y = height / 2 - 0.21 - i * 0.17
        for j, (label, key) in enumerate(group):
            yy = y - j * 0.062
            value = medians[key]
            color = ("teal", "coral")[j % 2]
            text(f"bars-{i}-{j}", label, 0.04, ink, anchor, location=(-0.06, yy - 0.012, 0), align="RIGHT")
            bar_w = max(min(value / tops[i], 1.0) * width, 0.002)
            box(f"bars-bar-{i}-{j}", materials.knit(color), anchor,
                location=(-0.04 + bar_w / 2, yy, 0), scale=(bar_w, 0.046, 0.004))
            shown = f"{value:.0%}" if percent else (f"{value:.3f}" if value < 1 else f"{value:.0f}")
            text(f"bars-value-{i}-{j}", shown, 0.04, ink, anchor, location=(-0.02 + bar_w, yy - 0.012, 0), align="LEFT")
    return lambda frame: None


def alike(beat, d, ctx):
    """Top right: how alike neighbors are (the share of adjacent pairs in
    the same tribe) and each tribe's share, at the tick shown."""
    anchor = ctx.screen.anchor("alike", 0.66, 0.78)
    card("alike-card", anchor, (0, 0, -0.01), (0.62, 0.24, 0.002))
    top = text("alike-top", "", 0.052, materials.fading("alike-ink", CREAM, 1.6), anchor,
               location=(-0.28, 0.035, 0), align="LEFT")
    blue = text("alike-blue", "", 0.052, materials.fading("alike-blue", materials.YARN["blue"], 3.0), anchor,
                location=(-0.28, -0.065, 0), align="LEFT")
    red = text("alike-red", "", 0.052, materials.fading("alike-red", materials.YARN["red"], 3.0), anchor,
               location=(0.02, -0.065, 0), align="LEFT")

    def update(frame):
        f = d.frames[min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)]
        top.data.body = f"neighbors alike: {tribes.neighbors_alike(f, d.width, d.height):.0%}"
        n = max(len(f.groups), 1)
        share = sum(g == 0 for g in f.groups.values()) / n
        blue.data.body = f"Blue {share:.0%}"
        red.data.body = f"Red {1 - share:.0%}"

    return update


def tally(beat, d, ctx):
    """Top right: how many helpers (group 0, blue) and cheats (group 1, red)
    are alive at the tick shown: the demographic PD's strategies."""
    anchor = ctx.screen.anchor("tally", 0.66, 0.78)
    card("tally-card", anchor, (0, 0, -0.01), (0.62, 0.17, 0.002))
    blue = text("tally-helpers", "", 0.052, materials.fading("tally-blue", materials.YARN["blue"], 3.0), anchor,
                location=(-0.28, -0.015, 0), align="LEFT")
    red = text("tally-cheats", "", 0.052, materials.fading("tally-red", materials.YARN["red"], 3.0), anchor,
               location=(0.03, -0.015, 0), align="LEFT")

    def update(frame):
        f = d.frames[min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)]
        cheats = sum(f.groups.values())
        blue.data.body = f"helpers {len(f.groups) - cheats}"
        red.data.body = f"cheats {cheats}"

    return update


def census(beat, d, ctx):
    """Top right: how many Flumps are alive at the tick shown, against how
    many there were at the start."""
    anchor = ctx.screen.anchor("census", 0.66, 0.78)
    card("census-card", anchor, (0, 0, -0.01), (0.62, 0.17, 0.002))
    line = text("census-line", "", 0.052, materials.fading("census-ink", CREAM, 1.6), anchor,
                location=(-0.28, -0.015, 0), align="LEFT")
    start = len(d.frames[0].agents)

    def update(frame):
        k = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        line.data.body = f"Flumps alive: {len(d.frames[k].agents)} of {start}"

    return update


# The price chart's log scale: from 1/2.5 to 2.5 times one-for-one, and its
# size. A tick's price swings when few pairs trade, so the chart shows a
# rolling average, and says so.
PRICE_RANGE = math.log(2.5)
PRICE_WINDOW = 20
CHART_W, CHART_H = 0.66, 0.3


def _polyline(name, parent, points, material):
    cu = bpy.data.curves.new(name, "CURVE")
    cu.dimensions = "3D"
    cu.bevel_depth = 0.004
    spline = cu.splines.new("POLY")
    spline.points.add(len(points) - 1)
    cu.materials.append(material)
    obj = bpy.data.objects.new(name, cu)
    obj.parent = parent
    bpy.context.scene.collection.objects.link(obj)
    return obj, spline


def prices(beat, d, ctx):
    """Bottom right: the price in spice per sugar (the geometric mean of the
    tick's trades, from the dump, averaged over PRICE_WINDOW ticks) on a log
    scale, drawn up to the tick shown, with a band as wide as the prices' spread and a line at one for
    one; beneath, the price and how far apart prices are."""
    price, spread = markets.smoothed(*markets.price_series(d.frames), PRICE_WINDOW)
    anchor = ctx.screen.anchor("prices", 0.56, -0.5)
    card("prices-card", anchor, (0, 0, -0.01), (0.84, 0.56, 0.002))
    ink = materials.fading("prices-ink", CREAM, 1.6)
    text("prices-title", f"price of sugar, in spice ({PRICE_WINDOW}-tick average)", 0.04, ink, anchor,
         location=(0, 0.225, 0))
    box("prices-one", materials.fading("prices-one-ink", CREAM, 0.6), anchor, location=(0, 0, -0.003),
        scale=(CHART_W, 0.004, 0.002))
    text("prices-one-label", "1 for 1", 0.03, ink, anchor, location=(-CHART_W / 2 - 0.01, -0.01, 0), align="RIGHT")
    readout = text("prices-readout", "", 0.036, ink, anchor, location=(0, -0.225, 0))

    def at(k, value):
        x = -CHART_W / 2 + CHART_W * k / max(d.ticks, 1)
        y = CHART_H / 2 * min(max(math.log(value) / PRICE_RANGE, -1), 1)
        return x, y

    first = next((k for k, v in enumerate(price) if v is not None), None)
    line, spline = _polyline("prices-line", anchor, price, materials.fading("prices-gold", (1.0, 0.55, 0.1), 2.0))
    band = bpy.data.meshes.new("prices-band")
    n = len(price)
    band.from_pydata([(0, 0, 0)] * (2 * n), [], [(2 * i, 2 * i + 1, 2 * i + 3, 2 * i + 2) for i in range(n - 1)])
    band.materials.append(materials.fading("prices-spread", (0.95, 0.45, 0.38), 0.7))
    band_obj = bpy.data.objects.new("prices-band", band)
    band_obj.parent = anchor
    bpy.context.scene.collection.objects.link(band_obj)

    def update(frame):
        now = min(int(round(ctx.timing.tick_at(frame))), d.ticks)
        if first is None or now < first:
            flump.stow(line)
            flump.stow(band_obj)
            readout.data.body = "no trades yet"
            return
        line.scale = band_obj.scale = (1, 1, 1)
        line.location.z, band_obj.location.z = 0.002, -0.001
        points, verts = [], []
        for k in range(n):
            j = min(max(k, first), now)
            x, y = at(j, price[j])
            points += [x, y, 0, 1]
            _, lo = at(j, price[j] / math.exp(spread[j]))
            _, hi = at(j, price[j] * math.exp(spread[j]))
            verts += [x, lo, 0, x, hi, 0]
        spline.points.foreach_set("co", points)
        band.vertices.foreach_set("co", verts)
        band.update()
        apart = math.exp(spread[now]) - 1
        readout.data.body = f"1 sugar = {price[now]:.2f} spice   ·   prices differ by about {apart:.0%}"

    return update


def kills(beat, d, ctx):
    """Top right: the Flumps killed so far, and each tribe's number alive, at
    the tick shown."""
    anchor = ctx.screen.anchor("kills", 0.66, 0.76)
    card("kills-card", anchor, (0, -0.02, -0.01), (0.62, 0.3, 0.002))
    ink = materials.fading("kills-ink", CREAM, 1.6)
    killed = text("kills-line", "", 0.052, ink, anchor, location=(-0.28, 0.06, 0), align="LEFT")
    blue = text("kills-blue", "", 0.048, materials.fading("kills-blue", materials.YARN["blue"], 3.0), anchor,
                location=(-0.28, -0.04, 0), align="LEFT")
    red = text("kills-red", "", 0.048, materials.fading("kills-red", materials.YARN["red"], 3.0), anchor,
               location=(0.02, -0.04, 0), align="LEFT")
    total = [0]
    for f in d.frames:
        total.append(total[-1] + len(f.kills))

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        f = d.frames[k]
        killed.data.body = f"Flumps killed: {total[k + 1]}"
        blue.data.body = f"Blue {sum(g == 0 for g in f.groups.values())}"
        red.data.body = f"Red {sum(g == 1 for g in f.groups.values())}"

    return update


LADDER_LEVELS = 10


def ladder(beat, d, ctx):
    """Right: the debt ladder — how many Flumps sit at each level of the loan
    network at the tick shown (level 1: pure lenders), bars on a square-root
    scale so one Flump at the bottom still shows beside a hundred at the top,
    and the count beside each."""
    anchor = ctx.screen.anchor("ladder", 0.66, 0.05)
    card("ladder-card", anchor, (0, 0, -0.01), (0.56, 0.92, 0.002))
    ink = materials.fading("ladder-ink", CREAM, 1.6)
    text("ladder-title", "the debt ladder", 0.045, ink, anchor, location=(0, 0.4, 0))
    text("ladder-note", "Flumps at each level; lenders at the top", 0.028, ink, anchor, location=(0, 0.35, 0))
    colors = [materials.knit("teal")] * 1 + [materials.knit("butter")] * 3 + [materials.knit("coral")] * 6
    rows = []
    for i in range(LADDER_LEVELS):
        y = 0.28 - i * 0.068
        text(f"ladder-level-{i}", f"{i + 1}", 0.032, ink, anchor, location=(-0.2, y - 0.01, 0), align="RIGHT")
        bar = box(f"ladder-bar-{i}", colors[i], anchor, location=(-0.17, y, 0), scale=(0.002, 0.045, 0.004))
        count = text(f"ladder-count-{i}", "", 0.03, ink, anchor, location=(-0.15, y - 0.01, 0), align="LEFT")
        rows.append((bar, count))
    width = 0.34

    def update(frame):
        k = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        counts = [0] * LADDER_LEVELS
        for level in credit.levels(d.frames[k].loans).values():
            counts[min(level, LADDER_LEVELS) - 1] += 1
        top = max(max(counts), 1) ** 0.5
        for (bar, count), n in zip(rows, counts):
            bar.scale.x = max(n ** 0.5 / top * width, 0.002)
            bar.location.x = -0.17 + bar.scale.x / 2
            count.location.x = -0.15 + bar.scale.x
            count.data.body = f"{n}" if n else ""

    return update


def sick(beat, d, ctx):
    """Top right: how many Flumps carry a disease at the tick shown, and what
    share of everyone that is."""
    anchor = ctx.screen.anchor("sick", 0.66, 0.78)
    card("sick-card", anchor, (0, 0, -0.01), (0.62, 0.17, 0.002))
    line = text("sick-line", "", 0.05, materials.fading("sick-ink", materials.YARN["sick"], 3.0), anchor,
                location=(-0.28, -0.015, 0), align="LEFT")

    def update(frame):
        f = d.frames[min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)]
        ill = sum(1 for n in f.diseases.values() if n)
        line.data.body = f"sick: {ill} of {len(f.agents)} ({ill / max(len(f.agents), 1):.0%})"

    return update


POP_TOP = 1200  # the chart's ceiling: past the book's doubling of 500


def popchart(beat, d, ctx):
    """Right: this run's population and the compare run's, drawn up to the
    tick shown, over the whole run's width, with a line at the book's
    doubling (1000). params: `labels` (this run's, the compare run's).
    (No "crash" line: both runs dip below any such line early and recover,
    and a viewer would read the dip as the book's crash.)"""
    here, there = beat.params.get("labels", ("this run", "the other"))
    anchor = ctx.screen.anchor("popchart", 0.52, -0.02)
    card("popchart-card", anchor, (0, 0, -0.01), (0.9, 0.8, 0.002))
    ink = materials.fading("popchart-ink", CREAM, 1.6)
    text("popchart-title", "Flumps alive", 0.045, ink, anchor, location=(0, 0.34, 0))
    width, height, left, bottom = 0.74, 0.54, -0.37, -0.28

    def at(k, pop):
        return left + width * k / max(d.ticks, 1), bottom + height * min(pop, POP_TOP) / POP_TOP

    for level, label in ((1000, "the book's doubling"),):
        _, y = at(0, level)
        box(f"popchart-ref-{level}", materials.fading(f"popchart-ref-{level}", CREAM, 0.5), anchor,
            location=(0, y, -0.004), scale=(width, 0.003, 0.002))
        text(f"popchart-ref-{level}-label", label, 0.026, ink, anchor, location=(left + width, y + 0.018, 0),
             align="RIGHT")
    series = [(d.stats["population"], "teal", here, 0), (ctx.compare.stats["population"], "coral", there, 1)]
    lines = []
    for pop, color, label, i in series:
        line, spline = _polyline(f"popchart-{i}", anchor, pop, materials.fading(f"popchart-{color}", materials.YARN[color], 2.5))
        text(f"popchart-label-{i}", label, 0.032, materials.fading(f"popchart-label-{i}", materials.YARN[color], 2.5),
             anchor, location=(left + 0.02, 0.27 - 0.045 * i, 0), align="LEFT")
        lines.append((pop, line, spline))

    def update(frame):
        now = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        for pop, line, spline in lines:
            points = []
            for k in range(len(pop)):
                x, y = at(min(k, now), pop[min(k, now)])
                points += [x, y, 0, 1]
            spline.points.foreach_set("co", points)
            line.location.z = 0.002

    return update


LEDGER_ROWS = 8


def ledger(beat, d, ctx):
    """Center: the book's claims against what we measured, a row at a time
    across the beat. params rows: (the book, here) pairs; `here` may name
    measured medians as {key} or {key:format}, filled from measurements.json.
    params heads: the two columns' headings (default the book / here)."""
    rows = beat.params["rows"][:LEDGER_ROWS]
    medians = ctx.measured.get("medians", {})
    anchor = ctx.screen.anchor("ledger", 0.0, 0.08)
    height = 0.2 + 0.1 * len(rows)
    card("ledger-card", anchor, (0, 0, -0.01), (1.6, height, 0.002))
    ink = materials.fading("ledger-ink", CREAM, 1.6)
    coral = materials.fading("ledger-book", materials.YARN["coral"], 2.5)
    teal = materials.fading("ledger-here", materials.YARN["teal"], 2.5)
    book_head, here_head = beat.params.get("heads", ("the book", "here, over 20 worlds"))
    text("ledger-book-head", book_head, 0.04, coral, anchor, location=(-0.36, height / 2 - 0.07, 0))
    text("ledger-here-head", here_head, 0.04, teal, anchor, location=(0.4, height / 2 - 0.07, 0))
    shown = []
    for i, (book, here) in enumerate(rows):
        y = height / 2 - 0.17 - 0.1 * i
        a = text(f"ledger-{i}-book", book, 0.034, ink, anchor, location=(-0.36, y, 0))
        b = text(f"ledger-{i}-here", here.format(**medians), 0.034, ink, anchor, location=(0.4, y, 0))
        shown.append((a, b))

    def update(frame):
        # Each row appears in turn over the first 80 % of the beat.
        visible = int(len(rows) * min(frame / (0.8 * beat.frames), 1.0) + 0.999)
        for i, (a, b) in enumerate(shown):
            s = 1.0 if i < visible else 1e-4
            a.scale = b.scale = (s, s, s)

    return update
