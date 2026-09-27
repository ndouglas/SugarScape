"""Episode 6, "Markets" (Animations IV-1 to IV-6, Figure IV-13).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
Sugar is golden, spice red; with two goods the felt rises over both goods'
hills, sugar sitting a little west of a cell's center and spice a little east.

What each shot does (from its dump; see measurements.md for the world beats):
pantry — a 10 × 10 board, sugar in the south-west, spice in the north-east;
  one Flump at (4, 5) holding 20 sugar and 4 spice, needing 2 spice a tick,
  walks to the spice hill by tick 4, fills up (27 spice by tick 14) and
  turns back for sugar.
swap — a 6 × 6 board packed with Flumps in a checkerboard, 30 sugar and 3
  spice or the reverse; nobody can move, so neighbors trade: 101 pairs, 340
  exchanges at a median price of 1.0 in tick 1, and by tick 4 all are even
  and trading stops. The followed pair start in the middle.
shuttle — iv-1-spice, seed 18 (typical of 20 for walking): 28 % of the
  Flumps alive through ticks 100–200 change hills twice or more; 400 → 119
  by tick 1000, all by starvation.
walk — the same seed with trade on: 30 % shuttle (walks count between sites deep in each hill).
market — iv-3-trade, seed 19 (typical of 20 for the market): 355 exchanges
  in tick 1, 108 at tick 10, 12 at tick 500; the price stays near one while
  the spread falls from 0.47 (ticks 1–50) to 0.02 (950–1000).
"""

from camera import Move
from episode import Beat

PANTRY_EYE, PANTRY_AT = (0, -12, 10), (0, -0.3, 0.4)
SWAP_EYE, SWAP_AT = (0, -7.2, 6.2), (0, -0.2, 0.5)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
SIDE_EYE, SIDE_AT = (0, -60, 34), (-8, 0, 0)


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("spice", "Now there's a second food: spice.", 5.0, shot="pantry", closeup=True,
         ticks_per_second=0.6, lead_in=0.6, focus=(0,), overlays=("belly",), params={"belly_full": 30},
         camera=hold(PANTRY_EYE, PANTRY_AT, lens=34)),
    Beat("two", "A Flump needs both. Run out of either, and it's gone.", 5.5, shot="pantry", closeup=True,
         ticks_per_second=1.5, start_tick=3, focus=(0,), overlays=("belly",), params={"belly_full": 30},
         camera=hold(PANTRY_EYE, PANTRY_AT, lens=36)),
    Beat("shuttle", "Without trade, about a quarter of the Flumps walk back and forth between the hills.", 7.0,
         shot="shuttle", ticks_per_second=12, start_tick=100, overlays=("rings-shuttlers",),
         camera=(Move(0, 7, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("cost", "Of 400 Flumps, about 120 survive.", 6.0, shot="shuttle", ticks_per_second=170, lead_in=0.3,
         overlays=("census",), camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 3, -1.5))),
    Beat("swap", "Now let neighbors swap what they have too much of for what they lack.", 5.5, shot="swap",
         closeup=True, ticks_per_second=0.4, lead_in=0.4, focus=(0, 1), overlays=("trades", "belly"),
         params={"belly_full": 30}, camera=hold(SWAP_EYE, SWAP_AT, lens=40)),
    Beat("haggle", "They settle on a price between what each thinks it's worth.", 4.5, shot="swap", closeup=True,
         ticks_per_second=0.4, start_tick=1, focus=(0, 1), overlays=("trades", "belly"),
         params={"belly_full": 30}, camera=hold(SWAP_EYE, SWAP_AT, lens=44)),
    Beat("busy", "At first, hundreds of trades a tick.", 6.0, shot="market", ticks_per_second=1.5, lead_in=0.5,
         overlays=("trades",), camera=(Move(0, 6, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.15),)),
    Beat("price", "Nobody sets the price. It settles near one sugar per spice…", 7.0, shot="market",
         ticks_per_second=30, start_tick=3, overlays=("trades", "prices"),
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("agree", "…and the prices agree more and more.", 7.0, shot="market", ticks_per_second=115,
         start_tick=200, overlays=("prices",), camera=hold(SIDE_EYE, SIDE_AT, lens=37)),
    Beat("more", "Trade feeds more Flumps: in 118 of 120 worlds.", 7.0, shot="market", start_tick=1000,
         overlays=("bars",),
         params={"title": "Flumps the land can feed", "format": "num", "top": 80,
                 "rows": [[("with trade", "pop_trade"), ("without", "pop_no_trade")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("travel", "But it doesn't stop the walking: about as many Flumps still shuttle.", 7.0, shot="walk",
         ticks_per_second=12, start_tick=100, overlays=("rings-shuttlers",),
         camera=(Move(0, 7, WIDE_EYE, WIDE_AT, (4, -52, 30), WIDE_AT, orbit=0.2),)),
    Beat("unequal", "And it makes them less equal, in all 20 worlds.", 7.5, shot="market", start_tick=1000,
         overlays=("bars",),
         params={"title": "how unequal (Gini of sugar and spice)", "format": "num", "top": 0.5,
                 "rows": [[("with trade", "gini_trade"), ("without", "gini")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=37)),
    Beat("question", "Trade made the pie bigger.\nIt didn't share it out.", 6.0, shot="market", start_tick=1000,
         caption_y=0.0, title=True, camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Markets — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
