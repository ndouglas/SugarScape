"""Episode 9, "Contagion" (Animations V-1 to V-3; McNeill).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
Flumps turn sickly green while they carry a disease; infection lines are the
book's transmission network (Animation V-3).

What each shot does (from its dump; see measurements.md for the world beats):
ward — an 8 × 8 flat board: seven neighbors, each carrying 1–3 diseases. In
  tick 1 diseases pass all round the cluster (six infections); by tick 4 all
  are well but one, a carrier with 2 diseases that stays sick through tick 14
  and re-infects its neighbors at ticks 3, 5, 9 and 10.
rid — v-1-rid, seed 17 (typical of 20): 87% sick at tick 0, 6% by tick 4,
  about 1.4% for good.
endemic — v-2-endemic, seed 17: 25 diseases, 10 each.
stranger — v-mcneill, seed 18 (typical of 20): no one sick by tick 299; the
  novel 10-bit disease given to 5 at tick 300 peaks at 2.1% and is gone by 305.
plague — the same seed with a 40-bit novel disease (our experiment): 49% sick
  by tick 305, 91% by 310.
"""

from camera import Move
from episode import Beat

WARD_EYE, WARD_AT = (0, -8.5, 7.5), (0, 0.2, 0.5)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
CLOSE = {"colors": "sick", "label": "diseases", "belly_full": 25, "line_width": 0.06}
WIDE = {"colors": "sick", "line_width": 0.12}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("carry", "Some Flumps carry diseases. A disease steals a little of their sugar every tick.", 6.0,
         shot="ward", closeup=True, ticks_per_second=0.12, lead_in=0.6, focus=(0, 1, 2, 3, 4, 5, 6),
         overlays=("labels",), params=CLOSE, camera=hold(WARD_EYE, WARD_AT, lens=36)),
    Beat("pass", "Diseases pass between neighbors…", 4.5, shot="ward", closeup=True, ticks_per_second=0.35,
         start_tick=1, focus=(0, 1, 2, 3, 4, 5, 6), overlays=("labels", "infections"), params=CLOSE,
         camera=hold(WARD_EYE, WARD_AT, lens=38)),
    Beat("learn", "…but each Flump's immune system learns, one step at a time, until it's well.", 7.0,
         shot="ward", closeup=True, ticks_per_second=1.3, start_tick=2, focus=(0, 1, 2, 3, 4, 5, 6),
         # They wander as they recover: the whole board, from higher.
         overlays=("labels", "infections"), params=CLOSE, camera=hold((0, -10.5, 12.5), (0, 0.3, 0.4), lens=34)),
    Beat("start", "At the start, almost every Flump is sick.", 5.0, shot="rid", ticks_per_second=0.2, lead_in=0.6,
         overlays=("sick",), params=WIDE,
         camera=(Move(0, 5, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("well", "Within a few dozen ticks at most, nearly all are well.", 6.5, shot="rid", ticks_per_second=6,
         start_tick=1, overlays=("sick", "infections"), params=WIDE, camera=hold(WIDE_EYE, WIDE_AT, lens=36)),
    Beat("linger", "The book says society rids itself of every disease.\nHere a little always lingers, in 18 of 20 worlds.",
         7.0, shot="rid", ticks_per_second=150, start_tick=40, overlays=("sick",), params=WIDE,
         camera=hold(WIDE_EYE, WIDE_AT, lens=37, drift=(0, 3, -1.5))),
    Beat("endemic", "Give them more diseases than an immune system can hold,\nand more of it stays for good, in 16 of 20 worlds.",
         7.5, shot="endemic", ticks_per_second=140, lead_in=0.3, overlays=("sick",), params=WIDE,
         camera=(Move(0, 7.5, WIDE_EYE, WIDE_AT, (4, -52, 30), WIDE_AT, orbit=0.2),)),
    Beat("stranger", "Now a healthy society meets a disease it has never seen.", 5.5, shot="stranger",
         ticks_per_second=3, start_tick=292, overlays=("sick", "infections"), params=WIDE,
         camera=hold((0, -44, 26), (0, 0, 0), lens=38)),
    Beat("fizzle", "The book expects a plague. Here it fizzles:\nabout one Flump in twenty catches it.", 7.0,
         shot="stranger", ticks_per_second=2.5, start_tick=299, overlays=("sick", "infections"), params=WIDE,
         camera=hold((0, -44, 26), (0, 0, 0), lens=40)),
    Beat("plague", "Only a disease four times longer than any in the book sweeps through,\n"
         "and costs a fifth of the Flumps. (Our experiment, not the book's.)", 9.0, shot="plague",
         ticks_per_second=3, start_tick=298, overlays=("sick", "infections"), params=WIDE,
         camera=(Move(0, 9, (0, -44, 26), (0, 0, 0), (0, -58, 32), WIDE_AT, orbit=0.15),)),
    Beat("question", "A plague needs something\nthis world has never known.", 6.0, shot="plague", start_tick=500,
         caption_y=0.0, title=True, params=WIDE, camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Contagion — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
