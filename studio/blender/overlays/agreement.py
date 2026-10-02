"""Measured uncertainty, example intervals, and protocol comparisons."""
import animate
from agreement_visual import interval, overlap, marker_length, select_pair
from blender import materials
from .parts import CREAM, box, card, text, flump_pose


def _frame(d, ctx, frame):
    return d.frames[min(max(round(ctx.timing.tick_at(frame)), 0), d.ticks)]


def agreement_markers(beat, d, ctx):
    cream = materials.knit('cream')
    objects = {i: box(f'uncertainty-{i}', cream, None, scale=(.1, .09, .09)) for i in ctx.tracks}
    # Actual histogram opinion bins span the physical width, not a decorative scale.
    front = -d.height / 2
    for opinion in (-1, -.5, 0, .5, 1):
        x = opinion * (d.width - 1) / 2
        box(f'opinion-tick-{opinion}', cream, None, location=(x, front - .7, .04), scale=(.12, 1, .04))
        label = text(f'opinion-ruler-{opinion}', f'{opinion:+g}' if opinion else '0', 2.4, cream, None,
                     location=(x, front - 3, .05))
    box('opinion-ruler-line', cream, None, location=(0, front - .7, .04), scale=(d.width - 1, .08, .04))
    anchor = ctx.screen.anchor('uncertainty-legend', -.55, .77)
    card('uncertainty-legend-card', anchor, (0, 0, -.01), (.8, .17, .002))
    text('uncertainty-legend', 'Cream length = current uncertainty\nBody yarn = starting opinion', .036,
         materials.fading('uncertainty-ink', CREAM, 1.6), anchor)
    def update(frame):
        f = _frame(d, ctx, frame)
        for i, obj in objects.items():
            p = flump_pose(ctx, d, i, frame)
            obj.hide_render = not p.visible
            obj.location = (p.x, p.y, p.z + 1.9)
            obj.scale.x = marker_length(f.uncertainties[i])
    return update


def agreement_pair(beat, d, ctx):
    f = d.frames[0]
    extreme, other = select_pair(f.opinions, f.uncertainties)
    anchor = ctx.screen.anchor('agreement-pair', -.42, .12)
    card('pair-card', anchor, (0, 0, -.01), (1.08, .49, .002))
    ink = materials.fading('pair-ink', CREAM, 1.8)
    text('pair-title', 'Example pair · period 0', .04, ink, anchor, location=(0, .18, 0))
    for row, i in enumerate((extreme, other)):
        a, b = interval(f.opinions[i], f.uncertainties[i])
        y = .06 - row * .12
        # Full intervals are untruncated: this ruler extends beyond ±1.
        box(f'pair-interval-{i}', ink, anchor, location=((a + b) / 8, y, .002), scale=((b - a) / 4, .018, .002))
        box(f'pair-opinion-{i}', ink, anchor, location=(f.opinions[i] / 4, y, .004), scale=(.008, .055, .002))
        text(f'pair-label-{i}', f'x {f.opinions[i]:+.2f} · uncertainty {f.uncertainties[i]:.2f}', .033, ink, anchor, location=(0, y - .035, .005))
    for value in (-2, 0, 2):
        text(f'pair-axis-{value}', f'{value:+g}' if value else '0', .025, ink, anchor, location=(value / 4, -.13, .005))
    h = overlap(f.opinions[extreme], f.uncertainties[extreme], f.opinions[other], f.uncertainties[other])
    text('pair-overlap', f'Overlap {h:.2f} · influence only if overlap > influencer uncertainty', .027, ink, anchor, location=(0, -.18, .005))
    return lambda frame: None


def agreement_cutoff(beat, d, ctx):
    anchor = ctx.screen.anchor('counting-lines', .68, .38)
    ink = materials.fading('cutoff-ink', CREAM, 2)
    lines = [box(f'cutoff-{s}', ink, anchor, scale=(.38, .003, .002)) for s in (-1, 1)]
    label = text('cutoff-label', '', .033, ink, anchor, location=(0, -.37, .005))
    def update(frame):
        fraction = (frame - 1) / max(beat.frames - 1, 1)
        value = .8 - .1 * min(max((fraction - .3) / .4, 0), 1) if beat.name == 'reply' else .8
        for obj, sign in zip(lines, (-1, 1)):
            obj.location = (0, -.16 + .364 * (sign * value + 1) / 2, .008)
        label.data.body = f'Counting past ±{value:.2f}' + ("\nReply's later choice" if beat.name == 'reply' and value < .8 else '')
    return update


def agreement_results(beat, d, ctx):
    anchor = ctx.screen.anchor('agreement-results', -.46, .28 if beat.compare else .15)
    card('results-card', anchor, (0, -.03 if beat.compare else 0, -.01), (1.0, .75 if beat.compare else .65, .002))
    ink = materials.fading('results-ink', CREAM, 1.8)
    label = text('results-text', '', .039, ink, anchor, location=(0, .12 if beat.compare else 0, .005))
    data = ctx.measured.get('agreement', ctx.measured)
    protocols = data.get('protocols', {})
    keys = {'readings': ('early', 'horizon-only', 'cutoff-only', 'reply'), 'size': ('size-small', 'size-big'), 'lean': ('lean-balanced', 'lean')}[beat.name]
    headings = {'early': '200 periods · ±0.8', 'horizon-only': '1,200 periods · ±0.8', 'cutoff-only': '200 periods · ±0.7', 'reply': '1,200 periods · ±0.7', 'size-small': '200 Flumps', 'size-big': '2,000 Flumps', 'lean-balanced': 'Equal initial sides', 'lean': 'Initial imbalance 0.1'}
    lines = []
    for key in keys:
        row = protocols.get(key)
        if row is None:
            lines.append(headings[key] + ': awaiting measurement')
        else:
            lines.append(f"{headings[key]}: {row['single']}/{row['seeds']} single\nmean y {row['mean_y']:.2f} · |mean opinion| {row['mean_drift']:.2f}")
    label.data.body = '\n\n'.join(lines)
    return lambda frame: None


def agreement_comparison(beat, d, ctx):
    """Two actual final distributions, each labeled with its own shot clock."""
    anchor = ctx.screen.anchor('agreement-comparison', -.46, .28)
    ink = materials.fading('comparison-ink', CREAM, 1.8)
    headings = {'size-small': '200 Flumps', 'size-big': '2,000 Flumps',
                'lean-balanced': 'Equal initial sides', 'lean': 'Initial imbalance 0.1'}
    for row_index, (name, source) in enumerate(((beat.shot, d), (beat.compare, ctx.compare))):
        final = source.frames[-1]
        y = -.18 - row_index * .12
        text(f'comparison-title-{row_index}', headings[name] + f' · period {final.period:,}', .028, ink, anchor, location=(0, y + .035, .005))
        for i, opinion in final.opinions.items():
            x = opinion * .43
            box(f'comparison-point-{row_index}-{i}', ink, anchor, location=(x, y, .006), scale=(.004, .045, .002))
        text(f'comparison-axis-{row_index}', '−1                   opinion                   +1', .025, ink, anchor, location=(0, y - .035, .005))
    return lambda frame: None


def agreement_horizon(beat, d, ctx):
    data = ctx.measured.get('agreement', ctx.measured)
    row = data['protocols']['neighbors']
    lattice = d.config['lattice']
    horizon = d.config['stop_at']
    anchor = ctx.screen.anchor('agreement-horizon', -.55, .50)
    card('horizon-card', anchor, (0, 0, -.01), (.82, .18, .002))
    ink = materials.fading('horizon-ink', CREAM, 1.8)
    body = (f"{lattice['width']} × {lattice['height']} Moore lattice · {row['seeds']} runs\n"
            f"{row['single']} single by {horizon:,} periods · {row['unsettled']} unsettled")
    text('horizon-note', body, .035, ink, anchor)
    return lambda frame: None
