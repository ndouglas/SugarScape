# Flump Studio

Short explainer videos rendered in Blender from real engine runs (see
`docs/superpowers/specs/2026-09-25-flump-studio-design.md`).

    python3 studio/build.py sugarscape            # the whole pilot, final quality
    python3 studio/build.py sugarscape --preview  # 960 × 540, low samples
    python3 studio/build.py sugarscape --beat 4   # one beat (1-based)
    python3 studio/measure.py                     # re-measure the pilot's captions over 20 seeds

Needs Blender 5.2 at /Applications/Blender.app (or `BLENDER=/path/to/blender`), ffmpeg, and cargo.
Outputs go to `studio/out/<episode>/` (ignored by git): `dumps/`, `beats/NN/` (PNG frames and the
beat's `.blend`), and `<episode>.mp4`. The `.blend` files open at the frame they were saved on;
the animation is driven by a handler installed at render time, so scrubbing them shows nothing.

## Music

The build lays a soundtrack under the cut, trimmed to the video and faded in and out:

- If `episodes/<episode>/music.wav` (or `.aif`, `.aiff`, `.m4a`) exists, that file is used.
- Otherwise `music.py` writes an original gånglåt, *Flumps' Walk*, in ABC notation, chooses its
  form and tempo so it ends with the video, and renders it with abc2midi and FluidSynth to
  `out/<episode>/music/flumps-walk.{abc,mid,wav}`. That needs `brew install fluid-synth abcmidi` and
  the FluidR3 GM SoundFont (MIT licence) at `out/soundfonts/FluidR3_GM.sf2`:

      curl -L -o studio/out/soundfonts/FluidR3_GM.sf2 --create-dirs \
        https://github.com/pianobooster/fluid-soundfont/releases/download/v3.1/FluidR3_GM.sf2

To give the tune better instruments, drag `flumps-walk.mid` into GarageBand (one track each for
accordion, fiddle drone and guitar), choose instruments, keep the tempo, and export the song as
audio to `episodes/<episode>/music.m4a`; then re-run the build with `--skip-render` to re-cut.
`--no-music` leaves the video silent.

## Beats

A beat is one shot (`episodes/<episode>/shots/*.json`, run by `sugarscape shot`) and one entry in
`episodes/<episode>/beats.py`. Everything a Flump does comes from the shot's run.

Tests: `python3 -m unittest discover -s studio/tests -t studio -v`
(`tests/fixtures/tiny.frames.json` is `sugarscape shot tests/fixtures/tiny.json`.)

Baloo 2 is © The Baloo 2 Project Authors, under the SIL Open Font License (`fonts/OFL.txt`).
