"""The Contagion episode's tune: an original tarantella, the dance once
believed to cure a spider's poison by dancing it out. Fast 6/8, written as
bars of six eighths (M:3/4), in A minor, for accordion on the melody, a
steel guitar in tremolo standing in for the mandolin (General MIDI has
none), bass and tambourine.

One form, section by section with the story: A (the close-ups, the dance in
A minor); C (recovery, brightening to A major); B (the endemic, heavier);
E (the stranger arrives and fizzles: almost nothing); D (the 40-bit plague
and the title: dark, chromatic, driving, with castanets). Sting: a castanet
rattle as the stranger disease arrives."""

from music import Tune, Voice

ACCORDION, MANDOLIN, BASS, TAMB, CASTANETS = "accordion", "mandolin", "bass", "tambourine", "castanets"

BAR = "z6"


def tremolo(chord):
    """A chord re-struck on every eighth, the mandolin's tremolo."""
    return f"{chord} {chord} {chord} {chord} {chord} {chord}"


CHORD = {"Am": "[A,CE]", "G": "[G,B,D]", "E": "[^G,B,E]", "Dm": "[A,DF]", "A": "[A,^CE]", "D": "[A,D^F]"}
ROOT = {"Am": "A,,3 E,,3", "G": "G,,3 D,,3", "E": "E,,3 B,,,3", "Dm": "D,,3 A,,3", "A": "A,,3 E,,3", "D": "D,,3 A,,3"}
# General MIDI drums (channel 10): tambourine ^F, (54), castanets ^c' (85).
TAMB_BAR = "^F,2 ^F, ^F,2 ^F,"
TAMB_DRIVE = "^F, ^F, ^F, ^F, ^F, ^F,"
CAST_BAR = "^c'2 ^c' ^c'2 ^c'"


def _bars(bars):
    return " | ".join(bars) + " |"


def _band(chords, melody, dynamic, tamb=TAMB_BAR, cast=None, bass=None):
    return {
        ACCORDION: _bars([f"!{dynamic}! {melody[0]}"] + melody[1:]),
        MANDOLIN: _bars([f"!{dynamic}! {tremolo(CHORD[chords[0]])}"] + [tremolo(CHORD[c]) for c in chords[1:]]),
        BASS: _bars([f"!{dynamic}! {(bass or ROOT)[chords[0]]}"] + [(bass or ROOT)[c] for c in chords[1:]]),
        TAMB: _bars([f"!{dynamic}! {tamb}"] + [tamb] * 7) if tamb else _bars([BAR] * 8),
        CASTANETS: _bars([f"!{dynamic}! {cast}"] + [cast] * 7) if cast else _bars([BAR] * 8),
    }


DRIVE_BASS = {"Am": "A,,A,,A,, E,,E,,E,,", "E": "E,,E,,E,, B,,,B,,,B,,,"}

TUNE = Tune(
    title="Tarantella of the Well",
    slug="tarantella-of-the-well",
    key="Am",
    beats_per_bar=3,
    voices=(Voice(ACCORDION, 21, 105), Voice(MANDOLIN, 25, 85, pan=40), Voice(BASS, 32, 100),
            Voice(TAMB, 0, 90, channel=10), Voice(CASTANETS, 0, 95, channel=10)),
    sections={
        "A": _band(("Am", "G", "Am", "E", "Am", "Dm", "Am", "Am"),
                   ["e2 A c2 e", "d2 B G2 B", "c2 A E2 A", "B2 ^G E3", "e2 A c2 e", "f2 d A2 d", "e2 c B2 ^G", "A3 A3"],
                   "mf"),
        "C": _band(("A", "D", "A", "E", "A", "D", "A", "A"),
                   ["e2 A ^c2 e", "^f2 d A2 d", "e2 ^c A2 ^c", "B3 E3", "e2 A ^c2 e", "^f2 d A2 ^f", "e2 ^c B2 ^G",
                    "A6"], "f"),
        "B": _band(("Am", "E", "Am", "E", "Am", "E", "Am", "E"),
                   ["A2 c B2 A", "^G2 B E3", "A2 c B2 A", "E6", "A2 c B2 A", "^G2 B E3", "A2 c B2 A", "E6"], "mp"),
        "E": {  # the stranger: almost nothing
            ACCORDION: _bars(["!pp! A6", "A6", "c6", "B6", "A6", "A6", "c6", "B6"]),
            MANDOLIN: _bars(["!pp! A,2 z4", BAR, "A,2 z4", BAR, "E,2 z4", BAR, "A,2 z4", BAR]),
            BASS: _bars(["!pp! A,,6", "A,,6", "A,,6", "E,,6", "A,,6", "A,,6", "A,,6", "E,,6"]),
            TAMB: _bars([BAR] * 8),
            CASTANETS: _bars([BAR] * 8),
        },
        "D": _band(("Am", "E", "Am", "E", "Am", "E", "Am", "E"),
                   ["e f e ^d e f", "e ^d e c B A", "e f e ^d e f", "^g a ^g f e ^d", "e f e ^d e f",
                    "e ^d e c B A", "a ^g a f e c", "B ^G E ^G B e"], "ff", tamb=TAMB_DRIVE, cast=CAST_BAR,
                   bass=DRIVE_BASS),
    },
    # One form, section by section with the story: A A (the close-ups), C C
    # (recovery), B (the endemic), E E (the stranger), D D (the plague and
    # the title).
    forms=("AACCBEEDD",),
    ending={ACCORDION: "!ff! [A,CEA]6", MANDOLIN: "[A,CEA]6", BASS: "A,,6", TAMB: BAR, CASTANETS: "^c'6"},
    bpm=(160, 185, 210),
    stings={"rattle": ({CASTANETS: "!f! ^c' ^c' ^c' ^c' ^c' ^c'2"}, 185)},
    # The stranger disease arrives at tick 300: (300 − 292) / 3 ≈ 2.7 s into
    # "stranger".
    cues=(("stranger", "rattle", 2.7),),
)
