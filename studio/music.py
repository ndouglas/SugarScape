"""Music for an episode: an original gånglåt (a Swedish walking tune) written
here in ABC notation, fitted to the video's length by choosing its form and
tempo, and rendered to audio by abc2midi and FluidSynth.

    python3 studio/music.py SECONDS OUT_DIR   # writes flumps-walk.abc, .mid, .wav

The .mid has one track per instrument, so it can also be dragged into
GarageBand, given better instruments, and exported; an audio file placed at
studio/episodes/<episode>/music.(wav|aif|aiff|m4a) replaces the draft.
"""

import pathlib
import re
import subprocess
import sys

SECTION_BARS = 8
MIN_BPM, TARGET_BPM, MAX_BPM = 92, 104, 116
RING_OUT = 1.5  # seconds the last chord rings under the end card

# The tune, in D major, 4/4, eighth-note units; a section is 8 bars.
MELODY = {
    "A": "A2 FA d2 AF | G2 FE D2 FA | B2 GB d2 BG | A2 ce a2 gf | e2 dc d2 AF | B2 dB f2 dB | e2 dc B2 A2 | d2 f2 d4 |",
    "B": "B2 dg b2 ag | f2 ed f2 a2 | e2 ce a2 ge | f2 df a4 | g2 bg e2 dB | A2 FA d2 fd | e2 fe dc BA | d2 A2 D4 |",
}
_D, _G = "D,2 [DFA]2 A,,2 [DFA]2", "G,,2 [GBd]2 D,2 [GBd]2"
_A, _BM = "A,,2 [Ace]2 E,2 [Ace]2", "B,,2 [Bdf]2 F,2 [Bdf]2"
_EM_A = "E,2 [EGB]2 A,,2 [Ace]2"
GUITAR = {
    "A": " | ".join([_D, _D, _G, _A, _D, _BM, _EM_A, _D]) + " |",
    "B": " | ".join([_G, _D, _A, _D, _G, _D, _EM_A, _D]) + " |",
}
DRONE_BAR = "[DA]8"  # the fiddle's open D and A, the nyckelharpa's hum
ENDING = {"melody": "d8", "guitar": "[D,A,D]8", "drone": "[DA]8"}

VOICES = [
    ("accordion", 21, 110),
    ("fiddle drone", 110, 55),
    ("guitar", 24, 85),
]


def eighths(bar):
    """The length of an ABC bar in eighth notes (L:1/8)."""
    return sum(int(n or 1) for _, n in re.findall(r"(\[[^\]]+\]|[_^=]?[A-Ga-gz][,']*)(\d*)", bar))


def tempo_for(total_bars, seconds):
    """Beats a minute so `total_bars` of 4/4 end RING_OUT before `seconds`."""
    return total_bars * 4 * 60 / (seconds - RING_OUT)


def form_for(seconds):
    """Sections (AABB AABB …, always ending on A) whose tempo is nearest a
    walking pace for a video `seconds` long."""
    forms = {"ABA", "ABBA"}
    for n in range(2, 16):
        form = ("AABB" * 4)[:n]
        forms.add(form if form[-1] == "A" else form + "A")
    candidates = []
    for form in sorted(forms):
        bpm = tempo_for(len(form) * SECTION_BARS, seconds)
        if MIN_BPM <= bpm <= MAX_BPM:
            candidates.append((abs(bpm - TARGET_BPM), form))
    if not candidates:
        raise ValueError(f"no form fits {seconds:.1f} s at {MIN_BPM}–{MAX_BPM} bpm")
    return min(candidates)[1]


def _bars(text):
    return [b.strip() for b in text.split("|") if b.strip()]


def _line(bars):
    return " | ".join(bars) + " |"


def score(seconds):
    form = form_for(seconds)
    bpm = round(tempo_for(len(form) * SECTION_BARS, seconds))
    melody = [b for s in form for b in _bars(MELODY[s])]
    guitar = [b for s in form for b in _bars(GUITAR[s])]
    drone = [DRONE_BAR] * len(melody)
    melody[-1], guitar[-1], drone[-1] = ENDING["melody"], ENDING["guitar"], ENDING["drone"]
    parts = [melody, drone, guitar]
    lines = [
        "X:1",
        "T:Flumps' Walk (gånglåt)",
        "C:original, written for the Flump studio",
        "M:4/4",
        "L:1/8",
        f"Q:1/4={bpm}",
        "K:D",
    ]
    for i, ((name, program, volume), bars) in enumerate(zip(VOICES, parts), start=1):
        lines += [f'V:{i} name="{name}"', f"%%MIDI program {program}", f"%%MIDI control 7 {volume}"]
        lines += [_line(bars[k : k + SECTION_BARS]) for k in range(0, len(bars), SECTION_BARS)]
    return "\n".join(lines) + "\n"


def render(seconds, out_dir, soundfont):
    """Writes flumps-walk.abc, .mid and .wav to `out_dir`; returns the .wav path."""
    out = pathlib.Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    abc, mid, wav = out / "flumps-walk.abc", out / "flumps-walk.mid", out / "flumps-walk.wav"
    abc.write_text(score(seconds))
    subprocess.run(["abc2midi", abc, "-o", mid], check=True, capture_output=True)
    subprocess.run(["fluidsynth", "-ni", "-q", "-g", "0.6", "-r", "48000", "-F", wav, soundfont, mid], check=True)
    return wav


if __name__ == "__main__":
    here = pathlib.Path(__file__).resolve().parent
    print(render(float(sys.argv[1]), sys.argv[2], here / "out" / "soundfonts" / "FluidR3_GM.sf2"))
