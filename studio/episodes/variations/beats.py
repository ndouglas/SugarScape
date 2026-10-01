"""Following the Crowd, episode 3: "Variations on a theme" (Pancs & Vriend
2007; Gauvin, Vannimenus & Nadal 2009; Singh, Vainchtein & Weiss 2009;
Zhang 2004).

Schelling's felt board from episode 1 at each paper's own size, a Flump per
agent, Red or Blue (colors="strategy"); a card top left names whose rules
these are, a number top right the paper's own measure. Board coordinates:
one unit per square, square (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).

What each shot does, each seed typical of 20 (see measurements.md):
board — s71-board, seed 4 (episode 1's): still from round 5.
flat, p50 — pv-flat and pv-p50 (5 × 5, 20 Flumps), seed 1: two clusters by
  step 2 (a step is 20 turns); flat's Flumps keep moving among squares they
  like as well, the clusters stay.
ring — pv-ring (20 in a ring), seed 4: two groups by step 3.
gvnrule — gvn-segregated (50 × 50), seed 1, the first 4 steps: everyone moves
  every step. segregated — the same to step 1,000, a frame every 20: 64
  clusters at step 1, 4 at 100, 2 at 1,000 (s 1.00).
frozen — gvn-frozen, seed 13: 70 move in step 1, then nobody (79 % discontent).
mixed — gvn-mixed, seed 13: s about 0.05 throughout.
small — svw-small (8 × 8), seed 1: two clusters, still by step 3.
large — svw-large (100 × 100), seed 9: 57 clusters, still by step 9.
zhang — zhang-checkerboard (100 × 100, 10,000 Flumps), seed 15, 20 steps;
  sorts — the same to step 1,000, a frame every 25: 20,000 mixed pairs to
  about 2,000.
spiked — pv-spiked, seed 3: never settles, about as mixed as chance.
"""

from camera import Move
from episode import Beat

BOARD_EYE, BOARD_AT = (0, -16, 16.5), (0, -1.8, 0)
PV_EYE, PV_AT = (0, -7.5, 8.8), (0, 0.4, 0)
RING_EYE, RING_AT = (0, -15, 9), (0, 1.5, 0)
GVN_EYE, GVN_AT = (0, -31, 76), (0, 3, 0)
SMALL_EYE, SMALL_AT = (0, -11, 13), (0, 0.6, 0)
BIG_EYE, BIG_AT = (0, -62, 150), (0, 6, 0)
S = {"colors": "strategy"}
PV = "Pancs & Vriend, 2007"
GVN = "Gauvin, Vannimenus\n& Nadal, 2009"
SVW = "Singh, Vainchtein\n& Weiss, 2009"
ZHANG = "Zhang, 2004"


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


def card(paper, **more):
    return {**S, "paper": paper, **more}


PV_CLUSTERS = {"stat": "pv_clusters", "label": "clusters", "format": "int"}
SVW_CLUSTERS = {"stat": "clusters8", "label": "clusters", "format": "int"}
GVN_S = {"stat": "seg_s", "label": "segregation (s)", "format": "2f"}
PAIRS = {"stat": "mixed_pairs", "label": "mixed neighbor pairs", "format": "int"}

BEATS = [
    Beat("theme", "Schelling's checkerboard has been rebuilt many times.\nEach rebuild changes one of his rules.", 6.5,
         shot="board", start_tick=6, caption_y=0.0, title=True, params=S,
         camera=(Move(0, 6.5, (0, -10, 7), (0, 1.5, 0), BOARD_EYE, BOARD_AT, lens0=32, lens1=32),)),
    Beat("pv-rule", "Pancs and Vriend: anyone may move, any time,\nto the square they like best.", 6.5, shot="flat",
         ticks_per_second=0.25, overlays=("paper", "figure"), params=card(PV, **PV_CLUSTERS),
         camera=hold(PV_EYE, PV_AT, lens=32, drift=(0, 0.3, -0.2))),
    Beat("pv-flat", "With Schelling's wishes, most runs end split in two.", 5.5, shot="flat", start_tick=1,
         ticks_per_second=2.0, overlays=("paper", "figure"), params=card(PV, **PV_CLUSTERS),
         camera=hold(PV_EYE, PV_AT, lens=32, drift=(0, 0.3, -0.2))),
    Beat("pv-p50", "Wanting a mixed street, up to half and half,\nsplits them even more often.", 6.5, shot="p50",
         ticks_per_second=1.8, overlays=("paper", "figure"), params=card(PV, **PV_CLUSTERS),
         camera=hold(PV_EYE, PV_AT, lens=32, drift=(0, 0.3, -0.2))),
    Beat("ring", "In a ring, even Flumps who like half and half best\nend in two groups.", 6.5, shot="ring",
         ticks_per_second=1.5, overlays=("paper", "ringjoin"), params=card(PV),
         camera=hold(RING_EYE, RING_AT, lens=30, drift=(0, 0.6, -0.3))),
    Beat("gvn-rule", "Gauvin and colleagues: anyone may move to a square that suits.\nTolerance works like temperature.",
         7.0, shot="gvnrule", ticks_per_second=0.55, overlays=("paper", "figure"), params=card(GVN, **GVN_S),
         camera=hold(GVN_EYE, GVN_AT, lens=32, drift=(0, 1, -1))),
    Beat("frozen", "Tolerate too little, and nobody can move.", 5.0, shot="frozen", ticks_per_second=0.8,
         overlays=("paper", "rings-unhappy"), params=card(GVN), camera=hold(GVN_EYE, GVN_AT, lens=32, drift=(0, 1, -1))),
    Beat("segregated", "Tolerate up to half, and two great clusters form.", 6.0, shot="segregated",
         ticks_per_second=9.0, overlays=("paper", "figure"), params=card(GVN, **GVN_S),
         camera=hold(GVN_EYE, GVN_AT, lens=32, drift=(0, 1, -1))),
    Beat("mixed", "Tolerate most, and the town stays mixed.", 5.0, shot="mixed", ticks_per_second=3.0,
         overlays=("paper", "figure"), params=card(GVN, **GVN_S),
         camera=hold(GVN_EYE, GVN_AT, lens=32, drift=(0, 1, -1))),
    Beat("small", "Singh and colleagues: in a small town,\nthe same wishes make two clusters.", 6.0, shot="small",
         ticks_per_second=1.2, overlays=("paper", "figure"), params=card(SVW, **SVW_CLUSTERS),
         camera=hold(SMALL_EYE, SMALL_AT, lens=32, drift=(0, 0.4, -0.3))),
    Beat("city", "In a city of 100 by 100, dozens.\nSchelling's striking picture is a small-town effect.", 7.0,
         shot="large", ticks_per_second=1.8, overlays=("paper", "figure"), params=card(SVW, **SVW_CLUSTERS),
         camera=hold(BIG_EYE, BIG_AT, lens=32, drift=(0, 2, -2))),
    Beat("zhang", "Zhang: no empty houses. Neighbors trade homes,\nand everyone likes half and half best.", 6.5,
         shot="zhang", ticks_per_second=0.6, overlays=("paper", "figure"), params=card(ZHANG, **PAIRS),
         camera=hold(BIG_EYE, BIG_AT, lens=32, drift=(0, 2, -2))),
    Beat("sorts", "From a perfect mix, they sort anyway.", 5.5, shot="sorts", ticks_per_second=7.5,
         overlays=("paper", "figure"), params=card(ZHANG, **PAIRS), camera=hold(BIG_EYE, BIG_AT, lens=32, drift=(0, 2, -2))),
    Beat("misses", "Two claims don't reproduce here: Zhang's waiting times, and that\n"
         "wanting only a perfect mix acts like wanting half and half.", 8.0, shot="spiked", ticks_per_second=3.5,
         overlays=("figure",), params={**S, **PV_CLUSTERS}, camera=hold(PV_EYE, PV_AT, lens=32, drift=(0, 0.3, -0.2))),
    Beat("point", "Change the rules, and the town still sorts,\nunless people tolerate a lot, or can't move at all.", 6.5,
         shot="segregated", start_tick=50, caption_y=0.0, title=True, params=S,
         camera=(Move(0, 6.5, GVN_EYE, GVN_AT, (0, -38, 86), GVN_AT, lens0=32, lens1=32, orbit=0.1),)),
    Beat("end", "Variations on a theme — after Schelling, 1971; Pancs & Vriend, 2007;\n"
         "Gauvin et al., Singh et al., 2009; Zhang, 2004 — ndouglas.github.io/SugarScape", 6.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
