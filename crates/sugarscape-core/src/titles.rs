//! Each preset's title: a plain headline of what happens or what the run
//! shows, for the presets menu. The rule notation (`name`) and the figure or
//! paper (`source`) stay on the preset as its reference.

/// Titles by preset id, in catalog order.
pub const TITLES: [(&str, &str); 424] = [
    (
        "ii-1-instant",
        "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
    ),
    (
        "ii-2-unit",
        "Sugar grows back slowly: foraging never stops and the population falls to about 224",
    ),
    (
        "ii-5-wealth",
        "Agents are born and die, and a lopsided distribution of wealth emerges",
    ),
    (
        "ii-6-waves",
        "A crowd bursts out of one corner, but the book's traveling waves never come",
    ),
    (
        "ii-7-seasons",
        "The seasons swap every 50 ticks: the far-sighted migrate and the frugal hibernate",
    ),
    (
        "ii-8-pollution",
        "Harvesting pollutes the land, and the pollution spreads",
    ),
    (
        "iii-2-sex",
        "Sexual reproduction keeps a steady population of overlapping generations",
    ),
    (
        "iii-4-inheritance",
        "Children inherit their parents' wealth; watch what it does to inequality",
    ),
    (
        "iii-6-culture",
        "Neighbors copy each other's culture until each mountain becomes one tribe",
    ),
    (
        "iii-6-three-tribes",
        "Culture spreads among three tribes: blue, green and red",
    ),
    (
        "iii-9-combat",
        "Quiet for hundreds of ticks, then one agent conquers the other tribe",
    ),
    (
        "iii-11-combat-fixed",
        "Two tribes fight forever, but the book's battle fronts never form",
    ),
    (
        "iii-12-collision",
        "Two tribes set out from opposite corners, but the book's colliding waves never meet",
    ),
    (
        "iii-14-combat-culture",
        "Culture meets combat: converts start a civil war that leaves almost no one",
    ),
    (
        "iv-1-spice",
        "Sugar on one mountain, spice on the other: half the agents shuttle between them",
    ),
    (
        "iv-3-trade",
        "Neighbors barter sugar for spice, and prices settle near 1",
    ),
    (
        "iv-15-trade-sex",
        "Trade across generations: prices never settle, and most societies die out",
    ),
    (
        "iv-3-pollution",
        "Sugar becomes a dirty good: its price rises, then the pollution clears",
    ),
    (
        "iv-18-foresight",
        "Agents plan ahead, and evolution barely trims how far",
    ),
    (
        "iv-5-credit",
        "The old lend to the young to raise children, and chains of credit form",
    ),
    (
        "v-1-rid",
        "Immune systems learn their diseases and wipe them out",
    ),
    (
        "v-2-endemic",
        "Too many diseases to remember, yet sickness clears: the book expected it to stay",
    ),
    (
        "v-mcneill",
        "A society that has beaten its old diseases meets a new one",
    ),
    (
        "vi-1-everything",
        "Every rule at once: trade, credit, culture, disease and new outbreaks",
    ),
    (
        "vi-2-no-trade",
        "Without trade the book's population crashes; here it recovers",
    ),
    (
        "vi-3-trade",
        "With trade the population dips, recovers and nearly doubles, as in the book",
    ),
    (
        "n-3-trade",
        "Three goods and three sets of prices: sugar, spice and salt",
    ),
    ("n-4-peaks", "Four goods on four peaks: travel far or trade"),
    (
        "n-2-pollutants",
        "Two pollutions: smoke drives agents off sugar, runoff spoils spice",
    ),
    (
        "dock-mobility-15",
        "Culture on the move: one culture takes everyone",
    ),
    (
        "dock-mobility-30",
        "Culture on the move with more traits: one or a few cultures",
    ),
    (
        "ifd-even",
        "Two equal sugar patches: the agents split evenly",
    ),
    (
        "ifd-two-to-one",
        "One patch yields twice as much, but draws fewer than twice the agents",
    ),
    (
        "ifd-four-to-one",
        "One patch yields four times as much, but draws under three times the agents",
    ),
    (
        "ifd-far-sighted",
        "Agents who can see across the gap come close to matching the yields",
    ),
    (
        "ifd-no-starving",
        "Nobody starves, and most agents never find sugar",
    ),
    (
        "ifd-wander",
        "Agents who see no sugar wander: nearly all find a patch, but the split strays from the yields",
    ),
    (
        "ifd-crowding",
        "Agents who avoid crowded sugar come closer to matching the yields",
    ),
    (
        "ifd-travel",
        "Agents who prefer nearby sugar stray further from matching the yields",
    ),
    (
        "walk-capacity",
        "Agents who walk instead of jump: fewer of them survive",
    ),
    (
        "walk-wealth",
        "Agents who walk are born and die, and wealth grows as lopsided as when they jump",
    ),
    (
        "walk-seasons",
        "Walking through the seasons: agents still migrate, but fewer of them",
    ),
    (
        "walk-waves",
        "Walking doesn't bring back the book's waves",
    ),
    (
        "walk-fast",
        "Agents who walk three steps a tick: most of the lost population comes back",
    ),
    (
        "ifd-fence",
        "A fence with a central gap: no clear change from no fence",
    ),
    (
        "ifd-fence-far",
        "The gap moves to the far end: a long walk to switch, and across patch sizes agents stray further from matching the yields",
    ),
    (
        "ifd-wall",
        "An opaque wall instead of a fence: no clear difference",
    ),
    (
        "mem-open",
        "Remembering on the open sugarscape: rememberers end up poorer",
    ),
    (
        "mem-catchment",
        "Wanderers who remember reach patches sooner but end slightly poorer",
    ),
    (
        "mem-walled",
        "Remembering beyond the wall: rememberers starve",
    ),
    (
        "mem-seasons",
        "Remembering the other hemisphere: rememberers end poorer",
    ),
    (
        "mem-truffles",
        "Hidden truffles: rememberers find over twice as many and still end poorer",
    ),
    (
        "mem-trapline",
        "Foragers who learn a route between truffle spots",
    ),
    (
        "mem-mvt",
        "When to leave a patch: foragers who find a rich one never leave",
    ),
    (
        "goap-mvt",
        "Planners who price travel stay longer when patches are farther apart",
    ),
    (
        "mvt-rule",
        "Leave when a patch falls below your average: stays lengthen only slightly with travel",
    ),
    (
        "goap-open",
        "Planners who remember end up richer on the open sugarscape",
    ),
    (
        "goap-truffles",
        "Planners who remember hidden truffles end up richer while they live",
    ),
    (
        "goap-walled",
        "Planners who remember what lies beyond the wall end up richer, and most survive",
    ),
    (
        "cache-winter-none",
        "Winter with nothing put away: half the population dies in the first winter",
    ),
    (
        "cache-winter-even",
        "Burying half of every surplus: three in four survive the first winter",
    ),
    (
        "cache-winter-compensate",
        "Burying more where food was scarce survives winter no better than an even share",
    ),
    (
        "cache-winter-plan",
        "Agents who plan for winter: nearly nine in ten survive the first, burying a sixth as much",
    ),
    (
        "cache-winter-mixed",
        "Four ways to face winter side by side: planners are a third of the survivors",
    ),
    (
        "central-near",
        "Loads carried home grow with the patches' distance from home",
    ),
    (
        "central-far",
        "Carrying loads home from a far patch: bigger loads, and half the trips fill up",
    ),
    (
        "central-linear",
        "A patch that never runs down: trips either turn back at once or fill up",
    ),
    (
        "cache-raby",
        "Raby's breakfast test: compensators and planners cache where breakfast was missing",
    ),
    (
        "cache-amodio",
        "Amodio's rotating compartments: each rule leaves its own signature, and the jays' looks like an even split",
    ),
    (
        "theft-winter",
        "Hoarders who can be robbed: theft spreads the stores, and more agents survive the winter",
    ),
    (
        "theft-winter-quarter",
        "A quarter never cache and live off the others' caches, outliving the hoarders",
    ),
    (
        "theft-winter-half",
        "Half never cache and outlive the hoarders, who dig back barely 1 % of what they bury",
    ),
    (
        "theft-arena-2",
        "Two agents in a room: the other takes half the hoarder's caches, and the hoarder ends the winter with a third as much in hand",
    ),
    (
        "theft-arena-4",
        "Four agents in a room: thieves take more than four fifths of the caches, and hoarders end the winter with a third as much in hand",
    ),
    (
        "theft-arena-8",
        "Eight agents in a room: thieves take nine tenths of the caches, and hoarders end the winter with two fifths as much in hand",
    ),
    (
        "watch-winter",
        "Everyone watches others bury: with a 7-tick memory fresh caches go twice as fast as by stumbling, and fewer survive the winter than without watching",
    ),
    (
        "watch-winter-stumble",
        "Watching and stumbling: fresh caches go three times as fast as by stumbling alone, and a quarter fewer survive the winter",
    ),
    (
        "watch-half",
        "Half never cache and everyone watches: nearly two hoarders in three die, against one cheater in six",
    ),
    (
        "watch-scroungers",
        "Half watch others bury: with a 7-tick memory watchers come out slightly ahead, and the more watch, the worse everyone does",
    ),
    (
        "watch-scroungers-only",
        "Half never cache but watch and steal: they trail the hoarders with or without watching, a little less with it",
    ),
    (
        "watch-scroungers-forgo",
        "Scroungers who forgo foraging: with a 7-tick memory they trail the hoarders further the more of them there are, mostly from scrounging rather than from never caching",
    ),
    (
        "watch-ak",
        "Owners who dig early, and everyone watching with a 7-tick memory: thieves now take more than owners dig back, and fewer hoarders survive",
    ),
    ("spatial-scatter", "Spatial scatter: fixed L=0, with no guard"),
    ("spatial-larder", "Spatial larder: fixed L=1, with no guard"),
    ("spatial-larder-guard", "Spatial larder: fixed L=1, with guarding"),
    (
        "watch-arena",
        "Four agents in a room, half watching: everyone survives, and the room ends the winter poorer",
    ),
    (
        "s71-board",
        "Wanting half your neighbors alike: four in five end up alike",
    ),
    (
        "s71-center-out",
        "Moving from the center out: a different town, just as sorted",
    ),
    (
        "s71-third",
        "Wanting a third alike: only slight sorting",
    ),
    (
        "s71-unequal-demands",
        "The more demanding color ends up more crowded, not more sorted",
    ),
    (
        "s71-minority",
        "A two-to-one minority ends up more sorted and more crowded",
    ),
    (
        "s71-wide",
        "Counting 24 neighbors while wanting half alike: more sorting, not less",
    ),
    (
        "s71-congregate",
        "Wanting company, not separation: four in five alike anyway",
    ),
    (
        "s71-integrate",
        "Wanting a mixed street: some can never be satisfied",
    ),
    (
        "s71-line",
        "Wanting half alike on a line: about seven clusters of ten",
    ),
    (
        "s71-line-3",
        "Counting three neighbors each side: clusters of about nine",
    ),
    (
        "s71-line-minority",
        "Halving the minority: it ends no more segregated",
    ),
    (
        "s71-line-reach",
        "Travel limited: most end content, a few never do",
    ),
    (
        "pv-flat",
        "Anyone may move to the square they like best: two clusters",
    ),
    (
        "pv-p50",
        "Liking some mix, up to half: still two clusters",
    ),
    (
        "pv-p100",
        "Liking half and half best: about five clusters",
    ),
    (
        "pv-spiked",
        "Only a perfect mix will do: the board stays mixed",
    ),
    (
        "pv-ring",
        "On a ring, even lovers of a mix end in two groups",
    ),
    (
        "gvn-frozen",
        "Too little tolerance: nobody can move",
    ),
    (
        "gvn-segregated",
        "Tolerating half unlike: two big clusters",
    ),
    (
        "gvn-mixed",
        "Tolerating most unlike: the board stays mixed",
    ),
    (
        "svw-small",
        "A small city: two clusters",
    ),
    (
        "svw-large",
        "The same rule in a big city: dozens of clusters",
    ),
    (
        "svw-t4",
        "Asking 4 of 8 alike in a big city: fewer, solid clusters",
    ),
    (
        "zhang-checkerboard",
        "Everyone wants a mix, yet from a perfect mix they sort",
    ),
    (
        "zhang-random",
        "Everyone wants a mix, yet they sort into blobs",
    ),
    (
        "tipping-fig18",
        "A mixed area with tolerance to spare still ends all one color",
    ),
    (
        "tipping-fig19",
        "Wider tolerance: the area settles at 80 and 80",
    ),
    (
        "tipping-fig20",
        "Twice as many of one color: the mixture is lost",
    ),
    (
        "tipping-fig21",
        "At the threshold of tolerance, a mix just holds",
    ),
    (
        "tipping-fig22",
        "Limit one color to 40, and a mixture holds",
    ),
    (
        "tipping-intolerant",
        "Make the least tolerant 60 % intolerant: a stable 40 and 40",
    ),
    (
        "tipping-minority",
        "A small minority no more tolerant than the rest is pushed out",
    ),
    (
        "tipping-less-tolerant",
        "Make the least tolerant less tolerant, and a mixture holds",
    ),
    (
        "vi-4-schelling-25",
        "Mild preferences, clear segregation: wanting a quarter alike is enough",
    ),
    (
        "vi-5-schelling-25-residence",
        "Mild preferences with turnover: the pattern never settles and segregation climbs",
    ),
    (
        "vi-6-schelling-50-residence",
        "Wanting half alike: the city sorts almost completely",
    ),
    (
        "vi-7-schelling-mixed",
        "Tolerant and less tolerant mixed: the tolerant can't undo segregation",
    ),
    (
        "vi-8-ring-world",
        "Foragers on a ring fall into flocks with no social rule at all",
    ),
    (
        "vi-9-ring-megagroup",
        "One big group on the ring breaks up into flocks",
    ),
    (
        "lhv-published",
        "The Anasazi of Long House Valley rise and fall, but don't vanish in 1300",
    ),
    (
        "lhv-published-defaults",
        "With the original defaults the valley fills to five times its recorded peak",
    ),
    (
        "lhv-documented",
        "The model as documented: households die out and the published curve is lost",
    ),
    (
        "cv-run-1-no-movement",
        "Citizens who can't move simmer in local outbursts",
    ),
    (
        "cv-run-2-punctuated",
        "Long calm, sudden outbursts, but only if arrest odds are rounded down",
    ),
    (
        "cv-run-3-salami",
        "Legitimacy erodes slowly and rebels are picked off one at a time (mostly)",
    ),
    (
        "cv-run-4-one-jump",
        "Legitimacy drops at once, and rebellion explodes about half the time",
    ),
    (
        "cv-run-5-cop-reductions",
        "Remove the police bit by bit and the society tips into rebellion",
    ),
    (
        "cv-run-6-coexistence",
        "Two ethnic groups, no police, and peace",
    ),
    (
        "cv-run-7-cleansing",
        "Two ethnic groups, no police: one wipes out the other",
    ),
    (
        "cv-run-8-nasty-regime",
        "Police were meant to keep both groups alive; here one still dies out",
    ),
    (
        "cv-safe-havens",
        "Peacekeepers arrive, but genocide still follows",
    ),
    (
        "cv-netlogo",
        "The NetLogo version, with all five of its departures from the paper",
    ),
    (
        "rca-published",
        "Cooperation without reciprocity: agents help others with similar tags",
    ),
    (
        "rca-literal",
        "Tag cooperation as the rules are written: coin-flip ties break the paper's tables",
    ),
    (
        "rca-published-p2",
        "Tag cooperation with fewer meetings: it never takes hold",
    ),
    (
        "rca-literal-p2",
        "Fewer meetings, coin-flip ties: 42 % donation where the paper reports 4 %",
    ),
    (
        "rca-strict",
        "Help only strictly within tolerance, and cooperation vanishes",
    ),
    (
        "rs-no-forced-clones",
        "Let agents refuse their own clones, and cooperation vanishes",
    ),
    (
        "eh-clones-only",
        "Help only exact copies: more cooperation, with tolerance doing nothing",
    ),
    (
        "eh-no-exact-clones",
        "No exact copies allowed: tolerance alone can't sustain cooperation",
    ),
    (
        "rca-adopt-p1",
        "Agents imitate the more successful instead of breeding: about half cooperate",
    ),
    (
        "ac-sample-run",
        "Neighbors copy each other's culture until the map freezes into a few regions",
    ),
    (
        "ac-many-regions",
        "Many traits, few features: dozens of cultures freeze apart",
    ),
    (
        "ac-large-territory",
        "A huge territory ends with almost one culture, after a billion events",
    ),
    (
        "ac-torus",
        "Culture on a torus: fewer regions than with edges",
    ),
    (
        "ac-random-activation-20",
        "One random site at a time, as Axelrod did it: about 16 regions",
    ),
    (
        "ac-sweep-activation",
        "Shuffled sweeps, as the Sugarscape does it: fewer regions from one small change",
    ),
    (
        "ac-neighbor-changes",
        "Who copies whom, reversed: slightly fewer regions",
    ),
    (
        "ac-soup",
        "Everyone can meet everyone: culture collapses to one",
    ),
    (
        "ac-drift",
        "A little random drift, and one culture spreads everywhere",
    ),
    (
        "aey-equity",
        "Bargainers settle on splitting the pie evenly",
    ),
    (
        "aey-fractious",
        "A fractious start was meant to last for ages; here it lasts a few periods",
    ),
    (
        "aey-transition",
        "Ten bargainers escape a fractious norm, sooner than the paper says",
    ),
    (
        "aey-tags",
        "Two meaningless tags: the paper's classes never form",
    ),
    (
        "aey-classes",
        "Start as a class system, and the classes last",
    ),
    (
        "pvplh-small-tags",
        "A small, forgetful society still forms no classes under the paper's rule",
    ),
    (
        "pvplh-mode",
        "Copy the most common demand instead, and classes appear in half the runs",
    ),
    (
        "pvplh-progressive",
        "Memories that start empty: no slower to equity",
    ),
    (
        "pvplh-lattice",
        "Two tags in two zones: each border settles on its own norm",
    ),
    (
        "hk-plurality",
        "Little confidence in others: opinions freeze into dozens of clusters",
    ),
    (
        "hk-polarisation",
        "Middling confidence: the paper's two camps, usually three here",
    ),
    (
        "hk-consensus",
        "Enough confidence in others, and everyone agrees",
    ),
    (
        "hk-regular-50",
        "Fifty evenly spread opinions split into two camps, exactly as published",
    ),
    (
        "hk-regular-plurality",
        "A hundred evenly spread opinions split into nine",
    ),
    (
        "hk-regular-consensus",
        "A hundred evenly spread opinions reach consensus",
    ),
    (
        "hk-asym-a",
        "Listening a little further to the right: opinion drifts right",
    ),
    (
        "hk-asym-b",
        "Listening much further to the right: a big camp forms near the right edge",
    ),
    (
        "hk-asym-c",
        "Listening further to the right: two lopsided camps, or consensus",
    ),
    ("hk-one-sided", "A one-sided split opens, then closes again"),
    (
        "hk-bias",
        "Each side listens outward, and the profile splits into two polarized camps",
    ),
    (
        "hk-serial",
        "One agent at a time instead of all at once: much the same outcome",
    ),
    (
        "hk-lattice",
        "Only neighbors listen: one big camp and stranded minorities, not two camps",
    ),
    (
        "cra-rwr",
        "Random partners every period: cooperation rarely takes hold",
    ),
    (
        "cra-2dk",
        "Neighbors on a torus: cooperation takes hold and stays",
    ),
    (
        "cra-frne",
        "Fixed random neighbors, both ways: cooperation even better than on a torus",
    ),
    (
        "cra-frn",
        "Fixed random partners alone are enough for cooperation",
    ),
    (
        "cra-ffr-01",
        "Swap a tenth of partners each period: cooperation mostly holds",
    ),
    (
        "cra-ffr-03",
        "Swap 30 % of partners: cooperation flips between high and low",
    ),
    (
        "cra-ffr-05",
        "Swap half the partners: cooperation collapses",
    ),
    (
        "cra-random-start",
        "Random starting strategies instead of an even spread: no difference",
    ),
    (
        "cra-copy-noise",
        "Noise only when copying: more cooperation than the paper's table",
    ),
    (
        "nm-1a-static",
        "The Prisoner's Dilemma on a grid: cooperators hold a static web against defectors",
    ),
    (
        "nm-1b-chaos",
        "A stronger temptation: cooperators and defectors churn in endless chaos",
    ),
    (
        "nm-2a-universal",
        "From any start, cooperation settles at 0.318",
    ),
    (
        "nm-3-kaleidoscope",
        "One defector among cooperators grows into a symmetric kaleidoscope",
    ),
    (
        "nm-no-self",
        "No playing against oneself: cooperation settles near 0.30",
    ),
    (
        "nm-four-neighbors",
        "Four neighbors instead of eight: cooperation near 0.38",
    ),
    (
        "hg-async-kaleidoscope",
        "The kaleidoscope updated one player at a time: everyone ends up defecting",
    ),
    (
        "nbm-probabilistic",
        "Winning in proportion to payoff: cooperators hold about 30 %",
    ),
    (
        "nbm-discrete",
        "Everyone updates at once: cooperators hold a mostly static 90 %",
    ),
    (
        "nbm-continuous",
        "Players update one at a time, and cooperators and defectors still coexist",
    ),
    (
        "nbm-random-array",
        "Players scattered at random: cooperation survives only if their reach is small",
    ),
    (
        "nbm-cube",
        "The Prisoner's Dilemma in 3D: spatial chaos in a cube",
    ),
    (
        "ha-standard",
        "Helping only your own color: ethnocentrism takes over the lattice",
    ),
    ("ha-figure-1", "Half the mutation, even more ethnocentrism"),
    (
        "ha-appendix-mutation",
        "The appendix's 5 % mutation: the lattice never sorts, so it must be a typo",
    ),
    (
        "ha-appendix-double-play",
        "Every helping decision made twice: five points more ethnocentrism",
    ),
    (
        "ha-java-five-colors",
        "Five colors instead of four, from a bug in the authors' code: no difference",
    ),
    (
        "ha-java-archive",
        "The authors' archived code as it actually runs: the same outcome",
    ),
    (
        "ha-egoist-start",
        "From a world of egoists, ethnocentrism still takes over",
    ),
    (
        "ha-cost-2",
        "Helping costs twice as much: ethnocentrism weakens",
    ),
    (
        "ha-cost-2-blind",
        "Color-blind agents at double cost: three times the paper's cooperation",
    ),
    (
        "ha-misperception",
        "Mistaking colors one time in ten: ethnocentrism still dominates",
    ),
    (
        "ha-each-color",
        "A choice for every color: most agents still favor their own",
    ),
    (
        "jansson-offspring-anywhere",
        "Offspring scattered at random: without kin nearby, cooperation collapses",
    ),
    (
        "jansson-tag-mutation-30",
        "Colors that mutate often stop signaling kinship, and altruists win",
    ),
    (
        "jansson-kin",
        "Favor family instead of color: kin favoritism wins, by less than published",
    ),
    (
        "jansson-kin-fixed",
        "Favor family, with the choice fixed for life: closer to the published numbers",
    ),
    (
        "hks-no-ethnocentrics",
        "Ban ethnocentrism, and humanitarians take over",
    ),
    (
        "dpd-run-1",
        "A Prisoner's Dilemma where agents breed and die: cooperators dominate",
    ),
    (
        "dpd-run-2",
        "Add old age: cooperators still dominate, but with twice the published defectors",
    ),
    (
        "dpd-run-3",
        "A smaller reward for cooperating: cooperators struggle, and some populations die out",
    ),
    (
        "dpd-run-4",
        "A tiny reward for cooperating: the published cycles end in extinction here",
    ),
    (
        "dpd-run-5",
        "Half of all offspring switch strategy: cooperation survives 10,000 cycles",
    ),
    (
        "dpd-working-paper",
        "The working paper's one game per turn: closer to the published table",
    ),
    (
        "dpd-closest",
        "Three unstated choices changed: both published tables match at once",
    ),
    (
        "dpd-soup",
        "Take away space so anyone meets anyone: defection wins",
    ),
    (
        "dpd-shifted",
        "Shifted payoffs were meant to bring pure defection; here cooperation persists",
    ),
    (
        "dpd-metabolism",
        "Shifted payoffs with a metabolism: equivalent only if charged per game",
    ),
    (
        "dpd-footnote-27",
        "Short lives were meant to bring a cooperative monopoly; here both strategies persist",
    ),
    (
        "dpd-rr-best",
        "A replication's best fit doesn't carry over to this one",
    ),
    (
        "dpd-coordination",
        "Driving on the left or the right: regions of each, with accidents at their borders",
    ),
    (
        "ax-norms",
        "The norms game: agents evolve how bold and how vengeful to be",
    ),
    (
        "ax-metanorms",
        "Punish those who fail to punish, and a norm against defection takes hold",
    ),
    (
        "ax-dominance",
        "A strong group and a weak one, no metanorms: both turn bold",
    ),
    (
        "ax-dominance-metanorms",
        "A strong group and a weak one, with metanorms: both are kept from boldness",
    ),
    (
        "gi-metanorms-long",
        "Run the metanorms game longer, and the norm slowly collapses",
    ),
    (
        "gi-low-mutation",
        "Less mutation: the norm collapses much sooner",
    ),
    (
        "gi-mild-metanorms",
        "Milder punishment for not punishing: the norm quickly collapses",
    ),
    ("gi-temptation-10", "More temptation keeps the norm alive"),
    (
        "gi-tournament",
        "A different selection rule, and the norm collapses",
    ),
    (
        "dnaw-consensus",
        "Pairs meet and meet in the middle: everyone ends up agreeing",
    ),
    (
        "dnaw-clusters",
        "Pairs meet only if they already half agree: two opinion clusters form",
    ),
    (
        "dnaw-lattice",
        "Opinions on a lattice: one big cluster and scattered holdouts, eventually",
    ),
    (
        "dnaw-lattice-clusters",
        "Opinions on a lattice with less tolerance: many small local clusters",
    ),
    (
        "ra-uniform",
        "Relative agreement without extremists: opinions settle into a few clusters",
    ),
    (
        "ra-central",
        "Confident extremists at both ends: most of the population stays central",
    ),
    (
        "ra-both",
        "Uncertain moderates split between the two extremes",
    ),
    (
        "ra-single",
        "The paper's single-extreme run: at its stated speed, both extremes instead",
    ),
    (
        "ra-literal",
        "A few extremists pull everyone to one extreme, as the paper states it",
    ),
    (
        "ra-meadows-cliff",
        "Measured too early, cut too strict: Meadows and Cliff see no extremists",
    ),
    (
        "ra-deffuant-2013",
        "The authors' reply: run longer, count looser, and the single extreme appears",
    ),
    (
        "ra-bc-extremists",
        "Bounded confidence with extremists: everyone drifts to one extreme",
    ),
    (
        "ra-bc-printed",
        "The same rule as printed in the paper: extremists get pulled to the center",
    ),
    (
        "ad-moore",
        "Extremists on a lattice: the population splits between both ends",
    ),
    (
        "ad-small-world",
        "Extremists in a small world: one extreme or the center, run by run",
    ),
    (
        "w-scale-free",
        "Opinions on a scale-free network: two clusters and many who never budge",
    ),
    (
        "ns-fig-1",
        "Help those with a good image: cooperation wins in some runs, defection in most",
    ),
    (
        "ns-fig-2",
        "With mutation, cooperation collapses and recovers in endless cycles",
    ),
    (
        "ns-fig-3-n20",
        "Reputations seen by a few, in a group of 20: cooperation holds",
    ),
    (
        "ns-fig-3-n50",
        "Reputations seen by a few, in a group of 50: cooperation half gone",
    ),
    (
        "ns-fig-3-n100",
        "Reputations seen by a few, in a group of 100: cooperation mostly gone",
    ),
    (
        "ns-fig-4a",
        "Help if they're good and you need it: cooperation in half the rounds",
    ),
    (
        "ns-fig-4b",
        "Help if they're good and you need it, a few watching: still about half",
    ),
    (
        "ns-fig-4c",
        "Help if they're good or you need it: most rounds cooperative",
    ),
    (
        "ns-fig-4d",
        "Help if they're good or you need it, a few watching: more cooperation still",
    ),
    (
        "ns-own-only",
        "Only your own image counts: cooperation all but vanishes",
    ),
    (
        "ns-no-offset",
        "Without the paper's payoff offset, cooperation does better",
    ),
    (
        "lh-fig-1a",
        "Help only to mend your own image: this strategy invades",
    ),
    (
        "lh-fig-1b",
        "The same with mistakes: it invades, far more slowly",
    ),
    (
        "lh-fig-2a",
        "Image scoring in one group: cooperation in over a third of rounds",
    ),
    (
        "lh-fig-2b",
        "Many groups, little drift: image scoring was meant to fade; here it persists",
    ),
    (
        "lh-fig-2c",
        "More migration between groups: cooperation falls, far less than published",
    ),
    (
        "lh-fig-3a",
        "A cheaper cost of helping: cooperation in about half the rounds",
    ),
    (
        "lh-fig-3b",
        "Add strategies that calculate whether helping pays: less cooperation",
    ),
    (
        "lh-fig-4a",
        "Standing beats image scoring: refusing the undeserving costs nothing",
    ),
    ("lh-fig-4b", "Standing still wins despite misperceptions"),
    (
        "lh-fig-4c",
        "All four strategies from the start: standing dominates",
    ),
    (
        "ef-arthur",
        "Who goes to the bar? Attendance averages 60 but swings far more than chance",
    ),
    (
        "ef-payoff",
        "Rate predictors by their advice, not their accuracy: the cycle goes",
    ),
    (
        "ef-random",
        "Coin-flippers also average 60, with far smaller swings",
    ),
    (
        "ef-shared",
        "Everyone holds the same predictors, and nobody is ever right",
    ),
    ("mg-m6", "Two sides, the minority wins: short memories"),
    ("mg-m8", "Two sides, the minority wins: longer memories"),
    ("mg-m10", "Two sides, the minority wins: long memories"),
    (
        "mg-mixed",
        "Long and short memories play together: the longer win, up to about six",
    ),
    (
        "mg-inverse",
        "Win more the smaller the minority: the paper's two peaks don't appear",
    ),
    (
        "mg-evolution",
        "The worst player is replaced by a mutated copy of the best",
    ),
    (
        "mg-inbred",
        "Perfect copies of the best player, and no mutation",
    ),
    (
        "mg-arms-race",
        "Memories that can grow: an arms race that levels off",
    ),
    ("mg-crowded", "Too little memory: worse than coin flips"),
    ("mg-critical", "Just enough memory: the best coordination"),
    ("mg-random-like", "Too much memory: no better than chance"),
    (
        "cmo-binary",
        "El Farol as a yes-or-no game: 60 seats, two rounds of memory",
    ),
    (
        "ants-1a",
        "Two identical sources, and most of the colony crowds one",
    ),
    (
        "ants-1b",
        "Every split of the colony between two sources is about equally likely",
    ),
    ("ants-1c", "Weak recruiting: the ants spread about evenly"),
    ("ants-2a", "The colony wanders around half and half"),
    (
        "ants-2b",
        "Nearly all the ants at one source, then a sudden flip",
    ),
    (
        "ants-crowd",
        "Ten times the ants, the same habits, and the herding is gone",
    ),
    (
        "ants-becker",
        "Following the crowd pays: the colony settles about 80–20, like the real ants",
    ),
    (
        "ants-lock",
        "Following the crowd pays too well: one source, forever",
    ),
    (
        "ants-three",
        "Three sources: one holds the colony for a while, then another",
    ),
    (
        "am-ring",
        "Herding over a ring: weaker than the theory predicts",
    ),
    (
        "am-random",
        "Herding over a random network: just as the theory predicts",
    ),
    (
        "am-scale-free",
        "Herding over a network with hubs: close to the theory",
    ),
    (
        "am-independent",
        "Five ants in a hundred who ignore everyone calm the whole colony",
    ),
    (
        "gr-uniform",
        "One instigator, and all 100 riot",
    ),
    (
        "gr-perturbed",
        "Move one person up one notch, and only the instigator riots",
    ),
    (
        "gr-normal-12",
        "Mean threshold 25, spread 12: a handful riot",
    ),
    (
        "gr-normal-13",
        "Mean threshold 25, spread 13: nearly everyone riots",
    ),
    (
        "gr-normal-sampled",
        "The same crowd drawn from real people: no sharp tipping point",
    ),
    (
        "gr-city",
        "Crowds drawn from a city that should riot: half end with no rioter or one",
    ),
    (
        "gr-friends",
        "Count friends double, and the crowd that should riot mostly doesn't",
    ),
    (
        "gr-friends-perturbed",
        "Close friends rescue the stalled crowd, now and then",
    ),
    (
        "gr-ceilings",
        "Join a crowd, leave a mob: the riot builds and collapses",
    ),
    (
        "gr-clusters",
        "Ten crowds with people drifting between them",
    ),
    (
        "watts-lower",
        "Few links: mostly small cascades, now and then a large one",
    ),
    (
        "watts-middle",
        "A middling network: most sparks spread everywhere",
    ),
    (
        "watts-upper",
        "Many links: almost never, then everything",
    ),
    (
        "watts-hetero",
        "Varied thresholds keep dense networks cascading",
    ),
    (
        "watts-hub",
        "Light the best-connected node",
    ),
    (
        "ae-rapid",
        "15 % decide rationally, and retiring at 65 sets in within a few years",
    ),
    (
        "ae-base",
        "A tenth decide rationally, and retiring at 65 takes hold in about 16 years",
    ),
    (
        "ae-slow",
        "5 % rational: retiring at 65 spreads slowly, up from the old",
    ),
    (
        "ae-policy",
        "Congress lowers the age to 62: here the new norm comes in a few years",
    ),
    (
        "ae-groups",
        "Two communities, one with no rational agents, loosely linked",
    ),
    (
        "ae-all-members",
        "Count every friend, not just the eligible, and no norm ever forms",
    ),
    (
        "ae-replace",
        "Replace friends who die, and 5 % rationality is no longer enough",
    ),
    (
        "bg-base",
        "Punishers keep about 70 % of groups of 32 cooperating",
    ),
    (
        "bg-either",
        "Either group can start a war, and the paper's figures appear",
    ),
    (
        "bg-none",
        "Without punishment, groups of 32 fall to defection",
    ),
    (
        "bg-large",
        "Groups of 128: punishment no longer holds",
    ),
    (
        "bg-weak",
        "A fine only twice the cost, and cooperation fades",
    ),
    (
        "bg-fixed",
        "Punishers who pay whether or not anyone defects, and cooperation fails",
    ),
    (
        "bg-mixing",
        "More mixing between groups, and cooperation falls",
    ),
    (
        "bg-benefit",
        "Cooperation benefits the group, and groups fight over payoffs",
    ),
    (
        "bg-continuous",
        "Cooperate and punish by degrees, and nearly everyone cooperates",
    ),
    (
        "bg-ring",
        "A ring of groups with no wars: cooperation stays low",
    ),
    (
        "bg-janssen",
        "Janssen's readings of the gaps together",
    ),
    (
        "gs-1",
        "Random traders who never take a loss capture nearly all the surplus",
    ),
    (
        "gs-2",
        "Market 2: random traders who never lose reach 99.8 % efficiency",
    ),
    (
        "gs-3",
        "Market 3: six units worth trading, and random traders find them",
    ),
    (
        "gs-4",
        "Market 4: loss-averse random traders capture 99.5 %; free ones, half",
    ),
    (
        "gs-5",
        "Market 5: marginal traders crowd equilibrium, and efficiency dips to 97 %",
    ),
    (
        "gs-1-u",
        "Traders free to lose money: 90 % efficiency and prices everywhere",
    ),
    (
        "gs-4-u",
        "Free to lose money in market 4, traders capture less than half",
    ),
    (
        "cliff-symmetric",
        "Symmetric supply and demand: random prices center on equilibrium",
    ),
    (
        "cliff-flat",
        "Flat supply: random prices settle well above equilibrium",
    ),
    (
        "cliff-excess-demand",
        "Too many buyers: random prices sit far below equilibrium",
    ),
    (
        "cliff-excess-supply",
        "Too many sellers: random prices sit far above equilibrium",
    ),
    (
        "zip-symmetric",
        "Traders who learn a margin converge on the equilibrium price",
    ),
    (
        "zip-flat",
        "Learning traders reach equilibrium within days when supply is flat",
    ),
    (
        "zip-excess-demand",
        "Where buyers abound, learning traders climb to equilibrium from below",
    ),
    (
        "zip-excess-supply",
        "Where sellers abound, learning traders fall to equilibrium from above",
    ),
    (
        "zip-demand-shift",
        "Demand jumps after day 10, and learning traders follow it",
    ),
    (
        "zip-supply-shift",
        "Supply drops after day 10, and learning traders follow it",
    ),
    (
        "zip-retail",
        "Only sellers post prices, and trades stay below equilibrium",
    ),
    (
        "lk-random",
        "Farmers copying their best neighbor's planting double the rice harvest",
    ),
    (
        "lk-random-fixed",
        "Random plans that never change reap half as much",
    ),
    (
        "lk-traditional",
        "Traditional rice gains from copying neighbors as fallows line up",
    ),
    (
        "lk-hyv",
        "High-yielding rice gains a little from copying neighbors",
    ),
    (
        "lk-perturbed",
        "Pests and drought strike in year 21, and the harvest never recovers",
    ),
    (
        "lk-stressed",
        "Pests and drought from the start hold the harvest lower",
    ),
    (
        "lk-temples",
        "One random plan per temple, fixed, reaps far less than copying",
    ),
    (
        "janssen-code",
        "Water routed as Janssen's code does barely changes the harvest",
    ),
    (
        "janssen-levels-14",
        "The best plan for each temple's subaks, found by search",
    ),
    (
        "janssen-two-node",
        "Two subaks on one river share water and pests",
    ),
    (
        "janssen-generalized",
        "Copying good farmers anywhere, discounted by distance, beats copying neighbors",
    ),
    (
        "janssen-adaptive",
        "Farmers plant when water is ample and pests are low",
    ),
    (
        "janssen-fewer-links",
        "With half the pest links gone, copying stalls early",
    ),
    (
        "hoard-threshold",
        "At the tipping point: larders take over in about two runs of five",
    ),
    (
        "hoard-scatter",
        "Scattered caches stay hidden: larders never take over",
    ),
    (
        "hoard-larder",
        "Conspicuous caches: larders take over almost every time, and half the agents die each season",
    ),
    (
        "hoard-no-free-recovery",
        "Owners find their own caches half the time, and larders still take over in about half the runs",
    ),
    (
        "hoard-cheaters",
        "A quarter never cache, and are gone after one generation",
    ),
    (
        "firms-base",
        "Workers team up, free-ride and scatter: firms stay small",
    ),
    (
        "firms-live",
        "Seeing others' current effort lets bigger firms form",
    ),
    (
        "firms-uniform",
        "Activating each worker once a period changes little",
    ),
    (
        "firms-beta-17",
        "Weaker returns to teamwork keep every firm small",
    ),
    (
        "firms-beta-21",
        "Stronger returns to teamwork grow firms of hundreds",
    ),
    (
        "firms-b-15",
        "A bigger team bonus grows bigger firms",
    ),
    (
        "firms-b-random",
        "Random firm technology grows bigger firms too",
    ),
    (
        "firms-theta-075",
        "Identical workers form many mid-sized firms and are happier",
    ),
    (
        "firms-friends-10",
        "More friends spread workers over mid-sized firms",
    ),
    (
        "firms-random-firms-10",
        "Shopping among random firms keeps firms mid-sized",
    ),
    (
        "firms-loyal-10",
        "Loyal workers keep firms alive about nine times longer",
    ),
    (
        "firms-sticky",
        "Slow-changing effort swallows everyone into one firm",
    ),
    (
        "firms-groping",
        "Trial-and-error effort swallows everyone into one firm",
    ),
    (
        "firms-seniority-5",
        "Paying founders most keeps every firm to four or fewer",
    ),
    (
        "firms-base-pay-80",
        "Guaranteed pay collapses effort, and firms pay out about three times what they make",
    ),
    (
        "firms-hiring-100",
        "Hiring only the as-eager keeps firms small",
    ),
    (
        "firms-random-choices",
        "Random moves never build a big firm",
    ),
    (
        "firms-2013",
        "Axtell's 2013 settings give Zipf's law, with brief giant firms",
    ),
    ("auctions-first-price", "Two bidders learn how much to pay"),
    ("auctions-second-price", "The winner pays the other bidder’s bid"),
    ("auctions-feedback", "Bidders learn from every bid they could have made"),
    ("auctions-unused-feedback", "Bidders receive information they do not use"),
    ("auctions-local", "Bidders try the next bid up or down"),
    ("auctions-biased", "Bidders start with a preference for low bids"),
    ("auctions-downward", "Bidders are nudged toward lower bids"),
    ("auctions-nonparticipation", "Bidders can choose to sit out"),
    ("auctions-reserve", "The seller sets a minimum bid"),
    ("auctions-three", "Three bidders learn together"),
    ("auctions-three-patient", "Three bidders put more weight on future rewards"),
    ("auctions-fringe", "A random bid joins the auction"),
    ("auctions-persistent", "Bidders keep experimenting for a hundred million auctions"),
    (
        "collusion-calvano",
        "Two pricing algorithms learn to keep prices high, but not as a best response",
    ),
    (
        "collusion-code",
        "The authors' own sessions, period for period",
    ),
    (
        "collusion-no-memory",
        "Algorithms that remember nothing price even higher",
    ),
    (
        "collusion-myopic",
        "Algorithms that ignore the future still price above Nash",
    ),
    (
        "collusion-two-phase",
        "Explore at random, then never: prices settle above Nash, but not where Lambin's theorem says",
    ),
    (
        "collusion-synchronous",
        "Learning from every price brings prices down",
    ),
    (
        "collusion-explore-more",
        "Ten times slower exploration decay barely lowers prices",
    ),
    (
        "collusion-every-price",
        "Prices are cut after a rival's price rise too",
    ),
    (
        "collusion-invitation",
        "An invitation to raise prices is met with a price cut",
    ),
    (
        "collusion-below-nash",
        "On a grid with no room above Nash, prices settle at the top",
    ),
    ("geosim-paper", "Technology and contested fronts shape the size of wars"),
    ("geosim-no-technology", "Fixed logistical limits constrain conflict cascades"),
    ("geosim-no-context", "States contemplate attacks without neighborhood alerts"),
    ("geosim-smaller-shocks", "Slower logistical expansion changes conflict clusters"),
    ("geosim-artifact-2017", "An archived GeoSim2 bundle exposes different source readings"),
    ("polarity-original", "Predators contest a world of independent governments"),
    ("polarity-defense", "A stronger defense changes the struggle for territory"),
    ("polarity-alliances", "Threatened neighbors can form defensive coalitions"),
    ("polarity-pra", "Governments direct resources toward hostile fronts"),
    ("polarity-two-level", "Provincial revolts challenge expanding governments"),
    ("polarity-overextension", "Uncertain victories can limit territorial expansion"),

];

/// The title of preset `id`, or "" if it has none.
pub fn title(id: &str) -> &'static str {
    TITLES.iter().find(|(k, _)| *k == id).map_or("", |(_, t)| t)
}
