"""Episode 8, "Credit" (Animation IV-5; rule L₁₀,₁₀).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
Loan lines take the book's colors: green from pure lenders, yellow through red
from Flumps deeper in the debt ladder.

What each shot does (from its dump; see measurements.md for the world beats):
loan — a 10 × 10 flat board: an old Flump (62, past childbearing, 60 sugar)
  beside a young one (25, holding 5 of a 40-sugar endowment). In tick 1 the
  old Flump lends 10, due as 20 at tick 10 (the engine's reading of the rate:
  10 % simple interest a tick). They wander apart, to opposite edges by
  tick 6, and meet again: in tick 11 the young one repays (its sugar 35 → 29
  as it eats) and the old one's rises 70 → 84; in tick 12 it borrows again.
world — iv-5-credit, seed 17 (typical of 20): 51 loans by tick 20, about
  170–240 outstanding after; the debt ladder is usually 4–5 levels deep and
  reaches 10 at ticks 966–968.
"""

from camera import Move
from episode import Beat

# The whole 10 × 10 board: the pair wander apart after the loan (across the
# wrapped edge, too) and meet again as it is repaid.
LOAN_EYE, LOAN_AT = (0, -8.5, 14.5), (0, 2.4, 0.4)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
SIDE_EYE, SIDE_AT = (0, -60, 34), (-8, 0, 0)
CLOSE = {"label": "age", "flows": True, "belly_full": 80, "line_width": 0.06, "reach": 12}
WIDE = {"line_width": 0.1}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("lend", "An old Flump, past having children, lends some of its sugar…", 5.5, shot="loan", closeup=True,
         ticks_per_second=0.35, lead_in=0.6, focus=(0, 1), overlays=("labels", "belly", "loanlines"), params=CLOSE,
         camera=hold(LOAN_EYE, LOAN_AT, lens=40)),
    Beat("repay", "…to a young neighbor, to pay back in ten ticks, with interest.", 6.0, shot="loan", closeup=True,
         ticks_per_second=2.0, start_tick=1, focus=(0, 1), overlays=("labels", "belly", "loanlines"), params=CLOSE,
         camera=hold(LOAN_EYE, LOAN_AT, lens=42)),
    Beat("flow", "Loans flow from old to young: lenders about 60, borrowers about 37.", 7.0, shot="world",
         ticks_per_second=30, start_tick=20, overlays=("loanlines",), params=WIDE,
         camera=(Move(0, 7, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("old", "Two in three loans come from Flumps too old to have children.", 7.0, shot="world",
         ticks_per_second=20, start_tick=240, overlays=("loanlines",), params=WIDE,
         camera=hold((0, -40, 26), (0, 0, 0), lens=38)),
    Beat("chains", "Some Flumps borrow and lend at once, and chains of debt form.", 8.0, shot="world",
         ticks_per_second=30, start_tick=400, overlays=("loanlines", "ladder"), params=WIDE,
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("deep", "The book saw five levels. Ours reach about ten.", 8.0, shot="world", ticks_per_second=2.0,
         start_tick=958, overlays=("loanlines", "ladder"), params=WIDE, camera=hold(SIDE_EYE, SIDE_AT, lens=38)),
    Beat("born", "With credit, about a fifth more Flumps are born, in all 20 worlds.", 7.0, shot="world",
         start_tick=1000, overlays=("bars",),
         params={"title": "Flumps born in 1000 ticks", "format": "num", "top": 6000,
                 "rows": [[("with credit", "births"), ("without", "births_off")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("unequal", "And they're a little less equal, in 18 of 20.", 7.0, shot="world", start_tick=1000,
         overlays=("bars",),
         params={"title": "how unequal (Gini)", "format": "num", "top": 0.25,
                 "rows": [[("with credit", "gini"), ("without", "gini_off")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=37)),
    Beat("question", "Every generation borrows\nfrom the one before.", 6.0, shot="world", start_tick=1000,
         caption_y=0.0, title=True, camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Credit — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
