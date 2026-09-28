"""Cooperation, episode 5: "Reputation" (Nowak & Sigmund 1998, image
scoring).

The board is a street (see street.py): twelve columns, one per threshold k,
from "help everyone" (k = −5) on the left to "help no one" (k = +6) on the
right; each Flump stands in its threshold's column in the yarn of its score
(red −5, pale 0, green +5). Each tick, the next generation first takes its
places, then plays its meetings one by one.

What each shot does (see measurements.md for the 20-seed claims):
early — ns-fig-1, seed 1, 6 generations, with meetings.
paper — ns-fig-1, seed 2: the discriminators (k = 0) have taken over by
  generation 65 (as in 5 of 20 worlds).
here — ns-fig-1, seed 10: a threshold that helps no one (k = +6) has taken
  over by generation 57 (as in 13 of 20; everyone helps in the other 7).
cycles — ns-fig-2 (mutation, 300 meetings a generation), seed 12 (typical of
  20): cooperation collapses at generation 12,065, recovers at 12,765 and
  collapses again at 13,048; the beat plays 11,800–13,400.
"""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -37, 41), (0, 5, 0)
LOW_EYE, LOW_AT = (0, -22, 13), (0, 1, 0)
GIFT_RATE = {"stat": "help_rate", "label": "of meetings end in a gift"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("score", "Now every Flump carries a reputation: a score from −5 to 5.", 6.0, shot="early",
         ticks_per_second=0.25, lead_in=0.5, overlays=("meetings",),
         camera=(Move(0, 6, LOW_EYE, LOW_AT, WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("earn", "Helping raises your score. Refusing lowers it.", 6.0, shot="early", ticks_per_second=0.25,
         start_tick=1, overlays=("meetings",), camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("rule", "Each Flump has a rule: help anyone whose score is at least k.", 6.0, shot="early",
         ticks_per_second=0.3, start_tick=2, overlays=("meetings",),
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, LOW_EYE, LOW_AT, lens0=35, lens1=35),)),
    Beat("pay", "Helping costs 0.1 and gives 1.\nThe more a Flump earns, the more children it has.", 6.0,
         shot="early", ticks_per_second=0.6, start_tick=3, camera=hold(LOW_EYE, LOW_AT, lens=35, drift=(0, -1, 3))),
    Beat("paper", "The paper shows one run, where everyone ends up helping those who help.", 7.0, shot="paper",
         ticks_per_second=12, overlays=("rate",), params=GIFT_RATE,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("here", "Here, everyone ends up helping in 7 of 20 worlds.\nIn 13, nobody helps anyone.", 7.5,
         shot="here", ticks_per_second=10, overlays=("rate",), params=GIFT_RATE,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("cycles", "Let rules mutate, and helping rises and collapses, again and again.", 7.5, shot="cycles",
         ticks_per_second=215, start_tick=11800, overlays=("rate",), params=GIFT_RATE,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("crowd", "And it works best in small groups:\nthe bigger the group, the less anyone helps.", 8.0, shot="paper",
         start_tick=80, overlays=("bars",),
         params={"title": "Flumps with a cooperative rule (k ≤ 0)", "format": "pct",
                 "rows": [[("20, the paper", "paper_n20"), ("20, here", "n20")],
                          [("50, the paper", "paper_n50"), ("50, here", "n50")],
                          [("100, the paper", "paper_n100"), ("100, here", "n100")]]},
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "A good name helps only\nwhere people can keep track.", 6.0, shot="paper", start_tick=80,
         caption_y=0.0, title=True,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -48, 50), WIDE_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Reputation — after Nowak & Sigmund, 1998\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
