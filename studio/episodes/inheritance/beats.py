"""Episode 4, "Inheritance" (Animations III-1–III-4).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).

What each shot does (from its dump; see measurements.md for the world beats):
family — a mother and father (placed side by side on a small hill, lifespan
  fixed at 30 for the close-up) have children at ticks 16, 18, 25 and 30, each
  starting with 6 sugar (half of each parent's starting endowment). At tick 32
  both die of old age, holding 47 and 51; their 98 sugar splits four ways, and
  the children jump 32 → 56, 25 → 48, 14 → 40 and 8 → 35. (Siblings may pair
  under rule S; a grandchild arrives at tick 32.)
family-plain — the same, without inheritance: identical until tick 32, when
  the parents' sugar vanishes and the children carry on as before.
heirs, plain — iii-4-inheritance and iii-2-sex (identical but for rule I),
  the same typical seed, 1000 ticks.
"""

from camera import Move
from episode import Beat

FAMILY_EYE, FAMILY_AT = (0.5, -8.6, 5.6), (0.5, -0.5, 1.4)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
BELLIES = {"belly": "all", "belly_full": 60, "colors": "family"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("pair", "Flumps can pair up and have children…", 6.0, shot="family", closeup=True,
         ticks_per_second=1.6, start_tick=12, params={"colors": "family"}, camera=hold(FAMILY_EYE, FAMILY_AT, lens=38)),
    Beat("endow", "…and a child starts with sugar from both parents.", 5.0, shot="family", closeup=True,
         ticks_per_second=1.2, start_tick=14, overlays=("belly",), params=BELLIES,
         camera=hold((0.5, -8.0, 5.2), FAMILY_AT, lens=36)),
    Beat("old", "Flumps grow old, and die.", 4.0, shot="family-plain", closeup=True, ticks_per_second=1.6,
         start_tick=26, params={"colors": "family"}, camera=hold(FAMILY_EYE, FAMILY_AT, lens=38)),
    Beat("vanish", "Without inheritance, a Flump's sugar dies with it.", 5.0, shot="family-plain", closeup=True,
         ticks_per_second=1.0, start_tick=29, overlays=("belly",), params=BELLIES,
         camera=hold(FAMILY_EYE, FAMILY_AT, lens=36)),
    Beat("inherit", "With inheritance, it goes to the children.", 6.0, shot="family", closeup=True,
         ticks_per_second=1.0, start_tick=29, overlays=("belly", "bequests"), params=BELLIES,
         camera=hold(FAMILY_EYE, FAMILY_AT, lens=36)),
    Beat("generations", "A thousand ticks. About sixty generations.", 8.0, shot="heirs", ticks_per_second=125,
         lead_in=0.4, params={"colors": "family"},
         camera=(Move(0, 8, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.3),)),
    Beat("more", "Kept in the family, sugar feeds more Flumps: nearly four times as many.", 7.0, shot="heirs",
         ticks_per_second=40, start_tick=60, compare="plain", overlays=("counter",),
         params={"colors": "family", "with": "with inheritance", "without": "without"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 3, -1.5))),
    Beat("uneven", "But it's shared far less evenly.", 8.0, shot="heirs", ticks_per_second=50, start_tick=600,
         overlays=("stacks", "bars"),
         params={"colors": "family", "stack_scale": 0.004, "title": "at tick 1000", "format": "num", "tops": [0.5, 0.5],
                 "rows": [[("inequality (Gini), with inheritance", "gini"), ("without", "gini_plain")],
                          [("richest tenth's share, with", "top10"), ("without", "top10_plain")]]},
         camera=(Move(0, 8, (-2, -44, 28), (9, 0, 0), (-2, -38, 24), (9, 0, 0), orbit=-0.2),)),
    Beat("last", "And fortunes last.", 8.0, shot="heirs", start_tick=1000, overlays=("bars",),
         params={"colors": "family", "title": "children who end up in the richest quarter",
                 "rows": [[("rich parents, with inheritance", "rich_stay"), ("without", "rich_stay_plain")],
                          [("poor parents, with inheritance", "poor_rise"), ("without", "poor_rise_plain")]]},
         camera=(Move(0, 8, (0, -60, 34), (-8, 0, 0), (-2, -58, 33), (-8, 0, 0)),)),
    Beat("question", "Nobody earned it.\nIt was waiting for them.", 6.0, shot="heirs", start_tick=1000,
         caption_y=0.0, title=True, params={"colors": "family"},
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Inheritance — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
