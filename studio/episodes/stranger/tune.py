"""The Cooperation finale's tune, *Home Again*: an original medley in G, 4/4,
quoting each episode's tune in turn, in its own instruments and set in G,
over one walking bass.

Bar by bar with the story (72 bars, about 1.07 s each):
1–5 the question: the medley's own theme on piano.
6–9 Neighbors in Phase (marimba, G dorian, its 6-note figure slipping across
  the 4/4 bar); 10–13 A Round for Neighbors (flute, clarinet a third
  below); 14–17 Cradle Song for Four Colors (music box); 18–21 The Twins'
  Slip Jig (whistle and fiddle in unison); 22–25 The Talk of the Town
  (clarinet, banjo); 26–29 A Fugue for Keeping Order (viola, then the
  fiddle answering, in G minor, over a cello pedal); 30–33 Swing Your
  Partner (fiddle, banjo).
34–37 strangers and the honest part: the bass alone.
38–63 the ledgers: the theme on piano, quietly, three times.
64–72 the title: every theme's opening bar in turn, then all together on
  G major.
"""

from music import Tune, Voice

PIANO, MARIMBA, FLUTE, CLARINET, BOX = "piano", "marimba", "flute", "clarinet", "music box"
WHISTLE, FIDDLE, BANJO, VIOLA, CELLO, BASS = "whistle", "fiddle", "banjo", "viola", "cello", "bass"
VOICES = (PIANO, MARIMBA, FLUTE, CLARINET, BOX, WHISTLE, FIDDLE, BANJO, VIOLA, CELLO, BASS)
BARS = 72
REST = "z8"

THEME = ["B2 d2 d2 e d", "B2 A2 G4", "c2 e2 e2 d c", "B6 z2", "B2 d2 g2 f e", "d2 B2 A2 G2", "A2 c2 B2 A2", "G8"]
# Each episode's opening phrase, set in G and in 4/4 (see the episodes' tune.py).
PHASE = ["G d =f g d a g =f", "d e =f d G d =f g", "d a g =f d e =f d", "G d =f g d a g2"]
ROUND = ["G2 B2 d4", "d4 B4", "c2 B2 A4", "B6 G2"]
ROUND_LOW = ["E2 G2 B4", "B4 G4", "A2 G2 F4", "G6 E2"]
CRADLE = ["d2 B G2 B z2", "A2 G F2 D z2", "E2 G B2 e z2", "d3 B3 z2"]
JIG = ["g2 d B2 d e d", "g2 d B2 d A B", "c2 e g2 e c e", "d2 B G2 B A2"]
TOWN = ["D2 E F G3 F", "A2 G F E4", "D2 E F G2 B2", "A6 z2"]
TOWN_CHORDS = ["[G,B,D]3 [G,B,D]5", "[F,A,D]3 [F,A,D]5", "[G,B,D]3 [G,B,D]5", "[F,A,D]3 [F,A,D]5"]
FUGUE = ["D G _B d c _B A G", "_B2 A2 G ^F G2", REST, REST]
ANSWER = [REST, REST, "d g _b d' c' _b a g", "_b2 a2 g ^f g2"]
REEL = ["G2 B d g d B d", "c A F A d A F A", "G B d B g B d B", "A B c A B G G2"]
ROLL = ["G B d g d B G B", "F A d f d A F A", "G B d g d B G B", "F A d f d A F A"]

WALK = {"G": "G,2 B,2 D2 B,2", "C": "C2 E2 G2 E2", "D": "D2 F2 A2 F2", "Gdor": "G,2 D2 =F2 D2",
        "Gm": "G,2 _B,2 D2 _B,2", "Dm": "D2 A,2 D,2 A,2", "Em": "E,2 G,2 B,2 G,2"}
QUOTE_CHORDS = {6: ["Gdor"] * 4, 10: ["G", "G", "C", "G"], 14: ["G", "D", "Em", "G"], 18: ["G", "G", "C", "D"],
                22: ["G", "D", "G", "D"], 26: ["Gm", "Gm", "Gm", "Dm"], 30: ["G", "D", "G", "D"]}


def build():
    bars = {v: [REST] * BARS for v in VOICES}

    def put(voice, first, phrase):
        for k, b in enumerate(phrase):
            bars[voice][first - 1 + k] = b

    put(PIANO, 1, THEME[:4] + ["G2 B2 d4"])
    put(MARIMBA, 6, PHASE)
    put(FLUTE, 10, ROUND)
    put(CLARINET, 10, ROUND_LOW)
    put(BOX, 14, CRADLE)
    put(WHISTLE, 18, JIG)
    put(FIDDLE, 18, JIG)
    put(CLARINET, 22, TOWN)
    put(BANJO, 22, TOWN_CHORDS)
    put(VIOLA, 26, FUGUE)
    put(FIDDLE, 26, ANSWER)
    put(CELLO, 26, ["G,,8"] * 3 + ["D,,8"])
    put(FIDDLE, 30, REEL)
    put(BANJO, 30, ROLL)
    for first, chords in QUOTE_CHORDS.items():
        put(BASS, first, [WALK[c] for c in chords])
    put(BASS, 1, [WALK[c] for c in ("G", "G", "C", "G", "G")])
    put(BASS, 34, ["G,,2 z6", REST, "D,,2 z6", REST])  # the bass alone, and sparse
    put(PIANO, 38, THEME * 3 + ["B2 d2 d2 e d", "G8"])
    put(BASS, 38, ["G,,8", "G,,8", "C,8", "G,,8", "G,,8", "D,8", "D,8", "G,,8"] * 3 + ["G,,8", "G,,8"])
    # The title: each theme's opening bar in turn, then the reel's, and all together.
    for bar, (voice, phrase) in enumerate([(MARIMBA, PHASE), (FLUTE, ROUND), (BOX, CRADLE), (WHISTLE, JIG),
                                           (CLARINET, TOWN), (VIOLA, FUGUE)], start=64):
        put(voice, bar, phrase[:1])
    put(FIDDLE, 70, REEL[:2])
    put(BANJO, 70, ROLL[:2])
    put(PIANO, 70, ["[GBd]8", "[FAd]8"])
    put(BASS, 64, [WALK[c] for c in ("Gdor", "G", "G", "G", "G", "Gm", "G", "D")])
    return bars


def sections(bars):
    names = "ABCDEFGHI"
    return {names[s]: {v: " | ".join(bars[v][8 * s : 8 * s + 8]) + " |" for v in VOICES} for s in range(BARS // 8)}


TUNE = Tune(
    title="Home Again",
    slug="home-again",
    key="G",
    beats_per_bar=4,
    voices=(Voice(PIANO, 0, 80, channel=1, pan=64), Voice(MARIMBA, 12, 95, channel=2, pan=50),
            Voice(FLUTE, 73, 90, channel=3, pan=70), Voice(CLARINET, 71, 85, channel=4, pan=56),
            Voice(BOX, 10, 100, channel=5, pan=64), Voice(WHISTLE, 78, 90, channel=6, pan=44),
            Voice(FIDDLE, 40, 95, channel=7, pan=84), Voice(BANJO, 105, 80, channel=8, pan=40),
            Voice(VIOLA, 41, 95, channel=9, pan=60), Voice(CELLO, 42, 80, channel=11, pan=72),
            Voice(BASS, 32, 95, channel=12, pan=64)),
    sections=sections(build()),
    forms=("ABCDEFGHI",),
    ending={PIANO: "[GBdg]8", MARIMBA: "[GBd]8", FLUTE: "g8", CLARINET: "B8", BOX: "[Bd]8", WHISTLE: "d8",
            FIDDLE: "g8", BANJO: "[GBd]8", VIOLA: "D8", CELLO: "G,8", BASS: "G,,8"},
    bpm=(200, 225, 250),
)
