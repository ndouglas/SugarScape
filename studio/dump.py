"""Loads a frame dump written by `sugarscape shot` (format 1)."""

import json
from dataclasses import dataclass

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


def parse(text):
    raw = json.loads(text)
    if raw.get("format") != FORMAT:
        raise ValueError(f"frame dump format {raw.get('format')!r}, expected {FORMAT}")
    frames = [
        Frame(
            tick=f["tick"],
            agents={row[0]: Agent(*row) for row in f["agents"]},
            sugar=f["sugar"],
            deaths=dict(f["deaths"]),
            born=f["born"],
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
    )


def load(path):
    with open(path, encoding="utf-8") as f:
        return parse(f.read())


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
