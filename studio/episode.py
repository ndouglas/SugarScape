"""An episode is an ordered list of beats, each rendered from one shot."""

import importlib.util
import pathlib
from dataclasses import dataclass

import animate

ROOT = pathlib.Path(__file__).resolve().parent
FPS = 30


@dataclass(frozen=True)
class Beat:
    """One beat: its caption, length, shot, pacing, camera moves (`camera.Move`s),
    whether its Flumps are close-up rigs, overlay names, `focus` — indexes
    into the dump's `placed` that overlays follow — the caption's height
    (−1 bottom, 1 top of the frame), whether the caption is a title
    (larger, over a scrim that dims the frame), and `compare`, a second
    shot whose dump overlays may set against this one's."""

    name: str
    caption: str
    seconds: float
    shot: str | None = None
    ticks_per_second: float = 4.0
    start_tick: int = 0
    lead_in: float = 0.0
    camera: tuple = ()
    closeup: bool = False
    overlays: tuple = ()
    focus: tuple = ()
    caption_y: float = -0.8
    title: bool = False
    compare: str | None = None

    @property
    def frames(self):
        return round(self.seconds * FPS)

    def timing(self, dump_ticks):
        return animate.Timing(self.ticks_per_second, self.start_tick, dump_ticks, self.lead_in, FPS)


def episode_dir(name):
    return ROOT / "episodes" / name


def load_module(name, module):
    """An episode's own module, e.g. its `beats` or `claims`."""
    path = episode_dir(name) / f"{module}.py"
    spec = importlib.util.spec_from_file_location(f"episodes.{name}.{module}", path)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


def load_episode(name):
    return list(load_module(name, "beats").BEATS)
