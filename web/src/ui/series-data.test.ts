import { describe, expect, it } from 'vitest';
import { chartKey } from '../protocol';
import type { Config, HoardConfig, ModelConfig } from '../types';
import type { HoardCharts } from '../protocol';
import {
  HOARD_CHARTS,
  hoardGenerationTable,
  hoardSeasonTable,
  bandData,
  barsData,
  SPATIAL_HOARDING_CHARTS,
  chartsBehind,
  distributionsDue,
  distributionWants,
  emptyTable,
  histTable,
  isEthnic,
  lineData,
  MODEL_CHARTS,
  overlayData,
  positionBars,
  positionSteps,
  shownCharts,
  stumbledData,
  showsAgeHist,
  showsGoodWealth,
  showsTagHist,
  showsTotalWealth,
  supplyDemandTable,
  showsForModel,
  timeAxisLabel,
  twoGoods,
  worldLines,
  yearTickLabels,
  type LineData,
} from './series-data';

const group = { ticks: Float64Array.of(0, 5, 9), columns: [Float64Array.of(1, NaN, 3), Float64Array.of(0.5, 0.5, NaN)] };

describe('series data', () => {
  it('uses the ticks as x and turns NaN into gaps', () => {
    expect(lineData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [0.5, 0.5, null],
    ]);
  });

  it('draws the price band around the mean', () => {
    expect(bandData(group)).toEqual([
      [0, 5, 9],
      [1, null, 3],
      [1.5, null, null],
      [0.5, null, null],
    ]);
  });
});

describe('chartsBehind', () => {
  const cache = (groups: Record<string, typeof group>) => (names: string[]) => groups[chartKey(names)];

  it('is behind for a group that is not cached', () => {
    expect(chartsBehind([['population']], 9, cache({}))).toBe(true);
  });

  it('is caught up while the cached group reaches the tick, and behind once the tick moves on', () => {
    const cached = cache({ population: group }); // last tick: 9
    expect(chartsBehind([['population']], 9, cached)).toBe(false);
    expect(chartsBehind([['population']], 10, cached)).toBe(true);
  });

  it('is behind if any group in the list is behind or missing', () => {
    expect(chartsBehind([['population'], ['gini']], 9, cache({ population: group, gini: group }))).toBe(false);
    expect(chartsBehind([['population'], ['gini']], 9, cache({ population: group }))).toBe(true);
  });

  it('looks multi-line groups up by their chart key', () => {
    expect(chartsBehind([['population', 'gini']], 9, cache({ [chartKey(['population', 'gini'])]: group }))).toBe(false);
  });
});

describe('two worlds on one axis', () => {
  it('puts tables on the union of their x values: holes are undefined (drawn through), gaps stay null', () => {
    const a: LineData = [[0, 2, 4], [1, null, 3]];
    const b: LineData = [[0, 3, 4], [5, 6, 7], [8, 9, 10]];
    expect(overlayData([a, b])).toEqual([
      [0, 2, 3, 4],
      [1, null, undefined, 3],
      [5, undefined, 6, 7],
      [8, undefined, 9, 10],
    ]);
  });

  it('gives a missing group an empty table with the right number of lines', () => {
    expect(emptyTable(2)).toEqual([[], [], []]);
    expect(overlayData([emptyTable(1), [[1, 2], [5, 6]]])).toEqual([[1, 2], [undefined, undefined], [5, 6]]);
  });

  it('draws a histogram as bars or as a step outline over its bin edges', () => {
    const hist = Float64Array.of(2, 5, 0, 1);
    expect(barsData(hist)).toEqual([[1, 3, 5], [5, 0, 1]]);
    expect(histTable(hist)).toEqual([[0, 2, 4, 6], [5, 0, 1, 0]]);
    expect(histTable(null)).toEqual([[], []]);
  });

  it('turns supply and demand into two curves and two marked points', () => {
    const sd = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, 1.1, 5, 2, 9);
    expect(supplyDemandTable(sd)).toEqual([
      [0.5, 1, 2],
      [9, 5, 1],
      [1, 5, 9],
      [null, 5, null],
      [null, null, 9],
    ]);
    const none = Float64Array.of(3, 0.5, 1, 2, 9, 5, 1, 1, 5, 9, NaN, NaN, NaN, NaN);
    expect(supplyDemandTable(none).slice(3)).toEqual([
      [null, null, null],
      [null, null, null],
    ]);
    expect(supplyDemandTable(undefined)).toEqual([[], [], [], [], []]);
  });
});

describe('Chapter VI histograms', () => {
  const config = (goods: number, lifespan: boolean, culture: boolean) =>
    ({ goods: Array.from({ length: goods }, () => ({})), lifespan: { enabled: lifespan }, culture: { enabled: culture } }) as unknown as Config;

  it('shows the age histogram while lifetimes are finite and the tag histogram while culture is on', () => {
    expect([showsAgeHist(config(1, true, false)), showsTagHist(config(1, true, false))]).toEqual([true, false]);
    expect([showsAgeHist(config(1, false, true)), showsTagHist(config(1, false, true))]).toEqual([false, true]);
  });

  it('asks for the distributions each world draws', () => {
    expect(distributionWants(config(1, false, false))).toEqual({ lorenz: true, wealthHist: true });
    expect(distributionWants(config(2, true, true))).toEqual({
      lorenz: true,
      wealthHist: true,
      supplyDemand: true,
      lorenzTotal: true,
      goodWealthHists: true,
      ageHist: true,
      tagHist: true,
    });
  });

  it('fetches distributions again only once the tick moved or they went stale, at most every 250 ms', () => {
    const d = { at: 1000, tick: 7, stale: false };
    expect(distributionsDue(d, 7, 5000, 250)).toBe(false); // paused and caught up
    expect(distributionsDue(d, 8, 1100, 250)).toBe(false); // too soon
    expect(distributionsDue(d, 8, 1250, 250)).toBe(true);
    expect(distributionsDue({ ...d, stale: true }, 7, 1250, 250)).toBe(true);
  });

  it('draws the tag histogram as bars at positions 1…L or as a step outline around them', () => {
    const pct = Float64Array.of(49, 100, 0);
    expect(positionBars(pct)).toEqual([[1, 2, 3], [49, 100, 0]]);
    expect(positionSteps(pct)).toEqual([[0.5, 1.5, 2.5, 3.5], [49, 100, 0, 0]]);
    expect(positionBars(null)).toEqual([[], []]);
    expect(positionSteps(undefined)).toEqual([[], []]);
  });

  it('draws the age histogram like the wealth histogram ([bin, counts…])', () => {
    const ages = Float64Array.of(5, 3, 0, 2);
    expect(barsData(ages)).toEqual([[2.5, 7.5, 12.5], [3, 0, 2]]);
    expect(histTable(ages)).toEqual([[0, 5, 10, 15], [3, 0, 2, 0]]);
  });
});

describe('wealth views', () => {
  const goods = (n: number) => ({ goods: Array.from({ length: n }, () => ({})) }) as unknown as Config;

  it("show total wealth and each good's histogram only with two or more goods", () => {
    expect(showsTotalWealth(goods(1))).toBe(false);
    expect(showsTotalWealth(goods(2))).toBe(true);
    expect(showsGoodWealth(0)(goods(1))).toBe(false);
    expect([0, 1, 2].map((g) => showsGoodWealth(g)(goods(2)))).toEqual([true, true, false]);
  });

  it('twoGoods and showsTotalWealth read the same world the same way, under their own names', () => {
    expect(twoGoods(goods(1))).toBe(false);
    expect(twoGoods(goods(2))).toBe(true);
    expect(twoGoods).toBe(showsTotalWealth);
  });
});

describe('shownCharts', () => {
  it('keeps only the charts on show, in order, each under its own caption', () => {
    const charts = [
      { name: 'Wealth distribution · sugar', hidden: false },
      { name: 'Wealth distribution · spice', hidden: false },
      { name: 'Wealth distribution · salt', hidden: true },
      { name: 'Age histogram', hidden: true },
      { name: 'Cultural tags (% zeros by position)', hidden: false },
    ];
    const shown = shownCharts(charts);
    expect(shown.map((c) => c.name)).toEqual(['Wealth distribution · sugar', 'Wealth distribution · spice', 'Cultural tags (% zeros by position)']);
    // Distinct captions (not the shared 'Wealth distribution' title) keep every exported file name unique.
    expect(new Set(shown.map((c) => c.name)).size).toBe(shown.length);
  });
});

describe('model charts', () => {
  it('shows a model’s charts while some world on screen runs it', () => {
    expect(showsForModel('schelling', ['schelling'])).toBe(true);
    expect(showsForModel('sugarscape', ['schelling'])).toBe(false);
    expect(showsForModel('ring', ['sugarscape', 'ring'])).toBe(true);
  });
});

describe('the anasazi’s charts', () => {
  it('count years: a group’s ticks move by the start year', () => {
    expect(lineData(group, 800)[0]).toEqual([800, 805, 809]);
    expect(timeAxisLabel('anasazi')).toBe('Year');
    expect(timeAxisLabel('schelling')).toBe('Tick');
  });

  it('label the year axis without digit grouping', () => {
    expect(yearTickLabels([800, 1000, 1350])).toEqual(['800', '1000', '1350']);
    expect(yearTickLabels([1000.5, 1000.1 + 0.2])).toEqual(['1000.5', '1000.3']);
  });

  it('draw households against the historical record, which Compare draws once', () => {
    expect(MODEL_CHARTS.anasazi.map((c) => c.title)).toEqual([
      'Households vs historical',
      'Carrying capacity',
      'Fit',
      'Mean stored corn',
      'Births, moves and departures',
    ]);
    const lines = MODEL_CHARTS.anasazi[0].lines;
    expect(worldLines(lines, 0).map((l) => l.key)).toEqual(['households', 'historical']);
    expect(worldLines(lines, 1).map((l) => l.key)).toEqual(['households']);
  });
});

describe('firms charts', () => {
  it('chart firms, sizes, effort and pay, output and the scaling exponent over periods', () => {
    expect(MODEL_CHARTS.firms.map((c) => c.title)).toEqual(['Firms', 'Sizes', 'Effort and pay', 'Output', 'Scaling']);
    expect(timeAxisLabel('firms')).toBe('Periods');
  });
});

describe('collusion charts', () => {
  it('chart prices, the profit gain, learning and settling over ticks', () => {
    expect(MODEL_CHARTS.collusion.map((c) => c.title)).toEqual(['Prices', 'Profit gain', 'Learning', 'Settling']);
    expect(timeAxisLabel('collusion')).toBe('Ticks');
  });
});

describe('bali charts', () => {
  it('chart harvest, changing plans, water and pests, patches and the temple match over months', () => {
    expect(MODEL_CHARTS.bali.map((c) => c.title)).toEqual(['Harvest', 'Changing plans', 'Water and pests', 'Patches', 'Temple match']);
    expect(timeAxisLabel('bali')).toBe('Months');
  });
});

describe('zi charts', () => {
  it('chart prices, efficiency, convergence, dispersion and volume over shouts', () => {
    expect(MODEL_CHARTS.zi.map((c) => c.title)).toEqual(['Prices', 'Efficiency', 'Convergence', 'Profit dispersion', 'Volume']);
    expect(timeAxisLabel('zi')).toBe('Shouts');
  });
});

describe('punishment charts', () => {
  it('chart the types, cooperation with its long-run average, payoff and conflict over periods', () => {
    expect(MODEL_CHARTS.punishment.map((c) => c.title)).toEqual(['Types', 'Cooperation', 'Payoff', 'Conflict']);
    expect(MODEL_CHARTS.punishment[1].lines.map((l) => l.key)).toEqual(['cooperation', 'long_run', 'acts']);
    expect(timeAxisLabel('punishment')).toBe('Periods');
  });
});

describe('retirement charts', () => {
  it('chart the retired share (by group when there are groups), retirement ages and the transition over periods', () => {
    expect(MODEL_CHARTS.retirement.map((c) => c.title)).toEqual(['Retired share', 'Retired share', 'Retirement age', 'Transition', 'Group transitions']);
    const one = { model: 'retirement', groups: { enabled: false } } as unknown as ModelConfig;
    const two = { model: 'retirement', groups: { enabled: true } } as unknown as ModelConfig;
    const [byGroup, single] = MODEL_CHARTS.retirement;
    expect([byGroup.shown!(one), byGroup.shown!(two), single.shown!(one), single.shown!(two)]).toEqual([false, true, true, false]);
    expect(timeAxisLabel('retirement')).toBe('Periods');
  });
});

describe('thresholds charts', () => {
  it('chart participation against theory, episodes, the last cascade and the swing over steps', () => {
    expect(MODEL_CHARTS.thresholds.map((c) => c.title)).toEqual(['Participation', 'Episodes', 'Last cascade', 'Swing']);
    expect(timeAxisLabel('thresholds')).toBe('Steps');
  });
});

describe('ants charts', () => {
  it('chart the split, its variance against theory, flips and extremes over steps', () => {
    expect(MODEL_CHARTS.ants.map((c) => c.title)).toEqual(['Share', 'Variance', 'Flips', 'Extremes']);
    expect(timeAxisLabel('ants')).toBe('Steps');
  });
});

describe('El Farol and minority game charts', () => {
  it('chart attendance, fluctuations, success, forecasts, switching and memory over rounds', () => {
    expect(MODEL_CHARTS.farol.map((c) => c.title)).toEqual(['Attendance', 'Fluctuations', 'Success', 'Forecasts', 'Switching', 'Memory']);
    const chart = (t: string) => MODEL_CHARTS.farol.find((c) => c.title === t)!;
    const el = { model: 'farol', game: 'el_farol', mixed_memory: { enabled: false }, evolution: { enabled: false } } as unknown as ModelConfig;
    const mg = { model: 'farol', game: 'minority', mixed_memory: { enabled: true }, evolution: { enabled: false } } as unknown as ModelConfig;
    expect([chart('Forecasts').shown!(el), chart('Forecasts').shown!(mg)]).toEqual([true, false]);
    expect([chart('Memory').shown!(el), chart('Memory').shown!(mg)]).toEqual([false, true]);
    expect(timeAxisLabel('farol')).toBe('Rounds');
  });
});

describe('relative agreement charts', () => {
  it('chart convergence, clusters, dispersion, opinion and uncertainty, and change over periods', () => {
    expect(MODEL_CHARTS.agreement.map((c) => c.title)).toEqual(['Convergence', 'Clusters', 'Dispersion', 'Opinion and uncertainty', 'Change']);
    expect(timeAxisLabel('agreement')).toBe('Periods');
    // p₊ and p₋ count moderates past the cutoff (the boundary less the margin), not at ±1.
    expect(MODEL_CHARTS.agreement[0].lines.map((l) => l.label)).toEqual(['y', 'Moderates turned extremist (+)', 'Moderates turned extremist (−)']);
  });
});

describe('norms charts', () => {
  it('chart boldness and vengefulness, events, payoff, the norm state and (with groups) each group', () => {
    expect(MODEL_CHARTS.norms.map((c) => c.title)).toEqual(['Boldness and vengefulness', 'Events', 'Mean payoff', 'Norm state', 'By group']);
    const grouped = { model: 'norms', groups: { enabled: true } } as unknown as ModelConfig;
    const plain = { model: 'norms', groups: { enabled: false } } as unknown as ModelConfig;
    const byGroup = MODEL_CHARTS.norms.find((c) => c.title === 'By group')!;
    expect([byGroup.shown!(grouped), byGroup.shown!(plain)]).toEqual([true, false]);
    expect(timeAxisLabel('norms')).toBe('Generations');
  });
});

describe('social-structure charts', () => {
  it('chart payoff, cooperation, strategy, high cooperation and copying over periods', () => {
    expect(MODEL_CHARTS.structure.map((c) => c.title)).toEqual(['Mean payoff', 'Cooperation', 'Strategy', 'High cooperation', 'Copying']);
    expect(timeAxisLabel('structure')).toBe('Periods');
  });
});

describe('bounded-confidence charts', () => {
  it('chart clusters, camps, the center, splits and change over periods', () => {
    expect(MODEL_CHARTS.opinions.map((c) => c.title)).toEqual(['Clusters', 'Largest camps', 'Mean and median', 'Splits', 'Change']);
    expect(timeAxisLabel('opinions')).toBe('Periods');
  });
});

describe('classes charts', () => {
  it('chart payoffs, outcomes, memory and the regime, and per-tag payoffs only with tags', () => {
    expect(MODEL_CHARTS.classes.map((c) => c.title)).toEqual(['Mean payoff', 'Outcomes', 'M in memory', 'Regime', 'Payoffs by tag', 'Inter-type advantage']);
    const tagged = { model: 'classes', tags: true } as unknown as ModelConfig;
    const plain = { model: 'classes', tags: false } as unknown as ModelConfig;
    expect(MODEL_CHARTS.classes.filter((c) => c.shown).map((c) => [c.shown!(tagged), c.shown!(plain)])).toEqual([
      [true, false],
      [true, false],
    ]);
    expect(timeAxisLabel('classes')).toBe('Periods');
  });
});

describe('culture charts', () => {
  it('chart regions, zones and cultures, the largest region, similarity, bonds and changes against events per site', () => {
    expect(MODEL_CHARTS.culture.map((c) => c.title)).toEqual(['Regions, zones and cultures', 'Largest region', 'Mean similarity', 'Active bonds', 'Changes']);
    expect(timeAxisLabel('culture')).toBe('Events per site');
  });
});

describe('tags charts', () => {
  it('chart donation, tolerance, clusters, tags and takeovers against the generation', () => {
    expect(MODEL_CHARTS.tags.map((c) => c.title)).toEqual(['Donation rate', 'Tolerance', 'Clusters', 'Distinct tags', 'Takeovers']);
    expect(MODEL_CHARTS.tags.every((c) => !c.shown)).toBe(true);
    expect([timeAxisLabel('tags'), timeAxisLabel('civil'), timeAxisLabel('anasazi')]).toEqual(['Generation', 'Tick', 'Year']);
  });
});

describe('civil charts', () => {
  it('show Model II’s groups and kills only in Model II', () => {
    const ethnic = { model: 'civil', variant: 'ethnic' } as unknown as ModelConfig;
    const rebellion = { model: 'civil', variant: 'rebellion' } as unknown as ModelConfig;
    expect([isEthnic(ethnic), isEthnic(rebellion)]).toEqual([true, false]);
    const conditional = MODEL_CHARTS.civil.filter((c) => c.shown).map((c) => c.title);
    expect(conditional).toEqual(['Groups', 'Killed']);
    expect(MODEL_CHARTS.civil.map((c) => c.title)).toEqual([
      'Actives, quiet and jailed',
      'Legitimacy',
      'Cops',
      'Tension',
      'Outbursts',
      'Wait between outbursts',
      'Activation per outburst',
      'Groups',
      'Killed',
    ]);
  });
});

describe('the spatial games’ charts', () => {
  it('charts the spatial games’ cooperators, changes, switches and payoffs', () => {
    expect(MODEL_CHARTS.spatial.map((c) => c.title)).toEqual(['Cooperators', 'Changes', 'Switches', 'Payoffs']);
  });
});

describe('the ethnocentrism model’s charts', () => {
  it('charts strategies, cooperation, population and kin against the period', () => {
    expect(MODEL_CHARTS.ethno.map((c) => c.title)).toEqual(['Strategies', 'Cooperation', 'Population', 'Kin']);
    expect(MODEL_CHARTS.ethno[0].lines.map((l) => [l.key, l.color])).toEqual([
      ['ethnocentric', '--lender'],
      ['humanitarian', '--blue'],
      ['selfish', '--red'],
      ['traitorous', '--both'],
      ['kin', '--c4'],
      ['nonkin', '--c2'],
      ['mixed', '--muted'],
    ]);
    expect(MODEL_CHARTS.ethno.every((c) => !c.shown)).toBe(true);
    expect(timeAxisLabel('ethno')).toBe('Period');
  });
});

describe('the demographic PD’s charts', () => {
  it('charts population, cooperator share, surrounded cooperators, wealth and births and deaths against the cycle', () => {
    expect(MODEL_CHARTS.dpd.map((c) => c.title)).toEqual(['Population', 'Cooperator share', 'Surrounded cooperators', 'Wealth', 'Births and deaths']);
    expect(MODEL_CHARTS.dpd.map((c) => c.lines.map((l) => [l.key, l.color]))).toEqual([
      [
        ['cooperators', '--blue'],
        ['defectors', '--red'],
      ],
      [['cooperator_share', '--blue']],
      [['surrounded', '--c1']],
      [
        ['wealth_c', '--blue'],
        ['wealth_d', '--red'],
      ],
      [
        ['births', '--c2'],
        ['deaths', '--muted'],
      ],
    ]);
    expect(MODEL_CHARTS.dpd[1].range).toEqual([0, 1]);
    expect(MODEL_CHARTS.dpd.every((c) => !c.shown)).toBe(true);
    expect(timeAxisLabel('dpd')).toBe('Cycle');
  });
});

describe('image scoring’s charts', () => {
  const image = (strategies: string[]) => ({ model: 'image', strategies }) as unknown as ModelConfig;

  it('charts the help rate with cooperative strategies, mean k, strategy shares and mean payoff against the generation', () => {
    expect(MODEL_CHARTS.image.map((c) => c.title)).toEqual(['Help rate', 'Mean k', 'Strategy shares', 'Binary scorers and standing', 'Mean payoff']);
    expect(MODEL_CHARTS.image.map((c) => c.lines.map((l) => [l.key, l.color]))).toEqual([
      [
        ['help_rate', '--c1'],
        ['cooperative', '--blue'],
      ],
      [['mean_k', '--c4']],
      [
        ['k_cooperative', '--blue'],
        ['k_defective', '--red'],
        ['h', '--c3'],
        ['own_only', '--muted'],
        ['and', '--c1'],
        ['or', '--c4'],
        ['standing', '--c2'],
        ['q', '--lender'],
      ],
      [
        ['binary_c', '--blue'],
        ['binary_x', '--c3'],
        ['binary_d', '--red'],
        ['standing', '--c2'],
      ],
      [['mean_payoff', '--c2']],
    ]);
    expect(MODEL_CHARTS.image.map((c) => c.range)).toEqual([[0, 1], [-5, 6], [0, 1], [0, 1], undefined]);
    expect(timeAxisLabel('image')).toBe('Generation');
  });

  it('shows mean k only with a class that has a k, and the binary scorers’ shares instead of the others', () => {
    const shown = (strategies: string[]) => MODEL_CHARTS.image.map((c) => !c.shown || c.shown(image(strategies)));
    expect(shown(['k'])).toEqual([true, true, true, false, true]);
    expect(shown(['and', 'q'])).toEqual([true, true, true, false, true]);
    expect(shown(['own_only'])).toEqual([true, false, true, false, true]);
    expect(shown(['binary', 'standing'])).toEqual([true, true, false, true, true]);
  });
});

describe('hoard charts (Minds 7)', () => {
  const generations = (values: Partial<Record<string, number[]>>) =>
    Object.fromEntries(
      ['generation', 'mean_larder_prob', 'hoarder_larder_prob', 'mean_defense', 'survivors', 'larder_share', 'larder_loss_rate', 'scatter_loss_rate'].map((k) => [
        k,
        Float64Array.from(values[k] ?? [0, 0]),
      ]),
    ) as HoardCharts['generations'];

  it('chart L, D, survivors, the larder share and the loss rates by generation, and this season by bout, one unit to a chart', () => {
    expect(HOARD_CHARTS.map((c) => [c.title, c.season === true])).toEqual([
      ['Larder probability (L) by generation', false],
      ['Defense propensity (D) by generation', false],
      ['Survivors by generation', false],
      ['Larder share by generation', false],
      ['Loss rates by generation (per item-day held)', false],
      ['Larder share this season (per bout)', true],
    ]);
    expect(HOARD_CHARTS[0].lines.map((l) => l.key)).toEqual(['mean_larder_prob', 'hoarder_larder_prob']);
    expect(HOARD_CHARTS[4].lines.map((l) => l.key)).toEqual(['larder_loss_rate', 'scatter_loss_rate']);
    expect(MODEL_CHARTS.hoard).toEqual([]);
    expect(timeAxisLabel('hoard')).toBe('Bouts');
  });

  it('puts generations on x and NaN (all cheaters, nothing held) as gaps', () => {
    const charts: HoardCharts = {
      generations: generations({ generation: [1, 2, 3], mean_larder_prob: [0.2, 0.5, 0.9], hoarder_larder_prob: [0.2, NaN, 0.9] }),
      season: new Float64Array(0),
    };
    expect(hoardGenerationTable(charts, ['mean_larder_prob', 'hoarder_larder_prob'])).toEqual([
      [1, 2, 3],
      [0.2, 0.5, 0.9],
      [0.2, null, 0.9],
    ]);
    expect(hoardGenerationTable(null, ['survivors'])).toEqual([[], []]);
  });

  it('counts this season in days from its first bout', () => {
    const c = { days: 100, bouts: 20 } as HoardConfig;
    const season = (flat: number[]): HoardCharts => ({ generations: generations({}), season: Float64Array.from(flat) });
    // Generation 1 starts at tick 0; generation 2's bouts are ticks 2001 to 4000.
    expect(hoardSeasonTable(season([0, NaN, 1, 0.5, 20, 0.25]), c)).toEqual([
      [0, 0.05, 1],
      [null, 0.5, 0.25],
    ]);
    expect(hoardSeasonTable(season([2001, 0.1, 2040, 0.2]), c)).toEqual([
      [0.05, 2],
      [0.1, 0.2],
    ]);
    expect(hoardSeasonTable(null, c)).toEqual([[], []]);
  });
});

describe('stumbledData', () => {
  it('turns the pilfered column into what was stumbled on (pilfered − raided), keeping gaps', () => {
    expect(stumbledData([[1, 2, 3], [0, 1, null], [2, 1, 4]])).toEqual([[1, 2, 3], [0, 1, null], [2, 0, null]]);
  });
});

describe('spatial episode chart series', () => {
  it('separates kinds and exposes unused-store gaps', () => {
    const keys = SPATIAL_HOARDING_CHARTS.flatMap(c => c.lines.map(l => l.key));
    expect(keys).toContain('scatter_stock_ticks');
    expect(keys).toContain('larder_stock_ticks');
    expect(keys).toContain('guard_executed');
    expect(keys).toContain('delivery_return_turns');
    expect(lineData({ ticks: Float64Array.of(0,1), columns: [Float64Array.of(NaN,0.25)] })).toEqual([[0,1],[null,0.25]]);
  });
});
