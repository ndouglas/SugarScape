"""On-screen text and data displays. Each overlay builder takes (beat, dump,
ctx) and returns a per-frame updater; `ctx` carries the camera's `Screen`,
the beat's timing, the agents' tracks, the board's corners and the close-up
rigs. Screen-space things hang from `Screen` anchors, whose units are half
the frame's width, so they keep their size on screen as the lens changes."""

from .retirement import retirement_population, retirement_panel, retirement_closing
from .farol import farol_stage, farol_panel
from .ants import ants_stage, ants_panel
from .caption import caption_scene
from .followers import belly, bequests, labels, rings_hungry, rings_migrants, rings_unhappy, rings_shuttlers, sight, stacks, trades, traits, loot, warlord, killmap, loanlines, infections
from .panels import alike, bars, census, tally, counter, dials, hills, histogram, kills, ladder, ledger, popchart, prices, sick, season_card, wealth
from .lattice import earnings, helpers, kinds, legend, play, scores
from .parts import Screen
from .ring import gifts, rate, tolerance
from .grid import payoff, ties
from .plane import events, generation, mean_traits
from .street import meetings
from .tipping import fence, tipplane
from .variations import figure, paper, ringjoin
from .culture import lanes
from .opinions import diagram, agreement_diagram
from .thresholds import thresholds_panel, thresholds_markers, thresholds_links, thresholds_comparison, thresholds_neighborhood
from .agreement import agreement_markers, agreement_pair, agreement_cutoff, agreement_results, agreement_comparison, agreement_horizon

__all__ = ["BUILDERS", "SCREEN", "Screen", "caption_scene"]


# The overlays drawn in screen space (on `Screen` anchors).
SCREEN = {"retirement-population", "retirement-panel", "farol-stage", "farol-panel", "ants-panel", "ants-stage", "thresholds-panel", "thresholds-comparison", "thresholds-neighborhood", "season-card", "counter", "hills", "alike", "survival", "bars", "dials", "histogram", "wealth",
          "census", "prices", "kills", "ladder", "sick", "popchart", "ledger", "helpers", "earnings", "tally", "kinds", "legend", "rate", "generation", "mean-traits", "payoff", "paper", "figure", "diagram", "agreement-diagram", "agreement-pair", "agreement-cutoff", "agreement-results", "agreement-markers", "agreement-comparison", "agreement-horizon"}


BUILDERS = {
    "retirement-population": retirement_population,
    "retirement-panel": retirement_panel,
    "retirement-closing": retirement_closing,
    "farol-stage": farol_stage,
    "farol-panel": farol_panel,
    "ants-stage": ants_stage,
    "ants-panel": ants_panel,
    "thresholds-panel": thresholds_panel,
    "thresholds-comparison": thresholds_comparison,
    "thresholds-neighborhood": thresholds_neighborhood,
    "thresholds-markers": thresholds_markers,
    "thresholds-links": thresholds_links,
    "season-card": season_card,
    "rings-migrants": rings_migrants,
    "rings-hungry": rings_hungry,
    "rings-unhappy": rings_unhappy,
    "fence": fence,
    "tipplane": tipplane,
    "counter": counter,
    "survival": bars,
    "bars": bars,
    "hills": hills,
    "bequests": bequests,
    "traits": traits,
    "alike": alike,
    "wealth": wealth,
    "belly": belly,
    "sight": sight,
    "labels": labels,
    "dials": dials,
    "stacks": stacks,
    "histogram": histogram,
    "census": census,
    "prices": prices,
    "rings-shuttlers": rings_shuttlers,
    "trades": trades,
    "loot": loot,
    "warlord": warlord,
    "killmap": killmap,
    "kills": kills,
    "loanlines": loanlines,
    "ladder": ladder,
    "infections": infections,
    "sick": sick,
    "popchart": popchart,
    "ledger": ledger,
    "helpers": helpers,
    "earnings": earnings,
    "scores": scores,
    "play": play,
    "tally": tally,
    "kinds": kinds,
    "legend": legend,
    "rate": rate,
    "gifts": gifts,
    "tolerance": tolerance,
    "meetings": meetings,
    "events": events,
    "generation": generation,
    "mean-traits": mean_traits,
    "ties": ties,
    "payoff": payoff,
    "paper": paper,
    "figure": figure,
    "ringjoin": ringjoin,
    "lanes": lanes,
    "diagram": diagram,
    "agreement-diagram": agreement_diagram,
    "agreement-markers": agreement_markers,
    "agreement-pair": agreement_pair,
    "agreement-cutoff": agreement_cutoff,
    "agreement-results": agreement_results,
    "agreement-comparison": agreement_comparison,
    "agreement-horizon": agreement_horizon,
}
