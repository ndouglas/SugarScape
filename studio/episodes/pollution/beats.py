"""Episode 3, "Pollution" (Animation II-8).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
On the 50 × 50 board the sugar hills sit around board (14, 14) and (−10, −10).

What each shot does (from its dump; see measurements.md for the world beats):
mess — a Flump grazing a small hill with pollution on: every site it gathers
  and eats on is stained, and the total climbs 0 → 6 → 11 → … → 41 in 8 ticks.
avoid — a heavy eater (eats 4) stains the rich hill (4, 6); at tick 6 a
  newcomer (sees 4) arrives at (8, 6), four cells from it and from a poorer,
  clean hill (12, 6). The rich site holds 4 sugar under 8 pollution (worth
  4 / 9); the clean one 3 (worth 3). It takes the clean hill.
world — ii-8-pollution, seed 6 (typical of 20): pollution from t = 50,
  diffusion from t = 100. The share on the hills falls from 91% (t = 49) to 36%
  by t = 60, and drifts back to 62% as diffusion spreads the soot everywhere.
clean — the same world and seed with its schedule removed: pollution never starts.
"""

from camera import Move
from episode import Beat

HILL = (14, 14, 1.8)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)



def hold(eye, at, lens=45, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("mess", "Gathering sugar leaves a mess. So does eating it.", 6.0, shot="mess", closeup=True,
         ticks_per_second=1.2, lead_in=0.8, camera=hold((0.5, -6.4, 4.2), (0.5, -0.5, 0.9), lens=38)),
    Beat("avoid", "Flumps avoid dirty ground: a polluted spot is worth less.", 6.5, shot="avoid", closeup=True,
         ticks_per_second=1.0, start_tick=4, focus=(1,), overlays=("labels",),
         camera=hold((0.5, -9.0, 5.0), (0.5, -0.5, 0.8), lens=30)),
    Beat("clean", "400 Flumps, and for 50 ticks, clean land.", 6.0, shot="world", ticks_per_second=8, lead_in=0.8,
         camera=(Move(0, 6, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("starts", "Then the pollution starts.", 4.0, shot="world", ticks_per_second=1.5, start_tick=48,
         camera=(Move(0, 4, (30, -6, 15), HILL, (26, -2, 13), HILL, lens0=38, lens1=42),)),
    Beat("flee", "Flumps flee the hills…", 4.6, shot="world", ticks_per_second=5, start_tick=52, compare="clean",
         overlays=("hills",), params={"without": "without pollution"},
         camera=(Move(0, 4.6, (26, -2, 13), HILL, (34, -14, 22), HILL, lens0=42, lens1=36),)),
    Beat("follows", "…but the mess goes wherever they go.", 6.2, shot="world", ticks_per_second=20, start_tick=75,
         compare="clean", overlays=("hills",), params={"without": "without pollution"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 4, -2))),
    Beat("dirtiest", "The best land gets the dirtiest.", 6.0, shot="world", ticks_per_second=50, start_tick=200,
         overlays=("bars",),
         params={"title": "pollution at tick 500", "format": "num", "top": 200,
                 "rows": [[("on the hills", "pol_hill"), ("on the plains", "pol_plain")]]},
         camera=(Move(0, 6, (30, -6, 15), HILL, (22, 0, 11), HILL, lens0=38, lens1=42),)),
    Beat("cost", "It costs this world a quarter of its Flumps.", 5.0, shot="world", start_tick=500, compare="clean",
         overlays=("counter",), params={"with": "with pollution", "without": "without pollution"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, -3, 2))),
    Beat("who", "The hungry go first. Then the far-sighted, who had the best land.", 8.0, shot="world",
         start_tick=500, overlays=("bars",),
         params={"title": "who survives once pollution starts",
                 "rows": [[("hungry, with pollution", "met_hi_alive"), ("hungry, without", "met_hi_alive_clean")],
                          [("far-sighted, with pollution", "vis_hi_alive"), ("far-sighted, without", "vis_hi_alive_clean")]]},
         camera=(Move(0, 8, (0, -60, 34), (-8, 0, 0), (0, -58, 33), (-8, 0, 0)),)),
    Beat("equal", "The survivors are more equal, because there are fewer of them.", 7.0, shot="world",
         start_tick=500, overlays=("bars",),
         params={"title": "at tick 500", "format": "num", "tops": [0.5, 250],
                 "rows": [[("inequality (Gini), with pollution", "gini"), ("without", "gini_clean")],
                          [("Flumps, with pollution", "pop"), ("without", "pop_clean")]]},
         camera=(Move(0, 7, (0, -60, 34), (-8, 0, 0), (-2, -58, 33), (-8, 0, 0)),)),
    Beat("question", "Everyone made the mess.\nEveryone lives in it.", 6.0, shot="world", start_tick=500,
         caption_y=0.0, title=True, camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Pollution — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
