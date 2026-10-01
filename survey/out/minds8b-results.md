# Minds 8b: second-round results

Assembled from `survey/out/results-watch-.json` (git-ignored), written by `cargo run --release -- --only watch-` in `survey/`; the usage check (`--usage`) and the presets' measures (`--presets`, `minds8b-presets.md`) are separate tracked files. The judges are `survey/src/claims/minds8b.rs`, binding word for word on docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md.

## Run order

1. `e4edce3` Minds 8b: second-round judges, committed before any run.
2. `55a8c23` Minds 8b: 3a rising reads Fails (clarified before any run).
3. This run: the survey binary built at `55a8c23`, unchanged, `--only watch-` (the second round's ten claims and, as context, the first round's five claims under the new defaults), seeds as the judges fix them (claims 1–2 seeds 1–20, claim 3 seeds 1–60); 18 min 37 s wall clock on 10 threads. The calibration (`b966312`) was not rerun for the judges: `WATCH_AK_ITEM` is `Some(3)`, item 4. Its claim reran here and reproduced it (20 of 20).
4. After the run, and not touching any judge: `--usage` and `--presets` (a reported-only section added for the presets' descriptions: survival counts at ticks 100 and 200 added to `Run`), same seeds 1–20.

## Verdicts

| id | verdict | headline |
|---|---|---|
| `watch-ak.calibration` | **Holds** | item 4 qualifies (20 of 20 seeds): `theft-winter-half` at `find` 0.02, owners digging below their whole reserve (`dig_below: reserve`) |
| `watch-winter.fresh` | **Holds** | h(watch-winter) > h(theft-winter) in 20 of 20 seeds (need 16; 0 without a value) |
| `watch-ak.flip` | **Holds** | p_s ÷ p_o < 1 in 20 of 20 seeds (need 16; 0 without a value) |
| `watch-ak.sign` | **Fails** | the signs agree in 62 of 100 runs, 62.0 % (need 80 %; 0 excluded) |
| `watch-ak.watched` | **Fails** | the hoarders' advantage under who: cheaters ≥ 0.05 below watching off in 0 of 20 seeds (need 16; 0 without a value) |
| `watch-ak.raiding` | **Fails** | hoarder fitness under who: hoarders ≥ 0.05 below watching off in 0 of 20 seeds (need 16; 0 without a value) |
| `watch-scroungers.frequency-forgo` | **Holds** | falls: mean per-seed slope -0.2012, 95 % CI -0.2283 to -0.1740, 90 % CI -0.2238 to -0.1785 (t, df 59); 59 of 60 slopes below 0 |
| `watch-scroungers.frequency-bury` | **Fails** | flat: mean per-seed slope 0.0112, 95 % CI -0.0075 to 0.0300, 90 % CI -0.0044 to 0.0269 (t, df 59); 26 of 60 slopes below 0 |
| `watch-scroungers.mix` | **Fails** | [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) |
| `watch-scroungers.dilemma` | **Holds** | [watchers ahead: Holds] pooled watcher advantage: mean 0.0093, 95 % CI 0.0027 to 0.0159 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0081, 95 % CI -0.0104 to -0.0058 (t, df 59) |

No claim read Inconclusive. No claim read Untestable: the calibration found a world (item 4), so 2a–2d were judged.

### The first round's claims, rerun under the new defaults (context, not judged in this round)

The first round's claims (survey/src/claims/minds8.rs) are unchanged; only the presets' default `raid_if` moved from `always` to `better`.

| id | verdict | headline |
|---|---|---|
| `watch-winter.pilferage` | Fails | median 0.0000 (IQR 0.0000–0.0000); 0/20 in [1.0000, 1.0000] |
| `watch-half.threshold` | Fails | [p_s ÷ p_o lower under watching: Fails] median 0.0000 (IQR 0.0000–0.0000); 2/20 in [1.0000, 1.0000] [hoarder advantage lower under watching: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] |
| `watch-scroungers.frequency` | Fails | mean per-seed slope -0.0269, 95 % CI -0.1009 to 0.0471 (t, df 19); 12 of 20 slopes below 0 |
| `watch-scroungers-only.frequency` | Fails | mean per-seed slope 0.0299, 95 % CI -0.0332 to 0.0930 (t, df 19); 8 of 20 slopes below 0 |
| `watch-winter.usage` | Holds | [watch-winter: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] [watch-winter-stumble: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] [watch-half: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] [watch-scroungers: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] [watch-scroungers-only: Holds] median 1.0000 (IQR 1.0000–1.0000); 20/20 in [1.0000, 1.0000] |

### `watch-scroungers-only.frequency` by share (first round's claim under the new defaults)

Seeds 1–20, span 7; watchers and cheaters at the same share s (the same agents, who never bury); the advantage is watcher − non-watcher survival per founder (alive at 200 ÷ founders). Result: mean per-seed slope 0.0299, 95 % CI -0.0332 to 0.0930 (t, df 19); 8 of 20 slopes below 0.

| s | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 | 0.9 |
|---|---|---|---|---|---|---|---|---|---|
| median advantage | -0.198 | -0.207 | -0.225 | -0.198 | -0.171 | -0.212 | -0.166 | -0.196 | -0.161 |

Without watching (watching off, same agents, at half; `minds8b-presets.md`): the cheaters survive 0.477 and the hoarders 0.699 per founder, a gap of 22 points; with watching, 0.489 and 0.653, a gap of 16.

## Claim 1: fresh caches (`watch-winter.fresh`)

**Holds**: h(watch-winter) > h(theft-winter) in 20 of 20 seeds (need 16; 0 without a value).

Per seed h (% a day), the cohort of caches first created in ticks 1–90:

| seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| watch-winter | 6.09 | 5.81 | 5.81 | 6.09 | 6.30 | 5.74 | 6.22 | 5.73 | 6.39 | 6.31 | 6.34 | 6.06 | 6.24 | 6.61 | 5.94 | 5.80 | 5.91 | 6.17 | 6.21 | 6.56 |
| theft-winter | 3.21 | 2.67 | 2.99 | 3.15 | 2.98 | 3.09 | 2.98 | 2.81 | 3.07 | 2.77 | 3.07 | 2.89 | 2.82 | 3.16 | 2.74 | 2.90 | 3.01 | 3.13 | 3.18 | 3.17 |

### The cohort hazard: P_1, P_3, P_7 and h (medians over seeds 1–20)

Context, not targets: Heinrich and Pepper's within-species next-day figure for P_1, at most 15 of 42 caches (36 %); Vander Wall and Jenkins's 2–30 % a day is loss of artificial caches to all pilferers, mostly rodents.

| world | P_1 | P_3 | P_7 | h a day | seeds with h above theft-winter's | cohort per seed | first round's stock rate |
|---|---|---|---|---|---|---|---|
| watch-winter | 6.63 % | 21.36 % | 35.78 % | 6.13 % | 20 of 20 (the judge) | 6555.500 (6465.250–6656.000) | 0.70 % |
| watch-winter-stumble | 7.84 % | 28.35 % | 50.48 % | 9.55 % | 20 of 20 | 6859.000 (6796.750–7034.000) | 3.48 % |
| theft-winter (stumbling only) | 0.82 % | 5.01 % | 19.20 % | 3.00 % | — | 5763.500 (5712.000–5831.000) | 2.31 % |

### Under the reported settings (switch variants at span 7; spans 1, 2, 3, 7 and 13)

| world, setting | P_1 | P_3 | P_7 | h a day | seeds with h above theft-winter's | raids per seed | raided sugar per seed | stock rate |
|---|---|---|---|---|---|---|---|---|
| watch-winter, span 7, raid_if always | 7.72 % | 26.61 % | 44.81 % | 8.14 % | 20 of 20 | 5103.500 (4970.000–5201.500) | 29810.779 (28178.998–30265.818) | 0.95 % |
| watch-winter-stumble, span 7, raid_if always | 8.57 % | 31.01 % | 54.75 % | 10.71 % | 20 of 20 | 6518.500 (6314.000–6578.250) | 42854.115 (41520.661–44482.377) | 3.60 % |
| watch-winter, span 7, value room | 6.67 % | 21.37 % | 36.02 % | 6.18 % | 20 of 20 | 4399.000 (4229.000–4483.000) | 27006.448 (26480.904–27952.624) | 0.70 % |
| watch-winter-stumble, span 7, value room | 7.98 % | 28.30 % | 50.38 % | 9.53 % | 20 of 20 | 6023.000 (5910.500–6090.500) | 44664.548 (43220.600–46340.499) | 3.49 % |
| watch-winter, span 1 | 3.97 % | 4.74 % | 5.80 % | 0.85 % | 0 of 20 | 730.500 (708.250–773.000) | 3696.261 (3493.565–3951.729) | 0.08 % |
| watch-winter-stumble, span 1 | 8.38 % | 18.70 % | 33.39 % | 5.64 % | 20 of 20 | 2357.500 (2293.750–2440.750) | 22880.060 (22560.192–24446.973) | 2.73 % |
| watch-winter, span 2 | 5.46 % | 12.59 % | 16.17 % | 2.49 % | 0 of 20 | 1920.000 (1868.500–1983.750) | 11423.527 (10909.226–11925.161) | 0.23 % |
| watch-winter-stumble, span 2 | 8.15 % | 26.21 % | 41.97 % | 7.48 % | 20 of 20 | 4162.500 (4061.250–4281.000) | 37663.766 (36814.184–38789.841) | 3.01 % |
| watch-winter, span 3 | 5.99 % | 17.29 % | 23.40 % | 3.74 % | 20 of 20 | 2699.500 (2634.500–2753.500) | 17178.038 (16628.676–17584.533) | 0.35 % |
| watch-winter-stumble, span 3 | 7.99 % | 28.45 % | 45.78 % | 8.37 % | 20 of 20 | 4942.500 (4858.000–5014.250) | 43243.304 (42479.016–43953.242) | 3.20 % |
| watch-winter, span 7 (judged) | 6.63 % | 21.36 % | 35.78 % | 6.13 % | 20 of 20 | 4351.000 (4249.500–4490.750) | 27011.030 (26494.726–28781.900) | 0.70 % |
| watch-winter-stumble, span 7 (judged) | 7.84 % | 28.35 % | 50.48 % | 9.55 % | 20 of 20 | 5989.000 (5888.750–6053.250) | 44688.010 (43308.285–46628.732) | 3.48 % |
| watch-winter, span 13 | 6.66 % | 21.34 % | 36.62 % | 6.31 % | 20 of 20 | 5347.000 (5288.250–5473.250) | 32986.402 (32118.643–33783.721) | 0.94 % |
| watch-winter-stumble, span 13 | 7.51 % | 27.90 % | 50.35 % | 9.52 % | 20 of 20 | 6380.000 (6301.250–6577.750) | 44232.965 (43075.734–46795.104) | 3.56 % |

### The probe (World::probe_raid_harvests: a raid that took something also harvests its site; a survey probe, not a setting)

| world | P_1 | P_3 | P_7 | h a day | raids per seed | raided sugar per seed | stock rate | fitness (ticks alive per founder ÷ 200) |
|---|---|---|---|---|---|---|---|---|
| watch-winter with the probe | 8.80 % | 25.50 % | 39.65 % | 6.96 % | 5133.000 (5040.750–5283.250) | 39695.821 (38716.959–40933.177) | 0.81 % | 0.911 (0.898–0.917) |
| watch-winter-stumble with the probe | 9.60 % | 31.79 % | 53.14 % | 10.26 % | 8059.000 (7874.750–8199.750) | 77346.723 (75924.259–79008.100) | 3.94 % | 0.950 (0.944–0.956) |

### The bottleneck (reported, classified by the spec's rule on the medians over seeds)

Knowledge-bound iff P(seen) < P(raid | seen), comparing the medians of the per-seed values; action-bound otherwise. P(seen) = burials seen ÷ burials; P(raid | seen) = caches seen that were raided within span ÷ caches seen. Medians (IQR) over seeds 1–20.

| world, setting | P(seen) | P(raid \| seen) | caches seen per seed | class (medians) | seeds knowledge-bound | logs skipped as full | checks: sightings ÷ burials seen, raid takes ÷ raids |
|---|---|---|---|---|---|---|---|
| watch-winter | 0.883 (0.873–0.889) | 0.541 (0.536–0.553) | 7992.000 (7863.500–8155.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble | 0.888 (0.883–0.897) | 0.528 (0.521–0.533) | 11249.000 (11067.750–11487.500) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| theft-winter (stumbling only) | 0.000 (0.000–0.000) | NaN (NaN–NaN) | 0.000 (0.000–0.000) | unclassified | 0 of 20 | 0 | NaN (NaN–NaN); NaN (NaN–NaN) |
| watch-winter, span 7, raid_if always | 0.882 (0.876–0.891) | 0.616 (0.612–0.620) | 8271.000 (8138.000–8411.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 7, raid_if always | 0.886 (0.876–0.892) | 0.579 (0.577–0.585) | 11142.000 (10902.000–11228.750) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 7, value room | 0.885 (0.876–0.891) | 0.544 (0.534–0.548) | 8029.500 (7821.000–8157.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 7, value room | 0.888 (0.882–0.896) | 0.527 (0.520–0.534) | 11349.000 (11106.500–11466.750) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 1 | 0.845 (0.837–0.853) | 0.105 (0.099–0.110) | 6977.000 (6861.750–7052.500) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 1 | 0.879 (0.870–0.883) | 0.200 (0.194–0.203) | 11707.000 (11574.500–11962.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 2 | 0.857 (0.851–0.868) | 0.267 (0.261–0.279) | 7164.000 (7070.000–7323.250) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 2 | 0.880 (0.874–0.888) | 0.363 (0.356–0.369) | 11385.500 (11222.500–11553.500) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 3 | 0.869 (0.864–0.875) | 0.368 (0.360–0.372) | 7371.500 (7262.250–7439.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 3 | 0.886 (0.872–0.892) | 0.435 (0.431–0.441) | 11229.000 (10989.750–11348.250) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 7 (judged) | 0.883 (0.873–0.889) | 0.541 (0.536–0.553) | 7992.000 (7863.500–8155.000) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 7 (judged) | 0.888 (0.883–0.897) | 0.528 (0.521–0.533) | 11249.000 (11067.750–11487.500) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter, span 13 | 0.885 (0.877–0.891) | 0.625 (0.622–0.635) | 8504.500 (8406.000–8697.750) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble, span 13 | 0.889 (0.877–0.893) | 0.572 (0.566–0.578) | 11124.500 (11062.750–11360.750) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter with the probe | 0.883 (0.875–0.892) | 0.589 (0.578–0.597) | 8661.500 (8497.250–8897.250) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |
| watch-winter-stumble with the probe | 0.904 (0.894–0.910) | 0.580 (0.574–0.586) | 13487.500 (13146.250–13600.250) | action-bound | 0 of 20 | 0 | 1.000 (1.000–1.000); 1.000 (1.000–1.000) |

## Claim 2: Andersson and Krebs under watching (`watch-ak.*`)

Andersson and Krebs's condition with free burying is p_s > p_o (1978, p. 708). `watch-ak` is the calibration's item 4 (`theft-winter-half` at `find` 0.02, `dig_below: reserve`) with every agent watching. p_s and p_o are Minds 6's, amount-weighted, sugar still buried at 200 excluded. Fitness is ticks alive per founder over ticks 1–200 ÷ 200 (a field world).

| id | verdict | headline |
|---|---|---|
| `watch-ak.calibration` | **Holds** | item 4 qualifies (20 of 20 seeds): `theft-winter-half` at `find` 0.02, owners digging below their whole reserve (`dig_below: reserve`) |
| `watch-ak.flip` | **Holds** | p_s ÷ p_o < 1 in 20 of 20 seeds (need 16; 0 without a value) |
| `watch-ak.sign` | **Fails** | the signs agree in 62 of 100 runs, 62.0 % (need 80 %; 0 excluded) |
| `watch-ak.watched` | **Fails** | the hoarders' advantage under who: cheaters ≥ 0.05 below watching off in 0 of 20 seeds (need 16; 0 without a value) |
| `watch-ak.raiding` | **Fails** | hoarder fitness under who: hoarders ≥ 0.05 below watching off in 0 of 20 seeds (need 16; 0 without a value) |

Per seed p_s ÷ p_o (2a):

| seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| watch-ak | 0.6452 | 0.6864 | 0.6719 | 0.6314 | 0.6750 | 0.6734 | 0.6084 | 0.6880 | 0.6537 | 0.6799 | 0.6595 | 0.6470 | 0.6876 | 0.6084 | 0.6639 | 0.7252 | 0.6772 | 0.6032 | 0.6946 | 0.6756 |
| watching off | 3.7109 | 3.3424 | 3.9892 | 3.2060 | 3.5135 | 3.6792 | 3.9228 | 4.1244 | 3.9583 | 3.3043 | 3.2338 | 3.4040 | 3.5591 | 3.3163 | 3.6693 | 3.8748 | 3.5752 | 3.4185 | 4.0073 | 3.5727 |

Per seed hoarders' advantage (hoarder − cheater fitness; 2c needs off − cheaters ≥ 0.05):

| seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| watching off | -0.024 | -0.008 | -0.030 | -0.019 | 0.076 | -0.014 | 0.023 | -0.009 | 0.023 | -0.025 | -0.000 | -0.002 | 0.020 | 0.053 | 0.012 | 0.033 | 0.031 | 0.022 | -0.039 | -0.028 |
| who: cheaters | -0.040 | -0.019 | -0.040 | -0.028 | 0.053 | -0.034 | 0.009 | -0.008 | -0.004 | -0.035 | -0.019 | 0.012 | -0.005 | 0.043 | -0.001 | 0.021 | 0.018 | 0.014 | -0.048 | -0.040 |

Per seed hoarder fitness (2d needs off − hoarders ≥ 0.05):

| seed | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| watching off | 0.914 | 0.935 | 0.913 | 0.928 | 0.981 | 0.936 | 0.955 | 0.932 | 0.955 | 0.920 | 0.932 | 0.926 | 0.967 | 0.965 | 0.928 | 0.974 | 0.968 | 0.951 | 0.895 | 0.925 |
| who: hoarders | 0.909 | 0.943 | 0.918 | 0.922 | 0.968 | 0.933 | 0.944 | 0.936 | 0.949 | 0.919 | 0.898 | 0.933 | 0.971 | 0.966 | 0.925 | 0.973 | 0.961 | 0.957 | 0.895 | 0.925 |

Paired over seeds 1–20 (computed from the per-seed lists above; reported, not judged): 2c, watching off − who: cheaters in the hoarders' advantage, mean 0.012, 95 % t interval 0.008 to 0.017, above 0 in 18 of 20 seeds, at most 0.027 in any seed; 2d, watching off − who: hoarders in hoarder fitness, mean 0.003, 95 % t interval −0.002 to 0.007, above 0 in 11 of 20, at most 0.034. Both are well under the 0.05 the claims asked for in each seed.

### 2b: the signs by run set (judged setting)

| runs | signs agree | p_s ÷ p_o | hoarders' advantage | hoarder fitness | cheater fitness | raided sugar per seed |
|---|---|---|---|---|---|---|
| watching off | 9 of 20 | 3.574 (3.389–3.887) | -0.001 (-0.020–0.023) | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | 0.000 (0.000–0.000) |
| span 1 | 10 of 20 | 1.720 (1.679–1.786) | -0.003 (-0.027–0.019) | 0.938 (0.926–0.960) | 0.948 (0.937–0.951) | 3340.729 (3194.351–3614.613) |
| span 3 | 12 of 20 | 0.910 (0.868–0.936) | -0.009 (-0.032–0.015) | 0.935 (0.920–0.956) | 0.948 (0.938–0.955) | 7803.169 (7350.899–8398.180) |
| span 7 | 15 of 20 | 0.673 (0.647–0.682) | -0.023 (-0.042–0.001) | 0.916 (0.908–0.936) | 0.943 (0.936–0.950) | 9712.706 (9108.366–10185.131) |
| span 13 | 16 of 20 | 0.536 (0.518–0.557) | -0.032 (-0.049–-0.007) | 0.908 (0.893–0.923) | 0.942 (0.932–0.948) | 11316.035 (10475.505–11686.946) |

### The factorial and the switches (reported)

Each setting's own factorial (watching off, `who: hoarders`, `who: cheaters`, everyone) and, at span 7, its own 2b sweep. Counts are seeds out of 20 (2b: runs out of 100).

| setting | 2a: p_s ÷ p_o < 1 | 2c: advantage ≥ 0.05 lower under who: cheaters | 2d: hoarder fitness ≥ 0.05 lower under who: hoarders | 2b: signs agree |
|---|---|---|---|---|
| span 7, raid_if always | 20 of 20 | 0 of 20 | 0 of 20 | 62 of 100 |
| span 7, value room | 20 of 20 | 0 of 20 | 0 of 20 | 62 of 100 |
| span 1 | 0 of 20 | 0 of 20 | 0 of 20 | — |
| span 2 | 0 of 20 | 0 of 20 | 0 of 20 | — |
| span 3 | 20 of 20 | 0 of 20 | 0 of 20 | — |
| span 7 (judged) | 20 of 20 | 0 of 20 | 0 of 20 | 62 of 100 |
| span 13 | 20 of 20 | 0 of 20 | 0 of 20 | — |

| setting | world | p_s | p_o | p_s ÷ p_o | below 1 | hoarder fitness | cheater fitness | hoarders' advantage | raided sugar per seed |
|---|---|---|---|---|---|---|---|---|---|
| span 7, raid_if always | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 7, raid_if always | who: hoarders | 0.334 (0.318–0.338) | 0.659 (0.656–0.675) | 0.507 (0.471–0.516) | 20 of 20 | 0.929 (0.919–0.950) | 0.932 (0.926–0.938) | -0.002 (-0.020–0.023) | 15348.213 (14399.366–16118.853) |
| span 7, raid_if always | who: cheaters | 0.576 (0.570–0.583) | 0.412 (0.403–0.419) | 1.398 (1.364–1.445) | 0 of 20 | 0.933 (0.922–0.959) | 0.950 (0.944–0.958) | -0.013 (-0.033–0.006) | 3650.239 (3573.357–3731.002) |
| span 7, raid_if always | everyone | 0.363 (0.353–0.372) | 0.614 (0.605–0.622) | 0.592 (0.571–0.617) | 20 of 20 | 0.912 (0.903–0.928) | 0.942 (0.929–0.948) | -0.022 (-0.043–-0.007) | 10530.614 (9700.179–11115.207) |
| span 7, value room | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 7, value room | who: hoarders | 0.335 (0.321–0.345) | 0.659 (0.648–0.670) | 0.509 (0.479–0.531) | 20 of 20 | 0.931 (0.922–0.956) | 0.935 (0.928–0.941) | 0.000 (-0.015–0.024) | 15410.871 (14267.077–16295.020) |
| span 7, value room | who: cheaters | 0.680 (0.670–0.693) | 0.319 (0.305–0.327) | 2.131 (2.045–2.271) | 0 of 20 | 0.942 (0.928–0.962) | 0.951 (0.941–0.958) | -0.008 (-0.023–0.015) | 2466.742 (2363.572–2550.602) |
| span 7, value room | everyone | 0.326 (0.314–0.330) | 0.663 (0.656–0.675) | 0.490 (0.467–0.502) | 20 of 20 | 0.931 (0.911–0.946) | 0.947 (0.939–0.955) | -0.017 (-0.036–-0.003) | 15120.748 (14229.466–16045.708) |
| span 1 | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 1 | who: hoarders | 0.648 (0.630–0.652) | 0.351 (0.347–0.369) | 1.845 (1.707–1.880) | 0 of 20 | 0.936 (0.925–0.961) | 0.941 (0.932–0.944) | 0.006 (-0.019–0.023) | 3541.336 (3192.201–3889.895) |
| span 1 | who: cheaters | 0.741 (0.733–0.745) | 0.259 (0.254–0.267) | 2.866 (2.751–2.927) | 0 of 20 | 0.937 (0.926–0.960) | 0.945 (0.935–0.949) | -0.001 (-0.023–0.017) | 676.368 (621.019–701.726) |
| span 1 | everyone | 0.632 (0.626–0.641) | 0.367 (0.359–0.373) | 1.720 (1.679–1.786) | 0 of 20 | 0.938 (0.926–0.960) | 0.948 (0.937–0.951) | -0.003 (-0.027–0.019) | 3340.729 (3194.351–3614.613) |
| span 2 | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 2 | who: hoarders | 0.504 (0.494–0.521) | 0.494 (0.476–0.504) | 1.020 (0.980–1.095) | 8 of 20 | 0.940 (0.926–0.963) | 0.936 (0.929–0.942) | 0.008 (-0.014–0.025) | 8583.034 (7775.486–9540.569) |
| span 2 | who: cheaters | 0.680 (0.671–0.685) | 0.318 (0.312–0.326) | 2.136 (2.062–2.195) | 0 of 20 | 0.936 (0.927–0.961) | 0.949 (0.940–0.958) | -0.007 (-0.035–0.011) | 1664.451 (1569.854–1711.921) |
| span 2 | everyone | 0.519 (0.515–0.536) | 0.474 (0.464–0.481) | 1.096 (1.071–1.157) | 0 of 20 | 0.939 (0.926–0.956) | 0.949 (0.940–0.954) | -0.010 (-0.027–0.019) | 6416.639 (6014.875–6680.817) |
| span 3 | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 3 | who: hoarders | 0.420 (0.405–0.430) | 0.579 (0.568–0.592) | 0.724 (0.683–0.757) | 20 of 20 | 0.940 (0.929–0.957) | 0.936 (0.926–0.944) | 0.008 (-0.013–0.028) | 12433.693 (11168.932–13186.258) |
| span 3 | who: cheaters | 0.649 (0.645–0.658) | 0.349 (0.337–0.352) | 1.859 (1.830–1.945) | 0 of 20 | 0.941 (0.928–0.962) | 0.951 (0.943–0.958) | -0.008 (-0.036–0.014) | 2223.148 (2130.402–2290.089) |
| span 3 | everyone | 0.472 (0.460–0.479) | 0.517 (0.512–0.531) | 0.910 (0.868–0.936) | 20 of 20 | 0.935 (0.920–0.956) | 0.948 (0.938–0.955) | -0.009 (-0.032–0.015) | 7803.169 (7350.899–8398.180) |
| span 7 (judged) | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 7 (judged) | who: hoarders | 0.335 (0.322–0.343) | 0.660 (0.651–0.674) | 0.509 (0.478–0.527) | 20 of 20 | 0.934 (0.921–0.958) | 0.934 (0.926–0.942) | 0.000 (-0.018–0.026) | 15066.759 (14460.687–16194.677) |
| span 7 (judged) | who: cheaters | 0.584 (0.577–0.591) | 0.404 (0.398–0.412) | 1.448 (1.402–1.480) | 0 of 20 | 0.941 (0.924–0.960) | 0.952 (0.943–0.958) | -0.007 (-0.034–0.012) | 3266.374 (3197.510–3345.756) |
| span 7 (judged) | everyone | 0.393 (0.383–0.397) | 0.587 (0.584–0.592) | 0.673 (0.647–0.682) | 20 of 20 | 0.916 (0.908–0.936) | 0.943 (0.936–0.950) | -0.023 (-0.042–0.001) | 9712.706 (9108.366–10185.131) |
| span 13 | watching off | 0.781 (0.772–0.795) | 0.219 (0.205–0.228) | 3.574 (3.389–3.887) | 0 of 20 | 0.933 (0.926–0.958) | 0.938 (0.932–0.943) | -0.001 (-0.020–0.023) | 0.000 (0.000–0.000) |
| span 13 | who: hoarders | 0.299 (0.286–0.310) | 0.694 (0.683–0.705) | 0.430 (0.406–0.454) | 20 of 20 | 0.933 (0.919–0.950) | 0.935 (0.927–0.940) | 0.000 (-0.019–0.013) | 16500.120 (15521.081–17067.280) |
| span 13 | who: cheaters | 0.560 (0.554–0.564) | 0.425 (0.422–0.432) | 1.319 (1.278–1.335) | 0 of 20 | 0.938 (0.923–0.960) | 0.953 (0.942–0.958) | -0.012 (-0.031–0.013) | 3805.093 (3710.034–3871.041) |
| span 13 | everyone | 0.341 (0.334–0.350) | 0.634 (0.628–0.640) | 0.536 (0.518–0.557) | 20 of 20 | 0.908 (0.893–0.923) | 0.942 (0.932–0.948) | -0.032 (-0.049–-0.007) | 11316.035 (10475.505–11686.946) |
| the probe (a raid also harvests its site) | watch-ak with the probe | 0.354 (0.343–0.372) | 0.633 (0.614–0.641) | 0.560 (0.539–0.604) | 20 of 20 | 0.927 (0.920–0.951) | 0.950 (0.938–0.954) | -0.018 (-0.037–0.007) | 12900.363 (12159.801–13538.066) |

## Claim 3: producers and scroungers (`watch-scroungers.*`, seeds 1–60)

Variant forgo: `watch-scroungers-forgo` with `cheaters` = `watchers` = s (the same agents, the scroungers). Variant bury: `watch-scroungers` with `watchers` = s and no cheaters. s = 0.1, …, 0.9; span 7 judged. The advantage is watcher (scrounger) − other fitness, ticks alive per founder ÷ 200; the world's fitness is everyone's.

| id | verdict | headline |
|---|---|---|
| `watch-scroungers.frequency-forgo` | **Holds** | falls: mean per-seed slope -0.2012, 95 % CI -0.2283 to -0.1740, 90 % CI -0.2238 to -0.1785 (t, df 59); 59 of 60 slopes below 0 |
| `watch-scroungers.frequency-bury` | **Fails** | flat: mean per-seed slope 0.0112, 95 % CI -0.0075 to 0.0300, 90 % CI -0.0044 to 0.0269 (t, df 59); 26 of 60 slopes below 0 |
| `watch-scroungers.mix` | **Fails** | [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) |
| `watch-scroungers.dilemma` | **Holds** | [watchers ahead: Holds] pooled watcher advantage: mean 0.0093, 95 % CI 0.0027 to 0.0159 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0081, 95 % CI -0.0104 to -0.0058 (t, df 59) |

### Variant forgo (scroungers who never bury and forgo producing)

Per-seed slopes of the advantage on s (seeds 1–60, in order): -0.135 -0.007 -0.146 -0.215 -0.356 -0.232 -0.337 -0.312 -0.220 -0.044 -0.192 -0.231 -0.271 -0.231 -0.208 -0.015 -0.086 -0.262 -0.169 -0.275 -0.165 -0.253 -0.082 -0.170 -0.314 -0.212 -0.150 -0.214 -0.170 -0.091 -0.237 -0.151 -0.134 -0.137 -0.469 -0.073 -0.239 -0.182 -0.275 -0.267 -0.119 -0.114 -0.260 -0.358 -0.174 -0.331 -0.306 -0.341 -0.119 -0.344 -0.185 0.147 -0.306 -0.280 -0.221 -0.103 -0.095 -0.212 -0.103 -0.316.

Per-seed down-crossings (first sign change from above 0 to at most 0, interpolated; "none" when there is none): none none none none 0.21 none 0.18 none none none none none 0.16 none 0.12 none none none 0.17 0.10 none 0.31 none none 0.41 0.22 0.22 0.31 none none none 0.11 none none 0.21 none none none none none none none none 0.13 none none none none none 0.12 none none 0.13 none 0.26 none none none none 0.24 (18 of 60 seeds cross).

By share (medians, IQR, over seeds 1–60):

| s | advantage | watchers | others | world | raided sugar per seed |
|---|---|---|---|---|---|
| 0.1 | -0.042 (-0.111–-0.006) | 0.856 (0.780–0.884) | 0.898 (0.882–0.907) | 0.893 (0.880–0.899) | 2652.720 (2369.436–2921.847) |
| 0.2 | -0.060 (-0.117–-0.032) | 0.825 (0.784–0.867) | 0.899 (0.883–0.910) | 0.881 (0.873–0.892) | 4832.060 (4467.049–5062.964) |
| 0.3 | -0.079 (-0.112–-0.049) | 0.810 (0.788–0.842) | 0.898 (0.882–0.911) | 0.871 (0.862–0.885) | 6251.267 (5984.838–6585.481) |
| 0.4 | -0.080 (-0.126–-0.052) | 0.808 (0.776–0.838) | 0.893 (0.877–0.910) | 0.859 (0.847–0.870) | 7524.417 (7314.533–7806.830) |
| 0.5 | -0.112 (-0.133–-0.083) | 0.781 (0.762–0.798) | 0.890 (0.876–0.909) | 0.838 (0.825–0.846) | 7989.380 (7838.042–8199.086) |
| 0.6 | -0.148 (-0.170–-0.125) | 0.745 (0.730–0.759) | 0.890 (0.875–0.904) | 0.805 (0.785–0.815) | 7963.671 (7694.891–8174.113) |
| 0.7 | -0.181 (-0.214–-0.165) | 0.702 (0.676–0.714) | 0.887 (0.871–0.901) | 0.754 (0.741–0.770) | 7253.924 (6958.114–7487.139) |
| 0.8 | -0.228 (-0.270–-0.189) | 0.656 (0.632–0.673) | 0.888 (0.857–0.911) | 0.700 (0.683–0.711) | 5655.367 (5436.254–5845.706) |
| 0.9 | -0.169 (-0.227–-0.127) | 0.671 (0.655–0.691) | 0.847 (0.806–0.884) | 0.687 (0.677–0.708) | 3426.295 (3217.667–3639.354) |

Under the reported settings (each judge rerun on that setting's worlds, seeds 1–60):

| setting | 3a | 3a: mean slope, 95 % CI, 90 % CI | 3b | median advantage by share (s = 0.1 … 0.9) |
|---|---|---|---|---|
| span 7, raid_if always | Holds (falls) | -0.2012; -0.2283 to -0.1740; -0.2238 to -0.1785; 59 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) | -0.042 -0.060 -0.079 -0.080 -0.112 -0.148 -0.181 -0.228 -0.169 |
| span 7, value room | Holds (falls) | -0.2012; -0.2283 to -0.1740; -0.2238 to -0.1785; 59 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) | -0.042 -0.060 -0.079 -0.080 -0.112 -0.148 -0.181 -0.228 -0.169 |
| span 1 | Fails (rising) | 0.3522; 0.3293 to 0.3752; 0.3331 to 0.3714; 0 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 0 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 48 of 60 seeds (need 48; 0 without a value) | -0.376 -0.365 -0.334 -0.319 -0.283 -0.271 -0.226 -0.148 -0.057 |
| span 2 | Fails (rising) | 0.1256; 0.0997 to 0.1514; 0.1040 to 0.1472; 5 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 1 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Fails] advantage < 0 in 33 of 60 seeds (need 48; 0 without a value) | -0.188 -0.190 -0.182 -0.198 -0.211 -0.217 -0.212 -0.147 -0.012 |
| span 3 | Holds (falls) | -0.0327; -0.0585 to -0.0070; -0.0542 to -0.0112; 39 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 4 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Fails] advantage < 0 in 47 of 60 seeds (need 48; 0 without a value) | -0.087 -0.115 -0.121 -0.148 -0.150 -0.181 -0.206 -0.176 -0.042 |
| span 7 (judged) | Holds (falls) | -0.2012; -0.2283 to -0.1740; -0.2238 to -0.1785; 59 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) | -0.042 -0.060 -0.079 -0.080 -0.112 -0.148 -0.181 -0.228 -0.169 |
| span 13 | Holds (falls) | -0.2282; -0.2555 to -0.2009; -0.2510 to -0.2054; 58 of 60 below 0 | 3b Fails: [above 0 at s = 0.1: Fails] advantage > 0 in 13 of 60 seeds (need 48; 0 without a value) [below 0 at s = 0.9: Holds] advantage < 0 in 60 of 60 seeds (need 48; 0 without a value) | -0.052 -0.057 -0.061 -0.080 -0.094 -0.116 -0.164 -0.222 -0.234 |

### Variant bury (watchers who also bury)

Per-seed slopes of the advantage on s (seeds 1–60, in order): 0.067 0.102 -0.001 -0.009 -0.118 0.076 -0.082 0.009 -0.047 0.014 0.049 0.071 -0.017 -0.051 -0.056 -0.007 0.151 -0.100 0.048 -0.025 0.074 -0.064 0.086 0.026 -0.023 0.082 0.065 0.027 0.056 0.006 0.036 0.104 -0.029 0.050 -0.071 0.068 -0.118 0.071 0.036 -0.008 0.112 0.001 -0.079 -0.076 0.052 -0.150 0.031 -0.074 0.015 -0.020 0.058 0.183 -0.066 -0.021 -0.047 0.191 0.055 0.005 0.034 -0.074.

Per-seed down-crossings (first sign change from above 0 to at most 0, interpolated; "none" when there is none): 0.34 0.31 0.32 0.34 0.26 0.54 0.84 0.34 0.15 0.16 0.23 0.29 0.46 0.47 0.48 0.30 0.89 0.64 0.28 0.54 0.57 0.37 0.35 0.70 0.63 0.23 0.47 0.38 none 0.57 0.71 0.57 0.28 0.59 0.45 0.13 0.44 0.37 0.24 0.36 0.55 0.60 0.28 0.46 0.49 0.43 0.13 0.53 0.29 0.31 0.51 0.36 none 0.63 none 0.40 0.53 0.47 0.38 0.48 (57 of 60 seeds cross).

By share (medians, IQR, over seeds 1–60):

| s | advantage | watchers | others | world | raided sugar per seed |
|---|---|---|---|---|---|
| 0.1 | 0.012 (-0.025–0.047) | 0.900 (0.869–0.951) | 0.897 (0.887–0.911) | 0.898 (0.889–0.909) | 2157.621 (1868.503–2442.093) |
| 0.2 | 0.014 (-0.026–0.035) | 0.906 (0.869–0.932) | 0.895 (0.886–0.914) | 0.899 (0.887–0.910) | 4826.002 (4427.028–5292.997) |
| 0.3 | 0.013 (-0.007–0.037) | 0.902 (0.887–0.924) | 0.895 (0.879–0.909) | 0.898 (0.888–0.907) | 7428.127 (6894.012–7709.132) |
| 0.4 | 0.009 (-0.028–0.033) | 0.902 (0.885–0.918) | 0.900 (0.878–0.916) | 0.900 (0.888–0.907) | 9912.953 (9296.361–10680.236) |
| 0.5 | 0.007 (-0.016–0.028) | 0.904 (0.892–0.918) | 0.894 (0.879–0.908) | 0.901 (0.889–0.908) | 12654.216 (12034.111–13118.868) |
| 0.6 | 0.003 (-0.016–0.024) | 0.897 (0.888–0.913) | 0.894 (0.874–0.908) | 0.894 (0.885–0.907) | 15236.372 (14682.802–15987.507) |
| 0.7 | 0.019 (-0.011–0.042) | 0.899 (0.887–0.914) | 0.885 (0.865–0.909) | 0.896 (0.889–0.904) | 18202.277 (17194.376–18596.858) |
| 0.8 | 0.005 (-0.016–0.027) | 0.898 (0.883–0.904) | 0.889 (0.859–0.910) | 0.894 (0.883–0.903) | 20996.696 (20376.568–21954.911) |
| 0.9 | 0.015 (-0.020–0.048) | 0.894 (0.883–0.906) | 0.881 (0.845–0.916) | 0.892 (0.881–0.903) | 24141.149 (23380.904–24760.153) |

Under the reported settings (each judge rerun on that setting's worlds, seeds 1–60):

| setting | 3a | 3a: mean slope, 95 % CI, 90 % CI | 3c | median advantage by share (s = 0.1 … 0.9) |
|---|---|---|---|---|
| span 7, raid_if always | Fails (flat) | 0.0147; -0.0034 to 0.0327; -0.0004 to 0.0298; 27 of 60 below 0 | 3c Holds: [watchers ahead: Holds] pooled watcher advantage: mean 0.0131, 95 % CI 0.0066 to 0.0196 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0192, 95 % CI -0.0218 to -0.0166 (t, df 59) | 0.009 0.010 0.011 0.007 0.015 0.011 0.017 0.019 0.013 |
| span 7, value room | Fails (flat) | 0.0113; -0.0074 to 0.0300; -0.0043 to 0.0269; 28 of 60 below 0 | 3c Holds: [watchers ahead: Holds] pooled watcher advantage: mean 0.0093, 95 % CI 0.0029 to 0.0157 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0091, 95 % CI -0.0109 to -0.0072 (t, df 59) | 0.012 0.014 0.012 0.009 0.008 0.006 0.018 0.006 0.008 |
| span 1 | Fails (flat) | -0.0062; -0.0248 to 0.0125; -0.0217 to 0.0094; 32 of 60 below 0 | 3c Fails: [watchers ahead: Fails] pooled watcher advantage: mean 0.0007, 95 % CI -0.0062 to 0.0077 (t, df 59) [the world's fitness falls with s: Fails] per-seed slope of world fitness on s: mean -0.0015, 95 % CI -0.0034 to 0.0005 (t, df 59) | 0.015 0.002 0.005 0.000 -0.002 -0.002 -0.006 -0.007 -0.007 |
| span 2 | Fails (flat) | -0.0056; -0.0224 to 0.0112; -0.0196 to 0.0084; 34 of 60 below 0 | 3c Fails: [watchers ahead: Fails] pooled watcher advantage: mean -0.0015, 95 % CI -0.0079 to 0.0049 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0062, 95 % CI -0.0082 to -0.0042 (t, df 59) | -0.003 0.005 0.002 -0.004 -0.003 -0.005 0.002 -0.006 -0.007 |
| span 3 | Fails (flat) | -0.0083; -0.0260 to 0.0094; -0.0230 to 0.0065; 35 of 60 below 0 | 3c Fails: [watchers ahead: Fails] pooled watcher advantage: mean -0.0027, 95 % CI -0.0093 to 0.0039 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0042, 95 % CI -0.0063 to -0.0021 (t, df 59) | 0.007 0.001 0.005 -0.009 -0.003 -0.005 -0.006 -0.008 -0.011 |
| span 7 (judged) | Fails (flat) | 0.0112; -0.0075 to 0.0300; -0.0044 to 0.0269; 26 of 60 below 0 | 3c Holds: [watchers ahead: Holds] pooled watcher advantage: mean 0.0093, 95 % CI 0.0027 to 0.0159 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0081, 95 % CI -0.0104 to -0.0058 (t, df 59) | 0.012 0.014 0.013 0.009 0.007 0.003 0.019 0.005 0.015 |
| span 13 | Fails (flat) | 0.0102; -0.0087 to 0.0291; -0.0056 to 0.0260; 28 of 60 below 0 | 3c Holds: [watchers ahead: Holds] pooled watcher advantage: mean 0.0243, 95 % CI 0.0180 to 0.0306 (t, df 59) [the world's fitness falls with s: Holds] per-seed slope of world fitness on s: mean -0.0074, 95 % CI -0.0099 to -0.0049 (t, df 59) | 0.024 0.028 0.024 0.024 0.018 0.026 0.025 0.026 0.029 |

### 3b per seed (variant forgo)

Advantage at s = 0.1 (seeds 1–60): -0.127 -0.124 -0.099 -0.046 0.079 -0.059 0.041 -0.007 -0.130 -0.078 -0.180 -0.032 0.082 -0.042 0.020 -0.262 -0.111 -0.107 0.068 0.000 -0.016 -0.012 -0.037 -0.151 -0.011 0.015 -0.048 -0.020 -0.154 -0.176 -0.145 0.017 -0.032 -0.174 0.037 -0.042 -0.116 -0.067 -0.028 -0.035 -0.084 -0.034 -0.066 0.035 -0.112 -0.002 -0.032 -0.007 -0.095 0.014 -0.172 -0.190 0.017 -0.104 -0.002 -0.081 -0.152 -0.041 -0.097 0.133. Above 0 in 13 of 60 (the judge's count, on unrounded values: seed 20's advantage prints as 0.000 at three decimals but is above 0, so a count of the printed values gives 12).

Advantage at s = 0.9 (seeds 1–60): -0.164 -0.104 -0.103 -0.218 -0.273 -0.234 -0.179 -0.226 -0.244 -0.100 -0.194 -0.132 -0.072 -0.170 -0.124 -0.188 -0.141 -0.313 -0.040 -0.153 -0.127 -0.132 -0.064 -0.176 -0.139 -0.091 -0.087 -0.155 -0.279 -0.137 -0.272 -0.100 -0.162 -0.197 -0.317 -0.001 -0.238 -0.159 -0.176 -0.136 -0.223 -0.130 -0.178 -0.258 -0.146 -0.283 -0.230 -0.278 -0.105 -0.246 -0.241 -0.016 -0.210 -0.241 -0.101 -0.168 -0.207 -0.220 -0.126 -0.174. Below 0 in 60 of 60.

The first round's probe is reported for claims 1 and 2 only, as the spec's "Also reported" list fixes it; claim 3 has none.

### 3c per seed (variant bury)

Pooled advantage, the per-seed mean over shares (seeds 1–60): -0.003 0.003 0.006 -0.018 0.013 -0.013 0.028 -0.000 -0.017 0.013 -0.010 0.018 0.032 0.007 0.041 -0.002 -0.041 0.003 0.059 0.024 0.016 0.021 0.039 0.006 0.032 0.010 0.058 -0.002 -0.034 0.018 -0.045 0.047 0.023 -0.015 0.016 0.025 -0.012 0.008 0.020 0.024 -0.036 -0.015 0.006 0.015 0.036 0.000 0.003 0.002 0.044 -0.007 -0.025 0.028 0.039 -0.053 0.060 -0.021 -0.011 0.024 0.027 0.041.

Per-seed slope of the world's fitness on s (seeds 1–60): -0.009 -0.001 -0.009 -0.005 -0.020 -0.005 -0.029 0.014 -0.015 -0.001 -0.021 -0.000 0.002 -0.014 -0.008 -0.011 0.000 -0.017 0.002 -0.030 0.013 0.005 -0.013 -0.019 -0.004 -0.001 -0.001 -0.020 -0.001 -0.004 -0.014 -0.010 -0.001 -0.011 0.001 0.003 -0.004 -0.014 -0.004 -0.006 -0.013 -0.015 -0.010 -0.004 -0.014 -0.009 -0.010 -0.016 -0.009 -0.016 -0.010 -0.018 -0.004 -0.025 -0.017 -0.006 -0.001 0.000 -0.010 0.000.

Partly seen before the run (the spec's pre-mortem): under `raid_if: always` the design audit saw world survival fall from 0.714 to 0.551; under `better` its variant cut the watchers' lead to 1 point.

## Notes on reading these results

- **Variant forgo is identical under `raid_if: always` and `value: room`.** By construction: a forgoing scrounger skips the `better` test (the site it forgoes is worth nothing to it; a Task 2 ruling), and under `loot: eat` its room is infinite, so `room` equals `amount`. The rows repeat the judged one exactly.
- **Spans 1 and 2 behave differently.** In `watch-winter`, h is below `theft-winter`'s in every seed at spans 1 and 2 (0.85 % and 2.49 % a day against 3.00 %), and above it at spans 3, 7 and 13. In `watch-ak`, p_s ÷ p_o stays above 1 at spans 1 and 2 (1.72 and 1.10) and falls below 1 at 3, 7 and 13. In variant forgo, 3a reads rising at spans 1 and 2 (the scroungers' shortfall is largest at low share) and falls at 3, 7 and 13. In variant bury, 3c's watcher lead fails at spans 1, 2 and 3 (pooled 0.0007, 95 % CI −0.0062 to 0.0077; −0.0015; −0.0027) and holds only at 7 and 13.
- **The bottleneck is action-bound in every world and setting:** watchers see 85–90 % of burials, and raid 10–63 % of the caches they see.
- **2b's 62 of 100 under all three span-7 settings is a coincidence of totals,** checked by recount: judged 9 + 10 + 12 + 15 + 16; `raid_if: always` 9 + 8 + 13 + 16 + 16; `value: room` 9 + 9 + 12 + 15 + 17 (watching off, then spans 1, 3, 7, 13).
- **Fitness in the field is compressed:** ticks alive per founder ÷ 200 sits near 0.9 because most agents who die do so in the winter (in `watch-winter`, from the medians, at about tick 150 on average), so a 0.05 drop (2c, 2d) is large on this scale; survival at 200 per founder moves several times as much (see `minds8b-presets.md`).

## Usage (a check, not a claim)

Generated by `cargo run --release -- --usage` in `survey/` (`survey/src/claims/minds8b.rs`). A check, not a claim (docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md, "Usage"): in every preset with watching on, raids take sugar (Σ `raided` over ticks 1–200 above 0) in at least 16 of 20 seeds.

| preset | seeds with raided sugar | passes | raids per seed | raided sugar per seed |
|---|---|---|---|---|
| `watch-winter` | 20 of 20 | yes | 4351.000 (IQR 4249.500–4490.750) | 27011.030 (IQR 26494.726–28781.900) |
| `watch-winter-stumble` | 20 of 20 | yes | 5989.000 (IQR 5888.750–6053.250) | 44688.010 (IQR 43308.285–46628.732) |
| `watch-half` | 20 of 20 | yes | 2812.500 (IQR 2676.750–2889.250) | 10767.678 (IQR 9607.415–11467.445) |
| `watch-scroungers` | 20 of 20 | yes | 2243.500 (IQR 2194.250–2331.250) | 12912.578 (IQR 12176.066–13127.126) |
| `watch-scroungers-only` | 20 of 20 | yes | 1285.500 (IQR 1262.500–1309.750) | 1432.044 (IQR 1406.652–1485.365) |
| `watch-scroungers-forgo` | 20 of 20 | yes | 2418.500 (IQR 2342.000–2451.500) | 8095.943 (IQR 7904.043–8262.386) |
| `watch-ak` | 20 of 20 | yes | 3011.000 (IQR 2892.000–3105.500) | 9712.706 (IQR 9108.366–10185.131) |
| `watch-arena` | 20 of 20 | yes | 43.500 (IQR 35.750–52.250) | 198.164 (IQR 176.897–252.602) |

**Result:** raids take sugar in every watching preset.

## The presets' measures

Reported, not judged, in `survey/out/minds8b-presets.md` (`--presets`): every watching preset at seeds 1–20 as set, and paired against the same world with watching off, `raid_if: always`, `value: room`, the probe and (where scroungers exist) the other `scrounge` setting. The presets' descriptions are written from it and from the claims above.
