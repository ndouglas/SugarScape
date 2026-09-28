"""Loads a frame dump written by `sugarscape shot` (format 1): a Sugarscape
shot as a `Dump`, a spatial-games shot as a `Lattice`."""

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


def parse(text):
    raw = json.loads(text)
    if raw.get("format") != FORMAT:
        raise ValueError(f"frame dump format {raw.get('format')!r}, expected {FORMAT}")
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
