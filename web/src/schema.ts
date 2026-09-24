import { defaultGroups, sameGroups } from './groups';
import type { Config } from './types';

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
      { kind: 'number', path: 'seasons.winter_divisor', label: 'Winter slowdown β', min: 1, max: 20, step: 1 },
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
    controls: [{ kind: 'range', path: 'lifespan.max_age', label: 'Max age [a, b]', min: 1, max: 300 }],
  },
  { title: 'Replacement (R)', enable: 'replacement.enabled', note: 'Needs lifespan; excludes sex.', controls: [] },
  {
    title: 'Sex (S)', enable: 'sex.enabled',
    controls: [
      { kind: 'range', path: 'sex.fertility_onset', label: 'Fertility begins', min: 0, max: 100 },
      { kind: 'range', path: 'sex.female_end', label: 'Female fertility ends', min: 0, max: 150 },
      { kind: 'range', path: 'sex.male_end', label: 'Male fertility ends', min: 0, max: 150 },
    ],
  },
  { title: 'Inheritance (I)', enable: 'inheritance.enabled', controls: [] },
  {
    title: 'Culture (K)', enable: 'culture.enabled',
    custom: 'groups',
    note: 'An agent belongs to the first group whose range holds its number of zero tags. Combat, the Tribe colors and the Group shares chart use the groups even while culture is off. Adding or removing a group or changing a range rebuilds the world; names and colors apply live.',
    controls: [],
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
    title: 'Disease (E)', enable: 'disease.enabled', enableResets: true,
    note: 'Immune strings learn the diseases agents carry; each disease raises metabolism. Turning disease on or off, the disease list and the immune length rebuild the world; the rest applies to the running world.',
    controls: [
      { kind: 'number', path: 'disease.count', label: 'Diseases', min: 1, max: 100, step: 1, reset: true },
      { kind: 'range', path: 'disease.length', label: 'Disease length', min: 1, max: 63, reset: true },
      { kind: 'number', path: 'disease.immune_length', label: 'Immune length', min: 2, max: 64, step: 1, reset: true },
      { kind: 'number', path: 'disease.initial', label: 'Diseases per new agent', min: 0, max: 100, step: 1 },
      { kind: 'number', path: 'disease.fee', label: 'Metabolism per disease', min: 0, max: 5, step: 0.5 },
      { kind: 'number', path: 'disease.flips_per_tick', label: 'Immune flips per tick (medicine)', min: 1, max: 10, step: 1 },
      { kind: 'number', path: 'disease.genome_mutation', label: 'Genome mutation rate', min: 0, max: 0.1, step: 0.001 },
      { kind: 'number', path: 'disease.disease_mutation', label: 'Disease mutation rate', min: 0, max: 1, step: 0.01 },
    ],
  },
];
