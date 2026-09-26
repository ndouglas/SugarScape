"""Music for an episode. Each episode writes its own tune in ABC notation
(episodes/<episode>/tune.py, a `Tune`); this module fits it to the video —
choosing one of the tune's forms and a tempo in its range so it ends with
the video — renders it with abc2midi and FluidSynth, and mixes in its stings
at the seconds their beats begin.

    python3 studio/music.py EPISODE SECONDS OUT_DIR   # writes <slug>.abc, .mid, .wav

The .mid has one track per instrument, so it can also be dragged into
GarageBand, given better instruments, and exported; an audio file placed at
studio/episodes/<episode>/music.(wav|aif|aiff|m4a) replaces the draft.
"""

import os
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass, field

# The SoundFont lives outside the repository (override with FLUMP_SOUNDFONT).
SOUNDFONT = pathlib.Path(
    os.environ.get("FLUMP_SOUNDFONT", pathlib.Path.home() / "Music" / "SoundFonts" / "FluidR3_GM.sf2")
)

SECTION_BARS = 8
RING_OUT = 1.5  # seconds the last chord rings under the end card
CUE_DELAY = 0.3  # a sting lands this long after its beat begins


@dataclass(frozen=True)
class Voice:
    """One instrument: a General MIDI program (0-based), its volume, and a
    channel (10 for percussion; None lets abc2midi choose)."""

    name: str
    program: int
    volume: int
    channel: int | None = None


@dataclass(frozen=True)
class Tune:
    """A tune in ABC (L:1/8): its sections, each 8 bars per voice; the forms
    it may be played in; the last bar of each voice; its tempo range as
    (lowest, preferred, highest) quarter notes a minute; and stings — short
    pieces, each {voice name: ABC} at a tempo — cued by beat name."""

    title: str
    slug: str
    key: str
    beats_per_bar: int
    voices: tuple
    sections: dict
    forms: tuple
    ending: dict
    bpm: tuple
    stings: dict = field(default_factory=dict)
    cues: tuple = ()


def eighths(bar):
    """The length of an ABC bar in eighth notes (L:1/8)."""
    return sum(int(n or 1) for _, n in re.findall(r"(\[[^\]]+\]|[_^=]?[A-Ga-gz][,']*)(\d*)", bar))


def tempo_for(tune, total_bars, seconds):
    """Quarter notes a minute so `total_bars` end RING_OUT before `seconds`."""
    return total_bars * tune.beats_per_bar * 60 / (seconds - RING_OUT)


def form_for(tune, seconds):
    """The tune's form whose tempo is nearest its preferred one."""
    low, target, high = tune.bpm
    candidates = []
    for form in tune.forms:
        bpm = tempo_for(tune, len(form) * SECTION_BARS, seconds)
        if low <= bpm <= high:
            candidates.append((abs(bpm - target), form))
    if not candidates:
        raise ValueError(f"no form of {tune.title} fits {seconds:.1f} s at {low}–{high} bpm")
    return min(candidates)[1]


def _bars(text):
    return [b.strip() for b in text.split("|") if b.strip()]


def _line(bars):
    return " | ".join(bars) + " |"


def _header(title, beats_per_bar, bpm, key):
    return ["X:1", f"T:{title}", "C:original, written for the Flump studio",
            f"M:{beats_per_bar}/4", "L:1/8", f"Q:1/4={bpm}", f"K:{key}"]


def _voice(i, v):
    lines = [f'V:{i} name="{v.name}"']
    if v.channel is not None:
        lines.append(f"%%MIDI channel {v.channel}")
    lines += [f"%%MIDI program {v.program}", f"%%MIDI control 7 {v.volume}"]
    return lines


def score(tune, seconds):
    form = form_for(tune, seconds)
    bpm = round(tempo_for(tune, len(form) * SECTION_BARS, seconds))
    lines = _header(tune.title, tune.beats_per_bar, bpm, tune.key)
    for i, v in enumerate(tune.voices, start=1):
        bars = [b for s in form for b in _bars(tune.sections[s][v.name])]
        bars[-1] = tune.ending[v.name]
        lines += _voice(i, v)
        lines += [_line(bars[k : k + SECTION_BARS]) for k in range(0, len(bars), SECTION_BARS)]
    return "\n".join(lines) + "\n"


def sting_score(tune, name):
    parts, bpm = tune.stings[name]
    lines = _header(f"{tune.title}: {name}", tune.beats_per_bar, bpm, tune.key)
    voices = {v.name: v for v in tune.voices}
    for i, (voice, text) in enumerate(parts.items(), start=1):
        lines += _voice(i, voices[voice]) + [text]
    return "\n".join(lines) + "\n"


def cue_times(cues, beat_names, frames, dissolve=12, fps=30):
    """Each cue (beat name, sting) as (sting, seconds into the cut): the
    moment its beat begins — after the dissolves before it — plus CUE_DELAY."""
    starts, elapsed = {}, 0
    for i, (name, n) in enumerate(zip(beat_names, frames)):
        starts[name] = elapsed / fps
        elapsed += n - dissolve
    return [(sting, round(starts[beat] + CUE_DELAY, 3)) for beat, sting in cues if beat in starts]


def sting_mix_command(track, stings, out):
    """ffmpeg argv laying each (wav, seconds) sting over `track`."""
    argv = ["ffmpeg", "-y", "-loglevel", "error", "-i", str(track)]
    parts, labels = [], ["[0:a]"]
    for i, (wav, at) in enumerate(stings):
        argv += ["-i", str(wav)]
        ms = round(at * 1000)
        parts.append(f"[{i + 1}:a]adelay={ms}|{ms}[s{i}]")
        labels.append(f"[s{i}]")
    parts.append(f"{''.join(labels)}amix=inputs={len(labels)}:duration=first:normalize=0[out]")
    return argv + ["-filter_complex", ";".join(parts), "-map", "[out]", str(out)]


def _synth(abc_text, stem, soundfont):
    abc, mid, wav = stem.with_suffix(".abc"), stem.with_suffix(".mid"), stem.with_suffix(".wav")
    abc.write_text(abc_text)
    subprocess.run(["abc2midi", abc, "-o", mid], check=True, capture_output=True)
    subprocess.run(["fluidsynth", "-ni", "-q", "-g", "0.6", "-r", "48000", "-F", wav, soundfont, mid], check=True)
    return wav


def render(tune, seconds, out_dir, soundfont, cues=()):
    """Writes <slug>.abc, .mid and .wav to `out_dir` (the tune, with any
    cued stings mixed in); returns the .wav path."""
    out = pathlib.Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    wav = _synth(score(tune, seconds), out / tune.slug, soundfont)
    if not cues:
        return wav
    stings = [(_synth(sting_score(tune, s), out / f"{tune.slug}-sting-{s}", soundfont), at) for s, at in cues]
    mixed = out / f"{tune.slug}-with-stings.wav"
    subprocess.run(sting_mix_command(wav, stings, mixed), check=True)
    return mixed


if __name__ == "__main__":
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
    import episode

    print(render(episode.load_module(sys.argv[1], "tune").TUNE, float(sys.argv[2]), sys.argv[3], SOUNDFONT))
