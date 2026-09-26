# Flump Studio

Short explainer videos rendered in Blender from real engine runs (see
`docs/superpowers/specs/2026-09-25-flump-studio-design.md`, and the series plan in
`docs/superpowers/specs/2026-09-26-sugarscape-series-plan.md`).

    python3 studio/build.py seasons --preview   # an episode, 960 × 540 (the shareable size)
    python3 studio/build.py seasons             # 1920 × 1080, final quality
    python3 studio/build.py seasons --beat 4    # one beat (1-based)
    python3 studio/measure.py seasons           # re-measure an episode's captions over 20 seeds

Episodes so far: `sugarscape` (the pilot) and `seasons`.

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
and Seasons' *The Thaw*, a waltz in A minor after the Russian waltz (violin and pizzicato for
summer; celesta and sleigh bells in C major for winter).

To give a tune better instruments, drag `out/<episode>/music/<slug>.mid` into GarageBand (one track
per instrument), choose instruments, keep the tempo, and export the song as audio to
`episodes/<episode>/music.m4a`; then re-run the build with `--skip-render` to re-cut. `--no-music`
leaves the video silent.

Tests: `python3 -m unittest discover -s studio/tests -t studio -v`
(`tests/fixtures/tiny.frames.json` is `sugarscape shot tests/fixtures/tiny.json`.)

Baloo 2 is © The Baloo 2 Project Authors, under the SIL Open Font License (`fonts/OFL.txt`).
