# Flump Studio

Short explainer videos rendered in Blender from real engine runs (see
`docs/superpowers/specs/2026-09-25-flump-studio-design.md`, and the series plan in
`docs/superpowers/specs/2026-09-26-sugarscape-series-plan.md`).

    python3 studio/build.py seasons --preview   # an episode, 960 × 540 (the shareable size)
    python3 studio/build.py seasons             # 1920 × 1080, final quality
    python3 studio/build.py seasons --beat 4    # one beat (1-based)
    python3 studio/measure.py seasons           # re-measure an episode's captions over 20 seeds

Episodes so far: `sugarscape` (the pilot), `seasons`, `pollution`, `inheritance`, `tribes`, `markets`, `war` and `credit`.

Needs Blender 5.2 at /Applications/Blender.app (or `BLENDER=/path/to/blender`), ffmpeg, and cargo.
Finished videos go to `~/Movies/Flump Studio/<episode>.mp4` (and `<episode>-preview.mp4`; set
`FLUMP_MOVIES` to change the folder). Working files go to `studio/out/<episode>/` (ignored by
git): `dumps/`, `beats/NN/` (PNG frames and the beat's `.blend`) and `music/`. The `.blend` files
open at the frame they were saved on; the animation is driven by a handler installed at render
time, so scrubbing them shows nothing.

Render times on an M1 Max (Blender 5.2, Eevee): the pilot's 19 beats (2,889 frames after dissolves,
96 s) took about 3 hours at final quality (1920 × 1080, 96 samples), and about 30 minutes as a
preview. The final encodes at CRF 21, so the pilot is 52 MB (Bluesky takes up to 100 MB).

## An episode

`studio/episodes/<episode>/` holds:

- `shots/*.json` — the runs (`sugarscape shot`); everything a Flump does comes from one.
- `beats.py` — the beats: caption, length, shot, pacing, camera, overlays. Its docstring records
  what each shot does, read from its dump.
- `claims.py` — every captioned claim, measured over 20 seeds by `measure.py`, which writes
  `measurements.md` (and `measurements.json`, when panels show the medians). A caption is only
  rendered once its claim holds.
- `tune.py` — the episode's own tune (see below).

## Music

Each episode has its own tune, written in ABC notation in `tune.py` (a `music.Tune`: voices as
General MIDI instruments, 8-bar sections, the forms it may be played in, a tempo range, and stings
cued to beats). The build fits it to the cut — choosing a form and a tempo so it ends with the
video — renders it with abc2midi and FluidSynth, mixes in the stings where their beats begin, and
lays it under the video, faded in and out.

- If `episodes/<episode>/music.wav` (or `.aif`, `.aiff`, `.m4a`) exists, that file is used instead.
- Rendering needs `brew install fluid-synth abcmidi` and the FluidR3 GM SoundFont (MIT licence) at
  `~/Music/SoundFonts/FluidR3_GM.sf2` (or wherever `FLUMP_SOUNDFONT` points):

      curl -L -o ~/Music/SoundFonts/FluidR3_GM.sf2 --create-dirs \
        https://github.com/pianobooster/fluid-soundfont/releases/download/v3.1/FluidR3_GM.sf2

The tunes so far: the pilot's *Flumps' Walk*, a gånglåt in D (accordion, fiddle drone, guitar),
Seasons' *The Thaw*, a waltz in A minor after the Russian waltz (violin and pizzicato for
summer; celesta and sleigh bells in C major for winter), and Pollution's *Smoke over the Mill*, a
brass-band chorale in E♭ that sinks into C minor under a tolling bell, and Inheritance's *The
Ground Beneath*, a passacaglia in D minor whose cello ground repeats under every variation, and
Tribes' *Two Choirs*, a polychoral canzona with recorders on the left answering oboe and bassoon on
the right until they play as one, and Markets' *The Spice Rag*, a slow rag in F for honky-tonk piano,
banjo, tuba and clarinet, whose oom-pah walks between two notes as the Flumps walk between two
hills, with a D-minor clarinet strain and a B♭ trio, and War's *Poseidon's Horn*, an original 5/4
march in D minor on a triplet figure, building under a drone from timpani and snare to the full
march, until its horn sting — a chord swelling across the brass patches, choir and organ — ducks
the march beneath it on the last title (a tune's `ducks` sink it under a sting), and Credit's
*Borrowed Time*, a 7/4 blues in E: a walking bass over coin percussion, then the band with tenor
sax, and a ka-ching as the first loan is made.

To give a tune better instruments, drag `out/<episode>/music/<slug>.mid` into GarageBand (one track
per instrument), choose instruments, keep the tempo, and export the song as audio to
`episodes/<episode>/music.m4a`; then re-run the build with `--skip-render` to re-cut. `--no-music`
leaves the video silent.

Tests: `python3 -m unittest discover -s studio/tests -t studio -v`
(`tests/fixtures/tiny.frames.json` is `sugarscape shot tests/fixtures/tiny.json`.) They check
every module, the Blender ones included, for names used but not defined, and that each episode's
beats, overlays, measurements and tune cues agree, without Blender.

Before a render, or after changing anything under `blender/`, run the smoke test:
`python3 studio/smoke.py [EPISODE ...]` renders one small still and caption of each kind of beat
from the episodes' real dumps (making any that are missing) and fails on any beat that can't be
built. All five episodes take under three minutes; stills go to `out/<episode>/smoke/`.

Overlays live in `blender/overlays/`: `parts.py` (anchors, text, cards), `followers.py` (things
that follow Flumps), `panels.py` (screen-space displays) and `caption.py`; a new overlay goes in
`BUILDERS` in `__init__.py`.

Baloo 2 is © The Baloo 2 Project Authors, under the SIL Open Font License (`fonts/OFL.txt`).
