import { defaultGroups, sameGroups } from './groups';
import type { Caching, Central, Config, Decision, Goap, Memory, Movement, Mvt, Truffles } from './types';

/** A config's decision, or the book's for older configs. */
const decision = (c: Config): Decision => c.decision ?? { rule: 'book', travel: 0, crowding: 0, idle: 'stay' };

/** A config's movement, or the book's jump at speed 1 for older configs. */
const movement = (c: Config): Movement => c.movement ?? { mode: 'jump', speed: 1 };

/** A config's memory, or the engine's default (span 0: memory off) for older configs. */
const memory = (c: Config): Memory => c.memory ?? { span: 0, share: 1, belief: 'project', prior: 'none' };

/** A config's GOAP search, or the engine's default for older configs. */
const goap = (c: Config): Goap => c.goap ?? { k: 8, horizon: 10, shortlist: 'rate' };

/** A config's marginal-value rule, or the engine's default for older configs. */
const mvt = (c: Config): Mvt => c.mvt ?? { alpha: 0.05 };

/** A config's truffles, or the engine's default (share 0: no truffles) for older configs. */
const truffles = (c: Config): Truffles => c.truffles ?? { share: 0, value: 5, regrow: 30, seed: 1 };

/** A config's caching, or the engine's default (rule none, no carrying limit) for older configs. */
const caching = (c: Config): Caching =>
  c.caching ?? { rule: 'none', capacity: 0, share: 0.5, lambda: 0.5, lookahead: 1, mixed: false };

/** A config's central-place foraging, or the engine's default (off) for older configs. */
const central = (c: Config): Central => c.central ?? { enabled: false };

interface Base {
  path: string;
  label: string;
  reset?: boolean;
  hint?: string;
  /** Called after the control sets its value on a copy of the config (`before` is the copy as it was). */
  adjust?: (next: Config, before: Config) => void;
}
export type Control =
  | (Base & { kind: 'toggle' })
  | (Base & { kind: 'number'; min: number; max: number; step: number })
  | (Base & { kind: 'range'; min: number; max: number })
  | (Base & { kind: 'select'; options: { value: string; label: string; apply: (c: Config) => void }[]; current: (c: Config) => string });

export interface Group {
  title: string;
  /** Path of the boolean that switches this rule on (shown in the header). */
  enable?: string;
  /** Switching the rule on or off rebuilds the world (reset) instead of editing it. */
  enableResets?: boolean;
  note?: string;
  controls: Control[];
  /** A hand-built editor shown after the controls. */
  custom?: 'goods' | 'pollution' | 'groups';
  /** One of the Minds rules: shown only for a Minds world (the model menu's Minds entry). */
  minds?: true;
}

export const GROUPS: Group[] = [
  {
    title: 'Setup',
    note: 'Changing these rebuilds the world.',
    controls: [
      { kind: 'number', path: 'width', label: 'Width', min: 10, max: 200, step: 1, reset: true },
      { kind: 'number', path: 'height', label: 'Height', min: 10, max: 200, step: 1, reset: true },
      { kind: 'number', path: 'population', label: 'Initial agents', min: 0, max: 4000, step: 10, reset: true },
      {
        kind: 'select', path: 'placement', label: 'Placement', reset: true,
        current: (c) => c.placement.kind,
        options: [
          { value: 'random', label: 'Random', apply: (c) => { c.placement = { kind: 'random' }; } },
          { value: 'block', label: 'Southwest block', apply: (c) => { c.placement = { kind: 'block', x: 0, y: Math.floor(c.height / 2), width: Math.floor(c.width / 2), height: Math.ceil(c.height / 2) }; } },
          { value: 'tribes', label: 'Two tribes in corners', apply: (c) => { c.placement = { kind: 'tribes', size: Math.floor(Math.min(c.width, c.height) * 0.4) }; } },
        ],
      },
      {
        kind: 'number', path: 'tag_length', label: 'Tag length', min: 1, max: 64, step: 1, reset: true,
        // Default groups follow the tag length; custom groups are kept (Decision 7).
        adjust: (next, before) => {
          if (sameGroups(before.culture.groups, defaultGroups(before.tag_length))) next.culture.groups = defaultGroups(next.tag_length);
        },
      },
    ],
  },
  {
    title: 'New agents',
    note: "Initial and replacement agents draw vision from this range; each good's metabolism and endowment ranges are in Goods.",
    controls: [{ kind: 'range', path: 'vision', label: 'Vision', min: 1, max: 25 }],
  },
  {
    title: 'Goods',
    custom: 'goods',
    note: 'Adding or removing a good or changing its map rebuilds the world; names, colors and trait ranges apply to the running world (traits to agents born from now on). Trade and foresight need two goods; combat needs one.',
    controls: [],
  },
  {
    title: 'Growback (G)',
    controls: [
      { kind: 'number', path: 'growback.rate', label: 'Rate α', min: 0.1, max: 10, step: 0.1 },
      { kind: 'toggle', path: 'growback.instant', label: 'Instant (G∞)' },
    ],
  },
  {
    title: 'Seasons', enable: 'seasons.enabled',
    controls: [
      { kind: 'number', path: 'seasons.period', label: 'Season length γ', min: 1, max: 500, step: 1 },
      { kind: 'number', path: 'seasons.winter_divisor', label: 'Winter slowdown β', min: 1, max: 32, step: 1 },
    ],
  },
  {
    title: 'Pollution (P)', enable: 'pollution.enabled',
    custom: 'pollution',
    note: 'Each pollutant forms from what agents gather and eat of each good, and devalues the goods it marks when they choose sites. Adding or removing a pollutant rebuilds the world; coefficients apply live.',
    controls: [],
  },
  {
    title: 'Diffusion (D)', enable: 'diffusion.enabled',
    controls: [{ kind: 'number', path: 'diffusion.every', label: 'Every α ticks', min: 1, max: 50, step: 1 }],
  },
  {
    title: 'Lifespan', enable: 'lifespan.enabled',
    controls: [
      { kind: 'range', path: 'lifespan.max_age', label: 'Max age [a, b]', min: 1, max: 300 },
      {
        kind: 'select', path: 'lifespan.founders', label: 'Founders start', reset: true,
        current: (c) => c.lifespan.founders ?? 'newborn',
        options: [
          { value: 'newborn', label: 'Newborn', apply: (c) => { c.lifespan.founders = 'newborn'; } },
          { value: 'random', label: 'At random ages (the book\'s VI-2 figure)', apply: (c) => { c.lifespan.founders = 'random'; } },
        ],
      },
    ],
  },
  { title: 'Replacement (R)', enable: 'replacement.enabled', note: 'Needs lifespan; excludes sex.', controls: [] },
  {
    title: 'Sex (S)', enable: 'sex.enabled',
    controls: [
      { kind: 'range', path: 'sex.fertility_onset', label: 'Fertility begins', min: 0, max: 100 },
      { kind: 'range', path: 'sex.female_end', label: 'Female fertility ends', min: 0, max: 150 },
      { kind: 'range', path: 'sex.male_end', label: 'Male fertility ends', min: 0, max: 150 },
      {
        kind: 'select', path: 'sex.fertile_wealth', label: 'Wealth to have children (two goods)',
        current: (c) => c.sex.fertile_wealth ?? 'each_good',
        options: [
          { value: 'each_good', label: 'Its endowment of each good', apply: (c) => { c.sex.fertile_wealth = 'each_good'; } },
          { value: 'total', label: 'Its endowment in total', apply: (c) => { c.sex.fertile_wealth = 'total'; } },
          { value: 'welfare', label: 'The welfare of its endowment', apply: (c) => { c.sex.fertile_wealth = 'welfare'; } },
          { value: 'sugar', label: 'Its sugar endowment alone', apply: (c) => { c.sex.fertile_wealth = 'sugar'; } },
        ],
      },
    ],
  },
  { title: 'Inheritance (I)', enable: 'inheritance.enabled', controls: [] },
  {
    title: 'Culture (K)', enable: 'culture.enabled',
    custom: 'groups',
    note: 'An agent belongs to the first group whose range holds its number of zero tags. Combat, the Tribe colors and the Group shares chart use the groups even while culture is off. Adding or removing a group or changing a range rebuilds the world; names and colors apply live. Axelrod\u2019s rule (Axtell et al. 1996\u2019s docking) gives agents features of several traits instead; the Culture color mode draws them.',
    controls: [
      {
        kind: 'select', path: 'culture.rule', label: 'Rule', reset: true,
        current: (c) => c.culture.rule ?? 'flip',
        options: [
          { value: 'flip', label: 'Flip a neighbor\u2019s tag (book)', apply: (c) => { c.culture.rule = 'flip'; } },
          { value: 'axelrod', label: 'Axelrod: copy from a similar neighbor', apply: (c) => { c.culture.rule = 'axelrod'; } },
        ],
      },
      { kind: 'number', path: 'culture.features', label: 'Axelrod features', min: 1, max: 32, step: 1, reset: true },
      { kind: 'number', path: 'culture.traits', label: 'Axelrod traits per feature', min: 2, max: 255, step: 1, reset: true },
      { kind: 'toggle', path: 'culture.stop_when_settled', label: 'Stop when cultures settle (Axelrod; with births, a child can unsettle them)' },
    ],
  },
  {
    title: 'Combat (C)', enable: 'combat.enabled',
    note: 'Needs exactly one good.',
    controls: [
      { kind: 'toggle', path: 'combat.unlimited', label: 'Unlimited reward (C∞)' },
      { kind: 'number', path: 'combat.reward', label: 'Reward cap α', min: 0, max: 50, step: 0.5 },
    ],
  },
  {
    title: 'Trade (T)', enable: 'trade.enabled',
    note: 'Needs at least two goods. The price rule applies to the running world and can be scheduled.',
    controls: [
      {
        kind: 'select', path: 'trade.price', label: 'Price rule',
        current: (c) => c.trade.price,
        options: [
          { value: 'geometric_mean', label: 'Geometric mean √(MRS_A·MRS_B) (book)', apply: (c) => { c.trade.price = 'geometric_mean'; } },
          { value: 'random', label: 'Random between the two MRSs (note 15)', apply: (c) => { c.trade.price = 'random'; } },
        ],
      },
    ],
  },
  {
    title: 'Credit (L)', enable: 'credit.enabled', note: 'Loans in every good, for childbearing; needs sex.',
    controls: [
      { kind: 'number', path: 'credit.duration', label: 'Duration d (ticks)', min: 1, max: 50, step: 1 },
      { kind: 'number', path: 'credit.rate', label: 'Interest r (% per tick)', min: 0, max: 100, step: 1 },
    ],
  },
  {
    title: 'Foresight', enable: 'foresight.enabled', note: 'Needs at least two goods.',
    controls: [{ kind: 'range', path: 'foresight.range', label: 'Foresight φ', min: 0, max: 20 }],
  },
  {
    title: 'Decision (Minds 1, 4)',
    minds: true,
    note: 'Which rule decides where an agent moves. The book’s rule M goes to the best site in sight. The utility mind multiplies that welfare by travel and crowding considerations; with both at 0 and Idle at Stay it is rule M exactly. Travel, crowding and idle apply only under the utility mind; rule C decides moves under combat. GOAP plans a run of harvests among the sites it knows that gathers enough food for the horizon, pricing each walk by its length; it needs one good. The marginal-value rule keeps a running average of its intake and leaves a patch once nothing within a step is worth that average; it also needs one good. Both need walking (Movement: Walk).',
    controls: [
      {
        kind: 'select', path: 'decision.rule', label: 'Rule', reset: true,
        current: (c) => c.decision?.rule ?? 'book',
        options: [
          { value: 'book', label: 'Rule M (book)', apply: (c) => { c.decision = { ...decision(c), rule: 'book' }; } },
          { value: 'utility', label: 'Utility mind', apply: (c) => { c.decision = { ...decision(c), rule: 'utility' }; } },
          { value: 'goap', label: 'GOAP (plan)', apply: (c) => { c.decision = { ...decision(c), rule: 'goap' }; } },
          { value: 'mvt', label: 'Marginal value (leave below your average)', apply: (c) => { c.decision = { ...decision(c), rule: 'mvt' }; } },
        ],
      },
      {
        kind: 'number', path: 'decision.travel', label: 'Travel k (welfare ÷ (1 + k·distance))', min: 0, max: 10, step: 0.1,
        adjust: (next) => { next.decision = { ...decision(next), ...next.decision }; },
      },
      {
        kind: 'number', path: 'decision.crowding', label: 'Crowding m (welfare × (1 + neighbors)^−m)', min: 0, max: 10, step: 0.1,
        adjust: (next) => { next.decision = { ...decision(next), ...next.decision }; },
      },
      {
        kind: 'select', path: 'decision.idle', label: 'When nothing in sight scores',
        current: (c) => c.decision?.idle ?? 'stay',
        options: [
          { value: 'stay', label: 'Stay (book)', apply: (c) => { c.decision = { ...decision(c), idle: 'stay' }; } },
          { value: 'wander', label: 'Wander to a random free site in sight', apply: (c) => { c.decision = { ...decision(c), idle: 'wander' }; } },
        ],
      },
      {
        kind: 'number', path: 'goap.k', label: 'GOAP: known sites a plan considers', min: 1, max: 12, step: 1,
        adjust: (next) => { next.goap = { ...goap(next), ...next.goap }; },
      },
      {
        kind: 'number', path: 'goap.horizon', label: 'GOAP: ticks of food a plan gathers', min: 1, max: 100, step: 1,
        adjust: (next) => { next.goap = { ...goap(next), ...next.goap }; },
      },
      {
        kind: 'select', path: 'goap.shortlist', label: 'GOAP: which known sites',
        current: (c) => goap(c).shortlist,
        options: [
          { value: 'rate', label: 'Most sugar per step (rate)', apply: (c) => { c.goap = { ...goap(c), shortlist: 'rate' }; } },
          { value: 'value', label: 'Most sugar (value)', apply: (c) => { c.goap = { ...goap(c), shortlist: 'value' }; } },
        ],
      },
      {
        kind: 'number', path: 'mvt.alpha', label: 'Marginal value: smoothing α of the average', min: 0.01, max: 1, step: 0.01,
        adjust: (next) => { next.mvt = { ...mvt(next), ...next.mvt }; },
      },
    ],
  },
  {
    title: 'Movement (Minds 2)',
    minds: true,
    note: 'How an agent reaches the site it chose. The book’s rule M jumps there in one tick. Walking takes that many steps a tick along an A* path around walls and other agents, and plans again every tick. Walls and fences come from presets (the Minds 2 fence presets); a wall also blocks sight.',
    controls: [
      {
        kind: 'select', path: 'movement.mode', label: 'Mode',
        current: (c) => c.movement?.mode ?? 'jump',
        options: [
          { value: 'jump', label: 'Jump (book)', apply: (c) => { c.movement = { ...movement(c), mode: 'jump' }; } },
          { value: 'walk', label: 'Walk', apply: (c) => { c.movement = { ...movement(c), mode: 'walk' }; } },
        ],
      },
      {
        kind: 'number', path: 'movement.speed', label: 'Speed (cells per tick)', min: 1, max: 50, step: 1,
        adjust: (next) => { next.movement = { ...movement(next), ...next.movement }; },
      },
    ],
  },
  {
    title: 'Memory (Minds 3)',
    minds: true,
    note: 'Memory needs walking (Movement: Walk). A remembered site’s belief is what was seen (recall) or that plus growback since (project).',
    controls: [
      {
        kind: 'number', path: 'memory.span', label: 'Span (ticks a site is remembered)', min: 0, max: 10_000, step: 1, reset: true,
        adjust: (next) => { next.memory = { ...memory(next), ...next.memory }; },
      },
      {
        kind: 'number', path: 'memory.share', label: 'Share born remembering', min: 0, max: 1, step: 0.05, reset: true,
        adjust: (next) => { next.memory = { ...memory(next), ...next.memory }; },
      },
      {
        kind: 'select', path: 'memory.belief', label: 'Belief about a remembered site',
        current: (c) => memory(c).belief,
        options: [
          { value: 'recall', label: 'Recall: what it saw there', apply: (c) => { c.memory = { ...memory(c), belief: 'recall' }; } },
          { value: 'project', label: 'Project: that plus growback since', apply: (c) => { c.memory = { ...memory(c), belief: 'project' }; } },
        ],
      },
      {
        kind: 'select', path: 'memory.prior', label: 'Founders start knowing', reset: true,
        current: (c) => memory(c).prior ?? 'none',
        options: [
          { value: 'none', label: 'Nothing (book)', apply: (c) => { c.memory = { ...memory(c), prior: 'none' }; } },
          { value: 'map', label: 'The whole map (needs a span)', apply: (c) => { c.memory = { ...memory(c), prior: 'map' }; } },
        ],
      },
    ],
  },
  {
    title: 'Truffles',
    minds: true,
    note: 'Hidden spots, found only by stopping on them; they ripen again a fixed time after a harvest.',
    controls: [
      {
        kind: 'number', path: 'truffles.share', label: 'Share of sites with a spot', min: 0, max: 1, step: 0.01, reset: true,
        adjust: (next) => { next.truffles = { ...truffles(next), ...next.truffles }; },
      },
      {
        kind: 'number', path: 'truffles.value', label: 'Value when picked', min: 0, max: 50, step: 0.5,
        adjust: (next) => { next.truffles = { ...truffles(next), ...next.truffles }; },
      },
      {
        kind: 'number', path: 'truffles.regrow', label: 'Regrow time (ticks)', min: 1, max: 10_000, step: 1,
        adjust: (next) => { next.truffles = { ...truffles(next), ...next.truffles }; },
      },
      {
        kind: 'number', path: 'truffles.seed', label: 'Layout seed', min: 0, max: 999_999, step: 1, reset: true,
        adjust: (next) => { next.truffles = { ...truffles(next), ...next.truffles }; },
      },
    ],
  },
  {
    title: 'Caching (Minds 5)',
    minds: true,
    note: 'A carrying limit, and caches an agent buries and digs back when it runs short. Even buries a share of its surplus wherever it is; compensate buries more where it has found food less often (each find lowers a place’s weight by λ); plan remembers where it was and what it found and buries for the shortfall it foresees, up to the lookahead in days, or, in a winter everywhere, for the winter ahead. Caching needs one good and walking, and no combat. With the Seasons rule on, the seasons mode can put winter on every row at once, with the same γ and β. Central-place foraging gives each agent a home to carry loads back to; it needs the marginal-value rule or GOAP, and a carrying limit.',
    controls: [
      {
        kind: 'select', path: 'caching.rule', label: 'Caching rule', reset: true,
        current: (c) => caching(c).rule,
        options: [
          { value: 'none', label: 'None (no caching)', apply: (c) => { c.caching = { ...caching(c), rule: 'none' }; } },
          { value: 'even', label: 'Even: bury a share of any surplus', apply: (c) => { c.caching = { ...caching(c), rule: 'even' }; } },
          { value: 'compensate', label: 'Compensate: more where food was scarce', apply: (c) => { c.caching = { ...caching(c), rule: 'compensate' }; } },
          { value: 'plan', label: 'Plan: for the shortfall it foresees', apply: (c) => { c.caching = { ...caching(c), rule: 'plan' }; } },
        ],
      },
      {
        kind: 'toggle', path: 'caching.mixed', label: 'Mix the rules (a quarter each, by id)', reset: true,
        adjust: (next) => { next.caching = { ...caching(next), ...next.caching }; },
      },
      {
        kind: 'number', path: 'caching.capacity', label: 'Carrying limit (0 for none)', min: 0, max: 500, step: 1, reset: true,
        adjust: (next) => { next.caching = { ...caching(next), ...next.caching }; },
      },
      {
        kind: 'number', path: 'caching.share', label: 'Share of surplus buried', min: 0.05, max: 1, step: 0.05,
        adjust: (next) => { next.caching = { ...caching(next), ...next.caching }; },
      },
      {
        kind: 'number', path: 'caching.lambda', label: 'Compensate: weight lost per find λ', min: 0.05, max: 1, step: 0.05,
        adjust: (next) => { next.caching = { ...caching(next), ...next.caching }; },
      },
      {
        kind: 'number', path: 'caching.lookahead', label: 'Plan: days looked ahead', min: 1, max: 10, step: 1,
        adjust: (next) => { next.caching = { ...caching(next), ...next.caching }; },
      },
      {
        kind: 'select', path: 'seasons.mode', label: 'Seasons', reset: true,
        current: (c) => c.seasons.mode ?? 'hemispheres',
        options: [
          { value: 'hemispheres', label: 'North and south take turns (book)', apply: (c) => { c.seasons = { ...c.seasons, mode: 'hemispheres' }; } },
          { value: 'global', label: 'Winter everywhere at once', apply: (c) => { c.seasons = { ...c.seasons, mode: 'global' }; } },
        ],
      },
      {
        kind: 'toggle', path: 'central.enabled', label: 'Central-place foraging (a home to carry loads to)', reset: true,
        adjust: (next) => { next.central = { ...central(next), ...next.central }; },
      },
    ],
  },
  {
    title: 'Disease (E)', enable: 'disease.enabled', enableResets: true,
    note: 'Immune strings learn the diseases agents carry; each disease raises metabolism. Turning disease on or off, the disease list and the immune length rebuild the world; the rest applies to the running world.',
    controls: [
      { kind: 'number', path: 'disease.count', label: 'Diseases', min: 1, max: 100, step: 1, reset: true },
      { kind: 'range', path: 'disease.length', label: 'Disease length', min: 1, max: 63, reset: true },
      { kind: 'number', path: 'disease.immune_length', label: 'Immune length', min: 2, max: 64, step: 1, reset: true },
      { kind: 'number', path: 'disease.initial', label: 'Diseases per new agent', min: 0, max: 100, step: 1 },
      { kind: 'number', path: 'disease.fee', label: 'Metabolism per disease', min: 0, max: 5, step: 0.5 },
      { kind: 'number', path: 'disease.flips_per_tick', label: 'Immune flips per tick (medicine)', min: 1, max: 10, step: 1 },
      {
        kind: 'select', path: 'disease.learning', label: 'Immune learning',
        current: (c) => c.disease.learning,
        options: [
          { value: 'per_agent', label: 'Flips per agent (note 16, book)', apply: (c) => { c.disease.learning = 'per_agent'; } },
          { value: 'per_disease', label: 'Flips per carried disease', apply: (c) => { c.disease.learning = 'per_disease'; } },
        ],
      },
      {
        kind: 'select', path: 'disease.cure', label: 'A learned disease is dropped',
        current: (c) => c.disease.cure,
        options: [
          { value: 'next_tick', label: 'Next tick, passed on meanwhile (book)', apply: (c) => { c.disease.cure = 'next_tick'; } },
          { value: 'immediate', label: 'At once', apply: (c) => { c.disease.cure = 'immediate'; } },
        ],
      },
      { kind: 'number', path: 'disease.genome_mutation', label: 'Genome mutation rate', min: 0, max: 0.1, step: 0.001 },
      { kind: 'number', path: 'disease.disease_mutation', label: 'Disease mutation rate', min: 0, max: 1, step: 0.01 },
    ],
  },
];
