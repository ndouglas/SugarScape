# Contagion (episode 9): spike, storyboard and tune

**Date:** 2026-09-27
**Builds on:** the series plan. Approved storyboard and tune. Every caption is re-measured by
`studio/episodes/contagion/claims.py` over 20 seeds.

## Spike: what the book says and what holds

| The book | Measured (20 seeds) | Verdict |
|---|---|---|
| V-1 (10 diseases, 4 each): "society is able to rid itself of all diseases" | 91 % sick at t = 0 (agents already immune to a disease aren't given it); under 5 % by tick 6.5 (median; 3–43); but 1.8 % still sick at t = 1000, and free of disease in only 2 of 20 | **Fails as stated**: a residue lingers in 18 of 20 |
| V-2 (25 diseases, 10 each): "an endemic level of infection is sustained" | 4.2 % sick over ticks 500–1000, against V-1's 1.8 %; more than V-1 in 16 of 20 | Holds, but it's closer to V-1 than the book's contrast suggests |
| McNeill: "would not be a difficult story to grow": a healthy society, a novel infection, "a terrifying toll" | the preset's novel 10-bit disease (the book's longest) reaches 4.7 % of the Flumps (median) and takes no measurable toll: population at t = 500 is 0.98 of the no-outbreak control, lower in 12 of 20 | **Doesn't reproduce** |
| (a departure, labeled as ours) a 40-bit novel disease | reaches everyone; population at t = 500 is 0.80 of the control, lower in 20 of 20 | The plague needs a disease four times longer than the book allows |

Why it fizzles (the spike's reading): immune systems learn one bit per tick, and a 10-bit disease
is usually only a few flips from some stretch of a random 50-bit immune string. In the spike's
sweep, 20 bits spreads to everyone but costs nothing, 30 bits costs about a tenth, and 40 bits
about a fifth.

**Captions that changed before rendering:**
- "within a few ticks, nearly all are well" became "within a few dozen ticks at most". The median
  world is well by tick 6.5, but only 14 of 20 are within 15 ticks and one takes 43.
- "a little more stays for good" became "…in 16 of 20 worlds", the measured count.

## Storyboard (about 75 s)

| # | Caption | Shot |
|---|---|---|
| 1 | "Some Flumps carry diseases. A disease steals a little of their sugar every tick." | close-up: the sick tinted green; a disease count over each |
| 2 | "Diseases pass between neighbors…" | close-up: infection lines |
| 3 | "…but each Flump's immune system learns, one step at a time, until it's well." | close-up: the tint fading |
| 4 | "At the start, almost every Flump is sick." | wide V-1, t = 0; a sick counter |
| 5 | "Within a few dozen ticks at most, nearly all are well." | wide, fast |
| 6 | "The book says society rids itself of every disease. Here a little always lingers, in 18 of 20 worlds." | wide, late |
| 7 | "Give them more diseases than an immune system can hold, and more of it stays for good, in 16 of 20 worlds." | wide V-2 |
| 8 | "Now a healthy society meets a disease it has never seen." | wide McNeill, the outbreak |
| 9 | "The book expects a plague. Here it fizzles: about one Flump in twenty catches it." | infection lines |
| 10 | "Only a disease four times longer than any in the book sweeps through, and costs a fifth of the Flumps." | our experiment: the 40-bit outbreak |
| 11 | "A plague needs something / this world has never known." | title |
| 12 | "Contagion — after Epstein & Axtell, 1996 / ndouglas.github.io/SugarScape" | end card |

## Tune: a tarantella

An original tarantella in 6/8 (written as 3/4 in eighths), A minor, for mandolin, accordion and
tambourine. The tarantella was once believed to cure a spider's poison by dancing it out. It
brightens to A major as immune systems learn, then turns dark and driving for the plague. Sting: a
castanet rattle at the outbreak.

## New studio work

- **Engine:** diseases carried and infections in the frame dump.
- **Blender:** a sick tint (colors="sick"), infection lines (the book's Animation V-3), a sick
  counter, and disease labels in the close-ups.
