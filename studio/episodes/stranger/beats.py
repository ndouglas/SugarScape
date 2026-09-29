"""Cooperation, episode 8: "Why help a stranger?", the series finale.

Each montage shot is a copy of an earlier episode's shot (the same preset,
seed and length), so every Flump still comes from a real run of its model;
see those episodes' beats for what each run does. The ledgers' right-hand
column comes from measurements.json (see claims.py): the earlier episodes'
20-seed measurements, and Epstein's Run 4 measured here.

neighbors — Spatial games' scatter (nm-1b-chaos, 99 × 99, seed 11).
relatives — Ethnocentrism's land (ha-standard, seed 15).
look — Tags' long run (rca-published, seed 6).
name — Reputation's paper run (ns-fig-1, seed 2): everyone ends up helping.
rule — Norms' holds (ax-metanorms, seed 2): established by 25.
faces, strangers — Friends and strangers' fixed partners (cra-frn, seed 15)
  and strangers (cra-rwr, seed 20).
"""

from camera import Move
from episode import Beat

GRID_EYE, GRID_AT = (0, -24, 26), (0, 0.5, 0)
TOP_EYE, TOP_AT = (0, -42, 108), (0, 1, 0)
LAND_EYE, LAND_AT = (0, -46, 44), (0, 2, 0)
RING_EYE, RING_AT = (0, -38, 62), (0, 2.5, 0)
STREET_EYE, STREET_AT = (0, -37, 41), (0, 5, 0)
PLANE_EYE, PLANE_AT = (0, -46, 40), (0, -1.5, 0)
FOLLOW = {"follow": [17, 60, 119, 150, 200, 238]}
PLAY = ("ties", "payoff")
HERE = "here, over 20 worlds"
HELD = {"heads": ("the paper", HERE), "rows": [
    ("Nowak & May, 1992: 31.8% keep helping", "{nm}"),
    ("Huberman & Glance, 1993: one cheat takes all", "{hg}"),
    ("Nowak, Bonhoeffer & May, 1994: helpers survive", "{nbm}"),
    ("Hammond & Axelrod, 2006: 76.3% favor their own", "{ha}"),
    ("Riolo, Cohen & Axelrod, 2001: 73.6% of meetings", "{rca}"),
    ("Nowak & Sigmund, 1998: 90, 47, 18% by group size", "{ns}"),
    ("Axelrod, 1986: the norm holds in 5 of 5", "{ax}"),
    ("Cohen, Riolo & Axelrod, 2001: Table 2", "{cra}"),
]}
DETAIL = {"heads": ("the detail", HERE), "rows": [
    ("Epstein, the book: play each neighbor", "{ep_book}"),
    ("Epstein, the working paper: one neighbor", "{ep_working}"),
    ("Axelrod, on ties: everyone has one child", "{ax_his}"),
    ("Galán & Izquierdo: two each, half removed", "{gi}"),
]}
DIDNT = {"heads": ("the paper", HERE), "rows": [
    ("Hammond & Axelrod: at double cost, 56% help", "{ha2}"),
    ("Epstein, Run 4: coexistence, cooperators alone, or extinction", "{run4}"),
]}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


def wide(eye, at):
    return hold(eye, at, lens=35, drift=(0, 1, -1))


BEATS = [
    Beat("question", "Seven episodes, one question:\nwhy would anyone help a stranger?", 6.0, shot="strangers",
         ticks_per_second=20, caption_y=0.0, title=True, params=FOLLOW,
         camera=(Move(0, 6, (0, -15, 11), (0, 1, 0), GRID_EYE, GRID_AT, lens0=35, lens1=35),)),
    Beat("neighbors", "Neighbors: about a third keep helping, in clusters.", 4.5, shot="neighbors",
         ticks_per_second=25, start_tick=150, overlays=("helpers",), camera=wide(TOP_EYE, TOP_AT)),
    Beat("relatives", "Relatives: most help goes to kin next door.", 4.5, shot="relatives", ticks_per_second=30,
         start_tick=1250, params={"colors": "tag"}, camera=wide(LAND_EYE, LAND_AT)),
    Beat("look", "A shared look: most gifts go to an exact look-alike.", 4.5, shot="look", ticks_per_second=20,
         start_tick=700, camera=wide(RING_EYE, RING_AT)),
    Beat("name", "A good name: in 7 of 20 worlds, everyone ends up helping.", 4.5, shot="name",
         ticks_per_second=12, start_tick=20, overlays=("rate",),
         params={"stat": "help_rate", "label": "of meetings end in a gift"}, camera=wide(STREET_EYE, STREET_AT)),
    Beat("rule", "A rule everyone enforces: the norm holds.\nHow long depends on one sentence.", 4.5, shot="rule",
         ticks_per_second=15, start_tick=20, overlays=("generation", "mean-traits"), camera=wide(PLANE_EYE, PLANE_AT)),
    Beat("faces", "Partners who stay: cooperation holds.", 4.5, shot="faces", ticks_per_second=50, start_tick=150,
         overlays=PLAY, params=FOLLOW, camera=wide(GRID_EYE, GRID_AT)),
    Beat("strangers", "New strangers every period: it almost never takes hold.", 4.5, shot="strangers",
         ticks_per_second=50, start_tick=60, overlays=PLAY, params=FOLLOW, camera=wide(GRID_EYE, GRID_AT)),
    Beat("honest", "Now the honest part: what reproduced, and what didn't.", 5.0, shot="strangers", start_tick=300,
         params=FOLLOW, camera=hold(GRID_EYE, GRID_AT, lens=35, drift=(0, 2, -2))),
    Beat("held", "Most of it reproduces, closely.", 12.0, shot="faces", start_tick=400, overlays=("ledger",),
         params=HELD, camera=hold((0, -32, 34), GRID_AT, lens=34)),
    Beat("detail", "Two results depend on a detail the papers give two ways.", 9.0, shot="faces", start_tick=400,
         overlays=("ledger",), params=DETAIL, camera=hold((0, -32, 34), GRID_AT, lens=34)),
    Beat("didnt", "And two didn't come back, as written.", 8.0, shot="faces", start_tick=400, overlays=("ledger",),
         params=DIDNT, camera=hold((0, -32, 34), GRID_AT, lens=34)),
    Beat("point", "Why help a stranger?\nThe Flumps do, when something makes the stranger less strange.", 6.5,
         shot="faces", start_tick=400, caption_y=0.0, title=True, params=FOLLOW,
         camera=(Move(0, 6.5, GRID_EYE, GRID_AT, (0, -32, 34), GRID_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Why help a stranger? — seven episodes, ten papers, 1986–2006\nndouglas.github.io/SugarScape", 5.5,
         caption_y=0.45, camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
