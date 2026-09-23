import type { Config } from './types';

interface Base { path: string; label: string; reset?: boolean; hint?: string }
export type Control =
  | (Base & { kind: 'toggle' })
  | (Base & { kind: 'number'; min: number; max: number; step: number })
  | (Base & { kind: 'range'; min: number; max: number })
  | (Base & { kind: 'select'; options: { value: string; label: string; apply: (c: Config) => void }[]; current: (c: Config) => string });

export interface Group {
  title: string;
  /** Path of the boolean that switches this rule on (shown in the header). */
  enable?: string;
  note?: string;
  controls: Control[];
}

export const GROUPS: Group[] = [
  {
    title: 'Setup',
    note: 'Changing these rebuilds the world.',
    controls: [
      {
        kind: 'select', path: 'landscape', label: 'Landscape', reset: true,
        current: (c) => c.landscape.kind,
        options: [
          { value: 'two_peaks', label: 'Two sugar mountains (50×50)', apply: (c) => { c.landscape = { kind: 'two_peaks' }; c.width = 50; c.height = 50; } },
          { value: 'flat', label: 'Flat', apply: (c) => { c.landscape = { kind: 'flat', capacity: 2 }; } },
        ],
      },
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
      { kind: 'number', path: 'tag_length', label: 'Tag length', min: 1, max: 64, step: 1, reset: true },
    ],
  },
  {
    title: 'New agents',
    note: 'Initial and replacement agents draw traits uniformly from these ranges.',
    controls: [
      { kind: 'range', path: 'vision', label: 'Vision', min: 1, max: 25 },
      { kind: 'range', path: 'metabolism', label: 'Metabolism', min: 0, max: 10 },
      { kind: 'range', path: 'endowment', label: 'Initial sugar', min: 0, max: 500 },
    ],
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
    controls: [
      { kind: 'number', path: 'pollution.production', label: 'Per sugar gathered α', min: 0, max: 5, step: 0.1 },
      { kind: 'number', path: 'pollution.consumption', label: 'Per sugar eaten β', min: 0, max: 5, step: 0.1 },
    ],
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
  { title: 'Culture (K)', enable: 'culture.enabled', controls: [] },
  {
    title: 'Combat (C)', enable: 'combat.enabled',
    controls: [
      { kind: 'toggle', path: 'combat.unlimited', label: 'Unlimited reward (C∞)' },
      { kind: 'number', path: 'combat.reward', label: 'Reward cap α', min: 0, max: 50, step: 0.5 },
    ],
  },
];
