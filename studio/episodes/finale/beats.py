"""Episode 10, "What didn't reproduce": the book's results that its stated
rules don't grow, over 20 seeds each.

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).

What each shot does (from its dump; see measurements.md for the numbers):
everything — vi-1-everything, seed 17: every rule at once; 400 → 117 by
  tick 100, 1,252 by 300.
without / with — vi-2-no-trade and vi-3-trade, seed 17 (typical of 20): both
  fall from 500 to about 110–170 by tick 100, pass 800 by 300 and track
  each other after (790 and 879 at tick 1000).
waves — ii-6-waves, seed 17: the book's full 20 × 20 block in the south-west
  corner; it spreads a little and stays.
collide — iii-12-collision, seed 17: opposed blocks in the corners; each tribe
  settles on its own hill and they never meet.
"""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
SIDE_EYE, SIDE_AT = (0, -60, 34), (-10, 0, 0)
TRIBE = {"colors": "tribe"}
CHART = {"labels": ("without trade", "with trade")}
LEDGER = {"rows": [
    ("battle fronts that hold", "no front; {newcomers:.0%} of the dead are newcomers"),
    ("conquest and conversion together", "a civil war: {iii14_pop:.0f} of 400 left"),
    ("nearly 150,000 trades", "about {trades_rounded:,.0f}"),
    ("disease that stays for good (V-2)", "it clears, in {v2_cleared} of 20"),
    ("a novel disease takes a terrible toll", "{fizzle:.0%} catch it; no more die"),
    ("foresight is selected away", "lower in only 12 of 20 worlds"),
]}
# Chapter VI under the book's unsaid details: four wealth tests for
# childbearing in each row, as ranges over them, in 20 worlds each.
UNSAID = {**CHART, "heads": ("what the book leaves unsaid", "crashes without trade · with trade · doublings"),
          "rows": [
              ("founders newborn, endowments 25–50", "{g0_crash} · {g0_crash3} · {g0_double}"),
              ("founders newborn, endowments 50–100", "{g1_crash} · {g1_crash3} · {g1_double}"),
              ("founders at random ages, endowments 25–50", "{g2_crash} · {g2_crash3} · {g2_double}"),
              ("founders at random ages, endowments 50–100", "{g3_crash} · {g3_crash3} · {g3_double}"),
          ]}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("nine", "Nine episodes, all grown from the book's rules.", 5.0, shot="everything", ticks_per_second=6,
         start_tick=150, caption_y=0.0, title=True, camera=(Move(0, 5, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("everything", "Everything at once: seasons, pollution, sex, culture, combat, trade, credit, disease.", 7.0,
         shot="everything", ticks_per_second=10, start_tick=180, camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 3, -1.5))),
    Beat("honest", "Now the honest part: what the book shows, and these rules don't.", 5.5, shot="everything",
         ticks_per_second=2, start_tick=250, camera=hold(WIDE_EYE, WIDE_AT, lens=38)),
    Beat("book", "The book: without trade, this society crashes. With trade, it doubles.", 6.0, shot="without",
         compare="with", ticks_per_second=0.5, overlays=("popchart",), params=CHART,
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("both", "Here, both dip, recover and settle near 800, with trade or without.", 9.0, shot="without",
         compare="with", ticks_per_second=110, lead_in=0.3, overlays=("popchart",), params=CHART,
         camera=hold(SIDE_EYE, SIDE_AT, lens=37)),
    Beat("none", "No crash after the first dip in 20 of 20 worlds. No doubling in 19. No 115-year cycles.", 7.0,
         shot="without", compare="with", start_tick=1000, overlays=("popchart",), params=CHART,
         camera=hold(SIDE_EYE, SIDE_AT, lens=38)),
    Beat("unsaid", "However we fill in what the book leaves unsaid,\ntrade never turns a crash into a boom.", 10.0,
         shot="without", start_tick=1000, overlays=("ledger",), params=UNSAID, camera=hold((0, -70, 40), WIDE_AT, lens=34)),
    Beat("waves", "Waves that sweep across the land (II-6): here the Flumps stay put.", 7.0, shot="waves",
         ticks_per_second=14, lead_in=0.4, camera=(Move(0, 7, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.15),)),
    Beat("collide", "Two tribes that collide (III-12): in 16 of 20 worlds they never meet.", 7.0, shot="collide",
         ticks_per_second=14,
         lead_in=0.4, params=TRIBE, camera=hold(WIDE_EYE, WIDE_AT, lens=36)),
    Beat("ledger", "And from our episodes:", 12.0, shot="collide", start_tick=100, overlays=("ledger",),
         params={**TRIBE, **LEDGER}, camera=hold((0, -70, 40), WIDE_AT, lens=34)),
    Beat("question", "The rules are simple.\nNot every story in the book is in them.", 6.0, shot="everything",
         start_tick=300, caption_y=0.0, title=True, camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "What didn't reproduce — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
