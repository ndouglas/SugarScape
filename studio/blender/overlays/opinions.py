"""Hegselmann & Krause's own picture beside the felt: opinion (up) against
period (across), a line per agent for a sample of them, colored by where
each started, drawn as far as the period shown."""

import dump
from blender import materials

from .panels import _polyline
from .parts import CREAM, box, card, text

SAMPLE = 120


def diagram(beat, d, ctx):
    """Top right: the opinion × time diagram. params: `periods` (the
    diagram's width in frames; default the shot's)."""
    anchor = ctx.screen.anchor("diagram", 0.72, 0.42)
    k = 0.7  # the panel's size on screen
    card("diagram-card", anchor, (0, 0, -0.01), (0.66 * k, 0.7 * k, 0.002))
    ink = materials.fading("diagram-ink", CREAM, 1.6)
    size_x, size_y, left, bottom = 0.56 * k, 0.52 * k, -0.28 * k, -0.29 * k
    text("diagram-y", "opinion", 0.032, ink, anchor, location=(left, bottom + size_y + 0.03, 0), align="LEFT")
    text("diagram-x", "period →", 0.032, ink, anchor, location=(left + size_x, bottom - 0.04, 0), align="RIGHT")
    box("diagram-axis-x", ink, anchor, location=(left + size_x / 2, bottom, 0.001), scale=(size_x, 0.003, 0.002))
    box("diagram-axis-y", ink, anchor, location=(left, bottom + size_y / 2, 0.001), scale=(0.003, size_y, 0.002))
    periods = beat.params.get("periods", d.ticks)
    inks = [materials.fading(f"diagram-{c}", materials.YARN[c], 2.5) for c in materials.START_YARN]
    n = len(d.placed)
    # Every (n / SAMPLE)th agent by starting rank: an even spread of starts.
    ranked = sorted(d.placed, key=d.start_rank.get)
    chosen = ranked[:: max(n // SAMPLE, 1)]
    lines = []
    for i in chosen:
        mat = inks[d.frames[0].groups[i] * len(inks) // dump.START_BINS]
        _, spline = _polyline(f"diagram-{i}", anchor, [(0, 0)] * (d.ticks + 1), mat)
        lines.append((i, spline))

    def at(k, i):
        x = left + size_x * min(k, periods) / max(periods, 1)
        return x, bottom + size_y * (d.frames[k].agents[i].x + 0.5) / d.width

    def update(frame):
        now = min(max(int(round(ctx.timing.tick_at(frame))), 0), d.ticks)
        for i, spline in lines:
            pts = []
            for k in range(d.ticks + 1):
                x, y = at(min(k, now), i)
                pts += [x, y, 0.002, 1]
            spline.points.foreach_set("co", pts)

    return update


def agreement_diagram(beat, d, ctx):
    """Actual opinion × true period, each segment colored by its uncertainty."""
    from agreement_visual import diagram_point, uncertainty_color, outcome_clock
    anchor = ctx.screen.anchor('agreement-diagram', .68, .38)
    k = .7
    card('agreement-diagram-card', anchor, (0, 0, -.01), (.66 * k, .78 * k, .002))
    ink = materials.fading('agreement-diagram-ink', CREAM, 1.6)
    left, bottom, sx, sy = -.196, -.16, .392, .364
    text('agreement-diagram-title', 'opinion × period', .031, ink, anchor, location=(0, .245, 0))
    text('agreement-range', '+1', .025, ink, anchor, location=(left - .025, bottom + sy, 0))
    text('agreement-range-low', '−1', .025, ink, anchor, location=(left - .025, bottom, 0))
    clock = text('agreement-clock', '', .029, ink, anchor, location=(0, -.205, 0))
    # Fixed 0–2 scale, independent of the run and its current frame.
    palette = [materials.fading(f'agreement-u-{j}', uncertainty_color(j / 10), 2.5) for j in range(21)]
    for j in range(21):
        box(f'agreement-legend-{j}', palette[j], anchor, location=(-.19 + j * .019, -.255, .003), scale=(.02, .013, .002))
    text('agreement-u-label', 'current uncertainty: 0      1      2', .025, ink, anchor, location=(0, -.28, .005))
    chosen = sorted(d.placed, key=d.start_rank.get)[::max((len(d.placed) + 59) // 60, 1)]
    text('agreement-sampling', f'{len(chosen)} agents · sampled periods', .022, ink, anchor, location=(0, -.31, .005))
    lines = []
    end_period = d.frames[-1].period
    import bpy
    samples = sorted(set(range(0, len(d.frames), max((d.ticks + 119) // 120, 1))) | {d.ticks})
    for i in chosen:
        curve = bpy.data.curves.new(f'agreement-path-{i}', 'CURVE')
        curve.dimensions = '3D'
        curve.bevel_depth = .0015
        for material in palette:
            curve.materials.append(material)
        obj = bpy.data.objects.new(f'agreement-path-{i}', curve)
        obj.parent = anchor
        bpy.context.scene.collection.objects.link(obj)
        for previous, j in zip(samples, samples[1:]):
            f, prev = d.frames[j], d.frames[previous]
            a, b = diagram_point(prev.period, prev.opinions[i], end_period), diagram_point(f.period, f.opinions[i], end_period)
            spline = curve.splines.new('POLY')
            spline.points.add(1)
            spline.material_index = min(max(round(f.uncertainties[i] * 10), 0), 20)
            coords = [(left + sx * a[0], bottom + sy * a[1]), (left + sx * b[0], bottom + sy * b[1])]
            spline.points.foreach_set('co', [v for x, y in coords for v in (x, y, .002, 1)])
            spline.points.foreach_set('radius', [0, 0])
            lines.append((j, spline, False))
    def update(frame):
        now = min(max(round(ctx.timing.tick_at(frame)), 0), d.ticks)
        for index, (j, spline, shown) in enumerate(lines):
            visible = j <= now
            if visible != shown:
                spline.points.foreach_set('radius', [1 if visible else 0] * 2)
                lines[index] = (j, spline, visible)
        clock.data.body = (outcome_clock(d.stats, d.config.get('stop_at', end_period), d.config.get('stop_when_stable', True)) if now == d.ticks
                           else f'period {d.frames[now].period:,} / {end_period:,}')
    return update
