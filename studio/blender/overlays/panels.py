"""Overlays on the screen: gauges, charts and cards hung from `Screen`
anchors, reading the frame shown, a compare world or the measurements."""

import animate
import seasons
import tribes
from blender import materials

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
