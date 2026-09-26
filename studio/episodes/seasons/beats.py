"""Episode 2, "Seasons" (Animation II-7).

Wide shots look from the east, so the equator runs up the frame with the
north (+y) on the right. Board coordinates: one unit per cell, cell (x, y) at
(x − w/2 + 0.5, h/2 − y − 0.5); the north is rows y < h // 2.

What each shot does (from its dump; see measurements.md for the world beats):
reprise — the pilot's patch: a lone Flump circling a small hill as it regrows.
regrow — two sites, one each side of the equator, from empty: the north's
  grows 0 → 1 → 2 → 3 → 4, the south's 0.125 a tick.
walk — a far-sighted Flump (sees 6) in the winter south hops 4 cells north
  into summer and prospers (8 → 19 sugar).
wait — a frugal, short-sighted Flump (eats 1, sees 1) stays in the south
  through its winter, running down 8 → 3.5, and recovers to 7.8 when summer
  comes.
world — ii-7-seasons, seed 7 (typical of 20 seeds). The hungry (metabolism ≥ 3)
  alive at tick 53 die in a wave at ticks 55–64, just after winter reaches the
  north at tick 50; none survive (as in every seed).
calm — the same world and seed with seasons off.
"""

from camera import Move
from episode import Beat

EAST_EYE, AT = (62, -4, 34), (0, 0, 0)
NORTH = (0, 12, 0)


def hold(eye, at, lens=45, seconds=10, drift=(-0.4, 0, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("reprise", "Flumps eat sugar. Sugar grows back.", 4.0, shot="reprise", closeup=True,
         ticks_per_second=1.6, start_tick=5,
         camera=(Move(0, 4, (0.5, -6.2, 4.6), (0.5, -0.5, 1.2), (0.5, -6.2, 4.6), (0.5, -0.5, 1.2), lens0=40, lens1=43, orbit=0.4),)),
    Beat("seasons", "Now the world has seasons.", 6.0, shot="world", ticks_per_second=8, lead_in=1.0,
         overlays=("season-card",), camera=hold(EAST_EYE, AT, lens=35, drift=(-4, 0, -2))),
    Beat("slower", "In winter, sugar grows back eight times slower.", 6.0, shot="regrow", closeup=True,
         ticks_per_second=2.5, lead_in=0.8, overlays=("season-card",),
         camera=hold((7.5, 0, 4.2), (0.5, 0, 0.8), lens=35)),
    Beat("walk", "A far-sighted Flump can walk to the summer…", 6.0, shot="walk", closeup=True,
         ticks_per_second=1.2, lead_in=0.8, focus=(0,), overlays=("labels", "season-card"),
         camera=hold((9.5, -1.5, 5.0), (0.5, -1.5, 0.8), lens=32)),
    Beat("wait", "…a frugal one can wait out the winter.", 7.0, shot="wait", closeup=True,
         ticks_per_second=2.2, lead_in=0.6, focus=(0,), overlays=("belly", "season-card"),
         camera=hold((1.0, -13.5, 5.5), (1.0, -5.5, 0.8), lens=35, drift=(0, 0.4, -0.15))),
    Beat("swap", "Every 50 ticks, summer and winter swap.", 4.5, shot="world", ticks_per_second=3, start_tick=40,
         overlays=("season-card",), camera=hold(EAST_EYE, AT, lens=35, drift=(-2, 0, -1))),
    Beat("hungry", "Winter is hard on the hungry.", 6.0, shot="world", ticks_per_second=12, start_tick=53,
         overlays=("rings-hungry", "season-card"),
         camera=(Move(0, 6, (40, 8, 26), NORTH, (34, 10, 22), NORTH, lens0=35, lens1=40),)),
    Beat("follow", "Many Flumps follow the summer.", 8.0, shot="world", ticks_per_second=25, start_tick=130,
         overlays=("rings-migrants", "season-card"),
         camera=(Move(0, 8, (58, -10, 30), AT, (54, 6, 28), AT, lens0=35, lens1=37),)),
    Beat("cost", "Seasons cost this world a third of its Flumps.", 6.0, shot="world", ticks_per_second=28,
         start_tick=330, compare="calm", overlays=("counter", "season-card"),
         camera=hold(EAST_EYE, AT, lens=33, drift=(6, 0, 3))),
    Beat("who", "Starting rich barely helps. Starting on a hill helps some.\nNeeding little helps most.", 8.0,
         shot="world", start_tick=500, overlays=("survival",),
         camera=(Move(0, 8, (68, -14, 37), (-6, -4, 0), (66, -12, 36), (-6, -4, 0), lens0=33, lens1=34),)),
    Beat("question", "Hard times don't just shrink a society.\nThey decide who's in it.", 6.0, shot="world",
         start_tick=500, caption_y=0.0, title=True,
         camera=(Move(0, 6, (60, -4, 33), AT, (74, -4, 42), AT, orbit=0.1),)),
    Beat("end", "Seasons — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
