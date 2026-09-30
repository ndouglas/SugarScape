"""Following the Crowd, episode 4: "One culture or many" (Axelrod 1997;
Castellano, Marsili & Vespignani 2000; Klemm et al. 2003; Axtell, Axelrod,
Epstein & Cohen 1996).

A still Flump on every village (site) of the felt, colored by culture: each
culture still present at the shot's end wears its own yarn (by size) from the
first frame it appears; cultures that die out wear cream. Axelrod's Fig. 1
lanes run between neighbors, darker the less they share, gone where they are
identical. Board coordinates: one unit per site, site (x, y) at
(x − w/2 + 0.5, h/2 − y − 0.5).

What each shot does, each seed typical of 20 (see measurements.md):
sample — ac-sample-run (10 × 10, 5 features of 10 traits), seed 5, to step
  800: 100 cultures to 5 regions, still from step 744.
traits — 15 traits, seed 6: 21 regions at rest (step 2,243).
features — 15 features, seed 5: one region (step 1,209).
range — 12 neighbors, seed 3: one region (step 374).
small — ac-many-regions (12 × 12, 15 traits), seed 3: 17 regions (step 2,181).
big — 50 × 50, 15 traits, seed 1: 5 regions (step 36,178), a frame every 400.
shatter — 30 × 30, 25 traits, seed 5: 417 regions (step 3,249).
drift — ac-drift (12 × 12, 15 traits, one random change in 10,000 events),
  seed 2: 6 regions at step 20,000.
wander — dock-mobility-15 (the Sugarscape), seed 8: 100 cultures to one by
  tick 1,773.
"""

from camera import Move
from episode import Beat

TEN_EYE, TEN_AT = (0, -11, 12.5), (0, 0.4, 0)
TWELVE_EYE, TWELVE_AT = (0, -13, 14.5), (0, 0.5, 0)
THIRTY_EYE, THIRTY_AT = (0, -19, 46), (0, 2, 0)
FIFTY_EYE, FIFTY_AT = (0, -31, 76), (0, 3, 0)
MOUNTAIN_EYE, MOUNTAIN_AT = (0, -19, 24), (0, 1, 0)
C = {"colors": "culture"}
REGIONS = {**C, "stat": "regions", "label": "regions", "format": "int"}
MAP = ("lanes", "figure")


def hold(eye, at, lens=32, seconds=10, drift=(0, 0.4, -0.3)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("villages", "Robert Axelrod's villages: a hundred on a map.\nEach has five features, each one of ten traits.",
         6.5, shot="sample", ticks_per_second=0.01, caption_y=0.0, title=True, overlays=("lanes",), params=C,
         camera=(Move(0, 6.5, (0, -8, 5), (0, 1, 0), TEN_EYE, TEN_AT, lens0=30, lens1=32),)),
    Beat("rule", "Neighbors talk as often as they're alike.\nWhen they talk, one copies a feature from the other.",
         7.0, shot="sample", ticks_per_second=2.5, overlays=MAP, params=REGIONS, camera=hold(TEN_EYE, TEN_AT)),
    Beat("converge", "Neighbors grow alike, until each pair is\neither the same or shares nothing. "
         "Then nothing changes.", 8.0, shot="sample", start_tick=18, ticks_per_second=100.0, overlays=MAP, params=REGIONS,
         camera=hold(TEN_EYE, TEN_AT)),
    Beat("regions", "A few cultures survive, about four here;\nAxelrod's runs, about three.", 6.0, shot="sample",
         start_tick=800, overlays=MAP, params=REGIONS, camera=hold(TEN_EYE, TEN_AT, drift=(0.6, 0.4, -0.3))),
    Beat("traits", "More traits to differ on, and more survive.", 5.5, shot="traits", ticks_per_second=17.0,
         overlays=MAP, params=REGIONS, camera=hold(TEN_EYE, TEN_AT)),
    Beat("features", "More features to share, and one culture wins.", 5.5, shot="features",
         ticks_per_second=9.5, overlays=MAP, params=REGIONS, camera=hold(TEN_EYE, TEN_AT)),
    Beat("range", "Talk to more neighbors, and fewer survive.", 5.5, shot="range", ticks_per_second=15.0,
         overlays=MAP, params=REGIONS, camera=hold(TEN_EYE, TEN_AT)),
    Beat("territory-small", "The surprise: a bigger map\nends with fewer cultures.", 5.0, shot="small",
         ticks_per_second=18.0, overlays=MAP, params=REGIONS, camera=hold(TWELVE_EYE, TWELVE_AT)),
    Beat("territory-big", "A map of 50 by 50: a handful.", 5.0, shot="big", ticks_per_second=18.5, overlays=MAP,
         params=REGIONS, camera=hold(FIFTY_EYE, FIFTY_AT, drift=(0, 1, -1))),
    Beat("shatter", "Unless there are enough traits.\nThen a big map shatters.", 6.0, shot="shatter",
         ticks_per_second=11.5, overlays=MAP + ("paper",),
         params={**REGIONS, "paper": "Castellano, Marsili\n& Vespignani, 2000"},
         camera=hold(THIRTY_EYE, THIRTY_AT, drift=(0, 1, -1))),
    Beat("drift", "Let a trait change at random, now and then,\nand the borders melt.", 6.5, shot="drift",
         ticks_per_second=12.5, overlays=MAP + ("paper",), params={**REGIONS, "paper": "Klemm et al., 2003"},
         camera=hold(TWELVE_EYE, TWELVE_AT)),
    Beat("wander", "Let them wander a sugar mountain,\nand one culture takes everyone.", 7.0, shot="wander",
         ticks_per_second=260.0, overlays=("figure", "paper"),
         params={**C, "stat": "distinct_cultures", "label": "cultures", "format": "int",
                 "paper": "Axtell, Axelrod, Epstein\n& Cohen, 1996"},
         camera=hold(MOUNTAIN_EYE, MOUNTAIN_AT, drift=(0, 0.6, -0.6))),
    Beat("reproduces", "Everything Axelrod reported, the Flumps reproduce.", 5.5, shot="sample", start_tick=800,
         overlays=("lanes",), params=C, camera=hold(TEN_EYE, TEN_AT, drift=(-0.6, 0.4, -0.3))),
    Beat("point", "Neighbors grow alike, and the map stays divided.", 6.0, shot="sample", start_tick=800,
         caption_y=0.0, title=True, overlays=("lanes",), params=C,
         camera=(Move(0, 6.0, TEN_EYE, TEN_AT, (0, -15, 17), TEN_AT, lens0=32, lens1=32, orbit=0.1),)),
    Beat("end", "One culture or many — after Axelrod, 1997\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
