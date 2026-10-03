"""How extremists win: the approved fourteen-beat relative-agreement film."""
from camera import Move
from episode import Beat

S = {'colors': 'start', 'motion_blur': False}
A = ('agreement-markers', 'agreement-diagram')

DEPTH = {'crowd': 65, 'middle': 20, 'both': 35, 'single': 65, 'early': 64, 'reply': 65,
         'size-small': 63, 'size-big': 598, 'lean': 317, 'lean-balanced': 297, 'printed': 334, 'neighbors': 158}
KEPT_TICKS = {'crowd': 63, 'middle': 135, 'both': 167, 'single': 63, 'early': 200, 'reply': 240,
              'size-small': 176, 'size-big': 120, 'lean': 220, 'lean-balanced': 72, 'printed': 46, 'neighbors': 791}


def hold(shot, seconds=7):
    front = -DEPTH[shot] / 2
    eye, target = (16, front - 65, 70), (16, front + min(DEPTH[shot] / 2, 30), 0)
    end = (16, front - 62, 69)
    return (Move(0, seconds, eye, target, end, target, lens0=26, lens1=26),)


def beat(name, caption, shot, seconds=7, speed=None, extra=(), compare=None, **kwargs):
    speed = KEPT_TICKS[shot] / (.75 * seconds) if speed is None else speed
    moves = hold(shot, seconds)
    if name in ('crowd', 'uncertainty', 'rule'):
        front = -DEPTH[shot] / 2
        eye, target = (-50, front - 22, 16), (-50, front + 6, 0)
        moves = (Move(0, seconds, eye, target, (-49, front - 23, 17), target, lens0=32, lens1=32),)
    return Beat(name, caption, seconds, shot=shot, ticks_per_second=speed, overlays=A + extra,
                camera=moves, params=S, compare=compare, **kwargs)

BEATS = [
    beat('crowd', '200 Flumps, each with an opinion.\nA few at the ends are very sure of themselves.', 'crowd', speed=.01),
    beat('uncertainty', 'Each has an uncertainty range around its opinion.\nA short range means more confidence.', 'crowd', speed=.01, extra=('agreement-pair',)),
    beat('rule', "Random pairs meet. Enough overlap lets one change\nthe other's opinion and uncertainty.", 'crowd', extra=('agreement-pair',)),
    beat('middle', 'A more certain crowd keeps a middle\nin 11 of our 20 runs.', 'middle'),
    beat('both', 'A less certain crowd can split between both extremes.', 'both'),
    beat('single', 'Even with equal extremists at both ends,\na few confident Flumps can pull nearly everyone to one side.', 'single', seconds=8),
    beat('early', "Stop early, and a crowd still drifting\nhasn't yet crossed the counting line.", 'early', extra=('agreement-cutoff',)),
    beat('reply', "The authors' reply waited for opinions to settle,\nand after seeing the runs, moved the counting line inward.", 'reply', seconds=8, extra=('agreement-cutoff',)),
    beat('readings', 'Together, those changes recover a frequent single extreme.', 'reply', extra=('agreement-results',)),
    beat('size', 'At these settings, a bigger crowd\nreaches one extreme much less often.', 'size-small', extra=('agreement-results', 'agreement-comparison'), compare='size-big'),
    beat('lean', 'A small imbalance makes one side more likely to win.', 'lean', extra=('agreement-results', 'agreement-comparison'), compare='lean-balanced'),
    beat('printed', "With their alternative's printed rule,\nthe moderates pull the extremists toward the middle.", 'printed'),
    beat('neighbors', 'Hear only neighbors on this lattice,\nand neither extreme takes over the crowd.', 'neighbors', seconds=8, extra=('agreement-horizon',)),
    Beat('end', 'How extremists win - After Deffuant et al., 2002\nndouglas.github.io/SugarScape', 7,
         caption_y=.45, params=S,
         camera=(Move(0, 7, (0, -5.5, 1.8), (0, 0, .6), (0, -5.2, 1.7), (0, 0, .6), lens0=50, lens1=53),)),
]
