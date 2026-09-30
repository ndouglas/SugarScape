"""Following the Crowd, episode 1: "Neighbors like me" (Schelling 1969, 1971).

Schelling's own models: his line (a row of 70 with no gaps) and his
checkerboard (13 × 16 with edges, 138 chips and 70 blanks). A Flump per agent,
Red (his stars) or Blue (his zeros) (colors="strategy"); a warm ring at the
feet of the discontented. Board coordinates: one unit per square, square
(x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).

What each shot does, each seed typical of 20 (see measurements.md):
line — s71-line, seed 1: 23 of 70 discontent; 20 move in round 1, 4 in
  round 2, 1 in round 3; still from round 4, in 7 groups.
board — s71-board, seed 4: 39 % discontent; 42 move in round 1, 13, 2, 1;
  still from round 5, 0.79 alike.
third — s71-third, seed 14: 18 move, then 2; still from round 3, 0.61 alike.
company — s71-congregate, seed 15: 56 move, then 4; 0.79 alike.
mixed — s71-integrate, seed 6: 38, 39, 16, 2 move; still from round 5 with
  9 % unsatisfied.
"""

from camera import Move
from episode import Beat

BOARD_EYE, BOARD_AT = (0, -16, 16.5), (0, -1.8, 0)
LOW_EYE, LOW_AT = (0, -10, 7), (0, 1.5, 0)
LINE_EYE, LINE_AT = (-24, -27, 11), (-2, 0, 0)
LINE_NEAR_EYE, LINE_NEAR_AT = (-44, -7, 4.5), (-24, 0, 0)
SCHELLING = {"colors": "strategy"}
UNHAPPY = ("rings-unhappy",)
ALIKE = {**SCHELLING, "stat": "segregation", "label": "of neighbors alike"}
LINE_ALIKE = {**SCHELLING, "stat": "like_share", "label": "of neighbors alike"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("hand", "Thomas Schelling worked it out by hand:\na row of stars and zeros, then a checkerboard.", 6.0,
         shot="line", ticks_per_second=0.01, caption_y=0.0, title=True, params=SCHELLING,
         camera=(Move(0, 6, LINE_NEAR_EYE, LINE_NEAR_AT, LINE_EYE, LINE_AT, lens0=30, lens1=28),)),
    Beat("line", "70 Flumps in a row. Each wants at least half\nof its eight nearest neighbors like itself.", 6.5,
         shot="line", ticks_per_second=0.01, overlays=UNHAPPY, params=SCHELLING,
         camera=hold(LINE_EYE, LINE_AT, lens=28, drift=(0, 1, -0.5))),
    Beat("squeeze", "The unhappy, from left to right, squeeze in\nat the nearest spot that suits them.", 7.0,
         shot="line", ticks_per_second=0.2, overlays=UNHAPPY, params=SCHELLING,
         camera=hold(LINE_NEAR_EYE, LINE_NEAR_AT, lens=30, drift=(8, 1, -0.5))),
    Beat("clusters", "A few rounds later: about seven clusters of ten.\nNobody asked for more than half.", 6.5,
         shot="line", ticks_per_second=0.6, start_tick=1, overlays=UNHAPPY + ("rate",), params=LINE_ALIKE,
         camera=hold(LINE_EYE, LINE_AT, lens=28, drift=(0, 1, -0.5))),
    Beat("board", "Then a checkerboard: 138 Flumps and 70 empty squares.", 5.0, shot="board", ticks_per_second=0.01,
         params=SCHELLING, camera=(Move(0, 5, LOW_EYE, LOW_AT, BOARD_EYE, BOARD_AT, lens0=32, lens1=32),)),
    Beat("nearest", "Each wants no fewer than half its neighbors alike,\nand moves to the nearest square that suits.", 7.0,
         shot="board", ticks_per_second=0.2, overlays=UNHAPPY, params=SCHELLING,
         camera=hold(BOARD_EYE, BOARD_AT, lens=32, drift=(0, 0.6, -0.4))),
    Beat("sorted", "Four in five neighbors end up alike.\nNearly two in five Flumps see no one of the other color.", 7.0,
         shot="board", ticks_per_second=0.7, start_tick=1, overlays=UNHAPPY + ("rate",), params=ALIKE,
         camera=hold(BOARD_EYE, BOARD_AT, lens=32, drift=(0, 0.6, -0.4))),
    Beat("hand-boards", "Schelling worked his boards by hand, and said they were\ntoo few to generalize. "
         "His came out a little more sorted.", 8.5, shot="board", start_tick=6, overlays=("bars",),
         params={**SCHELLING, "title": "neighbors alike", "format": "pct", "y": 0.12,
                 "rows": [[("his Fig. 8", "fig8_like"), ("here, the median of 20", "like")],
                          [("his Fig. 9", "fig9_like"), ("here, the median of 20", "like")]]},
         camera=hold(BOARD_EYE, BOARD_AT, lens=32)),
    Beat("third", "Ask for only a third, and the sorting is slight.", 6.0, shot="third", ticks_per_second=0.6,
         overlays=UNHAPPY + ("rate",), params=ALIKE, camera=hold(BOARD_EYE, BOARD_AT, lens=32, drift=(0, 0.6, -0.4))),
    Beat("company", "Ask only for company (three of your own)\nand the town sorts anyway.", 6.5, shot="company",
         ticks_per_second=0.6, overlays=UNHAPPY + ("rate",), params=ALIKE,
         camera=hold(BOARD_EYE, BOARD_AT, lens=32, drift=(0, 0.6, -0.4))),
    Beat("mixed", "Even Flumps who want a mixed street move more,\nand some are never satisfied.", 7.0, shot="mixed",
         ticks_per_second=0.8, overlays=UNHAPPY, params=SCHELLING,
         camera=hold(BOARD_EYE, BOARD_AT, lens=32, drift=(0, 0.6, -0.4))),
    Beat("point", "Nobody wanted a divided town.\nThey just didn't want to be outnumbered.", 6.0, shot="board", start_tick=6,
         caption_y=0.0, title=True, params=SCHELLING,
         camera=(Move(0, 6, BOARD_EYE, BOARD_AT, (0, -21, 21), BOARD_AT, lens0=32, lens1=32, orbit=0.1),)),
    Beat("end", "Neighbors like me — after Schelling, 1969, 1971\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
