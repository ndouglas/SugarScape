"""Loads a frame dump written by `sugarscape shot` (format 1): a Sugarscape
shot as a `Dump`, a spatial-games shot as a `Lattice`, and a demographic-PD
shot and an ethnocentrism shot as `Dump`s too (see `_dpd` and `_ethno`), a
tags shot as a `Ring`, an image-scoring shot as a `Street`, a norms shot as a
`Plane`, and a social-structure shot as a `Grid`."""

import json
from dataclasses import dataclass, field

FORMAT = 1


@dataclass(frozen=True)
class Agent:
    id: int
    x: int
    y: int
    sugar: float
    age: int
    vision: int
    metabolism: int


@dataclass(frozen=True)
class Frame:
    tick: int
    agents: dict
    sugar: list
    deaths: dict
    born: list
    pollution: list
    births: dict
    tags: dict = field(default_factory=dict)
    groups: dict = field(default_factory=dict)
    spice: list = field(default_factory=list)
    spice_agents: dict = field(default_factory=dict)
    trades: list = field(default_factory=list)
    kills: list = field(default_factory=list)
    loans: list = field(default_factory=list)
    fertility: dict = field(default_factory=dict)
    diseases: dict = field(default_factory=dict)
    infections: list = field(default_factory=list)
    # id → kind, as a letter (E, H, S, T): ethnocentrism's strategies.
    kinds: dict = field(default_factory=dict)
    # id → the site's traits, as a tuple: Axelrod's culture model.
    traits: dict = field(default_factory=dict)


@dataclass(frozen=True)
class Trade:
    """One tick's exchanges between two agents, merged: `sugar_giver` gave
    `sugar` and received `spice` from `spice_giver` in `exchanges` trades."""

    sugar_giver: int
    spice_giver: int
    sugar: float
    spice: float
    exchanges: int

    @property
    def price(self):
        """Spice per sugar, over the tick's exchanges."""
        return self.spice / self.sugar


@dataclass(frozen=True)
class Kill:
    """One kill under rule C: `attacker` took `victim`'s site and `loot` of
    its sugar."""

    attacker: int
    victim: int
    loot: float


@dataclass(frozen=True)
class Loan:
    """A loan outstanding under rule L: `borrower` owes `lender` `due` at
    `due_tick` (settled in the step after the frame of that tick)."""

    lender: int
    borrower: int
    due: float
    due_tick: int


@dataclass(frozen=True)
class Infection:
    """One infection under rule E: `infector` (None for an outbreak) gave
    `disease` to `infected`."""

    infector: int | None
    infected: int
    disease: int


@dataclass(frozen=True)
class Dump:
    seed: int
    ticks: int
    width: int
    height: int
    capacity: list
    placed: list
    config: dict
    frames: list
    stats: dict
    spice_capacity: list = field(default_factory=list)
    model: str = "sugarscape"
    # Axelrod cultures (traits tuple) still present in the last frame, ranked
    # 1… by how many hold them (ties by the traits); empty without cultures.
    culture_rank: dict = field(default_factory=dict)
    # Bounded confidence: each agent's rank by starting opinion (0 the lowest).
    start_rank: dict = field(default_factory=dict)


def _culture_rank(last):
    """Rank each culture in `last` (id → traits) by how many hold it."""
    sizes = {}
    for c in last.values():
        sizes[c] = sizes.get(c, 0) + 1
    return {c: k + 1 for k, c in enumerate(sorted(sizes, key=lambda c: (-sizes[c], c)))}


@dataclass(frozen=True)
class Track:
    """One agent's life: `cells[k]` and `sugar[k]` at tick `first + k`;
    `death` is the tick whose frame lists its death (None if it survives)."""

    id: int
    first: int
    cells: list
    sugar: list
    death: int | None
    cause: str | None


@dataclass(frozen=True)
class LatticeFrame:
    """A spatial-games generation: each square's player, row-major, as `C`,
    `D` or `.` (none), and each player's score (when the shot asked)."""

    tick: int
    strategies: str
    scores: list = field(default_factory=list)


@dataclass(frozen=True)
class Lattice:
    """A spatial-games shot."""

    seed: int
    ticks: int
    width: int
    height: int
    config: dict
    frames: list
    stats: dict

    def frame(self, tick):
        """The frame at `tick`, rounded and clamped to the shot."""
        return self.frames[min(max(int(round(tick)), 0), self.ticks)]


@dataclass(frozen=True)
class Tagger:
    """One agent of a tags generation, at its place in the population's list."""

    id: int
    parent: int
    tag: float
    tolerance: float
    given: int
    received: int


@dataclass(frozen=True)
class RingFrame:
    """A tags generation: its agents in list order, and its gifts as
    (giver, receiver) places (when the shot recorded them)."""

    tick: int
    agents: list
    gifts: list = field(default_factory=list)


@dataclass(frozen=True)
class Ring:
    """A tags shot."""

    seed: int
    ticks: int
    config: dict
    frames: list
    stats: dict

    def frame(self, tick):
        return self.frames[min(max(int(round(tick)), 0), self.ticks)]


@dataclass(frozen=True)
class StreetFrame:
    """An image-scoring generation after it played: each agent as (id, k or
    None, score, payoff) in list order, and its meetings as (donor,
    recipient, helped) places (when the shot recorded them)."""

    tick: int
    agents: list
    meetings: list = field(default_factory=list)


@dataclass(frozen=True)
class Street:
    """An image-scoring shot."""

    seed: int
    ticks: int
    config: dict
    frames: list
    stats: dict

    def frame(self, tick):
        return self.frames[min(max(int(round(tick)), 0), self.ticks)]


@dataclass(frozen=True)
class PlaneFrame:
    """A norms generation after it played: its true generation, each agent
    as (boldness, vengefulness, payoff, parent place or None), and its events
    by places (when the shot recorded them)."""

    tick: int
    agents: list
    cheats: list = field(default_factory=list)
    punishments: list = field(default_factory=list)
    metapunishments: list = field(default_factory=list)


@dataclass(frozen=True)
class Plane:
    """A norms shot. Frames may be every `every`th generation: `ticks` counts
    frames (what a beat's timing steps through), and each frame keeps its
    true generation."""

    seed: int
    ticks: int
    every: int
    config: dict
    frames: list
    stats: dict

    def frame(self, tick):
        return self.frames[min(max(int(round(tick)), 0), self.ticks)]


@dataclass(frozen=True)
class GridFrame:
    """A social-structure period: each agent's (y, p, q, score) as played,
    and each agent's partners (when the shot recorded them)."""

    tick: int
    agents: list
    partners: list = field(default_factory=list)


@dataclass(frozen=True)
class Grid:
    """A social-structure shot; `site` gives each agent's torus square (empty
    for the other structures)."""

    seed: int
    ticks: int
    config: dict
    site: list
    frames: list
    stats: dict

    def frame(self, tick):
        return self.frames[min(max(int(round(tick)), 0), self.ticks)]


def parse(text):
    raw = json.loads(text)
    if raw.get("format") != FORMAT:
        raise ValueError(f"frame dump format {raw.get('format')!r}, expected {FORMAT}")
    if raw.get("model") == "dpd":
        return _dpd(raw)
    if raw.get("model") in ("schelling", "line"):
        return _schelling(raw)
    if raw.get("model") == "culture":
        return _culture(raw)
    if raw.get("model") == "opinions":
        return _opinions(raw)
    if raw.get("model") == "tipping":
        return _tipping(raw)
    if raw.get("model") == "ethno":
        return _ethno(raw)
    if raw.get("model") == "structure":
        return Grid(seed=raw["seed"], ticks=raw["ticks"], config=raw["config"], site=raw["site"], stats=raw["stats"],
                    frames=[GridFrame(f["tick"], [tuple(a) for a in f["agents"]], f.get("partners", []))
                            for f in raw["frames"]])
    if raw.get("model") == "norms":
        frames = [PlaneFrame(f["tick"], [tuple(a) for a in f["agents"]], f.get("cheats", []),
                             [tuple(x) for x in f.get("punishments", [])], [tuple(x) for x in f.get("metapunishments", [])])
                  for f in raw["frames"]]
        return Plane(seed=raw["seed"], ticks=len(frames) - 1, every=raw["every"], config=raw["config"], frames=frames,
                     stats=raw["stats"])
    if raw.get("model") == "image":
        return Street(
            seed=raw["seed"], ticks=raw["ticks"], config=raw["config"], stats=raw["stats"],
            frames=[StreetFrame(f["tick"], [tuple(a) for a in f["agents"]], [tuple(m) for m in f.get("meetings", [])])
                    for f in raw["frames"]],
        )
    if raw.get("model") == "tags":
        return Ring(
            seed=raw["seed"], ticks=raw["ticks"], config=raw["config"], stats=raw["stats"],
            frames=[RingFrame(f["tick"], [Tagger(*a) for a in f["agents"]], [tuple(g) for g in f.get("gifts", [])])
                    for f in raw["frames"]],
        )
    if raw.get("model") == "spatial":
        return Lattice(
            seed=raw["seed"], ticks=raw["ticks"], width=raw["width"], height=raw["height"], config=raw["config"],
            frames=[LatticeFrame(f["tick"], f["strategies"], f.get("scores", [])) for f in raw["frames"]],
            stats=raw["stats"],
        )
    frames = [
        Frame(
            tick=f["tick"],
            agents={row[0]: Agent(*row) for row in f["agents"]},
            sugar=f["sugar"],
            deaths=dict(f["deaths"]),
            born=f["born"],
            # Dumps before pollution was recorded have none.
            pollution=f.get("pollution") or [0.0] * len(f["sugar"]),
            # id → (sex, (parent, parent) or None); dumps before births were
            # recorded know neither.
            births=(
                {i: (sex, tuple(ps) if ps else None) for i, sex, ps in f["births"]}
                if "births" in f
                else {i: (None, None) for i in f["born"]}
            ),
            # id → tag bit string and id → group (tribe); empty in older dumps.
            tags=dict(zip((row[0] for row in f["agents"]), f.get("tags", []))),
            groups=dict(zip((row[0] for row in f["agents"]), f.get("groups", []))),
            # Two-good worlds only: spice per site, id → (spice, spice
            # metabolism), and the tick's trades by pair.
            spice=f.get("spice", []),
            spice_agents={row[0]: tuple(s) for row, s in zip(f["agents"], f.get("spice_agents", []))},
            trades=[Trade(*t) for t in f.get("trades", [])],
            kills=[Kill(*k) for k in f.get("kills", [])],
            loans=[Loan(*l) for l in f.get("loans", [])],
            # id → (fertility onset, end), for this frame's newcomers.
            fertility={i: (on, end) for i, on, end in f.get("fertility", [])},
            # id → diseases carried, and the tick's infections (disease on).
            diseases=dict(zip((row[0] for row in f["agents"]), f.get("diseases", []))),
            infections=[Infection(*i) for i in f.get("infections", [])],
            # id → Axelrod traits (rule K Axelrod's only).
            traits={row[0]: tuple(c) for row, c in zip(f["agents"], f.get("cultures", []))},
        )
        for f in raw["frames"]
    ]
    return Dump(
        seed=raw["seed"],
        ticks=raw["ticks"],
        width=raw["width"],
        height=raw["height"],
        capacity=raw["capacity"],
        placed=raw["placed"],
        config=raw["config"],
        frames=frames,
        stats=raw["stats"],
        spice_capacity=raw.get("spice_capacity", []),
        culture_rank=_culture_rank(frames[-1].traits) if frames and frames[-1].traits else {},
    )


def _dpd(raw):
    """A demographic-PD shot as a `Dump`, so the crowd animates as the
    Sugarscape's does: each agent's wealth stands in its `sugar`, its
    strategy in its group (0 a helper, 1 a cheat), and each clone's parent in
    both its parents' places; the board has no sugar. `placed` lists the
    founders, so a beat's `focus` can follow them."""
    w, h = raw["width"], raw["height"]
    frames, seen = [], set()
    for f in raw["frames"]:
        ids = [row[0] for row in f["agents"]]
        born = [i for i in ids if i not in seen]
        seen.update(born)
        parents = {child: parent for child, parent in f["births"]}
        frames.append(Frame(
            tick=f["tick"],
            agents={i: Agent(i, x, y, wealth, age, 0, 0) for i, x, y, wealth, age, _ in f["agents"]},
            sugar=[0.0] * (w * h),
            deaths=dict(f["deaths"]),
            born=born,
            pollution=[0.0] * (w * h),
            births={i: (None, (parents[i], parents[i]) if i in parents else None) for i in born},
            groups={row[0]: 0 if row[5] == "C" else 1 for row in f["agents"]},
        ))
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=sorted(frames[0].agents), config=raw["config"], frames=frames, stats=raw["stats"], model="dpd",
    )


def _schelling(raw):
    """A Schelling shot (his board, or his line laid out as a row of squares)
    as a `Dump`, so the Flumps walk as the Sugarscape's do: each Flump's color
    in its group (1 Red, his stars; 0 Blue, his zeros, as the "strategy"
    colors draw them) and whether it is content in its `sugar` (1 or 0). Nobody
    is born or dies."""
    w, h = raw["width"], raw["height"]
    # A strided shot (`every`) is filmed a frame a tick: frames count as ticks.
    frames = [
        Frame(
            tick=k,
            agents={i: Agent(i, x, y, 1.0 if content else 0.0, 0, 0, 0) for i, x, y, _, content in f["agents"]},
            sugar=[0.0] * (w * h),
            deaths={},
            born=[],
            pollution=[0.0] * (w * h),
            births={},
            groups={row[0]: 1 if row[3] else 0 for row in f["agents"]},
        )
        for k, f in enumerate(raw["frames"])
    ]
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=sorted(frames[0].agents), config=raw["config"], frames=frames, stats=raw["stats"], model=raw["model"],
    )


def _culture(raw):
    """An Axelrod culture shot as a `Dump`: a still Flump on every site (id =
    the site, row-major), its traits in `traits`. `groups` colors it: the
    cultures still present in the last frame are ranked by how many sites
    hold them (1 the most, ties by the traits) and wear that rank wherever
    they appear; any other culture is 0. Frames count as ticks (a shot may
    film every `every`th step)."""
    w, h, f = raw["width"], raw["height"], raw["features"]

    def sites(frame):
        t = frame["traits"]
        return {i: tuple(t[i * f:(i + 1) * f]) for i in range(w * h)}

    rank = _culture_rank(sites(raw["frames"][-1]))
    frames = []
    for k, fr in enumerate(raw["frames"]):
        traits = sites(fr)
        frames.append(Frame(
            tick=k,
            agents={i: Agent(i, i % w, i // w, 1.0, 0, 0, 0) for i in range(w * h)},
            sugar=[0.0] * (w * h), deaths={}, born=[], pollution=[0.0] * (w * h), births={},
            groups={i: rank.get(c, 0) for i, c in traits.items()},
            traits=traits,
        ))
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=list(range(w * h)), config=raw["config"], frames=frames, stats=raw["stats"], model=raw["model"],
        culture_rank=rank,
    )


# Bounded confidence on the felt: opinion 0…1 across this many bins, or
# half as many for a small crowd (under SMALL_CROWD agents).
OPINION_COLUMNS = 40
SMALL_CROWD = 200
# Each opinion bin is a block this many Flumps wide.
BLOCK = 3
# Colors by start, red at 0 to magenta at 1 (Hegselmann & Krause's figures).
START_BINS = 10


def _opinions(raw):
    """A bounded-confidence shot as a `Dump`, a histogram lying on the felt:
    each agent a Flump in its opinion's bin (one of OPINION_COLUMNS across),
    the bin a block BLOCK wide filled row by row from the front in order of
    where they started, so the start is an even carpet and a camp a big
    block. The board is as deep as the biggest block. `groups` is the start's tenth (0 red … 9 magenta). Frames
    count as ticks (a shot may film every `every`th period)."""
    n = raw["agents"]
    bins_across = OPINION_COLUMNS if n >= SMALL_CROWD else OPINION_COLUMNS // 2
    w = bins_across * BLOCK
    starts = raw["starts"]
    rank = {i: r for r, i in enumerate(sorted(range(n), key=lambda i: (starts[i], i)))}
    groups = {i: min(int(starts[i] * START_BINS), START_BINS - 1) for i in range(n)}

    def layout(opinions):
        bins = {}
        for i, x in enumerate(opinions):
            bins.setdefault(min(int(x * bins_across), bins_across - 1), []).append(i)
        return {i: (b * BLOCK + j % BLOCK, j // BLOCK)
                for b, members in bins.items() for j, i in enumerate(sorted(members, key=rank.get))}

    spots = [layout(fr["opinions"]) for fr in raw["frames"]]
    h = max(max(r for _, r in sp.values()) + 1 for sp in spots)
    # Row 0 at the front: cell rows run from the back (y = 0) forward.
    spots = [{i: (x, h - 1 - r) for i, (x, r) in sp.items()} for sp in spots]
    frames = [
        Frame(
            tick=k,
            agents={i: Agent(i, x, y, 1.0, 0, 0, 0) for i, (x, y) in sp.items()},
            sugar=[0.0] * (w * h), deaths={}, born=[], pollution=[0.0] * (w * h), births={}, groups=groups,
        )
        for k, sp in enumerate(spots)
    ]
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=list(range(n)), config=raw["config"], frames=frames, stats=raw["stats"], model=raw["model"],
        start_rank=rank,
    )


def _tipping(raw):
    """A Schelling bounded-neighborhood shot as a `Dump` on `area.py`'s board:
    insiders in the area, outsiders in their queues, so the Flumps walk in and
    out. Red (his whites) are group 1 with ids 1…, Blue group 0 with ids 10001…;
    an insider who would leave has `sugar` 0 (everyone else 1). The tolerances
    ride in `config["tolerances"]` (Red's, then Blue's)."""
    import area
    red, blue = raw["red"], raw["blue"]
    keys = [[(True, r) for r, _ in f["red"]] + [(False, b) for b, _ in f["blue"]] for f in raw["frames"]]
    held = area.layout(keys)

    def ident(is_red, rank):
        return 1 + rank if is_red else 10001 + rank

    frames = []
    for f, where in zip(raw["frames"], held):
        content = {(True, r): c for r, c in f["red"]} | {(False, b): c for b, c in f["blue"]}
        agents, groups = {}, {}
        for is_red, n in ((True, red), (False, blue)):
            for rank in range(n):
                i = ident(is_red, rank)
                x, y = where.get((is_red, rank)) or area.queue_spot(is_red, rank)
                agents[i] = Agent(i, x, y, 0.0 if content.get((is_red, rank)) is False else 1.0, 0, 0, 0)
                groups[i] = 1 if is_red else 0
        w, h = area.WIDTH, area.HEIGHT
        frames.append(Frame(tick=f["tick"], agents=agents, sugar=[0.0] * (w * h), deaths={}, born=[],
                            pollution=[0.0] * (w * h), births={}, groups=groups))
    w, h = area.WIDTH, area.HEIGHT
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=sorted(frames[0].agents), config={**raw["config"], "tolerances": raw["tolerances"]}, frames=frames,
        stats=raw["stats"], model="tipping",
    )


def _ethno(raw):
    """An ethnocentrism shot as a `Dump`: each Flump's color (its tag) in
    its group, its kind in `kinds`; they never move, so births and deaths
    are the differences between frames (every death is the model's random
    one)."""
    w, h = raw["width"], raw["height"]
    frames, before = [], set()
    for f in raw["frames"]:
        ids = {row[0] for row in f["agents"]}
        born = [row[0] for row in f["agents"] if row[0] not in before]
        frames.append(Frame(
            tick=f["tick"],
            agents={i: Agent(i, x, y, 0.0, 0, 0, 0) for i, x, y, *_ in f["agents"]},
            sugar=[0.0] * (w * h),
            deaths={i: "random" for i in sorted(before - ids)},
            born=born,
            pollution=[0.0] * (w * h),
            births={i: (None, None) for i in born},
            groups={row[0]: row[3] for row in f["agents"]},
            kinds={row[0]: row[4] for row in f["agents"]},
        ))
        before = ids
    return Dump(
        seed=raw["seed"], ticks=raw["ticks"], width=w, height=h, capacity=[0.0] * (w * h),
        placed=sorted(frames[0].agents), config=raw["config"], frames=frames, stats=raw["stats"], model="ethno",
    )


def load(path):
    with open(path, encoding="utf-8") as f:
        return parse(f.read())


def survival(d, pred, at=None):
    """The share of the agents alive at tick 0 for which `pred(agent)` holds
    that are still alive at tick `at` (default: the last); NaN if none."""
    start, end = d.frames[0].agents, d.frames[d.ticks if at is None else at].agents
    ids = [i for i, a in start.items() if pred(a)]
    return sum(i in end for i in ids) / len(ids) if ids else float("nan")


def tracks(d):
    out = {}
    for frame in d.frames:
        for a in frame.agents.values():
            t = out.get(a.id)
            if t is None:
                t = out[a.id] = Track(a.id, frame.tick, [], [], None, None)
            t.cells.append((a.x, a.y))
            t.sugar.append(a.sugar)
        for id_, cause in frame.deaths.items():
            t = out[id_]
            out[id_] = Track(t.id, t.first, t.cells, t.sugar, frame.tick, cause)
    return out
